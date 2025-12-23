use syn::{Meta, spanned::Spanned};

use crate::{PromptAttributes, error::WizardError};

pub struct FieldAttributes {
    pub prompt: PromptAttributes,
    pub mask: bool,
    pub editor: bool,
    pub validate_on_submit: Option<proc_macro2::TokenStream>,
    pub validate_on_key: Option<proc_macro2::TokenStream>,
}

impl FieldAttributes {
    pub fn parse(field: &syn::Field) -> Result<Self, (WizardError, proc_macro2::Span)> {
        let mut prompt = PromptAttributes::None;
        let mut mask = false;
        let mut editor = false;
        let mut validate_on_submit = None;
        let mut validate_on_key = None;
        let mut validate = None;

        for attr in &field.attrs {
            if attr.path().is_ident("prompt") {
                prompt = match &attr.meta {
                    Meta::Path(_) => PromptAttributes::Wizard,
                    Meta::List(list) => PromptAttributes::WizardWithMessage(list.tokens.clone()),
                    Meta::NameValue(_) => {
                        return Err((WizardError::InvalidPromptAttribute, attr.span()));
                    }
                };
            } else if attr.path().is_ident("mask") {
                mask = true;
            } else if attr.path().is_ident("editor") {
                editor = true;
            } else if attr.path().is_ident("validate_on_submit") {
                validate_on_submit = match &attr.meta {
                    Meta::List(list) => {
                        // Parse the string literal and extract the function name
                        let lit: syn::LitStr = syn::parse2(list.tokens.clone())
                            .map_err(|_| (WizardError::InvalidValidateAttribute, attr.span()))?;
                        let func_name = lit.value();
                        let ident = syn::Ident::new(&func_name, lit.span());
                        Some(quote::quote! { #ident })
                    }
                    _ => return Err((WizardError::InvalidValidateAttribute, attr.span())),
                };
            } else if attr.path().is_ident("validate_on_key") {
                validate_on_key = match &attr.meta {
                    Meta::List(list) => {
                        // Parse the string literal and extract the function name
                        let lit: syn::LitStr = syn::parse2(list.tokens.clone())
                            .map_err(|_| (WizardError::InvalidValidateAttribute, attr.span()))?;
                        let func_name = lit.value();
                        let ident = syn::Ident::new(&func_name, lit.span());
                        Some(quote::quote! { #ident })
                    }
                    _ => return Err((WizardError::InvalidValidateAttribute, attr.span())),
                };
            } else if attr.path().is_ident("validate") {
                let validator_fn = match &attr.meta {
                    Meta::List(list) => {
                        let lit: syn::LitStr = syn::parse2(list.tokens.clone())
                            .map_err(|_| (WizardError::InvalidValidateAttribute, attr.span()))?;
                        let func_name = lit.value();
                        let ident = syn::Ident::new(&func_name, lit.span());
                        Some(quote::quote! {#ident})
                    }
                    _ => return Err((WizardError::InvalidValidateAttribute, attr.span())),
                };
                validate = Some((validator_fn, attr.span()));
            }
        }

        if let Some((validator_fn, span)) = validate {
            if validate_on_key.is_some() || validate_on_submit.is_some() {
                return Err((WizardError::AmbiguousValidation, span));
            }
            validate_on_key = validator_fn.clone();
            validate_on_submit = validator_fn;
        }

        if mask && editor {
            return Err((WizardError::BothMaskAndEditorSpecified, field.span()));
        }

        Ok(Self {
            prompt,
            mask,
            editor,
            validate_on_submit,
            validate_on_key,
        })
    }
}
