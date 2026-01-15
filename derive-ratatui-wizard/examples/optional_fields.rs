//! Optional fields example
//!
//! Run with: cargo run -p derive-ratatui-wizard --example optional_fields

use derive_ratatui_wizard::RatatuiBackend;
use example_surveys::ProjectConfig;

fn main() -> anyhow::Result<()> {
    let backend = RatatuiBackend::new();
    let result = ProjectConfig::builder().run(backend)?;
    println!("{result:#?}");
    Ok(())
}
