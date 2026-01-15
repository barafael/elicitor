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
/// - `#[validate_fields("fn_name")]` - Propagate a field-level validator to all numeric child fields
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
        validate_fields,
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

    // Generate typed field accessor methods
    let field_accessors = generate_field_accessors(input)?;

    // Generate ValidationContext struct for validators
    let validation_context = generate_validation_context(input)?;

    Ok(quote! {
        #validator_checks

        #validation_context

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
                path: &derive_survey::ResponsePath,
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

            #field_accessors
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
    /// Validator to propagate to all numeric child fields
    validate_fields: Option<Ident>,
}

impl TypeAttrs {
    fn extract(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut prelude = None;
        let mut epilogue = None;
        let mut validate = None;
        let mut validate_fields = None;

        for attr in attrs {
            if attr.path().is_ident("prelude") {
                prelude = Some(extract_string_attr(attr)?);
            } else if attr.path().is_ident("epilogue") {
                epilogue = Some(extract_string_attr(attr)?);
            } else if attr.path().is_ident("validate") {
                validate = Some(extract_ident_attr(attr)?);
            } else if attr.path().is_ident("validate_fields") {
                validate_fields = Some(extract_ident_attr(attr)?);
            }
        }

        Ok(Self {
            prelude,
            epilogue,
            validate,
            validate_fields,
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
        Data::Struct(data) => generate_struct_questions(data, type_attrs.validate_fields.as_ref())?,
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

fn generate_struct_questions(
    data: &syn::DataStruct,
    propagated_validator: Option<&Ident>,
) -> syn::Result<TokenStream2> {
    let mut questions = Vec::new();

    match &data.fields {
        Fields::Named(fields) => {
            for field in &fields.named {
                let field_name = field.ident.as_ref().unwrap();
                let field_name_str = field_name.to_string();
                let attrs = FieldAttrs::extract(&field.attrs)?;
                let question = generate_question_for_field(
                    &field_name_str,
                    &field.ty,
                    &attrs,
                    propagated_validator,
                )?;
                questions.push(question);
            }
        }
        Fields::Unnamed(fields) => {
            for (i, field) in fields.unnamed.iter().enumerate() {
                let field_name_str = i.to_string();
                let attrs = FieldAttrs::extract(&field.attrs)?;
                let question = generate_question_for_field(
                    &field_name_str,
                    &field.ty,
                    &attrs,
                    propagated_validator,
                )?;
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
                let q = generate_question_for_field("0", &field.ty, &attrs, None)?;
                quote! { derive_survey::QuestionKind::AllOf(derive_survey::AllOfQuestion::new(vec![#q])) }
            }
            Fields::Unnamed(fields) => {
                // Tuple variant - treat as AllOf
                let mut qs = Vec::new();
                for (i, field) in fields.unnamed.iter().enumerate() {
                    let name = i.to_string();
                    let attrs = FieldAttrs::extract(&field.attrs)?;
                    let q = generate_question_for_field(&name, &field.ty, &attrs, None)?;
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
                    let q = generate_question_for_field(&name, &field.ty, &attrs, None)?;
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
    propagated_validator: Option<&Ident>,
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
    let kind = generate_question_kind(ty, attrs, propagated_validator)?;

    Ok(quote! {
        derive_survey::Question::new(
            derive_survey::ResponsePath::new(#field_name),
            #ask.to_string(),
            #kind,
        )
    })
}

fn generate_question_kind(
    ty: &Type,
    attrs: &FieldAttrs,
    propagated_validator: Option<&Ident>,
) -> syn::Result<TokenStream2> {
    // Handle special attributes first
    if attrs.mask {
        let validate_opt = match (&attrs.validate, propagated_validator) {
            (Some(v), _) => {
                let v_str = v.to_string();
                quote! { Some(#v_str.to_string()) }
            }
            (None, Some(v)) => {
                let v_str = v.to_string();
                quote! { Some(#v_str.to_string()) }
            }
            (None, None) => quote! { None },
        };
        return Ok(quote! {
            derive_survey::QuestionKind::Masked(derive_survey::MaskedQuestion::with_validator(#validate_opt))
        });
    }

    if attrs.multiline {
        let validate_opt = match (&attrs.validate, propagated_validator) {
            (Some(v), _) => {
                let v_str = v.to_string();
                quote! { Some(#v_str.to_string()) }
            }
            (None, Some(v)) => {
                let v_str = v.to_string();
                quote! { Some(#v_str.to_string()) }
            }
            (None, None) => quote! { None },
        };
        return Ok(quote! {
            derive_survey::QuestionKind::Multiline(derive_survey::MultilineQuestion::with_validator(#validate_opt))
        });
    }

    // Check for Vec<T>
    if let Some(inner_ty) = extract_vec_inner_type(ty) {
        // If multiselect is set, use AnyOf for Vec<Enum>
        if attrs.multiselect {
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

        // For Vec<primitive>, generate ListQuestion
        let inner_type_name = type_to_string(&inner_ty);
        match inner_type_name.as_str() {
            "String" => {
                return Ok(quote! {
                    derive_survey::QuestionKind::List(derive_survey::ListQuestion::strings())
                });
            }
            "i8" | "i16" | "i32" | "i64" | "isize" | "u8" | "u16" | "u32" | "u64" | "usize" => {
                let min_opt = match attrs.min {
                    Some(m) => quote! { Some(#m) },
                    None => quote! { None },
                };
                let max_opt = match attrs.max {
                    Some(m) => quote! { Some(#m) },
                    None => quote! { None },
                };
                return Ok(quote! {
                    derive_survey::QuestionKind::List(derive_survey::ListQuestion::ints_with_bounds(#min_opt, #max_opt))
                });
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
                return Ok(quote! {
                    derive_survey::QuestionKind::List(derive_survey::ListQuestion::floats_with_bounds(#min_opt, #max_opt))
                });
            }
            _ => {
                // For other types (complex types without multiselect), fall through to error
            }
        }
    }

    // Handle types based on their name
    let type_name = type_to_string(ty);

    match type_name.as_str() {
        "String" | "&str" => {
            let validate_opt = match (&attrs.validate, propagated_validator) {
                (Some(v), _) => {
                    let v_str = v.to_string();
                    quote! { Some(#v_str.to_string()) }
                }
                (None, Some(v)) => {
                    let v_str = v.to_string();
                    quote! { Some(#v_str.to_string()) }
                }
                (None, None) => quote! { None },
            };
            Ok(quote! {
                derive_survey::QuestionKind::Input(derive_survey::InputQuestion::with_validator(#validate_opt))
            })
        }
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
            // Use field-level validator if present, otherwise use propagated validator
            let validate_opt = match (&attrs.validate, propagated_validator) {
                (Some(v), _) => {
                    let v_str = v.to_string();
                    quote! { Some(#v_str.to_string()) }
                }
                (None, Some(v)) => {
                    let v_str = v.to_string();
                    quote! { Some(#v_str.to_string()) }
                }
                (None, None) => quote! { None },
            };
            Ok(quote! {
                derive_survey::QuestionKind::Int(derive_survey::IntQuestion::with_bounds_and_validator(#min_opt, #max_opt, #validate_opt))
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
            // Use field-level validator if present, otherwise use propagated validator
            let validate_opt = match (&attrs.validate, propagated_validator) {
                (Some(v), _) => {
                    let v_str = v.to_string();
                    quote! { Some(#v_str.to_string()) }
                }
                (None, Some(v)) => {
                    let v_str = v.to_string();
                    quote! { Some(#v_str.to_string()) }
                }
                (None, None) => quote! { None },
            };
            Ok(quote! {
                derive_survey::QuestionKind::Float(derive_survey::FloatQuestion::with_bounds_and_validator(#min_opt, #max_opt, #validate_opt))
            })
        }
        "PathBuf" => Ok(quote! {
            derive_survey::QuestionKind::Input(derive_survey::InputQuestion::new())
        }),
        _ => {
            // Check if it's an Option<T>
            if let Some(inner_ty) = extract_option_inner_type(ty) {
                let inner_kind = generate_question_kind(&inner_ty, attrs, propagated_validator)?;
                // TODO: Handle Option properly - for now treat as inner type
                return Ok(inner_kind);
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
                    if responses.has_value(&#path_expr) {
                        Some(#inner_extraction)
                    } else {
                        None
                    }
                };
            }

            // Check for Vec<T>
            if let Some(inner_ty) = extract_vec_inner_type(ty) {
                let inner_type_name = type_to_string(&inner_ty);

                // For primitive types, use the list response values
                match inner_type_name.as_str() {
                    "String" => {
                        return quote! {
                            responses.get_string_list(&#path_expr)
                                .expect("missing string list")
                                .to_vec()
                        };
                    }
                    "i8" | "i16" | "i32" | "i64" | "isize" => {
                        return quote! {
                            responses.get_int_list(&#path_expr)
                                .expect("missing int list")
                                .iter()
                                .map(|&n| n as #inner_ty)
                                .collect()
                        };
                    }
                    "u8" | "u16" | "u32" | "u64" | "usize" => {
                        return quote! {
                            responses.get_int_list(&#path_expr)
                                .expect("missing int list")
                                .iter()
                                .map(|&n| n as #inner_ty)
                                .collect()
                        };
                    }
                    "f32" | "f64" => {
                        return quote! {
                            responses.get_float_list(&#path_expr)
                                .expect("missing float list")
                                .iter()
                                .map(|&n| n as #inner_ty)
                                .collect()
                        };
                    }
                    _ => {
                        // For complex types (enums with multiselect), use chosen_variants
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
                }
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
// Field Accessor Generation
// ============================================================================

/// Generate typed field accessor methods for retrieving values from Responses.
/// These methods allow validators to access field values without using string paths.
fn generate_field_accessors(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let mut accessors = Vec::new();

    match &input.data {
        Data::Struct(data) => {
            if let Fields::Named(fields) = &data.fields {
                for field in &fields.named {
                    let field_name = field.ident.as_ref().unwrap();
                    let field_name_str = field_name.to_string();
                    let ty = &field.ty;

                    let accessor = generate_field_accessor_method(&field_name_str, field_name, ty);
                    accessors.push(accessor);
                }
            }
        }
        Data::Enum(_) => {
            // Enums don't need field accessors in the same way
        }
        Data::Union(_) => {}
    }

    Ok(quote! {
        #(#accessors)*
    })
}

/// Generate a single field accessor method
fn generate_field_accessor_method(
    field_name_str: &str,
    field_name: &Ident,
    ty: &Type,
) -> TokenStream2 {
    let method_name = format_ident!("get_{}", field_name);
    let type_name = type_to_string(ty);
    let path_expr = quote! { derive_survey::ResponsePath::new(#field_name_str) };

    match type_name.as_str() {
        "String" => quote! {
            /// Get the value of this field from responses, if present.
            pub fn #method_name(responses: &derive_survey::Responses) -> Option<String> {
                responses.get_string(&#path_expr).ok().map(|s| s.to_string())
            }
        },
        "bool" => quote! {
            /// Get the value of this field from responses, if present.
            pub fn #method_name(responses: &derive_survey::Responses) -> Option<bool> {
                responses.get_bool(&#path_expr).ok()
            }
        },
        "i8" | "i16" | "i32" | "i64" | "isize" => quote! {
            /// Get the value of this field from responses, if present.
            pub fn #method_name(responses: &derive_survey::Responses) -> Option<#ty> {
                responses.get_int(&#path_expr).ok().map(|n| n as #ty)
            }
        },
        "u8" | "u16" | "u32" | "u64" | "usize" => quote! {
            /// Get the value of this field from responses, if present.
            pub fn #method_name(responses: &derive_survey::Responses) -> Option<#ty> {
                responses.get_int(&#path_expr).ok().map(|n| n as #ty)
            }
        },
        "f32" | "f64" => quote! {
            /// Get the value of this field from responses, if present.
            pub fn #method_name(responses: &derive_survey::Responses) -> Option<#ty> {
                responses.get_float(&#path_expr).ok().map(|n| n as #ty)
            }
        },
        "PathBuf" => quote! {
            /// Get the value of this field from responses, if present.
            pub fn #method_name(responses: &derive_survey::Responses) -> Option<std::path::PathBuf> {
                responses.get_string(&#path_expr).ok().map(std::path::PathBuf::from)
            }
        },
        _ => {
            // For complex types (nested structs, enums, etc.), we don't generate accessors
            // as they would require more complex handling
            quote! {}
        }
    }
}

// ============================================================================
// ValidationContext Generation
// ============================================================================

/// Generate a ValidationContext struct for accessing sibling fields during validation.
/// This struct wraps Responses and a prefix path, providing typed accessor methods.
fn generate_validation_context(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let name = &input.ident;
    let context_name = format_ident!("{}ValidationContext", name);

    let mut accessors = Vec::new();

    match &input.data {
        Data::Struct(data) => {
            if let Fields::Named(fields) = &data.fields {
                for field in &fields.named {
                    let field_name = field.ident.as_ref().unwrap();
                    let field_name_str = field_name.to_string();
                    let ty = &field.ty;

                    let accessor =
                        generate_context_accessor_method(&field_name_str, field_name, ty);
                    accessors.push(accessor);
                }
            }
        }
        Data::Enum(_) => {
            // Enums don't need field accessors in the same way
        }
        Data::Union(_) => {}
    }

    Ok(quote! {
        /// Validation context for #name, providing access to sibling fields.
        pub struct #context_name<'a> {
            responses: &'a derive_survey::Responses,
            prefix: derive_survey::ResponsePath,
        }

        impl<'a> #context_name<'a> {
            /// Create a new validation context with the given prefix path.
            pub fn new(responses: &'a derive_survey::Responses, prefix: derive_survey::ResponsePath) -> Self {
                Self { responses, prefix }
            }

            /// Get the prefix path for this context.
            pub fn prefix(&self) -> &derive_survey::ResponsePath {
                &self.prefix
            }

            /// Get the underlying responses.
            pub fn responses(&self) -> &derive_survey::Responses {
                self.responses
            }

            #(#accessors)*
        }
    })
}

/// Generate a single accessor method for ValidationContext
fn generate_context_accessor_method(
    field_name_str: &str,
    field_name: &Ident,
    ty: &Type,
) -> TokenStream2 {
    let method_name = format_ident!("get_{}", field_name);
    let type_name = type_to_string(ty);

    match type_name.as_str() {
        "String" => quote! {
            /// Get the value of this field from responses, if present.
            pub fn #method_name(&self) -> Option<String> {
                let path = self.prefix.child(#field_name_str);
                self.responses.get_string(&path).ok().map(|s| s.to_string())
            }
        },
        "bool" => quote! {
            /// Get the value of this field from responses, if present.
            pub fn #method_name(&self) -> Option<bool> {
                let path = self.prefix.child(#field_name_str);
                self.responses.get_bool(&path).ok()
            }
        },
        "i8" | "i16" | "i32" | "i64" | "isize" => quote! {
            /// Get the value of this field from responses, if present.
            pub fn #method_name(&self) -> Option<#ty> {
                let path = self.prefix.child(#field_name_str);
                self.responses.get_int(&path).ok().map(|n| n as #ty)
            }
        },
        "u8" | "u16" | "u32" | "u64" | "usize" => quote! {
            /// Get the value of this field from responses, if present.
            pub fn #method_name(&self) -> Option<#ty> {
                let path = self.prefix.child(#field_name_str);
                self.responses.get_int(&path).ok().map(|n| n as #ty)
            }
        },
        "f32" | "f64" => quote! {
            /// Get the value of this field from responses, if present.
            pub fn #method_name(&self) -> Option<#ty> {
                let path = self.prefix.child(#field_name_str);
                self.responses.get_float(&path).ok().map(|n| n as #ty)
            }
        },
        "PathBuf" => quote! {
            /// Get the value of this field from responses, if present.
            pub fn #method_name(&self) -> Option<std::path::PathBuf> {
                let path = self.prefix.child(#field_name_str);
                self.responses.get_string(&path).ok().map(std::path::PathBuf::from)
            }
        },
        _ => {
            // For complex types, don't generate accessors
            quote! {}
        }
    }
}

// ============================================================================
// Validation Generation
// ============================================================================

fn generate_validate_field_fn(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let mut validators = Vec::new();

    // Note: min/max validation is NOT generated here because:
    // 1. It's stored in IntQuestion/FloatQuestion and handled by backends directly
    // 2. This function runs ALL validators for ALL fields, so field-specific
    //    min/max checks would incorrectly apply to other fields

    // Add the propagated validator (from #[validate_fields]) if present
    // This runs for ALL fields (e.g., cross-field stat validation)
    let type_attrs = TypeAttrs::extract(&input.attrs)?;
    if let Some(validator) = &type_attrs.validate_fields {
        validators.push(quote! {
            if let Err(e) = #validator(value, responses, path) {
                return Err(e);
            }
        });
    }

    // Helper to check if a path matches a field name
    // The path could be "field_name" or "parent.field_name" etc.
    let path_matches_field = |field_name: &str| -> TokenStream2 {
        quote! {
            (path.as_str() == #field_name || path.as_str().ends_with(&format!(".{}", #field_name)))
        }
    };

    match &input.data {
        Data::Struct(data) => {
            if let Fields::Named(fields) = &data.fields {
                for field in &fields.named {
                    let attrs = FieldAttrs::extract(&field.attrs)?;
                    let ty = &field.ty;
                    let field_name = field.ident.as_ref().unwrap().to_string();

                    if let Some(validator) = &attrs.validate {
                        let path_check = path_matches_field(&field_name);
                        // Only run this validator if the path matches this field
                        validators.push(quote! {
                            if #path_check {
                                if let Err(e) = #validator(value, responses, path) {
                                    return Err(e);
                                }
                            }
                        });
                    }

                    // Delegate to nested Survey types for validation
                    let type_name = type_to_string(ty);
                    let is_primitive = matches!(
                        type_name.as_str(),
                        "String"
                            | "&str"
                            | "bool"
                            | "i8"
                            | "i16"
                            | "i32"
                            | "i64"
                            | "isize"
                            | "u8"
                            | "u16"
                            | "u32"
                            | "u64"
                            | "usize"
                            | "f32"
                            | "f64"
                            | "PathBuf"
                    );

                    // Skip Vec types (they're handled differently) and primitives
                    if !is_primitive
                        && extract_vec_inner_type(ty).is_none()
                        && extract_option_inner_type(ty).is_none()
                    {
                        validators.push(quote! {
                            // Delegate validation to nested Survey type
                            if let Err(e) = <#ty as derive_survey::Survey>::validate_field(value, responses, path) {
                                return Err(e);
                            }
                        });
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
                            let field_name = field.ident.as_ref().unwrap().to_string();

                            if let Some(validator) = &attrs.validate {
                                let path_check = path_matches_field(&field_name);
                                validators.push(quote! {
                                    if #path_check {
                                        if let Err(e) = #validator(value, responses, path) {
                                            return Err(e);
                                        }
                                    }
                                });
                            }
                        }
                    }
                    Fields::Unnamed(fields) => {
                        // For unnamed fields (tuple variants), use the variant name as identifier
                        let variant_name = variant.ident.to_string();
                        for (idx, field) in fields.unnamed.iter().enumerate() {
                            let attrs = FieldAttrs::extract(&field.attrs)?;
                            // For tuple variants, the path ends with the variant name
                            let field_name = if fields.unnamed.len() == 1 {
                                variant_name.clone()
                            } else {
                                format!("{}.{}", variant_name, idx)
                            };

                            if let Some(validator) = &attrs.validate {
                                let path_check = path_matches_field(&field_name);
                                validators.push(quote! {
                                    if #path_check {
                                        if let Err(e) = #validator(value, responses, path) {
                                            return Err(e);
                                        }
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

    // Check propagated field validator (validate_fields)
    if let Some(validator) = &type_attrs.validate_fields {
        checks.push(quote! {
            const _: fn(&derive_survey::ResponseValue, &derive_survey::Responses, &derive_survey::ResponsePath) -> Result<(), String> = #validator;
        });
    }

    // Check field validators
    let check_field = |field: &syn::Field, checks: &mut Vec<TokenStream2>| -> syn::Result<()> {
        let attrs = FieldAttrs::extract(&field.attrs)?;
        if let Some(validator) = &attrs.validate {
            checks.push(quote! {
                const _: fn(&derive_survey::ResponseValue, &derive_survey::Responses, &derive_survey::ResponsePath) -> Result<(), String> = #validator;
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

    // Generate the SuggestBuilder for this type
    let suggest_builder = generate_suggest_builder(input)?;

    // Generate Option builders for any Option<T> fields
    let option_builders = collect_option_builders(input);

    // Collect all fields for suggest/assume methods on the main builder
    let mut suggest_methods = Vec::new();
    let mut assume_methods = Vec::new();

    generate_builder_methods_for_type(
        input,
        "", // root prefix
        &mut suggest_methods,
        &mut assume_methods,
    )?;

    // Generate with_suggestions body
    let with_suggestions_body = generate_with_suggestions_body(input);

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
                #with_suggestions_body
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
                    &|value, responses, path| #name::validate_field(value, responses, path),
                ).map_err(Into::into)?;

                // Reconstruct the type
                Ok(#name::from_responses(&responses))
            }

            fn apply_to_definition(&self, definition: &mut derive_survey::SurveyDefinition) {
                for question in &mut definition.questions {
                    self.apply_to_question(question, "");
                }
            }

            fn apply_to_question(&self, question: &mut derive_survey::Question, parent_prefix: &str) {
                let path_str = if parent_prefix.is_empty() {
                    question.path().as_str().to_string()
                } else if question.path().as_str().is_empty() {
                    parent_prefix.to_string()
                } else {
                    format!("{}.{}", parent_prefix, question.path().as_str())
                };

                // Handle is_none marker for Option fields (assumptions only)
                let none_key = format!("{}.is_none", path_str);
                if let Some(derive_survey::ResponseValue::Bool(true)) = self.assumptions.get(&none_key) {
                    // For assumed None, skip this question entirely
                    question.set_assumption(derive_survey::ResponseValue::Bool(false));
                    return;
                }
                if let Some(derive_survey::ResponseValue::Bool(true)) = self.suggestions.get(&none_key) {
                    // For suggested None, set a suggestion marker (backend handles this)
                    question.set_suggestion(derive_survey::ResponseValue::Bool(false));
                }

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
                            self.apply_to_question(q, &path_str);
                        }
                    }
                    derive_survey::QuestionKind::OneOf(one_of) => {
                        // Handle selected_variant for enums
                        let variant_key = format!("{}.selected_variant", path_str);
                        if let Some(derive_survey::ResponseValue::ChosenVariant(idx)) = self.assumptions.get(&variant_key) {
                            one_of.default = Some(*idx);
                        } else if let Some(derive_survey::ResponseValue::ChosenVariant(idx)) = self.suggestions.get(&variant_key) {
                            one_of.default = Some(*idx);
                        }

                        // Recurse into variant fields
                        for variant in &mut one_of.variants {
                            match &mut variant.kind {
                                derive_survey::QuestionKind::AllOf(all_of) => {
                                    for q in all_of.questions_mut() {
                                        self.apply_to_question(q, &path_str);
                                    }
                                }
                                derive_survey::QuestionKind::Unit => {}
                                other => {
                                    // For newtype variants, create a temporary question wrapper
                                    let mut temp_q = derive_survey::Question::new(
                                        derive_survey::ResponsePath::new("0"),
                                        "",
                                        std::mem::replace(other, derive_survey::QuestionKind::Unit),
                                    );
                                    self.apply_to_question(&mut temp_q, &path_str);
                                    *other = std::mem::replace(temp_q.kind_mut(), derive_survey::QuestionKind::Unit);
                                }
                            }
                        }
                    }
                    derive_survey::QuestionKind::AnyOf(any_of) => {
                        // Handle selected_variants for multi-select
                        let variants_key = format!("{}.selected_variants", path_str);
                        if let Some(derive_survey::ResponseValue::ChosenVariants(indices)) = self.assumptions.get(&variants_key) {
                            any_of.defaults = indices.clone();
                        } else if let Some(derive_survey::ResponseValue::ChosenVariants(indices)) = self.suggestions.get(&variants_key) {
                            any_of.defaults = indices.clone();
                        }

                        for variant in &mut any_of.variants {
                            if let derive_survey::QuestionKind::AllOf(all_of) = &mut variant.kind {
                                for q in all_of.questions_mut() {
                                    self.apply_to_question(q, &path_str);
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

        #suggest_builder

        #(#option_builders)*
    })
}

/// Generate the SuggestBuilder struct and impl for a type.
/// This builder is used within closures to suggest/assume nested fields.
fn generate_suggest_builder(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let name = &input.ident;
    let suggest_builder_name = format_ident!("{}SuggestBuilder", name);

    match &input.data {
        Data::Struct(data) => {
            generate_suggest_builder_for_struct(name, &suggest_builder_name, data)
        }
        Data::Enum(data) => generate_suggest_builder_for_enum(name, &suggest_builder_name, data),
        Data::Union(_) => Ok(quote! {}),
    }
}

/// Generate SuggestBuilder for a struct type
fn generate_suggest_builder_for_struct(
    _name: &Ident,
    suggest_builder_name: &Ident,
    data: &syn::DataStruct,
) -> syn::Result<TokenStream2> {
    let mut field_methods = Vec::new();

    if let Fields::Named(fields) = &data.fields {
        for field in &fields.named {
            let field_name = field.ident.as_ref().unwrap();
            let field_name_str = field_name.to_string();
            let ty = &field.ty;

            let method = generate_suggest_builder_field_method(&field_name_str, ty)?;
            field_methods.push(method);
        }
    } else if let Fields::Unnamed(fields) = &data.fields {
        for (i, field) in fields.unnamed.iter().enumerate() {
            let field_name_str = i.to_string();
            let ty = &field.ty;

            let method = generate_suggest_builder_field_method(&field_name_str, ty)?;
            field_methods.push(method);
        }
    }

    Ok(quote! {
        /// Builder for suggesting/assuming values for nested fields
        pub struct #suggest_builder_name<'a> {
            map: &'a mut std::collections::HashMap<String, derive_survey::ResponseValue>,
            prefix: String,
        }

        impl<'a> #suggest_builder_name<'a> {
            fn new(
                map: &'a mut std::collections::HashMap<String, derive_survey::ResponseValue>,
                prefix: String,
            ) -> Self {
                Self { map, prefix }
            }

            fn path(&self, field: &str) -> String {
                if self.prefix.is_empty() {
                    field.to_string()
                } else {
                    format!("{}.{}", self.prefix, field)
                }
            }

            #(#field_methods)*
        }
    })
}

/// Generate SuggestBuilder for an enum type
fn generate_suggest_builder_for_enum(
    name: &Ident,
    suggest_builder_name: &Ident,
    data: &syn::DataEnum,
) -> syn::Result<TokenStream2> {
    let mut select_methods = Vec::new();
    let mut variant_methods = Vec::new();
    let mut variant_builders = Vec::new();

    for (idx, variant) in data.variants.iter().enumerate() {
        let variant_name = &variant.ident;
        let variant_snake = to_snake_case(&variant_name.to_string());
        let select_method_name = format_ident!("suggest_{}", variant_snake);

        // Generate suggest_<variant>() method to pre-select this variant
        select_methods.push(quote! {
            /// Pre-select this variant as the suggested default choice
            pub fn #select_method_name(self) -> Self {
                self.map.insert(
                    format!("{}.selected_variant", self.prefix),
                    derive_survey::ResponseValue::ChosenVariant(#idx),
                );
                self
            }
        });

        // Generate <variant>(closure) method for variants with fields
        match &variant.fields {
            Fields::Named(fields) if !fields.named.is_empty() => {
                let variant_builder_name = format_ident!("{}{}SuggestBuilder", name, variant_name);
                let method_name = format_ident!("{}", variant_snake);

                variant_methods.push(quote! {
                    /// Suggest values for this variant's fields
                    pub fn #method_name<F>(self, f: F) -> Self
                    where
                        F: FnOnce(#variant_builder_name<'_>) -> #variant_builder_name<'_>,
                    {
                        let builder = #variant_builder_name::new(self.map, self.prefix.clone());
                        f(builder);
                        self
                    }
                });

                // Generate the variant's field builder
                let mut field_methods = Vec::new();
                for field in &fields.named {
                    let field_name = field.ident.as_ref().unwrap();
                    let field_name_str = field_name.to_string();
                    let ty = &field.ty;

                    let method = generate_suggest_builder_field_method(&field_name_str, ty)?;
                    field_methods.push(method);
                }

                variant_builders.push(quote! {
                    /// Builder for suggesting values for variant fields
                    pub struct #variant_builder_name<'a> {
                        map: &'a mut std::collections::HashMap<String, derive_survey::ResponseValue>,
                        prefix: String,
                    }

                    impl<'a> #variant_builder_name<'a> {
                        fn new(
                            map: &'a mut std::collections::HashMap<String, derive_survey::ResponseValue>,
                            prefix: String,
                        ) -> Self {
                            Self { map, prefix }
                        }

                        fn path(&self, field: &str) -> String {
                            if self.prefix.is_empty() {
                                field.to_string()
                            } else {
                                format!("{}.{}", self.prefix, field)
                            }
                        }

                        #(#field_methods)*
                    }
                });
            }
            Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                // Newtype variant - use the inner type's builder directly for complex types
                let field = &fields.unnamed[0];
                let ty = &field.ty;
                let type_name = type_to_string(ty);
                let method_name = format_ident!("{}", variant_snake);

                // Check if it's a primitive type
                let is_primitive = matches!(
                    type_name.as_str(),
                    "String"
                        | "bool"
                        | "i8"
                        | "i16"
                        | "i32"
                        | "i64"
                        | "isize"
                        | "u8"
                        | "u16"
                        | "u32"
                        | "u64"
                        | "usize"
                        | "f32"
                        | "f64"
                        | "PathBuf"
                );

                if is_primitive {
                    // For primitives, generate a direct value method
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
                        _ => unreachable!(),
                    };

                    variant_methods.push(quote! {
                        /// Suggest a value for this newtype variant
                        pub fn #method_name(self, value: #param_type) -> Self {
                            self.map.insert(
                                format!("{}.0", self.prefix),
                                #conversion,
                            );
                            self
                        }
                    });
                } else {
                    // For complex types, use the inner type's builder directly
                    let inner_builder_name = format_ident!("{}SuggestBuilder", type_name);

                    variant_methods.push(quote! {
                        /// Suggest values for this newtype variant's inner type
                        pub fn #method_name<F>(self, f: F) -> Self
                        where
                            F: FnOnce(#inner_builder_name<'_>) -> #inner_builder_name<'_>,
                        {
                            let builder = #inner_builder_name::new(
                                self.map,
                                format!("{}.0", self.prefix),
                            );
                            f(builder);
                            self
                        }
                    });
                }
            }
            Fields::Unnamed(fields) if fields.unnamed.len() > 1 => {
                // Multi-field tuple variant - create intermediate builder
                let variant_builder_name = format_ident!("{}{}SuggestBuilder", name, variant_name);
                let method_name = format_ident!("{}", variant_snake);

                variant_methods.push(quote! {
                    /// Suggest values for this variant's fields
                    pub fn #method_name<F>(self, f: F) -> Self
                    where
                        F: FnOnce(#variant_builder_name<'_>) -> #variant_builder_name<'_>,
                    {
                        let builder = #variant_builder_name::new(self.map, self.prefix.clone());
                        f(builder);
                        self
                    }
                });

                // Generate the variant's field builder for tuple variants
                let mut field_methods = Vec::new();
                for (i, field) in fields.unnamed.iter().enumerate() {
                    let field_name_str = i.to_string();
                    let ty = &field.ty;

                    let method = generate_suggest_builder_field_method(&field_name_str, ty)?;
                    field_methods.push(method);
                }

                variant_builders.push(quote! {
                    /// Builder for suggesting values for variant fields
                    pub struct #variant_builder_name<'a> {
                        map: &'a mut std::collections::HashMap<String, derive_survey::ResponseValue>,
                        prefix: String,
                    }

                    impl<'a> #variant_builder_name<'a> {
                        fn new(
                            map: &'a mut std::collections::HashMap<String, derive_survey::ResponseValue>,
                            prefix: String,
                        ) -> Self {
                            Self { map, prefix }
                        }

                        fn path(&self, field: &str) -> String {
                            if self.prefix.is_empty() {
                                field.to_string()
                            } else {
                                format!("{}.{}", self.prefix, field)
                            }
                        }

                        #(#field_methods)*
                    }
                });
            }
            _ => {
                // Unit variants - no closure method needed
            }
        }
    }

    Ok(quote! {
        /// Builder for suggesting/assuming values for enum variants
        pub struct #suggest_builder_name<'a> {
            map: &'a mut std::collections::HashMap<String, derive_survey::ResponseValue>,
            prefix: String,
        }

        impl<'a> #suggest_builder_name<'a> {
            fn new(
                map: &'a mut std::collections::HashMap<String, derive_survey::ResponseValue>,
                prefix: String,
            ) -> Self {
                Self { map, prefix }
            }

            #(#select_methods)*
            #(#variant_methods)*
        }

        #(#variant_builders)*
    })
}

/// Generate a single field method for a SuggestBuilder
fn generate_suggest_builder_field_method(field_name: &str, ty: &Type) -> syn::Result<TokenStream2> {
    // For numeric field names (tuple structs), prefix with underscore
    let method_name = if field_name
        .chars()
        .next()
        .map(|c| c.is_ascii_digit())
        .unwrap_or(false)
    {
        format_ident!("_{}", field_name)
    } else {
        format_ident!("{}", field_name)
    };
    let type_name = type_to_string(ty);

    // Check for Option<T>
    if let Some(inner_ty) = extract_option_inner_type(ty) {
        return generate_option_suggest_method(field_name, &inner_ty);
    }

    // Skip Vec<T> types - they don't have a simple suggest pattern
    if extract_vec_inner_type(ty).is_some() {
        return Ok(quote! {});
    }

    // Check for primitives
    let (param_type, conversion) = match type_name.as_str() {
        "String" => (
            Some(quote! { impl Into<String> }),
            Some(quote! { derive_survey::ResponseValue::String(value.into()) }),
        ),
        "bool" => (
            Some(quote! { bool }),
            Some(quote! { derive_survey::ResponseValue::Bool(value) }),
        ),
        "i8" | "i16" | "i32" | "i64" | "isize" => (
            Some(quote! { #ty }),
            Some(quote! { derive_survey::ResponseValue::Int(value as i64) }),
        ),
        "u8" | "u16" | "u32" | "u64" | "usize" => (
            Some(quote! { #ty }),
            Some(quote! { derive_survey::ResponseValue::Int(value as i64) }),
        ),
        "f32" | "f64" => (
            Some(quote! { #ty }),
            Some(quote! { derive_survey::ResponseValue::Float(value as f64) }),
        ),
        "PathBuf" => (
            Some(quote! { impl Into<std::path::PathBuf> }),
            Some(
                quote! { derive_survey::ResponseValue::String(value.into().to_string_lossy().into_owned()) },
            ),
        ),
        _ => (None, None), // Complex type - closure-based
    };

    if let (Some(param_type), Some(conversion)) = (param_type, conversion) {
        // Primitive type - direct value method
        Ok(quote! {
            /// Suggest a value for this field
            pub fn #method_name(self, value: #param_type) -> Self {
                self.map.insert(self.path(#field_name), #conversion);
                self
            }
        })
    } else {
        // Complex type - closure-based method
        let inner_builder_name = format_ident!("{}SuggestBuilder", type_name);

        Ok(quote! {
            /// Suggest values for this nested field
            pub fn #method_name<F>(self, f: F) -> Self
            where
                F: FnOnce(#inner_builder_name<'_>) -> #inner_builder_name<'_>,
            {
                let builder = #inner_builder_name::new(self.map, self.path(#field_name));
                f(builder);
                self
            }
        })
    }
}

/// Generate suggest method for Option<T> fields within SuggestBuilder
fn generate_option_suggest_method(field_name: &str, inner_ty: &Type) -> syn::Result<TokenStream2> {
    let method_name = format_ident!("{}", field_name);
    let inner_type_name = type_to_string(inner_ty);
    let option_builder_name =
        format_ident!("Option{}SuggestBuilder", capitalize_first(&inner_type_name));

    Ok(quote! {
        /// Suggest a value for this optional field
        pub fn #method_name<F>(self, f: F) -> Self
        where
            F: FnOnce(#option_builder_name<'_>) -> #option_builder_name<'_>,
        {
            let builder = #option_builder_name::new(self.map, self.path(#field_name));
            f(builder);
            self
        }
    })
}

/// Generate an Option<T> SuggestBuilder type
fn generate_option_builder(inner_ty: &Type) -> TokenStream2 {
    let inner_type_name = type_to_string(inner_ty);
    let option_builder_name =
        format_ident!("Option{}SuggestBuilder", capitalize_first(&inner_type_name));

    // Check if inner type is primitive
    let is_primitive = matches!(
        inner_type_name.as_str(),
        "String"
            | "bool"
            | "i8"
            | "i16"
            | "i32"
            | "i64"
            | "isize"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "usize"
            | "f32"
            | "f64"
            | "PathBuf"
    );

    if is_primitive {
        // For primitive inner types, generate some(value) method
        let (some_param, some_conversion) = match inner_type_name.as_str() {
            "String" => (
                quote! { impl Into<String> },
                quote! { derive_survey::ResponseValue::String(value.into()) },
            ),
            "bool" => (
                quote! { bool },
                quote! { derive_survey::ResponseValue::Bool(value) },
            ),
            "i8" | "i16" | "i32" | "i64" | "isize" => (
                quote! { #inner_ty },
                quote! { derive_survey::ResponseValue::Int(value as i64) },
            ),
            "u8" | "u16" | "u32" | "u64" | "usize" => (
                quote! { #inner_ty },
                quote! { derive_survey::ResponseValue::Int(value as i64) },
            ),
            "f32" | "f64" => (
                quote! { #inner_ty },
                quote! { derive_survey::ResponseValue::Float(value as f64) },
            ),
            "PathBuf" => (
                quote! { impl Into<std::path::PathBuf> },
                quote! { derive_survey::ResponseValue::String(value.into().to_string_lossy().into_owned()) },
            ),
            _ => unreachable!(),
        };

        quote! {
            /// Builder for suggesting Option<T> values
            pub struct #option_builder_name<'a> {
                map: &'a mut std::collections::HashMap<String, derive_survey::ResponseValue>,
                prefix: String,
            }

            impl<'a> #option_builder_name<'a> {
                fn new(
                    map: &'a mut std::collections::HashMap<String, derive_survey::ResponseValue>,
                    prefix: String,
                ) -> Self {
                    Self { map, prefix }
                }

                /// Suggest None (leave empty/skip this field)
                pub fn none(self) -> Self {
                    self.map.insert(
                        format!("{}.is_none", self.prefix),
                        derive_survey::ResponseValue::Bool(true),
                    );
                    self
                }

                /// Suggest Some with a value
                pub fn some(self, value: #some_param) -> Self {
                    self.map.insert(self.prefix.clone(), #some_conversion);
                    self
                }
            }
        }
    } else {
        // For complex inner types, generate some(closure) method
        let inner_builder_name = format_ident!("{}SuggestBuilder", inner_type_name);

        quote! {
            /// Builder for suggesting Option<T> values
            pub struct #option_builder_name<'a> {
                map: &'a mut std::collections::HashMap<String, derive_survey::ResponseValue>,
                prefix: String,
            }

            impl<'a> #option_builder_name<'a> {
                fn new(
                    map: &'a mut std::collections::HashMap<String, derive_survey::ResponseValue>,
                    prefix: String,
                ) -> Self {
                    Self { map, prefix }
                }

                /// Suggest None (leave empty/skip this field)
                pub fn none(self) -> Self {
                    self.map.insert(
                        format!("{}.is_none", self.prefix),
                        derive_survey::ResponseValue::Bool(true),
                    );
                    self
                }

                /// Suggest Some with nested values
                pub fn some<F>(self, f: F) -> Self
                where
                    F: FnOnce(#inner_builder_name<'_>) -> #inner_builder_name<'_>,
                {
                    let builder = #inner_builder_name::new(self.map, self.prefix.clone());
                    f(builder);
                    self
                }
            }
        }
    }
}

/// Collect all Option<T> types used in a struct/enum and generate their builders
fn collect_option_builders(input: &DeriveInput) -> Vec<TokenStream2> {
    let mut option_types: Vec<Type> = Vec::new();
    let mut seen_names: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut builders = Vec::new();

    // Collect all Option types
    match &input.data {
        Data::Struct(data) => {
            collect_option_types_from_fields(&data.fields, &mut option_types, &mut seen_names);
        }
        Data::Enum(data) => {
            for variant in &data.variants {
                collect_option_types_from_fields(
                    &variant.fields,
                    &mut option_types,
                    &mut seen_names,
                );
            }
        }
        Data::Union(_) => {}
    }

    // Generate builders for each unique Option type
    for ty in option_types {
        builders.push(generate_option_builder(&ty));
    }

    builders
}

fn collect_option_types_from_fields(
    fields: &Fields,
    option_types: &mut Vec<Type>,
    seen_names: &mut std::collections::HashSet<String>,
) {
    match fields {
        Fields::Named(fields) => {
            for field in &fields.named {
                if let Some(inner) = extract_option_inner_type(&field.ty) {
                    let name = type_to_string(&inner);
                    if seen_names.insert(name) {
                        option_types.push(inner);
                    }
                }
            }
        }
        Fields::Unnamed(fields) => {
            for field in &fields.unnamed {
                if let Some(inner) = extract_option_inner_type(&field.ty) {
                    let name = type_to_string(&inner);
                    if seen_names.insert(name) {
                        option_types.push(inner);
                    }
                }
            }
        }
        Fields::Unit => {}
    }
}

/// Convert CamelCase to snake_case
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
        } else {
            result.push(c);
        }
    }
    result
}

/// Capitalize first letter
fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
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

    // Skip Vec<T> types - they use multiselect and don't have a simple suggest pattern
    if extract_vec_inner_type(ty).is_some() {
        return;
    }

    // Check for Option<T> - generate closure-based method
    if let Some(inner_ty) = extract_option_inner_type(ty) {
        let inner_type_name = type_to_string(&inner_ty);
        let option_builder_name =
            format_ident!("Option{}SuggestBuilder", capitalize_first(&inner_type_name));

        suggest_methods.push(quote! {
            /// Suggest a value for this optional field (user can modify)
            pub fn #suggest_name<F>(mut self, f: F) -> Self
            where
                F: FnOnce(#option_builder_name<'_>) -> #option_builder_name<'_>,
            {
                let builder = #option_builder_name::new(&mut self.suggestions, #path_key.to_string());
                f(builder);
                self
            }
        });

        assume_methods.push(quote! {
            /// Assume a value for this optional field (question is skipped)
            pub fn #assume_name<F>(mut self, f: F) -> Self
            where
                F: FnOnce(#option_builder_name<'_>) -> #option_builder_name<'_>,
            {
                let builder = #option_builder_name::new(&mut self.assumptions, #path_key.to_string());
                f(builder);
                self
            }
        });
        return;
    }

    // Check for primitive types
    let (param_type, conversion) = match type_name.as_str() {
        "String" => (
            Some(quote! { impl Into<String> }),
            Some(quote! { derive_survey::ResponseValue::String(value.into()) }),
        ),
        "bool" => (
            Some(quote! { bool }),
            Some(quote! { derive_survey::ResponseValue::Bool(value) }),
        ),
        "i8" | "i16" | "i32" | "i64" | "isize" => (
            Some(quote! { #ty }),
            Some(quote! { derive_survey::ResponseValue::Int(value as i64) }),
        ),
        "u8" | "u16" | "u32" | "u64" | "usize" => (
            Some(quote! { #ty }),
            Some(quote! { derive_survey::ResponseValue::Int(value as i64) }),
        ),
        "f32" | "f64" => (
            Some(quote! { #ty }),
            Some(quote! { derive_survey::ResponseValue::Float(value as f64) }),
        ),
        "PathBuf" => (
            Some(quote! { impl Into<std::path::PathBuf> }),
            Some(
                quote! { derive_survey::ResponseValue::String(value.into().to_string_lossy().into_owned()) },
            ),
        ),
        _ => (None, None), // Complex type
    };

    if let (Some(param_type), Some(conversion)) = (param_type, conversion) {
        // Primitive type - direct value methods
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
    } else {
        // Complex type - closure-based methods
        let inner_builder_name = format_ident!("{}SuggestBuilder", type_name);

        suggest_methods.push(quote! {
            /// Suggest values for this nested field (user can modify)
            pub fn #suggest_name<F>(mut self, f: F) -> Self
            where
                F: FnOnce(#inner_builder_name<'_>) -> #inner_builder_name<'_>,
            {
                let builder = #inner_builder_name::new(&mut self.suggestions, #path_key.to_string());
                f(builder);
                self
            }
        });

        assume_methods.push(quote! {
            /// Assume values for this nested field (questions are skipped)
            pub fn #assume_name<F>(mut self, f: F) -> Self
            where
                F: FnOnce(#inner_builder_name<'_>) -> #inner_builder_name<'_>,
            {
                let builder = #inner_builder_name::new(&mut self.assumptions, #path_key.to_string());
                f(builder);
                self
            }
        });
    }
}

/// Generate the with_suggestions body for a struct
fn generate_with_suggestions_struct(data: &syn::DataStruct) -> TokenStream2 {
    match &data.fields {
        Fields::Named(fields) => {
            let insertions: Vec<_> = fields
                .named
                .iter()
                .filter_map(|f| {
                    let field_name = f.ident.as_ref()?;
                    let field_name_str = field_name.to_string();
                    let ty = &f.ty;
                    let type_name = type_to_string(ty);

                    // Only handle primitive types directly
                    match type_name.as_str() {
                        "String" => Some(quote! {
                            self.suggestions.insert(
                                #field_name_str.to_string(),
                                derive_survey::ResponseValue::String(instance.#field_name.clone())
                            );
                        }),
                        "bool" => Some(quote! {
                            self.suggestions.insert(
                                #field_name_str.to_string(),
                                derive_survey::ResponseValue::Bool(instance.#field_name)
                            );
                        }),
                        "i8" | "i16" | "i32" | "i64" | "isize" => Some(quote! {
                            self.suggestions.insert(
                                #field_name_str.to_string(),
                                derive_survey::ResponseValue::Int(instance.#field_name as i64)
                            );
                        }),
                        "u8" | "u16" | "u32" | "u64" | "usize" => Some(quote! {
                            self.suggestions.insert(
                                #field_name_str.to_string(),
                                derive_survey::ResponseValue::Int(instance.#field_name as i64)
                            );
                        }),
                        "f32" | "f64" => Some(quote! {
                            self.suggestions.insert(
                                #field_name_str.to_string(),
                                derive_survey::ResponseValue::Float(instance.#field_name as f64)
                            );
                        }),
                        "PathBuf" => Some(quote! {
                            self.suggestions.insert(
                                #field_name_str.to_string(),
                                derive_survey::ResponseValue::String(instance.#field_name.display().to_string())
                            );
                        }),
                        _ => None, // Skip complex types
                    }
                })
                .collect();

            quote! { #(#insertions)* }
        }
        Fields::Unnamed(fields) => {
            let insertions: Vec<_> = fields
                .unnamed
                .iter()
                .enumerate()
                .filter_map(|(i, f)| {
                    let idx = syn::Index::from(i);
                    let field_name_str = i.to_string();
                    let ty = &f.ty;
                    let type_name = type_to_string(ty);

                    match type_name.as_str() {
                        "String" => Some(quote! {
                            self.suggestions.insert(
                                #field_name_str.to_string(),
                                derive_survey::ResponseValue::String(instance.#idx.clone())
                            );
                        }),
                        "bool" => Some(quote! {
                            self.suggestions.insert(
                                #field_name_str.to_string(),
                                derive_survey::ResponseValue::Bool(instance.#idx)
                            );
                        }),
                        "i8" | "i16" | "i32" | "i64" | "isize" => Some(quote! {
                            self.suggestions.insert(
                                #field_name_str.to_string(),
                                derive_survey::ResponseValue::Int(instance.#idx as i64)
                            );
                        }),
                        "u8" | "u16" | "u32" | "u64" | "usize" => Some(quote! {
                            self.suggestions.insert(
                                #field_name_str.to_string(),
                                derive_survey::ResponseValue::Int(instance.#idx as i64)
                            );
                        }),
                        "f32" | "f64" => Some(quote! {
                            self.suggestions.insert(
                                #field_name_str.to_string(),
                                derive_survey::ResponseValue::Float(instance.#idx as f64)
                            );
                        }),
                        _ => None,
                    }
                })
                .collect();

            quote! { #(#insertions)* }
        }
        Fields::Unit => quote! {},
    }
}

/// Generate with_suggestions body based on the input type
fn generate_with_suggestions_body(input: &DeriveInput) -> TokenStream2 {
    match &input.data {
        Data::Struct(data) => generate_with_suggestions_struct(data),
        Data::Enum(_) => {
            // Enums are complex - skip for now
            quote! {}
        }
        Data::Union(_) => quote! {},
    }
}
