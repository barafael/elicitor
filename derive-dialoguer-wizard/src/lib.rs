//! # derive-dialoguer-wizard
//!
//! Dialoguer wizard backend for derive-survey.
//!
//! This crate provides a command-line wizard interface for collecting survey responses
//! using the `dialoguer` library. Questions are presented step-by-step in a classic
//! CLI wizard style.
//!
//! ## Example
//!
//! ```rust,ignore
//! use derive_survey::Survey;
//! use derive_dialoguer_wizard::DialoguerBackend;
//!
//! #[derive(Survey)]
//! struct User {
//!     #[ask("What is your name?")]
//!     name: String,
//!
//!     #[ask("How old are you?")]
//!     age: i64,
//! }
//!
//! fn main() -> anyhow::Result<()> {
//!     let backend = DialoguerBackend::new();
//!     let user = User::builder().run(backend)?;
//!     println!("Hello, {} ({} years old)!", user.name, user.age);
//!     Ok(())
//! }
//! ```

mod backend;

pub use backend::{DialoguerBackend, DialoguerError};
