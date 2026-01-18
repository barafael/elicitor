#![doc = include_str!("../README.md")]

// Re-export all types from derive-survey-types
pub use elicitor_types::*;

// Re-export the derive macro
pub use elicitor_macro::Survey;

// Test backend for testing surveys without user interaction
mod test_backend;
pub use test_backend::TestBackend;
