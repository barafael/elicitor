//! Validation example
//!
//! Demonstrates:
//! - #[validate("fn_name")] for field-level validation
//! - Custom validator functions
//! - Using ResponseValue and Responses for validation
//!
//! Run with: cargo run --example validation

use derive_egui_form::EguiBackend;
use example_surveys::AccountCreation;

fn main() -> anyhow::Result<()> {
    let backend = EguiBackend::new();
    let result = AccountCreation::builder().run(backend)?;
    println!("{result:#?}");
    Ok(())
}
