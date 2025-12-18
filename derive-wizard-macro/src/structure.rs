use crate::{PromptAttr, error::WizardError, field_attrs, infer, is_promptable_type};
use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;

pub fn implement_struct_wizard(name: &syn::Ident, data_struct: &syn::DataStruct) -> TokenStream {
    let field_info: Result<Vec<_>, _> = data_struct
        .fields
        .iter()
        .map(|field| {
            let attrs = field_attrs::FieldAttrs::parse(field)?;
            let ident = field
                .ident
                .as_ref()
                .ok_or_else(|| (WizardError::UnnamedField, field.span()))?;

            match attrs.prompt {
                PromptAttr::None => Err((WizardError::MissingPrompt, field.span())),
                _ => Ok((ident.clone(), attrs, field.ty.clone())),
            }
        })
        .collect();

    let field_info = match field_info {
        Ok(info) => info,
        Err((err, span)) => return err.to_compile_error(span),
    };

    let (questions, prompts, field_idents): (Vec<_>, Vec<_>, Vec<_>) = field_info
        .into_iter()
        .map(|(ident, attrs, ty)| {
            let field_gen = generate_field_code(
                &ident,
                &ty,
                attrs.prompt,
                attrs.mask,
                attrs.editor,
                attrs.validate_on_submit,
                attrs.validate_on_key,
            )
            .expect("Field attributes");
            (field_gen.question, field_gen.prompt, ident)
        })
        .fold((vec![], vec![], vec![]), |(mut qs, ps, ids), (q, p, id)| {
            if let Some(question) = q {
                qs.push(question);
            }
            (qs, [ps, vec![p]].concat(), [ids, vec![id]].concat())
        });

    quote! {
        impl Wizard for #name {
            fn wizard() -> Self {
                use derive_wizard::{Question, prompt_one};
                #(#questions)*
                #(#prompts)*
                Self { #(#field_idents),* }
            }
        }
    }
}

struct FieldCode {
    question: Option<TokenStream>,
    prompt: TokenStream,
}

fn generate_field_code(
    ident: &syn::Ident,
    ty: &syn::Type,
    prompt_attr: PromptAttr,
    has_mask: bool,
    has_editor: bool,
    validate_on_submit: Option<TokenStream>,
    validate_on_key: Option<TokenStream>,
) -> Result<FieldCode, crate::WizardError> {
    match prompt_attr {
        PromptAttr::None => Err(crate::WizardError::MissingPromptAttributes),
        PromptAttr::Wizard => Ok(FieldCode {
            question: None,
            prompt: quote! { let #ident = <#ty>::wizard(); },
        }),
        PromptAttr::WizardWithMessage(prompt_text) => {
            if is_promptable_type(ty) {
                let field_name = ident.to_string();
                let question_type = infer::infer_question_type(ty, has_mask, has_editor);
                let into = infer::infer_target_type(ty)?;

                let validation = validate_on_submit.as_ref().map(|validator| {
                    quote! { .validate(#validator) }
                });

                let validation_on_key = validate_on_key.as_ref().map(|validator| {
                    quote! { .validate_on_key(|input, answers| #validator(input, answers).is_ok()) }
                });

                Ok(FieldCode {
                    question: Some(quote! {
                        let #ident = Question::#question_type(#field_name)
                            .message(#prompt_text)
                            #validation
                            #validation_on_key
                            .build();
                    }),
                    prompt: quote! { let #ident = prompt_one(#ident).unwrap() #into; },
                })
            } else {
                Ok(FieldCode {
                    question: None,
                    prompt: quote! { let #ident = <#ty>::wizard_with_message(#prompt_text); },
                })
            }
        }
    }
}
