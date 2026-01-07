//! # derive-egui-form
//!
//! An egui form backend for derive-survey that renders surveys as GUI forms.
//!
//! This backend uses the `eframe` and `egui` crates to provide a native
//! desktop form interface for surveys. All fields are displayed at once
//! and can be edited in any order.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use derive_survey::Survey;
//! use derive_egui_form::EguiBackend;
//!
//! #[derive(Survey, Debug)]
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
//!     let backend = EguiBackend::new()
//!         .with_title("User Profile")
//!         .with_window_size([400.0, 300.0]);
//!
//!     let profile: UserProfile = UserProfile::builder().run(backend)?;
//!     println!("{:?}", profile);
//!     Ok(())
//! }
//! ```

mod backend;

pub use backend::{EguiBackend, EguiError};
