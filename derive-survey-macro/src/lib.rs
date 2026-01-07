//! Procedural macro for deriving `Survey` implementations.
//!
//! This crate provides the `#[derive(Survey)]` macro which generates:
//! - `Survey` trait implementation
//! - Type-specific builder with `suggest_*` and `assume_*` methods

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{
    Attribute, Data, DeriveInput, Expr, Fields, Ident, Lit, LitStr, Meta, Type, parse_macro_input,
};

/// Derive the `Survey` trait for a struct or enum.
///
/// # Attributes
///
/// ## On structs and enums
/// - `#[prelude("...")]` - Message shown before the survey starts
/// - `#[epilogue("...")]` - Message shown after the survey completes
/// - `#[validate("fn_name")]` - Composite validator function
///
/// ## On fields
/// - `#[ask("...")]` - The prompt text shown to the user (required for non-primitive types)
/// - `#[mask]` - Hide input (for passwords)
/// - `#[multiline]` - Open text editor / show textarea
/// - `#[validate("fn_name")]` - Field-level validator function
/// - `#[min(n)]` / `#[max(n)]` - Numeric bounds
/// - `#[multiselect]` - For `Vec<Enum>` fields, enables multi-select
#[proc_macro_derive(
    Survey,
    attributes(
        ask,
        mask,
        multiline,
        validate,
        min,
        max,
        prelude,
        epilogue,
        multiselect
    )
)]
pub fn derive_survey(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    implement_survey(&input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

fn implement_survey(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let name = &input.ident;
    let builder_name = format_ident!("{}Builder", name);

    // Extract struct/enum level attributes
    let type_attrs = TypeAttrs::extract(&input.attrs)?;

    // Generate the survey() method
    let survey_fn = generate_survey_fn(input, &type_attrs)?;

    // Generate from_responses() method
    let from_responses_fn = generate_from_responses_fn(input)?;

    // Generate validate_field() method
    let validate_field_fn = generate_validate_field_fn(input)?;

    // Generate validate_all() method
    let validate_all_fn = generate_validate_all_fn(input, &type_attrs)?;

    // Generate the builder struct and its methods
    let builder_impl = generate_builder(input, &builder_name)?;

    // Generate validator compile-time checks
    let validator_checks = generate_validator_checks(input)?;

    Ok(quote! {
        #validator_checks

        impl derive_survey::Survey for #name {
            fn survey() -> derive_survey::SurveyDefinition {
                #survey_fn
            }

            fn from_responses(responses: &derive_survey::Responses) -> Self {
                #from_responses_fn
            }

            fn validate_field(
                value: &derive_survey::ResponseValue,
                responses: &derive_survey::Responses,
            ) -> Result<(), String> {
                #validate_field_fn
            }

            fn validate_all(
                responses: &derive_survey::Responses,
            ) -> std::collections::HashMap<derive_survey::ResponsePath, String> {
                #validate_all_fn
            }
        }

        impl #name {
            /// Returns a builder for running this survey.
            pub fn builder() -> #builder_name {
                #builder_name::new()
            }
        }

        #builder_impl
    })
}

// ============================================================================
// Attribute Extraction
// ============================================================================

/// Attributes that can appear on the struct/enum itself
struct TypeAttrs {
    prelude: Option<String>,
    epilogue: Option<String>,
    validate: Option<Ident>,
}

impl TypeAttrs {
    fn extract(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut prelude = None;
        let mut epilogue = None;
        let mut validate = None;

        for attr in attrs {
            if attr.path().is_ident("prelude") {
                prelude = Some(extract_string_attr(attr)?);
            } else if attr.path().is_ident("epilogue") {
                epilogue = Some(extract_string_attr(attr)?);
            } else if attr.path().is_ident("validate") {
                validate = Some(extract_ident_attr(attr)?);
            }
        }

        Ok(Self {
            prelude,
            epilogue,
            validate,
        })
    }
}

/// Attributes that can appear on fields
struct FieldAttrs {
    ask: Option<String>,
    mask: bool,
    multiline: bool,
    validate: Option<Ident>,
    min: Option<i64>,
    max: Option<i64>,
    multiselect: bool,
}

