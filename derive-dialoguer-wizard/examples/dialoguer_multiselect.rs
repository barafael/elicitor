//! Multiselect example. Run with: cargo run --example multiselect

use derive_dialoguer_wizard::DialoguerBackend;
use example_surveys::DeveloperProfile;

fn main() -> anyhow::Result<()> {
    let backend = DialoguerBackend::new();
    let result = DeveloperProfile::builder().run(backend)?;
    println!("{result:#?}");
    Ok(())
}
