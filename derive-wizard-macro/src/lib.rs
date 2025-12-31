use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{Data, Fields, Lit, Meta, Type, parse_macro_input};

use derive_wizard_types::interview::{
    ConfirmQuestion, FloatQuestion, InputQuestion, IntQuestion, Interview, MaskedQuestion,
    MultilineQuestion, Question, QuestionKind,
};

#[proc_macro_derive(
    Wizard,
    attributes(
        prompt,
        mask,
        editor,
        validate_on_submit,
        validate_on_key,
        validate,
        min,
        max
    )
)]
pub fn wizard_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    implement_wizard(&input)
}

fn implement_wizard(input: &syn::DeriveInput) -> TokenStream {
    let name = &input.ident;
    let interview = build_interview(input);
    let interview_code = generate_interview_code(&interview);

    let from_answers_code = match &input.data {
        Data::Struct(data) => generate_from_answers_struct(name, data),
        Data::Enum(data) => generate_from_answers_enum(name, data),
        Data::Union(_) => unimplemented!(),
    };

    let interview_with_suggestions_code = match &input.data {
        Data::Struct(data) => generate_interview_with_suggestions_struct(data, &interview),
        Data::Enum(_data) => {
            // For enums, we can't easily provide suggestions, so just return the base interview
            quote! { Self::interview() }
        }
        Data::Union(_) => unimplemented!(),
    };

    TokenStream::from(quote! {
        impl Wizard for #name {
            fn interview() -> derive_wizard::interview::Interview {
                #interview_code
            }

            fn interview_with_suggestions(&self) -> derive_wizard::interview::Interview {
                #interview_with_suggestions_code
            }

            fn from_answers(answers: &derive_wizard::Answers) -> Result<Self, derive_wizard::backend::BackendError> {
                #from_answers_code
            }
        }
    })
}

fn build_interview(input: &syn::DeriveInput) -> Interview {
    let sections = match &input.data {
        Data::Struct(data) => {
            if let Fields::Named(fields) = &data.fields {
                fields
                    .named
                    .iter()
                    .map(|f| build_question(f, None))
                    .collect()
            } else {
                vec![]
            }
        }
        Data::Enum(data) => {
            let alternatives = data
                .variants
                .iter()
                .map(|variant| {
                    let questions = match &variant.fields {
                        Fields::Unit => vec![],
                        Fields::Unnamed(fields) => fields
                            .unnamed
                            .iter()
                            .enumerate()
                            .map(|(i, f)| build_question(f, Some(i)))
                            .collect(),
                        Fields::Named(fields) => fields
                            .named
                            .iter()
                            .map(|f| build_question(f, None))
                            .collect(),
                    };
                    Question::new(
                        Some(variant.ident.to_string()),
                        variant.ident.to_string(),
                        variant.ident.to_string(),
                        QuestionKind::Alternative(0, questions),
                    )
                })
                .collect();
            vec![Question::new(
                Some("alternatives".to_string()),
                "alternatives".to_string(),
                "Select variant:".to_string(),
                QuestionKind::Sequence(alternatives),
            )]
        }
        Data::Union(_) => vec![],
    };

    Interview { sections }
}

fn build_question(field: &syn::Field, idx: Option<usize>) -> Question {
    let field_name = idx
        .map(|i| format!("field_{i}"))
        .or_else(|| field.ident.as_ref().map(Ident::to_string))
        .unwrap();

    let attrs = FieldAttrs::extract(&field.attrs, &field_name);
    let kind = determine_question_kind(&field.ty, &attrs);

    Question::new(Some(field_name.clone()), field_name, attrs.prompt, kind)
}

struct FieldAttrs {
    prompt: String,
    mask: bool,
    editor: bool,
    validate_on_key: Option<String>,
    validate_on_submit: Option<String>,
    min_int: Option<i64>,
    max_int: Option<i64>,
    min_float: Option<f64>,
    max_float: Option<f64>,
}

impl FieldAttrs {
    fn extract(attrs: &[syn::Attribute], field_name: &str) -> Self {
        let validate = extract_string_attr(attrs, "validate");
        Self {
            prompt: extract_string_attr(attrs, "prompt")
                .unwrap_or_else(|| format!("Enter {field_name}:")),
            mask: has_attr(attrs, "mask"),
            editor: has_attr(attrs, "editor"),
            validate_on_key: extract_string_attr(attrs, "validate_on_key").or(validate.clone()),
            validate_on_submit: extract_string_attr(attrs, "validate_on_submit").or(validate),
            min_int: extract_int_attr(attrs, "min"),
            max_int: extract_int_attr(attrs, "max"),
            min_float: extract_float_attr(attrs, "min"),
            max_float: extract_float_attr(attrs, "max"),
        }
    }
}

