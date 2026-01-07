//! Requestty backend for derive-survey.
//!
//! This crate provides a command-line interface for collecting survey responses
//! using the `requestty` library.
//!
//! # Example
//!
//! ```ignore
//! use derive_survey::Survey;
//! use derive_survey_requestty::RequesttyBackend;
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
//!     let backend = RequesttyBackend::new();
//!     let user = User::builder().run(backend)?;
//!     println!("Hello, {} ({} years old)!", user.name, user.age);
//!     Ok(())
//! }
//! ```

mod backend;

pub use backend::RequesttyBackend;
pub use backend::RequesttyError;
