//! Sandwich Builder - A minimal example showcasing all derive-survey features
//!
//! Features demonstrated:
//! - Prelude/epilogue messages
//! - Primitives: String, bool, i32, u32
//! - Optional fields (Option<T> - skipped = None)
//! - Text input with validation (#[validate])
//! - Password/masked input (#[mask])
//! - Multiline text input (#[multiline])
//! - Numeric bounds (#[min], #[max])
//! - Enum selection (unit, newtype, struct variants)
//! - Multi-select (#[multiselect])
//! - Nested struct with propagated validation (#[validate_fields])
//! - PathBuf support
//! - Builder API with suggestions and closures
//!
//! Run with: cargo run -p derive-requestty-wizard --example sandwich

use derive_requestty_wizard::RequesttyBackend;
use example_surveys::SandwichOrder;

fn main() -> anyhow::Result<()> {
    let backend = RequesttyBackend::new();

    let result = SandwichOrder::builder()
        .suggest_name("Ferris".to_string())
        .suggest_toasted(true)
        .suggest_tip(3)
        .suggest_bread(|b| b.suggest_italian())
        .suggest_filling(|f| f.suggest_turkey())
        .suggest_cheese(|c| c.suggest_provolone())
        .suggest_sauce(|s| s.suggest_oil_vinegar())
        .suggest_size(|sz| sz.suggest_six())
        .suggest_nutrition(|n| n.calories(600).protein(30))
        .run(backend)?;

    println!("{result:#?}");
    Ok(())
}