fn determine_question_kind(ty: &Type, attrs: &FieldAttrs) -> QuestionKind {
    if attrs.mask {
        return QuestionKind::Masked(MaskedQuestion {
            mask: Some('*'),
            validate_on_key: attrs.validate_on_key.clone(),
            validate_on_submit: attrs.validate_on_submit.clone(),
        });
    }

    if attrs.editor {
        return QuestionKind::Multiline(MultilineQuestion {
            default: None,
            validate_on_key: attrs.validate_on_key.clone(),
            validate_on_submit: attrs.validate_on_submit.clone(),
        });
    }

    match quote!(#ty).to_string().as_str() {
        "String" => QuestionKind::Input(InputQuestion {
            default: None,
            validate_on_key: attrs.validate_on_key.clone(),
            validate_on_submit: attrs.validate_on_submit.clone(),
        }),
        "bool" => QuestionKind::Confirm(ConfirmQuestion { default: false }),
        "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32" | "u64" | "u128"
        | "usize" => QuestionKind::Int(IntQuestion {
            default: None,
            min: attrs.min_int,
            max: attrs.max_int,
            validate_on_key: attrs.validate_on_key.clone(),
            validate_on_submit: attrs.validate_on_submit.clone(),
        }),
        "f32" | "f64" => QuestionKind::Float(FloatQuestion {
            default: None,
            min: attrs.min_float,
            max: attrs.max_float,
            validate_on_key: attrs.validate_on_key.clone(),
            validate_on_submit: attrs.validate_on_submit.clone(),
        }),
        "PathBuf" => QuestionKind::Input(InputQuestion {
            default: None,
            validate_on_key: attrs.validate_on_key.clone(),
            validate_on_submit: attrs.validate_on_submit.clone(),
        }),
        _type_path => {
            // For nested types, we'll need to handle them at the interview generation level
            // For now, default to Input for unknown types
            QuestionKind::Input(InputQuestion {
                default: None,
                validate_on_key: attrs.validate_on_key.clone(),
                validate_on_submit: attrs.validate_on_submit.clone(),
            })
        }
    }
}

