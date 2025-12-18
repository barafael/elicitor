use crate::{PromptAttr, WizardError, field_attrs, infer, is_primitive, is_string};
use quote::quote;


use proc_macro2::TokenStream;
use syn::spanned::Spanned;

pub fn implement_struct_wizard(name: &syn::Ident, data_struct: &syn::DataStruct) -> TokenStream {
    let field_info: Result<Vec<_>, (WizardError, proc_macro2::Span)> = data_struct.fields.iter().map(|field| {
        let attrs = field_attrs::FieldAttrs::parse(field)?;
        let ident = field.ident.as_ref()
            .ok_or((WizardError::UnnamedField, field.span()))?;
    
        if matches!(attrs.prompt, PromptAttr::None) {
            return Err((WizardError::MissingPrompt, field.span()));
        }
    
        Ok((ident.clone(), attrs, field.ty.clone()))
    }).collect();

    let field_info = match field_info {
        Ok(info) => info,
        Err((err, span)) => return err.to_compile_error(span),
    };

    let (questions, prompts, field_idents): (Vec<_>, Vec<_>, Vec<_>) = field_info
        .into_iter()
        .map(|(ident, attrs, ty)| {
            match attrs.prompt {
                PromptAttr::Wizard => {
                    // #[prompt] without message - nested wizard
                    let wizard_call = quote! { let #ident = <#ty>::wizard(); };
                    (None, wizard_call, ident)
                }
                PromptAttr::WizardWithMessage(prompt_text) => {
                    // #[prompt("...")] - check if it's a promptable type or wizard type
                    if is_primitive(&ty) || is_string(&ty){
                        // Regular promptable field
                        let field_name = ident.to_string();
                        let question_type = infer::infer_question_type(&ty, attrs.mask, attrs.editor);
                        let into = infer::infer_into(&ty);
                    
                        let question_def = quote! { let #ident = Question::#question_type(#field_name).message(#prompt_text).build(); };
                        let prompt_def = quote! { let #ident = prompt_one(#ident).unwrap() #into; };
                    
                        (Some(question_def), prompt_def, ident)
                    } else {
                        // Custom type with message - call wizard_with_message
                        let wizard_call = quote! { let #ident = <#ty>::wizard_with_message(#prompt_text); };
                        (None, wizard_call, ident)
                    }
                }
                PromptAttr::None => unreachable!(), // Already validated above
            }
        })
        .fold((vec![], vec![], vec![]), |(mut qs, mut ps, mut ids), (q, p, id)| {
            if let Some(question) = q {
                qs.push(question);
            }
            ps.push(p);
            ids.push(id);
            (qs, ps, ids)
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
