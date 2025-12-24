use crate::{
    PromptAttributes,
    error::WizardError,
    field_attrs::{self, FieldAttributes},
    infer, is_promptable_type,
};
use proc_macro2::{Literal, TokenStream};
use quote::quote;
use syn::{Field, spanned::Spanned};

pub fn implement_struct_wizard_2(name: &syn::Ident, data_struct: &syn::DataStruct) -> TokenStream {
    for field in &data_struct.fields {
        let field_ident = field.ident.to_owned();
        dbg!(&field_ident);
        let message_attribute = field
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("prompt"));
        let Some(message) = message_attribute else {
            return WizardError::MissingPrompt.to_compile_error(message_attribute.span());
        };
        let prompt_attributes = match &message.meta {
            syn::Meta::Path(_) => PromptAttributes::Wizard,
            syn::Meta::List(meta_list) => {
                let prompt = meta_list.tokens.clone();
                PromptAttributes::WizardWithMessage(prompt)
            }
            syn::Meta::NameValue(_) => {
                return WizardError::InvalidPromptAttribute.to_compile_error(message.span());
            }
        };
        let has_mask = field.attrs.iter().any(|attr| attr.path().is_ident("mask"));
    }
    todo!()
}

pub fn implement_struct_wizard(name: &syn::Ident, data_struct: &syn::DataStruct) -> TokenStream {
    let field_info: Result<Vec<_>, _> = data_struct
        .fields
        .iter()
        .map(|field| {
            let attrs = field_attrs::FieldAttributes::parse(field)?;
            let ident = field
                .ident
                .as_ref()
                .ok_or_else(|| (WizardError::UnnamedField, field.span()))?;

            match attrs.prompt {
                PromptAttributes::None => Err((WizardError::MissingPrompt, field.span())),
                _ => Ok((ident.clone(), attrs, field.ty.clone())),
            }
        })
        .collect();

    let field_info = match field_info {
        Ok(info) => info,
        Err((err, span)) => return err.to_compile_error(span),
    };

    let (questions, prompts, field_idents): (Vec<_>, Vec<_>, Vec<_>) = field_info
        .iter()
        .map(|(ident, attr, ty)| {
            let field_gen = generate_field_code(ident, ty, attr, false).expect("Field attributes");
            (field_gen.question, field_gen.prompt, ident.clone())
        })
        .fold((vec![], vec![], vec![]), |(mut qs, ps, ids), (q, p, id)| {
            if let Some(question) = q {
                qs.push(question);
            }
            (qs, [ps, vec![p]].concat(), [ids, vec![id]].concat())
        });

    let (questions_with_defaults, prompts_with_defaults): (Vec<_>, Vec<_>) = field_info
        .iter()
        .map(|(ident, attr, ty)| {
            let field_gen = generate_field_code(ident, ty, attr, true).expect("Field attributes");
            (field_gen.question, field_gen.prompt)
        })
        .fold((vec![], vec![]), |(mut qs, ps), (q, p)| {
            if let Some(question) = q {
                qs.push(question);
            }
            (qs, [ps, vec![p]].concat())
        });

    quote! {
        impl Wizard for #name {
            fn wizard(backend: impl derive_wizard::Promptable) -> Self {
                use derive_wizard::{Question, prompt_one};
                #(#questions)*
                #(#prompts)*
                Self { #(#field_idents),* }
            }

            fn wizard_with_defaults(self, backend: impl derive_wizard::Promptable) -> Self {
                use derive_wizard::{Question, prompt_one};
                #(#questions_with_defaults)*
                #(#prompts_with_defaults)*
                Self { #(#field_idents),* }
            }
        }
    }
}

fn generate_default_value_code(ident: &syn::Ident, ty: &syn::Type) -> TokenStream {
    // Check the type to determine how to generate the default value
    if let syn::Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            let type_name = segment.ident.to_string();

            match type_name.as_str() {
                "String" => {
                    // For String, check if it's not empty
                    quote! {
                        if !self.#ident.is_empty() {
                            self.#ident.clone()
                        } else {
                            String::new()
                        }
                    }
                }
                "bool" => {
                    quote! { self.#ident }
                }
                "u8" | "u16" | "u32" | "u64" | "u128" | "usize" | "i8" | "i16" | "i32" | "i64"
                | "i128" | "isize" => {
                    // For integers, cast to i64 since requestty uses i64 for int type
                    quote! { self.#ident as i64 }
                }
                "f32" | "f64" => {
                    // For floats, cast to f64 since requestty uses f64 for float type
                    quote! { self.#ident as f64 }
                }
                "char" => {
                    quote! { self.#ident.to_string() }
                }
                "PathBuf" => {
                    quote! { self.#ident.display().to_string() }
                }
                _ => {
                    quote! { self.#ident.to_string() }
                }
            }
        } else {
            quote! { self.#ident.to_string() }
        }
    } else {
        quote! { self.#ident.to_string() }
    }
}

struct FieldCode {
    question: Option<TokenStream>,
    prompt: TokenStream,
}

fn generate_field_code(
    ident: &syn::Ident,
    ty: &syn::Type,
    field_attr: &FieldAttributes,
    use_defaults: bool,
) -> Result<FieldCode, crate::WizardError> {
    let FieldAttributes {
        prompt,
        mask,
        editor,
        validate_on_submit,
        validate_on_key,
    } = field_attr;
    match prompt {
        PromptAttributes::None => Err(crate::WizardError::MissingPromptAttributes),
        PromptAttributes::Wizard => Ok(FieldCode {
            question: None,
            prompt: quote! { let #ident = <#ty>::wizard(backend.clone()); },
        }),
        PromptAttributes::WizardWithMessage(prompt_text) => {
            if is_promptable_type(ty) {
                let field_name = ident.to_string();
                let question_type = infer::infer_question_type(ty, *mask, *editor);
                let into = infer::infer_target_type(ty)?;

                let validation = validate_on_submit.as_ref().map(|validator| {
                    quote! { .validate(#validator) }
                });

                let validation_on_key = validate_on_key.as_ref().map(|validator| {
                    quote! { .validate_on_key(|input, answers| #validator(input, answers).is_ok()) }
                });

                // Only add .default() if:
                // 1. use_defaults is true
                // 2. The question type supports defaults (not password or editor)
                let default_value = if use_defaults && !mask && !editor {
                    Some(generate_default_value_code(ident, ty))
                } else {
                    None
                };

                let default_clause = default_value.as_ref().map(|default_expr| {
                    quote! { .default(#default_expr) }
                });

                Ok(FieldCode {
                    question: Some(quote! {
                        let #ident = Question::#question_type(#field_name)
                            .message(#prompt_text)
                            #default_clause
                            #validation
                            #validation_on_key
                            .build();
                    }),
                    prompt: quote! { let #ident = prompt_one(#ident).unwrap() #into; },
                })
            } else {
                Ok(FieldCode {
                    question: None,
                    prompt: quote! { let #ident = <#ty>::wizard_with_message(#prompt_text, backend); },
                })
            }
        }
    }
}
