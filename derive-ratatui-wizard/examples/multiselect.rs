//! Multiselect example
//!
//! Demonstrates:
//! - #[multiselect] attribute for Vec<Enum> fields
//! - Allows selecting multiple enum variants at once
//!
//! Run with: cargo run --example multiselect

use derive_ratatui_wizard::RatatuiBackend;
use example_surveys::DeveloperProfile;

fn main() -> anyhow::Result<()> {
    let backend = RatatuiBackend::new();
    let result = DeveloperProfile::builder().run(backend)?;
    println!("{result:#?}");
    Ok(())
}
