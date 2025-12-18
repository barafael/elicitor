use proc_macro2::TokenStream;
use quote::quote;
use syn::{Meta, parse_macro_input, spanned::Spanned};

struct FieldAttrs {
    prompt: Option<Option<TokenStream>>, // None = no attr, Some(None) = #[prompt], Some(Some(tokens)) = #[prompt("...")]
    mask: bool,
    editor: bool,
}

impl FieldAttrs {
    fn parse(field: &syn::Field) -> syn::Result<Self> {
        let mut prompt = None;
        let mut mask = false;
        let mut editor = false;

        for attr in &field.attrs {
            if attr.path().is_ident("prompt") {
                prompt = Some(match &attr.meta {
                    Meta::Path(_) => None, // #[prompt] without message
                    Meta::List(list) => Some(list.tokens.clone()), // #[prompt("...")]
                    _ => return Err(syn::Error::new_spanned(attr, "Expected #[prompt] or #[prompt(\"...\")]")),
                });
            } else if attr.path().is_ident("mask") {
                mask = true;
            } else if attr.path().is_ident("editor") {
                editor = true;
            }
        }

        if mask && editor {
            return Err(syn::Error::new(field.span(), 
                "Cannot use both #[mask] and #[editor] - they are mutually exclusive"));
        }

        Ok(Self { prompt, mask, editor })
    }
}

#[proc_macro_derive(Wizard, attributes(prompt, mask, editor))]
pub fn wizard_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input);
    let ast = implement_wizard(&input);
    proc_macro::TokenStream::from(ast)
}

fn implement_wizard(input: &syn::DeriveInput) -> TokenStream {
    let name = &input.ident;

    match input.data {
        syn::Data::Struct(ref data_struct) => implement_struct_wizard(name, data_struct),
        syn::Data::Enum(ref data_enum) => implement_enum_wizard(name, data_enum),
        _ => syn::Error::new_spanned(name, "Wizard can only be derived for structs and enums")
            .to_compile_error(),
    }
}

