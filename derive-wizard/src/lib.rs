#![doc = include_str!("../../README.md")]

pub use derive_wizard_macro::*;
pub use requestty::{Answers, ExpandItem, ListItem, Question, prompt_one};

pub trait Wizard: Sized {
    fn wizard() -> Self;

    fn wizard_with_message(message: &str) -> Self {
        let _ = message;
        Self::wizard()
    }

    fn wizard_with_defaults(self) -> Self {
        self
    }
}
