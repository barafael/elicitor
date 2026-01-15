//! Nested struct example. Run with: cargo run --example nested_struct

use derive_dialoguer_wizard::DialoguerBackend;
use example_surveys::UserRegistration;

fn main() -> anyhow::Result<()> {
    let backend = DialoguerBackend::new();
    let result = UserRegistration::builder().run(backend)?;
    println!("{result:#?}");
    Ok(())
}
