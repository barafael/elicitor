use syn::{Meta, spanned::Spanned};

use crate::{PromptAttr, error::WizardError};

pub struct FieldAttrs {
    pub prompt: PromptAttr,
    pub mask: bool,
    pub editor: bool,
}

impl FieldAttrs {
    pub fn parse(field: &syn::Field) -> Result<Self, (WizardError, proc_macro2::Span)> {
        let mut prompt = PromptAttr::None;
        let mut mask = false;
        let mut editor = false;

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
            }
        }

        if mask && editor {
            return Err((WizardError::ConflictingAttributes, field.span()));
        }

        Ok(Self {
            prompt,
            mask,
            editor,
        })
    }
}
