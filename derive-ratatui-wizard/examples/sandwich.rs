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

use derive_ratatui_wizard::{RatatuiBackend, Theme};
use example_surveys::SandwichOrder;
use ratatui::style::Color;

fn main() {
    let theme = Theme {
        primary: Color::Yellow,
        secondary: Color::LightYellow,
        background: Color::Reset,
        text: Color::White,
        highlight: Color::Green,
        error: Color::Red,
        success: Color::LightGreen,
        border: Color::DarkGray,
    };

    let backend = RatatuiBackend::new()
        .with_title("Rusty's Subs - Order")
        .with_theme(theme);

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
        .run(backend);

    match result {
        Ok(order) => {
            println!("\n=== Order Confirmed ===\n");
            println!("Name: {}", order.name);
            println!("Bread: {:?}", order.bread);
            println!("Filling: {:?}", order.filling);
            println!("Cheese: {:?}", order.cheese);
            println!("Toppings: {:?}", order.toppings);
            println!("Sauce: {:?}", order.sauce);
            println!("Size: {:?}", order.size);
            println!("Toasted: {}", if order.toasted { "Yes" } else { "No" });
            println!("Tip: ${}", order.tip);
            if !order.notes.is_empty() {
                println!("Notes: {}", order.notes);
            }
            println!("\n{:#?}", order);
        }
        Err(e) => {
            eprintln!("Order cancelled: {e}");
            std::process::exit(1);
        }
    }
}
