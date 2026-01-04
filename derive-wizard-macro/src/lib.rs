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
    attributes(prompt, mask, multiline, validate, min, max, prelude, epilogue)
)]
pub fn wizard_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input);
    implement_wizard(&input)
}

fn implement_wizard(input: &syn::DeriveInput) -> TokenStream {
    let name = &input.ident;

    // Check for non-builtin fields without #[prompt] and emit compile errors
    let errors = collect_missing_prompt_errors(&input.data);
    if !errors.is_empty() {
        return TokenStream::from(errors);
    }

    let interview = build_interview(input);
    let interview_code = generate_interview_code(&interview, &input.data);

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

    let validate_field_code = match &input.data {
        Data::Struct(data) => generate_validate_field_method_struct(data),
        Data::Enum(data) => generate_validate_field_method_enum(data),
        Data::Union(_) => unimplemented!(),
    };

    // Generate compile-time checks for validator functions
    let validator_checks = generate_validator_checks(&input.data);

    TokenStream::from(quote! {
        #[allow(dead_code)]
        const _: fn(&derive_wizard::Answers) = |answers: &derive_wizard::Answers| {
            let _ = answers.iter();
        };

        #validator_checks

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

            #validate_field_code
        }
    })
}

/// Check for non-builtin type fields that are missing the #[prompt] attribute.
/// These fields would silently fail at runtime, so we catch them at compile time.
fn collect_missing_prompt_errors(data: &Data) -> proc_macro2::TokenStream {
    let mut errors = proc_macro2::TokenStream::new();

    let check_field = |field: &syn::Field, errors: &mut proc_macro2::TokenStream| {
        let field_name = field
            .ident
            .as_ref()
            .map(|i| i.to_string())
            .unwrap_or_else(|| "unnamed".to_string());
        let attrs = FieldAttrs::extract(&field.attrs, &field_name);
        let field_ty = &field.ty;

        // If it's a non-builtin type without #[prompt], it's likely a mistake
        if !is_builtin_type(field_ty)
            && !attrs.has_explicit_prompt
            && !attrs.mask
            && !attrs.multiline
        {
            let type_name = type_ident_name(field_ty).unwrap_or_else(|| "unknown".to_string());
            let msg = format!(
                "field `{}` has non-builtin type `{}` but is missing #[prompt]. \
                Add #[prompt(\"...\")] to treat it as a nested Wizard, \
                or use #[mask] / #[multiline] if it should be a string input.",
                field_name, type_name
            );
            let span = field
                .ident
                .as_ref()
                .map(|i| i.span())
                .unwrap_or_else(proc_macro2::Span::call_site);
            errors.extend(quote::quote_spanned! { span =>
                compile_error!(#msg);
            });
        }
    };

    match data {
        Data::Struct(data) => {
            if let Fields::Named(fields) = &data.fields {
                for field in &fields.named {
                    check_field(field, &mut errors);
                }
            }
        }
        Data::Enum(data) => {
            for variant in &data.variants {
                match &variant.fields {
                    Fields::Named(fields) => {
                        for field in &fields.named {
                            check_field(field, &mut errors);
                        }
                    }
                    Fields::Unnamed(fields) => {
                        for field in &fields.unnamed {
                            check_field(field, &mut errors);
                        }
                    }
                    Fields::Unit => {}
                }
            }
        }
        Data::Union(_) => {}
    }

    errors
}

/// Generate compile-time checks that verify validator functions exist and have the correct signature.
/// This catches typos in #[validate("func_name")] at compile time rather than runtime.
fn generate_validator_checks(data: &Data) -> proc_macro2::TokenStream {
    let mut checks = Vec::new();

    let collect_from_field = |field: &syn::Field, checks: &mut Vec<proc_macro2::TokenStream>| {
        let field_name = field
            .ident
            .as_ref()
            .map(|i| i.to_string())
            .unwrap_or_else(|| "unnamed".to_string());
        let attrs = FieldAttrs::extract(&field.attrs, &field_name);

        if let Some(validator_ident) = attrs.validate_ident {
            // Generate a const assertion that the validator function has the correct signature
            checks.push(quote! {
                const _: fn(&str, &derive_wizard::Answers) -> Result<(), String> = #validator_ident;
            });
        }
    };

    match data {
        Data::Struct(data) => {
            if let Fields::Named(fields) = &data.fields {
                for field in &fields.named {
                    collect_from_field(field, &mut checks);
                }
            }
        }
        Data::Enum(data) => {
            for variant in &data.variants {
                match &variant.fields {
                    Fields::Named(fields) => {
                        for field in &fields.named {
                            collect_from_field(field, &mut checks);
                        }
                    }
                    Fields::Unnamed(fields) => {
                        for field in &fields.unnamed {
                            collect_from_field(field, &mut checks);
                        }
                    }
                    Fields::Unit => {}
                }
            }
        }
        Data::Union(_) => {}
    }

    quote! {
        #(#checks)*
    }
}

fn build_interview(input: &syn::DeriveInput) -> Interview {
    let prelude = extract_string_attr(&input.attrs, "prelude");
    let epilogue = extract_string_attr(&input.attrs, "epilogue");

    let sections = match &input.data {
        Data::Struct(data) => {
            if let Fields::Named(fields) = &data.fields {
                fields
                    .named
                    .iter()
                    .flat_map(|f| build_question(f, None, None))
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
                            .flat_map(|(i, f)| build_question(f, Some(i), None))
                            .collect(),
                        Fields::Named(fields) => fields
                            .named
                            .iter()
                            .flat_map(|f| build_question(f, None, None))
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

    Interview {
        sections,
        prelude,
        epilogue,
    }
}

fn build_question(
    field: &syn::Field,
    idx: Option<usize>,
    parent_prefix: Option<&str>,
) -> Vec<Question> {
    let field_name = idx
        .map(|i| format!("field_{i}"))
        .or_else(|| field.ident.as_ref().map(Ident::to_string))
        .unwrap();

    let attrs = FieldAttrs::extract(&field.attrs, &field_name);
    let kind = determine_question_kind(&field.ty, &attrs);

    // Apply parent prefix if present
    let prefixed_name = if let Some(prefix) = parent_prefix {
        format!("{}.{}", prefix, field_name)
    } else {
        field_name.clone()
    };

    // Detect nested Wizard types: non-builtin type with explicit #[prompt] attribute
    let field_ty = &field.ty;
    let is_nested =
        attrs.has_explicit_prompt && !is_builtin_type(field_ty) && !attrs.mask && !attrs.multiline;

    if is_nested {
        // Nested Wizard type - will be expanded at runtime
        vec![Question::new(
            Some(prefixed_name.clone()),
            prefixed_name,
            attrs.prompt,
            QuestionKind::Sequence(vec![]), // Empty sequence, will be populated at runtime
        )]
    } else {
        vec![Question::new(
            Some(prefixed_name.clone()),
            prefixed_name,
            attrs.prompt,
            kind,
        )]
    }
}

fn type_ident_name(ty: &Type) -> Option<String> {
    match ty {
        Type::Path(tp) if tp.qself.is_none() => tp
            .path
            .segments
            .last()
            .map(|segment| segment.ident.to_string()),
        _ => None,
    }
}

fn is_integer_type(ty: &Type) -> bool {
    matches!(
        type_ident_name(ty).as_deref(),
        Some(
            "i8" | "i16"
                | "i32"
                | "i64"
                | "i128"
                | "isize"
                | "u8"
                | "u16"
                | "u32"
                | "u64"
                | "u128"
                | "usize"
        )
    )
}

fn is_float_type(ty: &Type) -> bool {
    matches!(type_ident_name(ty).as_deref(), Some("f32" | "f64"))
}

fn is_builtin_type(ty: &Type) -> bool {
    matches!(
        type_ident_name(ty).as_deref(),
        Some(
            "String"
                | "bool"
                | "PathBuf"
                | "i8"
                | "i16"
                | "i32"
                | "i64"
                | "i128"
                | "isize"
                | "u8"
                | "u16"
                | "u32"
                | "u64"
                | "u128"
                | "usize"
                | "f32"
                | "f64"
        )
    )
}

struct FieldAttrs {
    prompt: String,
    has_explicit_prompt: bool,
    mask: bool,
    multiline: bool,
    validate: Option<String>,
    validate_ident: Option<proc_macro2::Ident>,
    min_int: Option<i64>,
    max_int: Option<i64>,
    min_float: Option<f64>,
    max_float: Option<f64>,
}

impl FieldAttrs {
    fn extract(attrs: &[syn::Attribute], field_name: &str) -> Self {
        let validate_str = extract_string_attr(attrs, "validate");
        let validate_ident = extract_validator_ident(attrs, "validate");
        let explicit_prompt = extract_string_attr(attrs, "prompt");
        Self {
            has_explicit_prompt: explicit_prompt.is_some(),
            prompt: explicit_prompt.unwrap_or_else(|| format!("Enter {field_name}:")),
            mask: has_attr(attrs, "mask"),
            multiline: has_attr(attrs, "multiline"),
            validate: validate_str,
            validate_ident,
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
            validate: attrs.validate.clone(),
        });
    }

    if attrs.multiline {
        return QuestionKind::Multiline(MultilineQuestion {
            default: None,
            validate: attrs.validate.clone(),
        });
    }

    match type_ident_name(ty).as_deref() {
        Some("String") => QuestionKind::Input(InputQuestion {
            default: None,
            validate: attrs.validate.clone(),
        }),
        Some("bool") => QuestionKind::Confirm(ConfirmQuestion { default: false }),
        Some(_) if is_integer_type(ty) => QuestionKind::Int(IntQuestion {
            default: None,
            min: attrs.min_int,
            max: attrs.max_int,
            validate: attrs.validate.clone(),
        }),
        Some(_) if is_float_type(ty) => QuestionKind::Float(FloatQuestion {
            default: None,
            min: attrs.min_float,
            max: attrs.max_float,
            validate: attrs.validate.clone(),
        }),
        Some("PathBuf") => QuestionKind::Input(InputQuestion {
            default: None,
            validate: attrs.validate.clone(),
        }),
        _ => QuestionKind::Input(InputQuestion {
            default: None,
            validate: attrs.validate.clone(),
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
            Meta::Path(_) => None,
        }
    })
}

fn extract_validator_ident(attrs: &[syn::Attribute], name: &str) -> Option<proc_macro2::Ident> {
    attrs.iter().find_map(|attr| {
        if !attr.path().is_ident(name) {
            return None;
        }

        match &attr.meta {
            Meta::List(list) => {
                // Parse the string literal and extract the function name
                syn::parse2::<Lit>(list.tokens.clone())
                    .ok()
                    .and_then(|lit| {
                        if let Lit::Str(s) = lit {
                            let func_name = s.value();
                            Some(syn::Ident::new(&func_name, s.span()))
                        } else {
                            None
                        }
                    })
            }
            Meta::NameValue(nv) => {
                if let syn::Expr::Lit(expr) = &nv.value {
                    if let Lit::Str(s) = &expr.lit {
                        let func_name = s.value();
                        Some(syn::Ident::new(&func_name, s.span()))
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

fn generate_interview_code(interview: &Interview, data: &Data) -> proc_macro2::TokenStream {
    let prelude = interview
        .prelude
        .as_ref()
        .map(|s| quote! { Some(#s.to_string()) })
        .unwrap_or_else(|| quote! { None });
    let epilogue = interview
        .epilogue
        .as_ref()
        .map(|s| quote! { Some(#s.to_string()) })
        .unwrap_or_else(|| quote! { None });

    // Generate runtime code that builds the interview, populating nested Wizard sequences
    let section_builders: Vec<_> = if let Data::Struct(struct_data) = data {
        if let Fields::Named(fields) = &struct_data.fields {
            fields.named.iter().zip(&interview.sections).map(|(field, question)| {
                let field_ty = &field.ty;
                let field_name = field.ident.as_ref().unwrap().to_string();

                // Check if this is a Sequence (nested Wizard marker)
                if matches!(question.kind(), QuestionKind::Sequence(seq) if seq.is_empty()) {
                    // Generate code to call FieldType::interview() and prefix the nested questions
                    quote! {
                        {
                            let mut nested_interview = <#field_ty as derive_wizard::Wizard>::interview();
                            // Prefix all nested question names
                            for question in &mut nested_interview.sections {
                                let old_name = question.name().to_string();
                                let new_name = format!("{}.{}", #field_name, old_name);
                                *question = derive_wizard::interview::Question::new(
                                    Some(new_name.clone()),
                                    new_name,
                                    question.prompt().to_string(),
                                    question.kind().clone(),
                                );
                            }
                            nested_interview.sections
                        }
                    }
                } else {
                    // Regular question
                    let q_code = generate_question_code(question);
                    quote! { vec![#q_code] }
                }
            }).collect()
        } else {
            interview
                .sections
                .iter()
                .map(|q| {
                    let q_code = generate_question_code(q);
                    quote! { vec![#q_code] }
                })
                .collect()
        }
    } else {
        interview
            .sections
            .iter()
            .map(|q| {
                let q_code = generate_question_code(q);
                quote! { vec![#q_code] }
            })
            .collect()
    };

    quote! {
        {
            let mut all_sections = Vec::new();
            #(all_sections.extend(#section_builders);)*
            derive_wizard::interview::Interview {
                sections: all_sections,
                prelude: #prelude,
                epilogue: #epilogue,
            }
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
            let validate = opt_str!(&q.validate);
            quote! {
                derive_wizard::interview::QuestionKind::Input(derive_wizard::interview::InputQuestion {
                    default: #default,
                    validate: #validate,
                })
            }
        }
        QuestionKind::Multiline(q) => {
            let default =
                default_value.map_or_else(|| opt_str!(&q.default), |v| quote! { Some(#v) });
            let validate = opt_str!(&q.validate);
            quote! {
                derive_wizard::interview::QuestionKind::Multiline(derive_wizard::interview::MultilineQuestion {
                    default: #default,
                    validate: #validate,
                })
            }
        }
        QuestionKind::Masked(q) => {
            let mask = q.mask.map_or_else(|| quote!(None), |v| quote! { Some(#v) });
            let validate = opt_str!(&q.validate);
            quote! {
                derive_wizard::interview::QuestionKind::Masked(derive_wizard::interview::MaskedQuestion {
                    mask: #mask,
                    validate: #validate,
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
            let validate = opt_str!(&q.validate);
            quote! {
                derive_wizard::interview::QuestionKind::Int(derive_wizard::interview::IntQuestion {
                    default: #default,
                    min: #min,
                    max: #max,
                    validate: #validate,
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
            let validate = opt_str!(&q.validate);
            quote! {
                derive_wizard::interview::QuestionKind::Float(derive_wizard::interview::FloatQuestion {
                    default: #default,
                    min: #min,
                    max: #max,
                    validate: #validate,
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
    let match_arms = data.variants.iter().enumerate().map(|(idx, variant)| {
        let variant_name = &variant.ident;
        let idx_lit = idx as i64;

        match &variant.fields {
            Fields::Unit => quote! {
                #idx_lit => Ok(#name::#variant_name),
            },
            Fields::Unnamed(fields) => {
                let constructions = fields.unnamed.iter().enumerate().map(|(i, field)| {
                    let field_name = format!("field_{i}");
                    generate_answer_extraction(&field.ty, &field_name)
                });
                quote! {
                    #idx_lit => Ok(#name::#variant_name(#(#constructions),*)),
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
                    #idx_lit => Ok(#name::#variant_name { #(#constructions),* }),
                }
            }
        }
    });

    quote! {
        let selected = answers.as_int("selected_alternative")?;
        match selected {
            #(#match_arms)*
            _ => Err(derive_wizard::backend::BackendError::ExecutionError(
                format!("Unknown variant index: {}", selected)
            ))
        }
    }
}

fn generate_answer_extraction(ty: &Type, field_name: &str) -> proc_macro2::TokenStream {
    if let Some(type_name) = type_ident_name(ty) {
        match type_name.as_str() {
            "String" => return quote! { answers.as_string(#field_name)? },
            "bool" => return quote! { answers.as_bool(#field_name)? },
            "PathBuf" => {
                return quote! { std::path::PathBuf::from(answers.as_string(#field_name)?) };
            }
            _ => {}
        }

        if is_integer_type(ty) {
            return quote! { answers.as_int(#field_name)? as #ty };
        }

        if is_float_type(ty) {
            return quote! { answers.as_float(#field_name)? as #ty };
        }
    }

    quote! {
        {
            let mut nested_answers = derive_wizard::Answers::default();
            let prefix = format!("{}.", #field_name);
            for (key, value) in answers.iter() {
                if let Some(stripped) = key.strip_prefix(&prefix) {
                    nested_answers.insert(stripped.to_string(), value.clone());
                }
            }

            // Run nested validators before constructing the nested Wizard type.
            for (nested_key, nested_value) in nested_answers.iter() {
                let nested_value_str = match nested_value {
                    derive_wizard::AnswerValue::String(s) => s.clone(),
                    derive_wizard::AnswerValue::Int(i) => i.to_string(),
                    derive_wizard::AnswerValue::Float(f) => f.to_string(),
                    derive_wizard::AnswerValue::Bool(b) => b.to_string(),
                    derive_wizard::AnswerValue::Nested(_) => continue,
                };

                <#ty as derive_wizard::Wizard>::validate_field(
                    nested_key,
                    &nested_value_str,
                    &nested_answers,
                )
                .map_err(|err| derive_wizard::backend::BackendError::ExecutionError(err))?;
            }

            <#ty as derive_wizard::Wizard>::from_answers(&nested_answers)?
        }
    }
}

fn generate_validate_field_method_struct(data: &syn::DataStruct) -> proc_macro2::TokenStream {
    let Fields::Named(fields) = &data.fields else {
        return quote! {
            fn validate_field(_field: &str, _value: &str, _answers: &derive_wizard::Answers) -> Result<(), String> {
                Ok(())
            }
        };
    };

    // Collect all fields with validators
    let mut validator_arms = Vec::new();
    // Collect nested wizard fields for delegation
    let mut nested_arms = Vec::new();

    for field in &fields.named {
        let field_name = field.ident.as_ref().unwrap().to_string();
        let attrs = FieldAttrs::extract(&field.attrs, &field_name);
        let field_ty = &field.ty;

        if let Some(ident) = attrs.validate_ident {
            validator_arms.push(quote! {
                #field_name => #ident(value, answers),
            });
        }

        // Check if this is a nested wizard type (non-builtin with explicit #[prompt])
        let is_nested = attrs.has_explicit_prompt
            && !is_builtin_type(field_ty)
            && !attrs.mask
            && !attrs.multiline;
        if is_nested {
            let prefix = format!("{}.", field_name);
            nested_arms.push(quote! {
                if field.starts_with(#prefix) {
                    let nested_field = &field[#prefix.len()..];
                    return <#field_ty as derive_wizard::Wizard>::validate_field(nested_field, value, answers);
                }
            });
        }
    }

    if validator_arms.is_empty() && nested_arms.is_empty() {
        // No validators, return a stub
        quote! {
            fn validate_field(_field: &str, _value: &str, _answers: &derive_wizard::Answers) -> Result<(), String> {
                Ok(())
            }
        }
    } else {
        quote! {
            fn validate_field(field: &str, value: &str, answers: &derive_wizard::Answers) -> Result<(), String> {
                // Check for nested wizard fields first
                #(#nested_arms)*

                // Then check direct field validators
                match field {
                    #(#validator_arms)*
                    _ => Ok(()),
                }
            }
        }
    }
}

fn generate_validate_field_method_enum(data: &syn::DataEnum) -> proc_macro2::TokenStream {
    // Collect all validators from all variants
    let mut validator_arms = Vec::new();
    // Collect nested wizard fields for delegation
    let mut nested_arms = Vec::new();

    for variant in &data.variants {
        match &variant.fields {
            Fields::Unit => {
                // No fields, no validation needed
            }
            Fields::Unnamed(fields) => {
                // Tuple variant fields
                for (i, field) in fields.unnamed.iter().enumerate() {
                    let field_name = format!("field_{}", i);
                    let attrs = FieldAttrs::extract(&field.attrs, &field_name);
                    let field_ty = &field.ty;

                    if let Some(ident) = attrs.validate_ident {
                        validator_arms.push(quote! {
                            #field_name => #ident(value, answers),
                        });
                    }

                    // Check if this is a nested wizard type (non-builtin with explicit #[prompt])
                    let is_nested = attrs.has_explicit_prompt
                        && !is_builtin_type(field_ty)
                        && !attrs.mask
                        && !attrs.multiline;
                    if is_nested {
                        let prefix = format!("{}.", field_name);
                        nested_arms.push(quote! {
                            if field.starts_with(#prefix) {
                                let nested_field = &field[#prefix.len()..];
                                return <#field_ty as derive_wizard::Wizard>::validate_field(nested_field, value, answers);
                            }
                        });
                    }
                }
            }
            Fields::Named(fields) => {
                // Struct variant fields
                for field in &fields.named {
                    let field_name = field.ident.as_ref().unwrap().to_string();
                    let attrs = FieldAttrs::extract(&field.attrs, &field_name);
                    let field_ty = &field.ty;

                    if let Some(ident) = attrs.validate_ident {
                        validator_arms.push(quote! {
                            #field_name => #ident(value, answers),
                        });
                    }

                    // Check if this is a nested wizard type (non-builtin with explicit #[prompt])
                    let is_nested = attrs.has_explicit_prompt
                        && !is_builtin_type(field_ty)
                        && !attrs.mask
                        && !attrs.multiline;
                    if is_nested {
                        let prefix = format!("{}.", field_name);
                        nested_arms.push(quote! {
                            if field.starts_with(#prefix) {
                                let nested_field = &field[#prefix.len()..];
                                return <#field_ty as derive_wizard::Wizard>::validate_field(nested_field, value, answers);
                            }
                        });
                    }
                }
            }
        }
    }

    if validator_arms.is_empty() && nested_arms.is_empty() {
        // No validators, return a stub
        quote! {
            fn validate_field(_field: &str, _value: &str, _answers: &derive_wizard::Answers) -> Result<(), String> {
                Ok(())
            }
        }
    } else {
        quote! {
            fn validate_field(field: &str, value: &str, answers: &derive_wizard::Answers) -> Result<(), String> {
                // Check for nested wizard fields first
                #(#nested_arms)*

                // Then check direct field validators
                match field {
                    #(#validator_arms)*
                    _ => Ok(()),
                }
            }
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
                QuestionKind::Input(_) => match type_ident_name(field_type).as_deref() {
                    Some("String") => Some(quote! {
                        interview.sections[#i].set_suggestion(self.#field_name.clone());
                    }),
                    Some("PathBuf") => Some(quote! {
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
