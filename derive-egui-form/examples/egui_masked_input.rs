//! Masked input example
//!
//! Demonstrates:
//! - #[mask] attribute for hiding sensitive input like passwords
//! - Cross-field validation for password confirmation
//!
//! Run with: cargo run --example masked_input

use derive_egui_form::EguiBackend;
use example_surveys::Login;

fn main() -> anyhow::Result<()> {
    let backend = EguiBackend::new();
    let result = Login::builder().run(backend)?;
    println!("{result:#?}");
    Ok(())
}
