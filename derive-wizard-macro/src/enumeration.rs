use crate::{PromptAttr, WizardError, field_attrs, infer, is_primitive, is_string};

use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;

pub fn implement_enum_wizard(name: &syn::Ident, data_enum: &syn::DataEnum) -> TokenStream {
    let variant_names: Vec<_> = data_enum
        .variants
        .iter()
        .map(|v| v.ident.to_string())
        .collect();

    let match_arms: Result<Vec<_>, (WizardError, proc_macro2::Span)> = data_enum
        .variants
        .iter()
        .map(|variant| {
            let variant_ident = &variant.ident;
            let variant_name = variant_ident.to_string();

            match &variant.fields {
                syn::Fields::Named(fields) => {
                    let field_data: Result<Vec<_>, _> = fields
                        .named
                        .iter()
                        .enumerate()
                        .map(|(i, f)| process_enum_field(f, i, &variant_name))
                        .collect();

                    let field_data = field_data?;
                    let (idents, code): (Vec<_>, Vec<_>) = field_data.into_iter().unzip();

                    Ok(quote! {
                        #variant_name => {
                            #(#code)*
                            #name::#variant_ident { #(#idents),* }
                        }
                    })
                }
                syn::Fields::Unnamed(fields) => {
                    let field_data: Result<Vec<_>, _> = fields
                        .unnamed
                        .iter()
                        .enumerate()
                        .map(|(i, f)| process_enum_field(f, i, &variant_name))
                        .collect();

                    let field_data = field_data?;
                    let (idents, code): (Vec<_>, Vec<_>) = field_data.into_iter().unzip();

                    Ok(quote! {
                        #variant_name => {
                            #(#code)*
                            #name::#variant_ident(#(#idents),*)
                        }
                    })
                }
                syn::Fields::Unit => Ok(quote! { #variant_name => #name::#variant_ident }),
            }
        })
        .collect();

    let match_arms = match match_arms {
        Ok(arms) => arms,
        Err((err, span)) => return err.to_compile_error(span),
    };

    quote! {
        impl Wizard for #name {
            fn wizard() -> Self {
                Self::wizard_with_message("Select variant:")
            }

            fn wizard_with_message(message: &str) -> Self {
                use derive_wizard::{Question, prompt_one};

                let variant_question = Question::select("variant")
                    .message(message)
                    .choices(vec![#(#variant_names),*])
                    .build();

                let selected_variant = prompt_one(variant_question)
                    .unwrap()
                    .try_into_list_item()
                    .unwrap();

                match selected_variant.text.as_str() {
                    #(#match_arms,)*
                    _ => unreachable!()
                }
            }
        }
    }
}

fn process_enum_field(
    field: &syn::Field,
    index: usize,
    variant_name: &str,
) -> Result<(syn::Ident, TokenStream), (WizardError, proc_macro2::Span)> {
    let field_ident = field
        .ident
        .clone()
        .unwrap_or_else(|| syn::Ident::new(&format!("field_{}", index), field.span()));

    let attrs = field_attrs::FieldAttrs::parse(field)?;
    let ty = &field.ty;

    match attrs.prompt {
        PromptAttr::None => Err((WizardError::MissingPrompt, field.span())),
        PromptAttr::Wizard => {
            // #[prompt] without message - nested wizard
            Ok((
                field_ident.clone(),
                quote! {
                    let #field_ident = <#ty>::wizard();
                },
            ))
        }
        PromptAttr::WizardWithMessage(prompt_text) => {
            let field_name = field
                .ident
                .as_ref()
                .map(|id| id.to_string())
                .unwrap_or_else(|| format!("{} field {}", variant_name, index));

            if is_primitive(ty) || is_string(ty) {
                // Regular promptable field
                let question_type = infer::infer_question_type(ty, attrs.mask, attrs.editor);
                let into = infer::infer_into(ty);

                Ok((
                    field_ident.clone(),
                    quote! {
                        let #field_ident = Question::#question_type(#field_name).message(#prompt_text).build();
                        let #field_ident = prompt_one(#field_ident).unwrap() #into;
                    },
                ))
            } else {
                // Custom type with message - call wizard_with_message
                Ok((
                    field_ident.clone(),
                    quote! {
                        let #field_ident = <#ty>::wizard_with_message(#prompt_text);
                    },
                ))
            }
        }
    }
}