fn implement_struct_wizard(name: &syn::Ident, data_struct: &syn::DataStruct) -> TokenStream {
    let field_info: Result<Vec<_>, _> = data_struct.fields.iter().map(|field| {
        let attrs = FieldAttrs::parse(field)?;
        let ident = field.ident.as_ref().ok_or_else(|| 
            syn::Error::new(field.span(), "Field must have a name"))?;
        
        if attrs.prompt.is_none() {
            return Err(syn::Error::new(field.span(), "Missing required #[prompt(\"...\")] or #[prompt] attribute"));
        }
        
        Ok((ident.clone(), attrs, field.ty.clone()))
    }).collect();

    let field_info = match field_info {
        Ok(info) => info,
        Err(e) => return e.to_compile_error(),
    };

    let (questions, prompts, field_idents): (Vec<_>, Vec<_>, Vec<_>) = field_info
        .into_iter()
        .map(|(ident, attrs, ty)| {
            match attrs.prompt {
                Some(None) => {
                    // #[prompt] without message - nested wizard
                    let wizard_call = quote! { let #ident = <#ty>::wizard(); };
                    (None, wizard_call, ident)
                }
                Some(Some(prompt_text)) => {
                    // #[prompt("...")] - check if it's a promptable type or wizard type
                    if is_rust_primitive_or_string(&ty) {
                        // Regular promptable field
                        let field_name = ident.to_string();
                        let question_type = infer_question_type(&ty, attrs.mask, attrs.editor);
                        let into = infer_into(&ty);
                        
                        let question_def = quote! { let #ident = Question::#question_type(#field_name).message(#prompt_text).build(); };
                        let prompt_def = quote! { let #ident = prompt_one(#ident).unwrap() #into; };
                        
                        (Some(question_def), prompt_def, ident)
                    } else {
                        // Custom type with message - call wizard_with_message
                        let wizard_call = quote! { let #ident = <#ty>::wizard_with_message(#prompt_text); };
                        (None, wizard_call, ident)
                    }
                }
                None => unreachable!(), // Already validated above
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

fn is_rust_primitive_or_string(ty: &syn::Type) -> bool {
    const PRIMITIVES: &[&str] = &[
        "String", "bool", "u8", "u16", "u32", "u64", "u128", "usize",
        "i8", "i16", "i32", "i64", "i128", "isize", "f32", "f64", "char", "PathBuf",
    ];
    
    matches!(ty, syn::Type::Path(type_path) 
        if type_path.path.segments.last()
            .map_or(false, |s| PRIMITIVES.contains(&s.ident.to_string().as_str())))
}

fn process_enum_field(
    field: &syn::Field,
    index: usize,
    variant_name: &str,
) -> syn::Result<(syn::Ident, TokenStream)> {
    let field_ident = field.ident.clone().unwrap_or_else(|| 
        syn::Ident::new(&format!("field_{}", index), field.span()));
    
    let attrs = FieldAttrs::parse(field)?;
    let ty = &field.ty;

    match attrs.prompt {
        None => {
            Err(syn::Error::new(field.span(), 
                "Missing required #[prompt(\"...\")] or #[prompt] attribute"))
        }
        Some(None) => {
            // #[prompt] without message - nested wizard
            Ok((field_ident.clone(), quote! { 
                let #field_ident = <#ty>::wizard(); 
            }))
        }
        Some(Some(prompt_text)) => {
            let field_name = field.ident.as_ref()
                .map(|id| id.to_string())
                .unwrap_or_else(|| format!("{} field {}", variant_name, index));
            
            if is_rust_primitive_or_string(ty) {
                // Regular promptable field
                let question_type = infer_question_type(ty, attrs.mask, attrs.editor);
                let into = infer_into(ty);

                Ok((field_ident.clone(), quote! {
                    let #field_ident = Question::#question_type(#field_name).message(#prompt_text).build();
                    let #field_ident = prompt_one(#field_ident).unwrap() #into;
                }))
            } else {
                // Custom type with message - call wizard_with_message
                Ok((field_ident.clone(), quote! {
                    let #field_ident = <#ty>::wizard_with_message(#prompt_text);
                }))
            }
        }
    }
}

fn implement_enum_wizard(name: &syn::Ident, data_enum: &syn::DataEnum) -> TokenStream {
    let variant_names: Vec<_> = data_enum.variants.iter()
        .map(|v| v.ident.to_string())
        .collect();

    let match_arms: Result<Vec<_>, syn::Error> = data_enum.variants.iter().map(|variant| {
        let variant_ident = &variant.ident;
        let variant_name = variant_ident.to_string();

        match &variant.fields {
            syn::Fields::Named(fields) => {
                let field_data: Result<Vec<_>, _> = fields.named.iter()
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
                let field_data: Result<Vec<_>, _> = fields.unnamed.iter()
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
            syn::Fields::Unit => {
                Ok(quote! { #variant_name => #name::#variant_ident })
            }
        }
    }).collect();

    let match_arms = match match_arms {
        Ok(arms) => arms,
        Err(e) => return e.to_compile_error(),
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

fn infer_question_type(ty: &syn::Type, has_mask: bool, has_editor: bool) -> TokenStream {
    if has_editor { return quote! { editor }; }
    if has_mask { return quote! { password }; }

    let type_str = match ty {
        syn::Type::Path(tp) => tp.path.segments.iter()
            .map(|s| s.ident.to_string())
            .collect::<Vec<_>>()
            .join("::"),
        _ => return quote! { input },
    };

    match type_str.as_str() {
        "PathBuf" | "String" => quote! { input },
        "bool" => quote! { confirm },
        "i8" | "i16" | "i32" | "i64" | "i128" | "isize" |
        "u8" | "u16" | "u32" | "u64" | "u128" | "usize" => quote! { int },
        "f32" | "f64" => quote! { float },
        "ListItem" => quote! { select },
        "ExpandItem" => quote! { expand },
        s if s.starts_with("Vec") => quote! { multi_select },
        _ => quote! { input },
    }
}

fn infer_into(ty: &syn::Type) -> TokenStream {
    let type_str = match ty {
        syn::Type::Path(tp) => tp.path.segments.iter()
            .map(|s| s.ident.to_string())
            .collect::<Vec<_>>()
            .join("::"),
        _ => return quote! {},
    };

    match type_str.as_str() {
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
        _ => quote! {},
    }
}