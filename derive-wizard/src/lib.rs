#![doc = include_str!("../../README.md")]

pub use derive_wizard_macro::*;
pub use requestty::Question;
pub use requestty::prompt_one;
pub use requestty::{ExpandItem, ListItem};

pub trait Wizard: Sized {
    fn wizard() -> Self;

    fn wizard_with_message(message: &str) -> Self {
        let _ = message;
        Self::wizard()
    }
}
