use proc_macro2::TokenStream;
use quote::quote;
use syn::{Meta, parse_macro_input};

#[proc_macro_derive(Wizard, attributes(prompt))]
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
                for attr in &field.attrs {
                    if attr.path().is_ident("prompt") {
                        fields.push((field.clone(), attr.clone()));
                    }
                }
            }
        }
        _ => {
            return syn::Error::new_spanned(name, "Wizard can only be derived for structs")
                .to_compile_error();
        }
    }

    let mut identifiers = Vec::new();
    for (field, attribute) in fields {
        // Parse the attribute to extract the prompt string
        let prompt_text = match &attribute.meta {
            Meta::List(meta_list) => meta_list.tokens.clone(),
            _ => {
                return syn::Error::new_spanned(attribute, "Expected #[prompt(\"...\")]")
                    .to_compile_error();
            }
        };

        let field_ident = field.ident.clone().unwrap();
        let field_name = field_ident.to_string();
        let question_type = match &field.ty {
            syn::Type::Path(type_path) => {
                match type_path
                    .path
                    .get_ident()
                    .map(|id| id.to_string())
                    .as_deref()
                {
                    Some("String") => quote! { input },
                    Some("u8") | Some("u16") => quote! { int },
                    _ => {
                        return syn::Error::new_spanned(
                            &field.ty,
                            "Unsupported field type for Wizard derive",
                        )
                        .to_compile_error();
                    }
                }
            }
            _ => unimplemented!(),
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
    println!("Type: {:?}", quote! {#typ});
    match typ {
        syn::Type::Path(type_path) => match type_path
            .path
            .get_ident()
            .map(|id| id.to_string())
            .as_deref()
        {
            Some(ty @ ("u8" | "u16")) => {
                let id = syn::Ident::new(ty, proc_macro2::Span::call_site());
                quote! { .try_into_int().unwrap() as #id }
            }
            Some("String") => quote! { .try_into_string().unwrap() },
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    }
}
