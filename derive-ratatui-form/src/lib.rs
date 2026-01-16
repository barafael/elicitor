//! # derive-ratatui-form
//!
//! Ratatui form backend for derive-survey.
//!
//! This backend displays all survey fields at once in a scrollable TUI form,
//! similar to the egui backend but for the terminal. Users can navigate
//! between fields using Tab/Shift+Tab or arrow keys.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use derive_survey::Survey;
//! use derive_ratatui_form::RatatuiFormBackend;
//!
//! #[derive(Survey)]
//! struct UserProfile {
//!     #[ask("What is your name?")]
//!     name: String,
//!
//!     #[ask("How old are you?")]
//!     #[min(0)]
//!     #[max(150)]
//!     age: i64,
//! }
//!
//! fn main() -> anyhow::Result<()> {
//!     let backend = RatatuiFormBackend::new();
//!     let result = UserProfile::builder().run(backend)?;
//!     println!("{result:#?}");
//!     Ok(())
//! }
//! ```

mod backend;

pub use backend::{RatatuiFormBackend, RatatuiFormError, Theme};