impl FieldAttrs {
    fn extract(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut ask = None;
        let mut mask = false;
        let mut multiline = false;
        let mut validate = None;
        let mut min = None;
        let mut max = None;
        let mut multiselect = false;

        for attr in attrs {
            if attr.path().is_ident("ask") {
                ask = Some(extract_string_attr(attr)?);
            } else if attr.path().is_ident("mask") {
                mask = true;
            } else if attr.path().is_ident("multiline") {
                multiline = true;
            } else if attr.path().is_ident("validate") {
                validate = Some(extract_ident_attr(attr)?);
            } else if attr.path().is_ident("min") {
                min = Some(extract_int_attr(attr)?);
            } else if attr.path().is_ident("max") {
                max = Some(extract_int_attr(attr)?);
            } else if attr.path().is_ident("multiselect") {
                multiselect = true;
            }
        }

        Ok(Self {
            ask,
            mask,
            multiline,
            validate,
            min,
            max,
            multiselect,
        })
    }
}

fn extract_string_attr(attr: &Attribute) -> syn::Result<String> {
    let meta = &attr.meta;
    match meta {
        Meta::List(list) => {
            let lit: LitStr = list.parse_args()?;
            Ok(lit.value())
        }
        _ => Err(syn::Error::new_spanned(
            attr,
            "expected #[attr(\"string\")]",
        )),
    }
}

fn extract_ident_attr(attr: &Attribute) -> syn::Result<Ident> {
    let meta = &attr.meta;
    match meta {
        Meta::List(list) => {
            // Try parsing as a string literal first (e.g., #[validate("fn_name")])
            if let Ok(lit) = list.parse_args::<LitStr>() {
                return Ok(Ident::new(&lit.value(), lit.span()));
            }
            // Then try parsing as an identifier (e.g., #[validate(fn_name)])
            list.parse_args()
        }
        _ => Err(syn::Error::new_spanned(
            attr,
            "expected #[attr(identifier)] or #[attr(\"string\")]",
        )),
    }
}

fn extract_int_attr(attr: &Attribute) -> syn::Result<i64> {
    let meta = &attr.meta;
    match meta {
        Meta::List(list) => {
            let expr: Expr = list.parse_args()?;
            match expr {
                Expr::Lit(lit) => match &lit.lit {
                    Lit::Int(int) => int.base10_parse(),
                    _ => Err(syn::Error::new_spanned(lit, "expected integer literal")),
                },
                Expr::Unary(ref unary) => {
                    if matches!(unary.op, syn::UnOp::Neg(_))
                        && let Expr::Lit(ref lit) = *unary.expr
                        && let Lit::Int(int) = &lit.lit
                    {
                        let val: i64 = int.base10_parse()?;
                        return Ok(-val);
                    }
                    Err(syn::Error::new_spanned(expr, "expected integer literal"))
                }
                _ => Err(syn::Error::new_spanned(expr, "expected integer literal")),
            }
        }
        _ => Err(syn::Error::new_spanned(attr, "expected #[attr(number)]")),
    }
}

// ============================================================================
// Survey Generation
// ============================================================================

