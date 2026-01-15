//! Basic field types example. Run with: cargo run --example basic_fields

use derive_dialoguer_wizard::DialoguerBackend;
use example_surveys::BasicFields;

fn main() -> anyhow::Result<()> {
    let backend = DialoguerBackend::new();
    let result = BasicFields::builder().run(backend)?;
    println!("{result:#?}");
    Ok(())
}
