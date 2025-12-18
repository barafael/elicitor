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

    match &input.data {
        syn::Data::Struct(data_struct) => structure::implement_struct_wizard(name, data_struct),
        syn::Data::Enum(data_enum) => implement_enum_wizard(name, data_enum),
        syn::Data::Union(_) => WizardError::UnionsNotSupported.to_compile_error(name.span()),
    }
}

enum PromptAttr {
    None,
    Wizard,
    WizardWithMessage(TokenStream),
}

fn is_promptable_type(ty: &syn::Type) -> bool {
    const PROMPTABLE: &[&str] = &[
        "String", "bool", "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64",
        "i128", "isize", "f32", "f64", "char", "PathBuf",
    ];

    matches!(ty, syn::Type::Path(tp)
        if tp.path.segments.last()
            .is_some_and(|s| PROMPTABLE.contains(&s.ident.to_string().as_str())))
}
