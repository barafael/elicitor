use proc_macro2::TokenStream;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WizardError {
    #[error("Missing required #[prompt(\"...\")] or #[prompt] attribute")]
    MissingPrompt,

    #[error("Expected #[prompt] or #[prompt(\"...\")]")]
    InvalidPromptAttribute,

    #[error("Cannot use both #[mask] and #[editor] - they are mutually exclusive")]
    ConflictingAttributes,

    #[error("Field must have a name")]
    UnnamedField,

    #[error("Wizard can only be derived for structs and enums")]
    UnionsNotSupported,

    #[error("Unsupported type for prompting")]
    UnsupportedTypeForPrompting,

    #[error("Missing required prompt attributes on one or more fields")]
    MissingPromptAttributes,
}

impl WizardError {
    pub fn to_compile_error(&self, span: proc_macro2::Span) -> TokenStream {
        let msg = self.to_string();
        syn::Error::new(span, msg).to_compile_error()
    }
}
