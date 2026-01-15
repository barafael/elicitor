//! Optional fields example. Run with: cargo run -p derive-dialoguer-wizard --example optional_fields

use derive_dialoguer_wizard::DialoguerBackend;
use example_surveys::ProjectConfig;

fn main() -> anyhow::Result<()> {
    let backend = DialoguerBackend::new();
    let result = ProjectConfig::builder().run(backend)?;
    println!("{result:#?}");
    Ok(())
}
