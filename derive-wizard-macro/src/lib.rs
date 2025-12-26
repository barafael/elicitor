use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, Fields, Lit, Meta, Type, parse_macro_input};

use derive_wizard_types::interview::{Alternative, Interview, Section, Sequence};
use derive_wizard_types::question::{
    ConfirmQuestion, FloatQuestion, InputQuestion, IntQuestion, MaskedQuestion, MultilineQuestion,
    NestedQuestion, Question, QuestionKind,
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
        _ => unimplemented!(),
    };

    TokenStream::from(quote! {
        impl Wizard for #name {
            fn interview() -> derive_wizard::interview::Interview {
                #interview_code
            }

            fn from_answers(answers: &derive_wizard::backend::Answers) -> Result<Self, derive_wizard::backend::BackendError> {
                #from_answers_code
            }
        }
    })
}

fn build_interview(input: &syn::DeriveInput) -> Interview {
    let sections = match &input.data {
        Data::Struct(data) => {
            if let Fields::Named(fields) = &data.fields {
                let questions = fields
                    .named
                    .iter()
                    .map(|f| build_question(f, None))
                    .collect();
                vec![Section::Sequence(Sequence {
                    sequence: questions,
                })]
            } else {
                vec![]
            }
        }
        Data::Enum(data) => {
            let alternatives = data
                .variants
                .iter()
                .map(|variant| {
                    let section = match &variant.fields {
                        Fields::Unit => Section::Empty,
                        Fields::Unnamed(fields) => {
                            let questions = fields
                                .unnamed
                                .iter()
                                .enumerate()
                                .map(|(i, f)| build_question(f, Some(i)))
                                .collect();
                            Section::Sequence(Sequence {
                                sequence: questions,
                            })
                        }
                        Fields::Named(fields) => {
                            let questions = fields
                                .named
                                .iter()
                                .map(|f| build_question(f, None))
                                .collect();
                            Section::Sequence(Sequence {
                                sequence: questions,
                            })
                        }
                    };
                    Alternative {
                        name: variant.ident.to_string(),
                        section,
                    }
                })
                .collect();
            vec![Section::Alternatives(0, alternatives)]
        }
        _ => vec![],
    };

    Interview { sections }
}

fn build_question(field: &syn::Field, idx: Option<usize>) -> Question {
    let field_name = idx
        .map(|i| format!("field_{}", i))
        .or_else(|| field.ident.as_ref().map(|i| i.to_string()))
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
                .unwrap_or_else(|| format!("Enter {}:", field_name)),
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
        type_path => QuestionKind::Nested(NestedQuestion {
            type_path: type_path.to_string(),
        }),
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
            _ => None,
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
            _ => None,
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
            _ => None,
        }
    })
}

