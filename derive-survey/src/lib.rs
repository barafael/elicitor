//! # derive-survey
//!
//! Derive interactive surveys for Rust types. Backend-agnostic.
//!
//! This crate provides the `#[derive(Survey)]` macro for defining surveys
//! that can be collected through various backends (CLI wizards, GUI forms, etc.)
//!
//! ## Usage
//!
//! ```rust,ignore
//! use derive_survey::Survey;
//!
//! #[derive(Survey, Debug)]
//! struct UserProfile {
//!     #[ask("What is your name?")]
//!     name: String,
//!
//!     #[ask("How old are you?")]
//!     #[min(0)]
//!     #[max(150)]
//!     age: u32,
//!
//!     #[ask("Are you a student?")]
//!     student: bool,
//! }
//!
//! // Run the survey with a backend
//! let profile: UserProfile = UserProfile::builder()
//!     .suggest_name("Alice")
//!     .run(backend)
//!     .unwrap();
//! ```
//!
//! ## Attributes
//!
//! ### On structs and enums
//! - `#[prelude("...")]` - Message shown before the survey starts
//! - `#[epilogue("...")]` - Message shown after the survey completes
//! - `#[validate("fn_name")]` - Composite validator function
//!
//! ### On fields
//! - `#[ask("...")]` - The prompt text shown to the user
//! - `#[mask]` - Hide input (for passwords)
//! - `#[multiline]` - Open text editor / show textarea
//! - `#[validate("fn_name")]` - Field-level validator function
//! - `#[min(n)]` / `#[max(n)]` - Numeric bounds
//! - `#[multiselect]` - For `Vec<Enum>` fields, enables multi-select
//!
//! ## Backends
//!
//! Backends are separate crates that implement `SurveyBackend`:
//! - `derive-requestty-wizard` - CLI prompts via requestty
//! - `derive-dialoguer-wizard` - CLI prompts via dialoguer
//! - `derive-ratatui-wizard` - TUI wizard
//! - `derive-egui-form` - GUI form via egui

// Re-export all types from derive-survey-types
pub use derive_survey_types::*;

// Re-export the derive macro
pub use derive_survey_macro::Survey;

// Test backend for testing surveys without user interaction
mod test_backend;
pub use test_backend::TestBackend;
