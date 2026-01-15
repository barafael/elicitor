//! Multiselect example
//!
//! Demonstrates:
//! - #[multiselect] attribute for Vec<Enum> fields
//! - Allows selecting multiple enum variants at once
//!
//! Run with: cargo run --example multiselect

use derive_requestty_wizard::RequesttyBackend;
use example_surveys::DeveloperProfile;

fn main() -> anyhow::Result<()> {
    let backend = RequesttyBackend::new();
    let result = DeveloperProfile::builder().run(backend)?;
    println!("{result:#?}");
    Ok(())
}
