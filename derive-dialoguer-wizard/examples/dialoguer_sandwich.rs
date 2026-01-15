//! Sandwich Builder - Minimal all-features example. Run with: cargo run -p derive-dialoguer-wizard --example sandwich

use derive_dialoguer_wizard::DialoguerBackend;
use example_surveys::SandwichOrder;

fn main() -> anyhow::Result<()> {
    let backend = DialoguerBackend::new();

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
