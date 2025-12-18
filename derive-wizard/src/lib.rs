pub use derive_wizard_macro::*;
pub use requestty::Question;
pub use requestty::prompt_one;
pub use requestty::{ListItem, ExpandItem};

pub trait Wizard {
    fn wizard() -> Self;
}