fn generate_interview_code(interview: &Interview) -> proc_macro2::TokenStream {
    let has_nested = interview.sections.iter().any(|section| {
        matches!(section, Section::Sequence(seq) if seq.sequence.iter()
            .any(|q| matches!(q.kind(), QuestionKind::Nested(_))))
    });

    if !has_nested {
        let sections = interview.sections.iter().map(generate_section_code);
        return quote! {
            derive_wizard::interview::Interview {
                sections: vec![#(#sections),*],
            }
        };
    }

    // Handle nested types dynamically
    let mut builders = Vec::new();
    for section in &interview.sections {
        if let Section::Sequence(seq) = section {
            let mut batch = Vec::new();

            for question in &seq.sequence {
                if let QuestionKind::Nested(nested) = question.kind() {
                    if !batch.is_empty() {
                        let questions = batch.iter().map(generate_question_code);
                        builders.push(quote! {
                            sections.push(derive_wizard::interview::Section::Sequence(
                                derive_wizard::interview::Sequence { sequence: vec![#(#questions),*] }
                            ));
                        });
                        batch.clear();
                    }
                    let type_ident = syn::parse_str::<syn::Ident>(&nested.type_path).unwrap();
                    builders.push(quote! {
                        sections.extend(#type_ident::interview().sections);
                    });
                } else {
                    batch.push(question.clone());
                }
            }

            if !batch.is_empty() {
                let questions = batch.iter().map(generate_question_code);
                builders.push(quote! {
                    sections.push(derive_wizard::interview::Section::Sequence(
                        derive_wizard::interview::Sequence { sequence: vec![#(#questions),*] }
                    ));
                });
            }
        } else {
            let section_code = generate_section_code(section);
            builders.push(quote! { sections.push(#section_code); });
        }
    }

    quote! {{
        let mut sections = Vec::new();
        #(#builders)*
        derive_wizard::interview::Interview { sections }
    }}
}

fn generate_section_code(section: &Section) -> proc_macro2::TokenStream {
    match section {
        Section::Empty => quote! { derive_wizard::interview::Section::Empty },
        Section::Sequence(seq) => {
            let questions = seq.sequence.iter().map(generate_question_code);
            quote! {
                derive_wizard::interview::Section::Sequence(
                    derive_wizard::interview::Sequence { sequence: vec![#(#questions),*] }
                )
            }
        }
        Section::Alternatives(idx, alts) => {
            let alternatives = alts.iter().map(|alt| {
                let name = &alt.name;
                let section = generate_section_code(&alt.section);
                quote! {
                    derive_wizard::interview::Alternative {
                        name: #name.to_string(),
                        section: #section,
                    }
                }
            });
            quote! {
                derive_wizard::interview::Section::Alternatives(#idx, vec![#(#alternatives),*])
            }
        }
    }
}

fn generate_question_code(question: &Question) -> proc_macro2::TokenStream {
    let id = question
        .id()
        .map(|id| quote! { Some(#id.to_string()) })
        .unwrap_or_else(|| quote! { None });
    let name = question.name();
    let prompt = question.prompt();
    let kind = generate_question_kind_code(question.kind());

    quote! {
        derive_wizard::question::Question::new(#id, #name.to_string(), #prompt.to_string(), #kind)
    }
}

fn generate_question_kind_code(kind: &QuestionKind) -> proc_macro2::TokenStream {
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
            let default = opt_str!(&q.default);
            let validate_on_key = opt_str!(&q.validate_on_key);
            let validate_on_submit = opt_str!(&q.validate_on_submit);
            quote! {
                derive_wizard::question::QuestionKind::Input(derive_wizard::question::InputQuestion {
                    default: #default,
                    validate_on_key: #validate_on_key,
                    validate_on_submit: #validate_on_submit,
                })
            }
        }
        QuestionKind::Multiline(q) => {
            let default = opt_str!(&q.default);
            let validate_on_submit = opt_str!(&q.validate_on_submit);
            quote! {
                derive_wizard::question::QuestionKind::Multiline(derive_wizard::question::MultilineQuestion {
                    default: #default,
                    validate_on_submit: #validate_on_submit,
                })
            }
        }
        QuestionKind::Masked(q) => {
            let mask = q
                .mask
                .map(|v| quote! { Some(#v) })
                .unwrap_or_else(|| quote! { None });
            let validate_on_key = opt_str!(&q.validate_on_key);
            let validate_on_submit = opt_str!(&q.validate_on_submit);
            quote! {
                derive_wizard::question::QuestionKind::Masked(derive_wizard::question::MaskedQuestion {
                    mask: #mask,
                    validate_on_key: #validate_on_key,
                    validate_on_submit: #validate_on_submit,
                })
            }
        }
        QuestionKind::Int(q) => {
            let default = q
                .default
                .map(|v| quote! { Some(#v) })
                .unwrap_or_else(|| quote! { None });
            let min = q
                .min
                .map(|v| quote! { Some(#v) })
                .unwrap_or_else(|| quote! { None });
            let max = q
                .max
                .map(|v| quote! { Some(#v) })
                .unwrap_or_else(|| quote! { None });
            let validate_on_key = opt_str!(&q.validate_on_key);
            let validate_on_submit = opt_str!(&q.validate_on_submit);
            quote! {
                derive_wizard::question::QuestionKind::Int(derive_wizard::question::IntQuestion {
                    default: #default,
                    min: #min,
                    max: #max,
                    validate_on_key: #validate_on_key,
                    validate_on_submit: #validate_on_submit,
                })
            }
        }
        QuestionKind::Float(q) => {
            let default = q
                .default
                .map(|v| quote! { Some(#v) })
                .unwrap_or_else(|| quote! { None });
            let min = q
                .min
                .map(|v| quote! { Some(#v) })
                .unwrap_or_else(|| quote! { None });
            let max = q
                .max
                .map(|v| quote! { Some(#v) })
                .unwrap_or_else(|| quote! { None });
            let validate_on_key = opt_str!(&q.validate_on_key);
            let validate_on_submit = opt_str!(&q.validate_on_submit);
            quote! {
                derive_wizard::question::QuestionKind::Float(derive_wizard::question::FloatQuestion {
                    default: #default,
                    min: #min,
                    max: #max,
                    validate_on_key: #validate_on_key,
                    validate_on_submit: #validate_on_submit,
                })
            }
        }
        QuestionKind::Confirm(q) => {
            let default = q.default;
            quote! {
                derive_wizard::question::QuestionKind::Confirm(derive_wizard::question::ConfirmQuestion {
                    default: #default,
                })
            }
        }
        QuestionKind::Nested(q) => {
            let type_path = &q.type_path;
            quote! {
                derive_wizard::question::QuestionKind::Nested(derive_wizard::question::NestedQuestion {
                    type_path: #type_path.to_string(),
                })
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
                    let field_name = format!("field_{}", i);
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
