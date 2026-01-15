//! Masked input example
//!
//! Demonstrates:
//! - #[mask] attribute for hiding sensitive input like passwords
//! - Cross-field validation for password confirmation
//!
//! Run with: cargo run --example masked_input

use derive_requestty_wizard::RequesttyBackend;
use example_surveys::Login;

fn main() -> anyhow::Result<()> {
    let backend = RequesttyBackend::new();
    let result = Login::builder().run(backend)?;
    println!("{result:#?}");
    Ok(())
}
