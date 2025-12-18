use proc_macro2::TokenStream;
use quote::quote;
use syn::{Meta, parse_macro_input};

#[proc_macro_derive(Wizard, attributes(prompt, mask, editor))]
pub fn wizard_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input);
    let ast = implement_wizard(&input);
    proc_macro::TokenStream::from(ast)
}

fn implement_wizard(input: &syn::DeriveInput) -> TokenStream {
    let name = &input.ident;

    let mut fields = Vec::new();

    match input.data {
        syn::Data::Struct(ref data_struct) => {
            for field in &data_struct.fields {
                let mut prompt_attr = None;
                let mut has_mask = false;
                let mut has_editor = false;

                for attr in &field.attrs {
                    if attr.path().is_ident("prompt") {
                        prompt_attr = Some(attr.clone());
                    } else if attr.path().is_ident("mask") {
                        has_mask = true;
                    } else if attr.path().is_ident("editor") {
                        has_editor = true;
                    }
                }

                if let Some(prompt) = prompt_attr {
                    // Check for mutually exclusive attributes
                    if has_mask && has_editor {
                        return syn::Error::new_spanned(
                            field,
                            "Cannot use both #[mask] and #[editor] on the same field. They are mutually exclusive.",
                        )
                        .to_compile_error();
                    }
                    fields.push((field.clone(), prompt, has_mask, has_editor));
                } else {
                    return syn::Error::new_spanned(
                        field,
                        "Missing required #[prompt(\"...\")] attribute",
                    )
                    .to_compile_error();
                }
            }
        }
        _ => {
            return syn::Error::new_spanned(
                name,
                "Wizard can only be derived for structs (for now)",
            )
            .to_compile_error();
        }
    }

    let mut identifiers = Vec::new();
    for (field, prompt_attribute, has_mask, has_editor) in fields {
        // Parse the prompt attribute to extract the prompt string
        let prompt_text = match &prompt_attribute.meta {
            Meta::List(meta_list) => meta_list.tokens.clone(),
            _ => {
                return syn::Error::new_spanned(prompt_attribute, "Expected #[prompt(\"...\")]")
                    .to_compile_error();
            }
        };

        let field_ident = field.ident.clone().unwrap();
        let field_name = field_ident.to_string();

        // Determine question type - priority: editor > mask > type inference
        let question_type = if has_editor {
            // #[editor] attribute means editor question type
            quote! { editor }
        } else if has_mask {
            // #[mask] attribute means password question type
            quote! { password }
        } else {
            // Infer from type
            match &field.ty {
                syn::Type::Path(type_path) => {
                    let type_str = type_path
                        .path
                        .segments
                        .iter()
                        .map(|seg| seg.ident.to_string())
                        .collect::<Vec<_>>()
                        .join("::");

                    match type_str.as_str() {
                        "PathBuf" => quote! { input },
                        "String" => quote! { input },
                        "bool" => quote! { confirm },
                        "i8" | "i16" | "i32" | "i64" | "isize" => quote! { int },
                        "u8" | "u16" | "u32" | "u64" | "usize" => quote! { int },
                        "f32" | "f64" => quote! { float },
                        "ListItem" => quote! { select },
                        "ExpandItem" => quote! { expand },
                        _ if type_str.starts_with("Vec") => {
                            // Check if it's Vec<ListItem>
                            if let Some(syn::PathSegment {
                                arguments: syn::PathArguments::AngleBracketed(args),
                                ..
                            }) = type_path.path.segments.last()
                            {
                                if let Some(syn::GenericArgument::Type(syn::Type::Path(
                                    inner_type,
                                ))) = args.args.first()
                                {
                                    if let Some(inner_ident) = inner_type.path.get_ident() {
                                        if inner_ident == "ListItem" {
                                            quote! { multi_select }
                                        } else {
                                            return syn::Error::new_spanned(
                                                &field.ty,
                                                "Unsupported Vec type for Wizard derive. Only Vec<ListItem> is supported.",
                                            )
                                            .to_compile_error();
                                        }
                                    } else {
                                        return syn::Error::new_spanned(
                                            &field.ty,
                                            "Unsupported Vec type for Wizard derive. Only Vec<ListItem> is supported.",
                                        )
                                        .to_compile_error();
                                    }
                                } else {
                                    return syn::Error::new_spanned(
                                        &field.ty,
                                        "Unsupported Vec type for Wizard derive. Only Vec<ListItem> is supported.",
                                    )
                                    .to_compile_error();
                                }
                            } else {
                                return syn::Error::new_spanned(
                                    &field.ty,
                                    "Unsupported Vec type for Wizard derive. Only Vec<ListItem> is supported.",
                                )
                                .to_compile_error();
                            }
                        }
                        _ => {
                            return syn::Error::new_spanned(
                                &field.ty,
                                format!("Unsupported field type '{}' for Wizard derive. Supported types: String, bool, i8-i64, u8-u64, f32, f64, ListItem, ExpandItem, Vec<ListItem>", type_str),
                            )
                            .to_compile_error();
                        }
                    }
                }
                _ => {
                    return syn::Error::new_spanned(
                        &field.ty,
                        "Unsupported field type for Wizard derive",
                    )
                    .to_compile_error();
                }
            }
        };
        let question =
            quote::quote! { Question::#question_type(#field_name).message(#prompt_text).build() };
        identifiers.push((field_ident, question, field.ty));
    }

    let questions = identifiers
        .iter()
        .map(|(ident, q, _)| quote::quote! {let #ident = #q;})
        .collect::<TokenStream>();

    let prompts = identifiers
        .iter()
        .map(|(ident, _, t)| {
            let into = infer_into(t);
            quote::quote! {
                let #ident = prompt_one(#ident).unwrap()
                    #into;
            }
        })
        .collect::<TokenStream>();

    let target = identifiers
        .iter()
        .map(|(ident, _, _)| {
            quote::quote! {
                #ident,
            }
        })
        .collect::<TokenStream>();

    let code = quote::quote! {
        use derive_wizard::Question;
        use derive_wizard::prompt_one;

        impl Wizard for #name {
            fn wizard() -> Self {
                #questions

                #prompts

                let result = Self {
                    #target
                };

                result
            }
        }
    };

    code
}

fn infer_into(typ: &syn::Type) -> TokenStream {
    match typ {
        syn::Type::Path(type_path) => {
            let type_str = type_path
                .path
                .segments
                .iter()
                .map(|seg| seg.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");

            match type_str.as_str() {
                "PathBuf" => quote! { .try_into_string().map(PathBuf::from).unwrap() },
                "String" => quote! { .try_into_string().unwrap() },
                "bool" => quote! { .try_into_bool().unwrap() },
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
                "ListItem" => quote! { .try_into_list_item().unwrap() },
                "ExpandItem" => quote! { .try_into_expand_item().unwrap() },
                _ if type_str.starts_with("Vec") => {
                    quote! { .try_into_list_items().unwrap() }
                }
                _ => unimplemented!(),
            }
        }
        _ => unimplemented!(),
    }
}
