//! Prelude and epilogue example. Run with: cargo run --example prelude_epilogue

use derive_dialoguer_wizard::DialoguerBackend;
use example_surveys::FitnessProfile;

fn main() -> anyhow::Result<()> {
    let backend = DialoguerBackend::new();
    let result = FitnessProfile::builder().run(backend)?;
    println!("{result:#?}");
    Ok(())
}
