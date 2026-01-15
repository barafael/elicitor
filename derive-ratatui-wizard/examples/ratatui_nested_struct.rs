//! Nested struct example
//!
//! Demonstrates:
//! - Nested structs create "AllOf" question groups
//! - All nested fields are collected together
//!
//! Run with: cargo run --example nested_struct

use derive_ratatui_wizard::RatatuiBackend;
use example_surveys::UserRegistration;

fn main() -> anyhow::Result<()> {
    let backend = RatatuiBackend::new();
    let result = UserRegistration::builder().run(backend)?;
    println!("{result:#?}");
    Ok(())
}
