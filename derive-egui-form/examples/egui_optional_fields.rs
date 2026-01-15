//! Optional fields example
//!
//! Run with: cargo run -p derive-egui-form --example optional_fields

use derive_egui_form::EguiBackend;
use example_surveys::ProjectConfig;

fn main() -> anyhow::Result<()> {
    let backend = EguiBackend::new();
    let result = ProjectConfig::builder().run(backend)?;
    println!("{result:#?}");
    Ok(())
}
