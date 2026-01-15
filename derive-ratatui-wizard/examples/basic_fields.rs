//! Basic field types example
//!
//! Demonstrates:
//! - String input
//! - Boolean confirmation
//! - Numeric types (u32, i32, f64)
//!
//! Run with: cargo run --example basic_fields

use derive_ratatui_wizard::RatatuiBackend;
use example_surveys::BasicFields;

fn main() -> anyhow::Result<()> {
    let backend = RatatuiBackend::new();
    let result = BasicFields::builder().run(backend)?;
    println!("{result:#?}");
    Ok(())
}