fn extract_string_attr(attrs: &[syn::Attribute], name: &str) -> Option<String> {
    attrs.iter().find_map(|attr| {
        if !attr.path().is_ident(name) {
            return None;
        }

        match &attr.meta {
            Meta::List(list) => syn::parse2::<Lit>(list.tokens.clone())
                .ok()
                .and_then(|lit| {
                    if let Lit::Str(s) = lit {
                        Some(s.value())
                    } else {
                        None
                    }
                }),
            Meta::NameValue(nv) => {
                if let syn::Expr::Lit(expr) = &nv.value {
                    if let Lit::Str(s) = &expr.lit {
                        Some(s.value())
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            Meta::Path(_) => None,
        }
    })
}

fn has_attr(attrs: &[syn::Attribute], name: &str) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident(name))
}

fn extract_int_attr(attrs: &[syn::Attribute], name: &str) -> Option<i64> {
    attrs.iter().find_map(|attr| {
        if !attr.path().is_ident(name) {
            return None;
        }

        let parse_lit = |lit: &Lit| match lit {
            Lit::Int(i) => i.base10_parse().ok(),
            _ => None,
        };

        match &attr.meta {
            Meta::List(list) => syn::parse2::<Lit>(list.tokens.clone())
                .ok()
                .and_then(|lit| parse_lit(&lit)),
            Meta::NameValue(nv) => {
                if let syn::Expr::Lit(expr) = &nv.value {
                    parse_lit(&expr.lit)
                } else {
                    None
                }
            }
            Meta::Path(_) => None,
        }
    })
}

fn extract_float_attr(attrs: &[syn::Attribute], name: &str) -> Option<f64> {
    attrs.iter().find_map(|attr| {
        if !attr.path().is_ident(name) {
            return None;
        }

        let parse_lit = |lit: &Lit| match lit {
            Lit::Float(f) => f.base10_parse().ok(),
            Lit::Int(i) => i.base10_parse::<i64>().ok().map(|v| v as f64),
            _ => None,
        };

        match &attr.meta {
            Meta::List(list) => syn::parse2::<Lit>(list.tokens.clone())
                .ok()
                .and_then(|lit| parse_lit(&lit)),
            Meta::NameValue(nv) => {
                if let syn::Expr::Lit(expr) = &nv.value {
                    parse_lit(&expr.lit)
                } else {
                    None
                }
            }
            Meta::Path(_) => None,
        }
    })
}

fn generate_interview_code(interview: &Interview) -> proc_macro2::TokenStream {
    let sections = interview.sections.iter().map(generate_question_code);
    quote! {
        derive_wizard::interview::Interview {
            sections: vec![#(#sections),*],
        }
    }
}

fn generate_question_code(question: &Question) -> proc_macro2::TokenStream {
    generate_question_code_impl(question, None)
}

fn generate_question_code_impl(
    question: &Question,
    default_value: Option<proc_macro2::TokenStream>,
) -> proc_macro2::TokenStream {
    let id = question
        .id()
        .map_or_else(|| quote!(None), |id| quote! { Some(#id.to_string()) });
    let name = question.name();
    let prompt = question.prompt();
    let kind = generate_question_kind_code_impl(question.kind(), default_value);

    quote! {
        derive_wizard::interview::Question::new(#id, #name.to_string(), #prompt.to_string(), #kind)
    }
}

fn generate_question_kind_code_impl(
    kind: &QuestionKind,
    default_value: Option<proc_macro2::TokenStream>,
) -> proc_macro2::TokenStream {
    macro_rules! opt_str {
        ($opt:expr) => {
            match $opt {
                Some(v) => quote! { Some(#v.to_string()) },
                None => quote! { None },
            }
        };
    }

    match kind {
        QuestionKind::Input(q) => {
            let default =
                default_value.map_or_else(|| opt_str!(&q.default), |v| quote! { Some(#v) });
            let validate_on_key = opt_str!(&q.validate_on_key);
            let validate_on_submit = opt_str!(&q.validate_on_submit);
            quote! {
                derive_wizard::interview::QuestionKind::Input(derive_wizard::interview::InputQuestion {
                    default: #default,
                    validate_on_key: #validate_on_key,
                    validate_on_submit: #validate_on_submit,
                })
            }
        }
        QuestionKind::Multiline(q) => {
            let default =
                default_value.map_or_else(|| opt_str!(&q.default), |v| quote! { Some(#v) });
            let validate_on_key = opt_str!(&q.validate_on_key);
            let validate_on_submit = opt_str!(&q.validate_on_submit);
            quote! {
                derive_wizard::interview::QuestionKind::Multiline(derive_wizard::interview::MultilineQuestion {
                    default: #default,
                    validate_on_key: #validate_on_key,
                    validate_on_submit: #validate_on_submit,
                })
            }
        }
        QuestionKind::Masked(q) => {
            let mask = q.mask.map_or_else(|| quote!(None), |v| quote! { Some(#v) });
            let validate_on_key = opt_str!(&q.validate_on_key);
            let validate_on_submit = opt_str!(&q.validate_on_submit);
            quote! {
                derive_wizard::interview::QuestionKind::Masked(derive_wizard::interview::MaskedQuestion {
                    mask: #mask,
                    validate_on_key: #validate_on_key,
                    validate_on_submit: #validate_on_submit,
                })
            }
        }
        QuestionKind::Int(q) => {
            let default = default_value.map_or_else(
                || {
                    q.default
                        .map_or_else(|| quote!(None), |v| quote! { Some(#v) })
                },
                |v| quote! { Some(#v as i64) },
            );
            let min = q.min.map_or_else(|| quote!(None), |v| quote! { Some(#v) });
            let max = q.max.map_or_else(|| quote!(None), |v| quote! { Some(#v) });
            let validate_on_key = opt_str!(&q.validate_on_key);
            let validate_on_submit = opt_str!(&q.validate_on_submit);
            quote! {
                derive_wizard::interview::QuestionKind::Int(derive_wizard::interview::IntQuestion {
                    default: #default,
                    min: #min,
                    max: #max,
                    validate_on_key: #validate_on_key,
                    validate_on_submit: #validate_on_submit,
                })
            }
        }
        QuestionKind::Float(q) => {
            let default = default_value.map_or_else(
                || {
                    q.default
                        .map_or_else(|| quote!(None), |v| quote! { Some(#v) })
                },
                |v| quote! { Some(#v as f64) },
            );
            let min = q.min.map_or_else(|| quote!(None), |v| quote! { Some(#v) });
            let max = q.max.map_or_else(|| quote!(None), |v| quote! { Some(#v) });
            let validate_on_key = opt_str!(&q.validate_on_key);
            let validate_on_submit = opt_str!(&q.validate_on_submit);
            quote! {
                derive_wizard::interview::QuestionKind::Float(derive_wizard::interview::FloatQuestion {
                    default: #default,
                    min: #min,
                    max: #max,
                    validate_on_key: #validate_on_key,
                    validate_on_submit: #validate_on_submit,
                })
            }
        }
        QuestionKind::Confirm(q) => {
            let default = default_value.unwrap_or_else(|| {
                let d = q.default;
                quote! { #d }
            });
            quote! {
                derive_wizard::interview::QuestionKind::Confirm(derive_wizard::interview::ConfirmQuestion {
                    default: #default,
                })
            }
        }
        QuestionKind::Sequence(questions) => {
            let question_codes = questions.iter().map(generate_question_code);
            quote! {
                derive_wizard::interview::QuestionKind::Sequence(vec![#(#question_codes),*])
            }
        }
        QuestionKind::Alternative(idx, questions) => {
            let question_codes = questions.iter().map(generate_question_code);
            quote! {
                derive_wizard::interview::QuestionKind::Alternative(#idx, vec![#(#question_codes),*])
            }
        }
    }
}

fn generate_from_answers_struct(
    name: &syn::Ident,
    data: &syn::DataStruct,
) -> proc_macro2::TokenStream {
    let Fields::Named(fields) = &data.fields else {
        return quote! { unimplemented!("from_answers for non-named struct fields") };
    };

    let field_assignments = fields.named.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        let field_name_str = field_name.to_string();
        let extraction = generate_answer_extraction(&field.ty, &field_name_str);
        quote! { #field_name: #extraction }
    });

    quote! {
        Ok(#name { #(#field_assignments),* })
    }
}

fn generate_from_answers_enum(name: &syn::Ident, data: &syn::DataEnum) -> proc_macro2::TokenStream {
    let match_arms = data.variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let variant_str = variant_name.to_string();

        match &variant.fields {
            Fields::Unit => quote! {
                #variant_str => Ok(#name::#variant_name),
            },
            Fields::Unnamed(fields) => {
                let constructions = fields.unnamed.iter().enumerate().map(|(i, field)| {
                    let field_name = format!("field_{i}");
                    generate_answer_extraction(&field.ty, &field_name)
                });
                quote! {
                    #variant_str => Ok(#name::#variant_name(#(#constructions),*)),
                }
            }
            Fields::Named(fields) => {
                let constructions = fields.named.iter().map(|field| {
                    let field_name = field.ident.as_ref().unwrap();
                    let field_str = field_name.to_string();
                    let extraction = generate_answer_extraction(&field.ty, &field_str);
                    quote! { #field_name: #extraction }
                });
                quote! {
                    #variant_str => Ok(#name::#variant_name { #(#constructions),* }),
                }
            }
        }
    });

    quote! {
        let selected = answers.as_string("selected_alternative")?;
        match selected.as_str() {
            #(#match_arms)*
            _ => Err(derive_wizard::backend::BackendError::ExecutionError(
                format!("Unknown variant: {}", selected)
            ))
        }
    }
}

fn generate_answer_extraction(ty: &Type, field_name: &str) -> proc_macro2::TokenStream {
    match quote!(#ty).to_string().as_str() {
        "String" => quote! { answers.as_string(#field_name)? },
        "bool" => quote! { answers.as_bool(#field_name)? },
        "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32" | "u64" | "u128"
        | "usize" => {
            quote! { answers.as_int(#field_name)? as #ty }
        }
        "f32" | "f64" => quote! { answers.as_float(#field_name)? as #ty },
        "PathBuf" => quote! { std::path::PathBuf::from(answers.as_string(#field_name)?) },
        type_str => {
            let type_ident = syn::parse_str::<syn::Ident>(type_str).unwrap();
            quote! { #type_ident::from_answers(answers)? }
        }
    }
}

fn generate_interview_with_suggestions_struct(
    data: &syn::DataStruct,
    base_interview: &Interview,
) -> proc_macro2::TokenStream {
    let Fields::Named(fields) = &data.fields else {
        return quote! { Self::interview() };
    };

    let suggestion_setters: Vec<_> = fields
        .named
        .iter()
        .enumerate()
        .filter_map(|(i, field)| {
            let field_name = field.ident.as_ref().unwrap();
            let field_type = &field.ty;
            let question = &base_interview.sections[i];

            // Generate the suggested value based on field type
            match question.kind() {
                QuestionKind::Input(_) => match quote!(#field_type).to_string().as_str() {
                    "String" => Some(quote! {
                        interview.sections[#i].set_suggestion(self.#field_name.clone());
                    }),
                    "PathBuf" => Some(quote! {
                        interview.sections[#i].set_suggestion(self.#field_name.display().to_string());
                    }),
                    _ => None,
                },
                QuestionKind::Multiline(_) => Some(quote! {
                    interview.sections[#i].set_suggestion(self.#field_name.clone());
                }),
                QuestionKind::Int(_) => Some(quote! {
                    interview.sections[#i].set_suggestion(self.#field_name as i64);
                }),
                QuestionKind::Float(_) => Some(quote! {
                    interview.sections[#i].set_suggestion(self.#field_name as f64);
                }),
                QuestionKind::Confirm(_) => Some(quote! {
                    interview.sections[#i].set_suggestion(self.#field_name);
                }),
                _ => None,
            }
        })
        .collect();

    quote! {{
        let mut interview = Self::interview();
        #(#suggestion_setters)*
        interview
    }}
}
