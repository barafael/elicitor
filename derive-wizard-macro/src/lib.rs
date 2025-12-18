mod enumeration;
mod error;
mod field_attrs;
mod infer;
mod structure;

use error::WizardError;

use proc_macro2::TokenStream;
use syn::parse_macro_input;

use crate::enumeration::implement_enum_wizard;

#[proc_macro_derive(Wizard, attributes(prompt, mask, editor))]
pub fn wizard_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input);
    let ast = implement_wizard(&input);
    proc_macro::TokenStream::from(ast)
}

fn implement_wizard(input: &syn::DeriveInput) -> TokenStream {
    let name = &input.ident;

    match input.data {
        syn::Data::Struct(ref data_struct) => structure::implement_struct_wizard(name, data_struct),
        syn::Data::Enum(ref data_enum) => implement_enum_wizard(name, data_enum),
        _ => WizardError::UnsupportedDataType.to_compile_error(name.span()),
    }
}


enum PromptAttr {
    None,
    Wizard,
    WizardWithMessage(TokenStream),
}

fn is_primitive(ty: &syn::Type) -> bool {
    const PRIMITIVES: &[&str] = &[
        "bool", "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128",
        "isize", "f32", "f64", "char", "PathBuf",
    ];

    matches!(ty, syn::Type::Path(type_path) 
        if type_path.path.segments.last()
            .is_some_and(|s| PRIMITIVES.contains(&s.ident.to_string().as_str())))
}

fn is_string(ty: &syn::Type) -> bool {
    matches!(ty, syn::Type::Path(type_path) 
        if type_path.path.segments.last()
            .is_some_and(|s| s.ident == "String"))
}
