//! Masked input example
//!
//! Demonstrates:
//! - #[mask] attribute for hiding sensitive input like passwords
//! - Cross-field validation for password confirmation
//!
//! Run with: cargo run --example masked_input

use derive_dialoguer_wizard::DialoguerBackend;
use example_surveys::Login;

fn main() -> anyhow::Result<()> {
    let backend = DialoguerBackend::new();
    let result = Login::builder().run(backend)?;
    println!("{result:#?}");
    Ok(())
}
