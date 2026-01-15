//! Enum (OneOf) example
//!
//! Demonstrates:
//! - Enum fields for selecting one option
//! - Unit variants (simple choices)
//! - Newtype variants (with follow-up question)
//! - Struct variants (with multiple follow-up questions)
//!
//! Run with: cargo run --example enum_oneof

use derive_dialoguer_wizard::DialoguerBackend;
use example_surveys::Checkout;

fn main() -> anyhow::Result<()> {
    let backend = DialoguerBackend::new();
    let result = Checkout::builder().run(backend)?;
    println!("{result:#?}");
    Ok(())
}
