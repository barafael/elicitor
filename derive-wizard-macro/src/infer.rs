use proc_macro2::TokenStream;
use quote::quote;

pub fn infer_question_type(ty: &syn::Type, has_mask: bool, has_editor: bool) -> TokenStream {
    if has_editor {
        return quote! { editor };
    }
    if has_mask {
        return quote! { password };
    }

    let type_str = match ty {
        syn::Type::Path(tp) => tp
            .path
            .segments
            .iter()
            .map(|s| s.ident.to_string())
            .collect::<Vec<_>>()
            .join("::"),
        _ => return quote! { input },
    };

    match type_str.as_str() {
        "PathBuf" | "String" => quote! { input },
        "bool" => quote! { confirm },
        "i8" | "i16" | "i32" | "i64" | "i128" | "isize" => quote! { int },
        "u8" | "u16" | "u32" | "u64" | "u128" | "usize" => quote! { int },
        "f32" | "f64" => quote! { float },
        "ListItem" => quote! { select },
        "ExpandItem" => quote! { expand },
        s if s.starts_with("Vec") => quote! { multi_select },
        _ => quote! { input },
    }
}

pub fn infer_target_type(ty: &syn::Type) -> Result<TokenStream, crate::WizardError> {
    let syn::Type::Path(tp) = ty else {
        return Err(crate::WizardError::UnsupportedTypeForPrompting);
    };
    let type_str = tp
        .path
        .segments
        .iter()
        .map(|s| s.ident.to_string())
        .collect::<Vec<_>>()
        .join("::");

    let tokens = match type_str.as_str() {
        "PathBuf" => quote! { .try_into_string().map(PathBuf::from).unwrap() },
        "String" => quote! { .try_into_string().unwrap() },
        "bool" => quote! { .try_into_bool().unwrap() },
        "ListItem" => quote! { .try_into_list_item().unwrap() },
        "ExpandItem" => quote! { .try_into_expand_item().unwrap() },
        ty if ty.starts_with("Vec") => quote! { .try_into_list_items().unwrap() },
        ty @ ("i8" | "i16" | "i32" | "i64" | "isize") => {
            let id = syn::Ident::new(ty, proc_macro2::Span::call_site());
            quote! { .try_into_int().unwrap() as #id }
        }
        ty @ ("u8" | "u16" | "u32" | "u64" | "usize") => {
            let id = syn::Ident::new(ty, proc_macro2::Span::call_site());
            quote! { .try_into_int().unwrap() as #id }
        }
        ty @ ("f32" | "f64") => {
            let id = syn::Ident::new(ty, proc_macro2::Span::call_site());
            quote! { .try_into_float().unwrap() as #id }
        }
        _ => return Err(crate::WizardError::UnsupportedTypeForPrompting),
    };
    Ok(tokens)
}
