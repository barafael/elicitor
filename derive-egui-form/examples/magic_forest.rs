//! Magic Forest example - comprehensive demo of all features in egui.
//!
//! This example combines:
//! - Text input with validation
//! - Password/masked fields
//! - Integer fields with bounds
//! - Enum selection (OneOf)
//! - Multi-select (AnyOf)
//! - Nested struct fields within enum variants
//! - Prelude and epilogue messages
//!
//! Run with: cargo run -p derive-egui-form --example magic_forest

use derive_survey::{ResponseValue, Responses, Survey};
use derive_egui_form::EguiBackend;

/// The magic forest adventure survey.
#[allow(dead_code)]
#[derive(Debug, Survey)]
#[prelude("🌲 Welcome to the Magic Forest! 🌲\n\nA journey begins...")]
#[epilogue("Good luck on your adventure!")]
struct MagicForest {
    #[ask("What is your name, adventurer?")]
    #[validate(validate_name)]
    name: String,

    #[ask("What's the secret passphrase?")]
    #[mask]
    passphrase: String,

    #[ask("How old are you?")]
    #[min(18)]
    #[max(233)]
    age: i64,

    #[ask("What is your role?")]
    role: Role,

    #[ask("Pick your starting inventory:")]
    #[multiselect]
    #[validate(validate_budget)]
    inventory: Vec<Item>,
}

/// Character role selection.
#[allow(dead_code)]
#[derive(Debug, Survey)]
enum Role {
    #[ask("⚔️ Streetfighter")]
    Streetfighter,

    #[ask("🧙 Mage")]
    Mage,

    #[ask("🏹 Archer")]
    Archer,

    #[ask("🗡️ Thief")]
    Thief,

    #[ask("❓ Other")]
    Other(#[ask("What role then?!")] String),
}

/// Inventory items with costs.
#[allow(dead_code)]
#[derive(Debug, Survey)]
enum Item {
    #[ask("⚔️ Sword (value: 80 gold)")]
    Sword,

    #[ask("🛡️ Shield (value: 50 gold)")]
    Shield,

    #[ask("🧪 Potion (value: 20 gold)")]
    Potion,

    #[ask("📜 Scroll (value: 10 gold)")]
    Scroll,

    #[ask("🍬 Chewing Gum (value: 2 gold each)")]
    ChewingGum {
        #[ask("Flavor:")]
        flavor: String,
        #[ask("Quantity:")]
        #[min(1)]
        #[max(10)]
        quantity: i64,
    },
}

fn validate_name(value: &ResponseValue, _responses: &Responses) -> Result<(), String> {
    let ResponseValue::String(name) = value else {
        return Ok(());
    };

    if name.len() < 3 {
        return Err("Name must be at least 3 characters".to_string());
    }

    if name.len() > 30 {
        return Err("Name must be at most 30 characters".to_string());
    }

    // Check for valid characters
    if let Some(c) = name
        .chars()
        .find(|c| !c.is_alphabetic() && !c.is_whitespace())
    {
        return Err(format!("Invalid character in name: '{c}'"));
    }

    Ok(())
}

fn validate_budget(value: &ResponseValue, _responses: &Responses) -> Result<(), String> {
    let ResponseValue::ChosenVariants(selections) = value else {
        return Ok(());
    };

    const STARTING_BUDGET: i64 = 150;
    let mut total_cost: i64 = 0;

    for &variant_idx in selections {
        let item_cost = match variant_idx {
            0 => 80, // Sword
            1 => 50, // Shield
            2 => 20, // Potion
            3 => 10, // Scroll
            4 => 2,  // ChewingGum base cost
            _ => 0,
        };
        total_cost += item_cost;
    }

    if total_cost > STARTING_BUDGET {
        return Err(format!(
            "Over budget! Total: {} gold, limit: {} gold",
            total_cost, STARTING_BUDGET
        ));
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    println!("=== Magic Forest Adventure - egui Complete Demo ===\n");

    let backend = EguiBackend::new()
        .with_title("Magic Forest Adventure")
        .with_window_size([550.0, 700.0]);

    let adventure: MagicForest = MagicForest::builder().run(backend)?;

    println!("\n=== Your Adventure Character ===");
    println!("Name: {}", adventure.name);
    println!("Age: {}", adventure.age);
    println!("Role: {:?}", adventure.role);
    println!("Inventory: {:?}", adventure.inventory);

    println!("\n🌲 May the forest spirits guide you! 🌲");

    Ok(())
}
