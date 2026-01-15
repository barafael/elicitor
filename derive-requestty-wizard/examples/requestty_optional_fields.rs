//! Optional fields example
//!
//! Run with: cargo run -p derive-requestty-wizard --example optional_fields

use derive_requestty_wizard::RequesttyBackend;
use example_surveys::ProjectConfig;

fn main() -> anyhow::Result<()> {
    let backend = RequesttyBackend::new();
    let result = ProjectConfig::builder().run(backend)?;
    println!("{result:#?}");
    Ok(())
}
