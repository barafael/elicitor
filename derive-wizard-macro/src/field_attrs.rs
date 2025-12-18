use syn::{Meta, spanned::Spanned};

use crate::{PromptAttr, error::WizardError};

pub struct FieldAttrs {
    pub prompt: PromptAttr,
    pub mask: bool,
    pub editor: bool,
    pub validate_on_submit: Option<proc_macro2::TokenStream>,
    pub validate_on_key: Option<proc_macro2::TokenStream>,
}

impl FieldAttrs {
    pub fn parse(field: &syn::Field) -> Result<Self, (WizardError, proc_macro2::Span)> {
        let mut prompt = PromptAttr::None;
        let mut mask = false;
        let mut editor = false;
        let mut validate_on_submit = None;
        let mut validate_on_key = None;

        for attr in &field.attrs {
            if attr.path().is_ident("prompt") {
                prompt = match &attr.meta {
                    Meta::Path(_) => PromptAttr::Wizard,
                    Meta::List(list) => PromptAttr::WizardWithMessage(list.tokens.clone()),
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
            }
        }

        if mask && editor {
            return Err((WizardError::ConflictingAttributes, field.span()));
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