fn generate_survey_fn(input: &DeriveInput, type_attrs: &TypeAttrs) -> syn::Result<TokenStream2> {
    let prelude = match &type_attrs.prelude {
        Some(s) => quote! { Some(#s.to_string()) },
        None => quote! { None },
    };

    let epilogue = match &type_attrs.epilogue {
        Some(s) => quote! { Some(#s.to_string()) },
        None => quote! { None },
    };

    let questions = match &input.data {
        Data::Struct(data) => generate_struct_questions(data)?,
        Data::Enum(data) => generate_enum_questions(data, &input.ident)?,
        Data::Union(_) => {
            return Err(syn::Error::new_spanned(
                input,
                "Survey cannot be derived for unions",
            ));
        }
    };

    Ok(quote! {
        derive_survey::SurveyDefinition {
            prelude: #prelude,
            questions: #questions,
            epilogue: #epilogue,
        }
    })
}

fn generate_struct_questions(data: &syn::DataStruct) -> syn::Result<TokenStream2> {
    let mut questions = Vec::new();

    match &data.fields {
        Fields::Named(fields) => {
            for field in &fields.named {
                let field_name = field.ident.as_ref().unwrap();
                let field_name_str = field_name.to_string();
                let attrs = FieldAttrs::extract(&field.attrs)?;
                let question = generate_question_for_field(&field_name_str, &field.ty, &attrs)?;
                questions.push(question);
            }
        }
        Fields::Unnamed(fields) => {
            for (i, field) in fields.unnamed.iter().enumerate() {
                let field_name_str = i.to_string();
                let attrs = FieldAttrs::extract(&field.attrs)?;
                let question = generate_question_for_field(&field_name_str, &field.ty, &attrs)?;
                questions.push(question);
            }
        }
        Fields::Unit => {}
    }

    Ok(quote! { vec![#(#questions),*] })
}

fn generate_enum_questions(data: &syn::DataEnum, _enum_name: &Ident) -> syn::Result<TokenStream2> {
    // For enums, we generate a single OneOf question containing all variants
    let mut variants = Vec::new();

    for variant in &data.variants {
        let variant_name = variant.ident.to_string();

        // Check for #[ask] on the variant itself for display text
        let variant_attrs = FieldAttrs::extract(&variant.attrs)?;
        let display_name = variant_attrs.ask.unwrap_or_else(|| variant_name.clone());

        let kind = match &variant.fields {
            Fields::Unit => quote! { derive_survey::QuestionKind::Unit },
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                // Newtype variant - wrap in AllOf with a single Question to preserve prompt
                let field = &fields.unnamed[0];
                let attrs = FieldAttrs::extract(&field.attrs)?;
                let q = generate_question_for_field("0", &field.ty, &attrs)?;
                quote! { derive_survey::QuestionKind::AllOf(derive_survey::AllOfQuestion::new(vec![#q])) }
            }
            Fields::Unnamed(fields) => {
                // Tuple variant - treat as AllOf
                let mut qs = Vec::new();
                for (i, field) in fields.unnamed.iter().enumerate() {
                    let name = i.to_string();
                    let attrs = FieldAttrs::extract(&field.attrs)?;
                    let q = generate_question_for_field(&name, &field.ty, &attrs)?;
                    qs.push(q);
                }
                quote! { derive_survey::QuestionKind::AllOf(derive_survey::AllOfQuestion::new(vec![#(#qs),*])) }
            }
            Fields::Named(fields) => {
                // Struct variant
                let mut qs = Vec::new();
                for field in &fields.named {
                    let name = field.ident.as_ref().unwrap().to_string();
                    let attrs = FieldAttrs::extract(&field.attrs)?;
                    let q = generate_question_for_field(&name, &field.ty, &attrs)?;
                    qs.push(q);
                }
                quote! { derive_survey::QuestionKind::AllOf(derive_survey::AllOfQuestion::new(vec![#(#qs),*])) }
            }
        };

        variants.push(quote! {
            derive_survey::Variant {
                name: #display_name.to_string(),
                kind: #kind,
            }
        });
    }

    // Return a single-element vec with the OneOf question
    Ok(quote! {
        vec![derive_survey::Question::new(
            derive_survey::ResponsePath::empty(),
            String::new(),  // No prompt for root enum
            derive_survey::QuestionKind::OneOf(derive_survey::OneOfQuestion {
                variants: vec![#(#variants),*],
                default: None,
            }),
        )]
    })
}

fn generate_question_for_field(
    field_name: &str,
    ty: &Type,
    attrs: &FieldAttrs,
) -> syn::Result<TokenStream2> {
    // Use field name as default prompt, converting snake_case to Title Case
    let default_prompt = field_name
        .split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");
    let ask = attrs.ask.clone().unwrap_or(default_prompt);
    let kind = generate_question_kind(ty, attrs)?;

    Ok(quote! {
        derive_survey::Question::new(
            derive_survey::ResponsePath::new(#field_name),
            #ask.to_string(),
            #kind,
        )
    })
}

fn generate_question_kind(ty: &Type, attrs: &FieldAttrs) -> syn::Result<TokenStream2> {
    // Handle special attributes first
    if attrs.mask {
        return Ok(quote! {
            derive_survey::QuestionKind::Masked(derive_survey::MaskedQuestion::new())
        });
    }

    if attrs.multiline {
        return Ok(quote! {
            derive_survey::QuestionKind::Multiline(derive_survey::MultilineQuestion::new())
        });
    }

    // Check for multiselect (Vec<Enum>)
    if attrs.multiselect
        && let Some(inner_ty) = extract_vec_inner_type(ty)
    {
        return Ok(quote! {
            derive_survey::QuestionKind::AnyOf(derive_survey::AnyOfQuestion {
                variants: <#inner_ty as derive_survey::Survey>::survey()
                    .questions
                    .into_iter()
                    .flat_map(|q| match q.kind() {
                        derive_survey::QuestionKind::OneOf(one_of) => one_of.variants.clone(),
                        _ => vec![],
                    })
                    .collect(),
                defaults: vec![],
            })
        });
    }

    // Handle types based on their name
    let type_name = type_to_string(ty);

    match type_name.as_str() {
        "String" | "&str" => Ok(quote! {
            derive_survey::QuestionKind::Input(derive_survey::InputQuestion::new())
        }),
        "bool" => Ok(quote! {
            derive_survey::QuestionKind::Confirm(derive_survey::ConfirmQuestion::new())
        }),
        "i8" | "i16" | "i32" | "i64" | "isize" | "u8" | "u16" | "u32" | "u64" | "usize" => {
            let min_opt = match attrs.min {
                Some(m) => quote! { Some(#m) },
                None => quote! { None },
            };
            let max_opt = match attrs.max {
                Some(m) => quote! { Some(#m) },
                None => quote! { None },
            };
            Ok(quote! {
                derive_survey::QuestionKind::Int(derive_survey::IntQuestion::with_bounds(#min_opt, #max_opt))
            })
        }
        "f32" | "f64" => {
            let min_opt = match attrs.min {
                Some(m) => {
                    let f = m as f64;
                    quote! { Some(#f) }
                }
                None => quote! { None },
            };
            let max_opt = match attrs.max {
                Some(m) => {
                    let f = m as f64;
                    quote! { Some(#f) }
                }
                None => quote! { None },
            };
            Ok(quote! {
                derive_survey::QuestionKind::Float(derive_survey::FloatQuestion::with_bounds(#min_opt, #max_opt))
            })
        }
        "PathBuf" => Ok(quote! {
            derive_survey::QuestionKind::Input(derive_survey::InputQuestion::new())
        }),
        _ => {
            // Check if it's an Option<T>
            if let Some(inner_ty) = extract_option_inner_type(ty) {
                let inner_kind = generate_question_kind(&inner_ty, attrs)?;
                // TODO: Handle Option properly - for now treat as inner type
                return Ok(inner_kind);
            }

            // Check if it's a Vec<T> (non-multiselect)
            if let Some(_inner_ty) = extract_vec_inner_type(ty) {
                // For Vec without multiselect, we need AnyOf
                return Ok(quote! {
                    derive_survey::QuestionKind::Input(derive_survey::InputQuestion::new())
                });
            }

            // Assume it's a nested Survey type
            Ok(quote! {
                derive_survey::QuestionKind::AllOf(
                    derive_survey::AllOfQuestion::new(<#ty as derive_survey::Survey>::survey().questions)
                )
            })
        }
    }
}

fn type_to_string(ty: &Type) -> String {
    match ty {
        Type::Path(path) => path
            .path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default(),
        _ => String::new(),
    }
}

fn extract_option_inner_type(ty: &Type) -> Option<Type> {
    if let Type::Path(path) = ty
        && let Some(segment) = path.path.segments.last()
        && segment.ident == "Option"
        && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
        && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
    {
        return Some(inner.clone());
    }
    None
}

fn extract_vec_inner_type(ty: &Type) -> Option<Type> {
    if let Type::Path(path) = ty
        && let Some(segment) = path.path.segments.last()
        && segment.ident == "Vec"
        && let syn::PathArguments::AngleBracketed(args) = &segment.arguments
        && let Some(syn::GenericArgument::Type(inner)) = args.args.first()
    {
        return Some(inner.clone());
    }
    None
}

// ============================================================================
// from_responses Generation
// ============================================================================

fn generate_from_responses_fn(input: &DeriveInput) -> syn::Result<TokenStream2> {
    match &input.data {
        Data::Struct(data) => generate_from_responses_struct(&input.ident, data),
        Data::Enum(data) => generate_from_responses_enum(&input.ident, data),
        Data::Union(_) => Err(syn::Error::new_spanned(
            input,
            "Survey cannot be derived for unions",
        )),
    }
}

fn generate_from_responses_struct(
    name: &Ident,
    data: &syn::DataStruct,
) -> syn::Result<TokenStream2> {
    match &data.fields {
        Fields::Named(fields) => {
            let field_inits: Vec<_> = fields
                .named
                .iter()
                .map(|f| {
                    let field_name = f.ident.as_ref().unwrap();
                    let field_name_str = field_name.to_string();
                    let ty = &f.ty;
                    let extraction = generate_value_extraction(&field_name_str, ty);
                    quote! { #field_name: #extraction }
                })
                .collect();

            Ok(quote! {
                #name {
                    #(#field_inits),*
                }
            })
        }
        Fields::Unnamed(fields) => {
            let field_inits: Vec<_> = fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(i, f)| {
                    let field_name_str = i.to_string();
                    let ty = &f.ty;
                    generate_value_extraction(&field_name_str, ty)
                })
                .collect();

            Ok(quote! {
                #name(#(#field_inits),*)
            })
        }
        Fields::Unit => Ok(quote! { #name }),
    }
}

fn generate_from_responses_enum(name: &Ident, data: &syn::DataEnum) -> syn::Result<TokenStream2> {
    let variant_arms: Vec<_> = data
        .variants
        .iter()
        .enumerate()
        .map(|(idx, variant)| {
            let variant_name = &variant.ident;

            let construction = match &variant.fields {
                Fields::Unit => quote! { #name::#variant_name },
                Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                    let ty = &fields.unnamed[0].ty;
                    let extraction = generate_value_extraction("0", ty);
                    quote! { #name::#variant_name(#extraction) }
                }
                Fields::Unnamed(fields) => {
                    let extractions: Vec<_> = fields
                        .unnamed
                        .iter()
                        .enumerate()
                        .map(|(i, f)| {
                            let ty = &f.ty;
                            generate_value_extraction(&i.to_string(), ty)
                        })
                        .collect();
                    quote! { #name::#variant_name(#(#extractions),*) }
                }
                Fields::Named(fields) => {
                    let field_inits: Vec<_> = fields
                        .named
                        .iter()
                        .map(|f| {
                            let field_name = f.ident.as_ref().unwrap();
                            let field_name_str = field_name.to_string();
                            let ty = &f.ty;
                            let extraction = generate_value_extraction(&field_name_str, ty);
                            quote! { #field_name: #extraction }
                        })
                        .collect();
                    quote! { #name::#variant_name { #(#field_inits),* } }
                }
            };

            quote! { #idx => #construction }
        })
        .collect();

    Ok(quote! {
        let variant_idx = responses
            .get_chosen_variant(&derive_survey::ResponsePath::new(derive_survey::SELECTED_VARIANT_KEY))
            .expect("missing selected_variant");
        match variant_idx {
            #(#variant_arms,)*
            _ => panic!("invalid variant index"),
        }
    })
}

fn generate_value_extraction(field_name: &str, ty: &Type) -> TokenStream2 {
    let type_name = type_to_string(ty);
    let path_expr = quote! { derive_survey::ResponsePath::new(#field_name) };

    match type_name.as_str() {
        "String" => quote! {
            responses.get_string(&#path_expr).expect("missing string").to_string()
        },
        "bool" => quote! {
            responses.get_bool(&#path_expr).expect("missing bool")
        },
        "i8" | "i16" | "i32" | "i64" | "isize" => quote! {
            responses.get_int(&#path_expr).expect("missing int") as #ty
        },
        "u8" | "u16" | "u32" | "u64" | "usize" => quote! {
            responses.get_int(&#path_expr).expect("missing int") as #ty
        },
        "f32" | "f64" => quote! {
            responses.get_float(&#path_expr).expect("missing float") as #ty
        },
        "PathBuf" => quote! {
            std::path::PathBuf::from(responses.get_string(&#path_expr).expect("missing path"))
        },
        _ => {
            // Check for Option<T>
            if let Some(inner_ty) = extract_option_inner_type(ty) {
                let inner_extraction = generate_value_extraction(field_name, &inner_ty);
                return quote! {
                    if responses.get(&#path_expr).is_some() {
                        Some(#inner_extraction)
                    } else {
                        None
                    }
                };
            }

            // Check for Vec<T> with chosen_variants (AnyOf)
            if let Some(inner_ty) = extract_vec_inner_type(ty) {
                let variants_path = quote! {
                    derive_survey::ResponsePath::new(
                        &format!("{}.{}", #field_name, derive_survey::SELECTED_VARIANTS_KEY)
                    )
                };
                return quote! {
                    {
                        let indices = responses
                            .get_chosen_variants(&#variants_path)
                            .map(|s| s.to_vec())
                            .unwrap_or_default();

                        // Reconstruct each item from its indexed responses
                        indices
                            .iter()
                            .enumerate()
                            .map(|(item_idx, _variant_idx)| {
                                let item_prefix = derive_survey::ResponsePath::new(
                                    &format!("{}.{}", #field_name, item_idx)
                                );
                                let item_responses = responses.filter_prefix(&item_prefix);
                                <#inner_ty as derive_survey::Survey>::from_responses(&item_responses)
                            })
                            .collect()
                    }
                };
            }

            // Nested Survey type - filter responses and call its from_responses
            quote! {
                {
                    let prefix = derive_survey::ResponsePath::new(#field_name);
                    let nested_responses = responses.filter_prefix(&prefix);
                    <#ty as derive_survey::Survey>::from_responses(&nested_responses)
                }
            }
        }
    }
}

// ============================================================================
// Validation Generation
// ============================================================================

fn generate_validate_field_fn(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let mut validators = Vec::new();

    match &input.data {
        Data::Struct(data) => {
            if let Fields::Named(fields) = &data.fields {
                for field in &fields.named {
                    let attrs = FieldAttrs::extract(&field.attrs)?;

                    if let Some(validator) = &attrs.validate {
                        // The validator is called directly with the value being validated
                        validators.push(quote! {
                            if let Err(e) = #validator(value, responses) {
                                return Err(e);
                            }
                        });
                    }

                    // Add min/max validation for numeric types
                    if attrs.min.is_some() || attrs.max.is_some() {
                        let type_name = type_to_string(&field.ty);
                        if matches!(
                            type_name.as_str(),
                            "i8" | "i16"
                                | "i32"
                                | "i64"
                                | "isize"
                                | "u8"
                                | "u16"
                                | "u32"
                                | "u64"
                                | "usize"
                        ) {
                            let min_check = attrs.min.map(|m| {
                                quote! {
                                    if parsed < #m {
                                        return Err(format!("Value must be at least {}", #m));
                                    }
                                }
                            });
                            let max_check = attrs.max.map(|m| {
                                quote! {
                                    if parsed > #m {
                                        return Err(format!("Value must be at most {}", #m));
                                    }
                                }
                            });

                            validators.push(quote! {
                                if let derive_survey::ResponseValue::Int(parsed) = value {
                                    let parsed = *parsed;
                                    #min_check
                                    #max_check
                                }
                            });
                        }
                    }
                }
            }
        }
        Data::Enum(data) => {
            for variant in &data.variants {
                match &variant.fields {
                    Fields::Named(fields) => {
                        for field in &fields.named {
                            let attrs = FieldAttrs::extract(&field.attrs)?;

                            if let Some(validator) = &attrs.validate {
                                validators.push(quote! {
                                    if let Err(e) = #validator(value, responses) {
                                        return Err(e);
                                    }
                                });
                            }
                        }
                    }
                    Fields::Unnamed(fields) => {
                        for field in &fields.unnamed {
                            let attrs = FieldAttrs::extract(&field.attrs)?;

                            if let Some(validator) = &attrs.validate {
                                validators.push(quote! {
                                    if let Err(e) = #validator(value, responses) {
                                        return Err(e);
                                    }
                                });
                            }
                        }
                    }
                    Fields::Unit => {}
                }
            }
        }
        Data::Union(_) => {}
    }

    Ok(quote! {
        #(#validators)*
        Ok(())
    })
}

fn generate_validate_all_fn(
    _input: &DeriveInput,
    type_attrs: &TypeAttrs,
) -> syn::Result<TokenStream2> {
    let composite_call = if let Some(validator) = &type_attrs.validate {
        quote! { #validator(responses) }
    } else {
        quote! { std::collections::HashMap::new() }
    };

    // For now, just call the composite validator if present
    // TODO: Also validate nested Survey types
    Ok(quote! {
        #composite_call
    })
}

fn generate_validator_checks(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let mut checks = Vec::new();

    // Check composite validator
    let type_attrs = TypeAttrs::extract(&input.attrs)?;
    if let Some(validator) = &type_attrs.validate {
        checks.push(quote! {
            const _: fn(&derive_survey::Responses) -> std::collections::HashMap<derive_survey::ResponsePath, String> = #validator;
        });
    }

    // Check field validators
    let check_field = |field: &syn::Field, checks: &mut Vec<TokenStream2>| -> syn::Result<()> {
        let attrs = FieldAttrs::extract(&field.attrs)?;
        if let Some(validator) = &attrs.validate {
            checks.push(quote! {
                const _: fn(&derive_survey::ResponseValue, &derive_survey::Responses) -> Result<(), String> = #validator;
            });
        }
        Ok(())
    };

    match &input.data {
        Data::Struct(data) => {
            if let Fields::Named(fields) = &data.fields {
                for field in &fields.named {
                    check_field(field, &mut checks)?;
                }
            }
        }
        Data::Enum(data) => {
            for variant in &data.variants {
                match &variant.fields {
                    Fields::Named(fields) => {
                        for field in &fields.named {
                            check_field(field, &mut checks)?;
                        }
                    }
                    Fields::Unnamed(fields) => {
                        for field in &fields.unnamed {
                            check_field(field, &mut checks)?;
                        }
                    }
                    Fields::Unit => {}
                }
            }
        }
        Data::Union(_) => {}
    }

    Ok(quote! { #(#checks)* })
}

// ============================================================================
// Builder Generation
// ============================================================================

fn generate_builder(input: &DeriveInput, builder_name: &Ident) -> syn::Result<TokenStream2> {
    let name = &input.ident;

    // Collect all fields for suggest/assume methods
    let mut suggest_methods = Vec::new();
    let mut assume_methods = Vec::new();

    generate_builder_methods_for_type(
        input,
        "", // root prefix
        &mut suggest_methods,
        &mut assume_methods,
    )?;

    Ok(quote! {
        /// Builder for running surveys with suggestions and assumptions
        pub struct #builder_name {
            suggestions: std::collections::HashMap<String, derive_survey::ResponseValue>,
            assumptions: std::collections::HashMap<String, derive_survey::ResponseValue>,
        }

        impl #builder_name {
            /// Create a new builder
            pub fn new() -> Self {
                Self {
                    suggestions: std::collections::HashMap::new(),
                    assumptions: std::collections::HashMap::new(),
                }
            }

            /// Set suggestions from an existing instance (all fields become suggested defaults)
            pub fn with_suggestions(mut self, instance: &#name) -> Self {
                // TODO: Walk the instance and populate suggestions
                self
            }

            #(#suggest_methods)*
            #(#assume_methods)*

            /// Run the survey with the given backend
            pub fn run<B: derive_survey::SurveyBackend>(
                self,
                backend: B,
            ) -> Result<#name, anyhow::Error> {
                let mut definition = #name::survey();

                // Apply suggestions and assumptions to questions
                self.apply_to_definition(&mut definition);

                // Collect responses
                let responses = backend.collect(
                    &definition,
                    &|value, responses| #name::validate_field(value, responses),
                ).map_err(Into::into)?;

                // Reconstruct the type
                Ok(#name::from_responses(&responses))
            }

            fn apply_to_definition(&self, definition: &mut derive_survey::SurveyDefinition) {
                for question in &mut definition.questions {
                    self.apply_to_question(question);
                }
            }

            fn apply_to_question(&self, question: &mut derive_survey::Question) {
                let path_str = question.path().as_str().to_string();

                // Check for assumption first (takes priority)
                if let Some(value) = self.assumptions.get(&path_str) {
                    question.set_assumption(value.clone());
                    return;
                }

                // Then check for suggestion
                if let Some(value) = self.suggestions.get(&path_str) {
                    question.set_suggestion(value.clone());
                }

                // Recurse into nested questions
                match question.kind_mut() {
                    derive_survey::QuestionKind::AllOf(all_of) => {
                        for q in all_of.questions_mut() {
                            self.apply_to_question(q);
                        }
                    }
                    derive_survey::QuestionKind::OneOf(one_of) => {
                        for variant in &mut one_of.variants {
                            if let derive_survey::QuestionKind::AllOf(all_of) = &mut variant.kind {
                                for q in all_of.questions_mut() {
                                    self.apply_to_question(q);
                                }
                            }
                        }
                    }
                    derive_survey::QuestionKind::AnyOf(any_of) => {
                        for variant in &mut any_of.variants {
                            if let derive_survey::QuestionKind::AllOf(all_of) = &mut variant.kind {
                                for q in all_of.questions_mut() {
                                    self.apply_to_question(q);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        impl Default for #builder_name {
            fn default() -> Self {
                Self::new()
            }
        }
    })
}

fn generate_builder_methods_for_type(
    input: &DeriveInput,
    prefix: &str,
    suggest_methods: &mut Vec<TokenStream2>,
    assume_methods: &mut Vec<TokenStream2>,
) -> syn::Result<()> {
    match &input.data {
        Data::Struct(data) => {
            generate_builder_methods_for_fields(
                &data.fields,
                prefix,
                suggest_methods,
                assume_methods,
            )?;
        }
        Data::Enum(data) => {
            // For enums, generate methods for each variant's fields
            for variant in &data.variants {
                let variant_prefix = if prefix.is_empty() {
                    variant.ident.to_string().to_lowercase()
                } else {
                    format!("{}_{}", prefix, variant.ident.to_string().to_lowercase())
                };
                generate_builder_methods_for_fields(
                    &variant.fields,
                    &variant_prefix,
                    suggest_methods,
                    assume_methods,
                )?;
            }
        }
        Data::Union(_) => {}
    }
    Ok(())
}

fn generate_builder_methods_for_fields(
    fields: &Fields,
    prefix: &str,
    suggest_methods: &mut Vec<TokenStream2>,
    assume_methods: &mut Vec<TokenStream2>,
) -> syn::Result<()> {
    match fields {
        Fields::Named(fields) => {
            for field in &fields.named {
                let field_name = field.ident.as_ref().unwrap();
                let field_name_str = field_name.to_string();
                let ty = &field.ty;

                let method_suffix = if prefix.is_empty() {
                    field_name_str.clone()
                } else {
                    format!("{}_{}", prefix, field_name_str)
                };

                let path_key = if prefix.is_empty() {
                    field_name_str.clone()
                } else {
                    format!("{}.{}", prefix.replace('_', "."), field_name_str)
                };

                generate_suggest_assume_methods(
                    &method_suffix,
                    &path_key,
                    ty,
                    suggest_methods,
                    assume_methods,
                );
            }
        }
        Fields::Unnamed(fields) => {
            for (i, field) in fields.unnamed.iter().enumerate() {
                let field_name_str = i.to_string();
                let ty = &field.ty;

                let method_suffix = if prefix.is_empty() {
                    field_name_str.clone()
                } else {
                    format!("{}_{}", prefix, field_name_str)
                };

                let path_key = if prefix.is_empty() {
                    field_name_str
                } else {
                    format!("{}.{}", prefix.replace('_', "."), i)
                };

                generate_suggest_assume_methods(
                    &method_suffix,
                    &path_key,
                    ty,
                    suggest_methods,
                    assume_methods,
                );
            }
        }
        Fields::Unit => {}
    }
    Ok(())
}

fn generate_suggest_assume_methods(
    method_suffix: &str,
    path_key: &str,
    ty: &Type,
    suggest_methods: &mut Vec<TokenStream2>,
    assume_methods: &mut Vec<TokenStream2>,
) {
    let suggest_name = format_ident!("suggest_{}", method_suffix);
    let assume_name = format_ident!("assume_{}", method_suffix);

    let type_name = type_to_string(ty);
    let (param_type, conversion) = match type_name.as_str() {
        "String" => (
            quote! { impl Into<String> },
            quote! { derive_survey::ResponseValue::String(value.into()) },
        ),
        "bool" => (
            quote! { bool },
            quote! { derive_survey::ResponseValue::Bool(value) },
        ),
        "i8" | "i16" | "i32" | "i64" | "isize" => (
            quote! { #ty },
            quote! { derive_survey::ResponseValue::Int(value as i64) },
        ),
        "u8" | "u16" | "u32" | "u64" | "usize" => (
            quote! { #ty },
            quote! { derive_survey::ResponseValue::Int(value as i64) },
        ),
        "f32" | "f64" => (
            quote! { #ty },
            quote! { derive_survey::ResponseValue::Float(value as f64) },
        ),
        "PathBuf" => (
            quote! { impl Into<std::path::PathBuf> },
            quote! { derive_survey::ResponseValue::String(value.into().to_string_lossy().into_owned()) },
        ),
        _ => {
            // For complex types, don't generate individual methods
            // They would need special handling
            return;
        }
    };

    suggest_methods.push(quote! {
        /// Suggest a default value for this field (user can modify)
        pub fn #suggest_name(mut self, value: #param_type) -> Self {
            self.suggestions.insert(#path_key.to_string(), #conversion);
            self
        }
    });

    assume_methods.push(quote! {
        /// Assume a value for this field (question is skipped)
        pub fn #assume_name(mut self, value: #param_type) -> Self {
            self.assumptions.insert(#path_key.to_string(), #conversion);
            self
        }
    });
}
