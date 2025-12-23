use crate::{PromptAttributes, WizardError, field_attrs, infer, is_promptable_type};
use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;

pub fn implement_enum_wizard(name: &syn::Ident, data_enum: &syn::DataEnum) -> TokenStream {
    let variant_names: Vec<_> = data_enum
        .variants
        .iter()
        .map(|v| v.ident.to_string())
        .collect();

    let match_arms: Result<Vec<_>, _> = data_enum
        .variants
        .iter()
        .map(|variant| process_variant(name, variant))
        .collect();

    let match_arms = match match_arms {
        Ok(arms) => arms,
        Err((err, span)) => return err.to_compile_error(span),
    };

    quote! {
        impl Wizard for #name {
            fn wizard(backend: impl derive_wizard::Promptable) -> Self {
                Self::wizard_with_message("Select variant:", backend)
            }

            fn wizard_with_message(message: &str, backend: impl derive_wizard::Promptable) -> Self {
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

fn process_variant(
    enum_name: &syn::Ident,
    variant: &syn::Variant,
) -> Result<TokenStream, (WizardError, proc_macro2::Span)> {
    let variant_ident = &variant.ident;
    let variant_name = variant_ident.to_string();

    match &variant.fields {
        syn::Fields::Named(fields) => {
            let (idents, code): (Vec<_>, Vec<_>) = fields
                .named
                .iter()
                .enumerate()
                .map(|(i, f)| process_enum_field(f, i, &variant_name))
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .unzip();

            Ok(quote! {
                #variant_name => {
                    #(#code)*
                    #enum_name::#variant_ident { #(#idents),* }
                }
            })
        }
        syn::Fields::Unnamed(fields) => {
            let (idents, code): (Vec<_>, Vec<_>) = fields
                .unnamed
                .iter()
                .enumerate()
                .map(|(i, f)| process_enum_field(f, i, &variant_name))
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .unzip();

            Ok(quote! {
                #variant_name => {
                    #(#code)*
                    #enum_name::#variant_ident(#(#idents),*)
                }
            })
        }
        syn::Fields::Unit => Ok(quote! { #variant_name => #enum_name::#variant_ident }),
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
        .unwrap_or_else(|| syn::Ident::new(&format!("field_{index}"), field.span()));

    let attrs = field_attrs::FieldAttributes::parse(field)?;

    let code = match attrs.prompt {
        PromptAttributes::None => return Err((WizardError::MissingPrompt, field.span())),
        PromptAttributes::Wizard => quote! { let #field_ident = <#field.ty>::wizard(); },
        PromptAttributes::WizardWithMessage(prompt_text) => {
            let field_name = field.ident.as_ref().map_or_else(
                || format!("{variant_name} field {index}"),
                syn::Ident::to_string,
            );

            if is_promptable_type(&field.ty) {
                let question_type = infer::infer_question_type(&field.ty, attrs.mask, attrs.editor);
                let into = infer::infer_target_type(&field.ty).map_err(|e| (e, field.span()))?;

                let validation = attrs.validate_on_submit.as_ref().map(|validator| {
                    quote! { .validate(#validator) }
                });

                let validation_on_key = attrs.validate_on_key.as_ref().map(|validator| {
                    quote! { .validate_on_key(|input, answers| #validator(input, answers).is_ok()) }
                });

                quote! {
                    let #field_ident = Question::#question_type(#field_name)
                        .message(#prompt_text)
                        #validation
                        #validation_on_key
                        .build();
                    let #field_ident = prompt_one(#field_ident).unwrap() #into;
                }
            } else {
                quote! { let #field_ident = <#field.ty>::wizard_with_message(#prompt_text, backend); }
            }
        }
    };

    Ok((field_ident, code))
}
