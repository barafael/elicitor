//! Magic Forest - A comprehensive example demonstrating ALL derive-survey features
//!
//! This example showcases:
//! - Prelude and epilogue messages
//! - All primitive types (String, integers, floats, bool)
//! - Text input with validation
//! - Password/masked input
//! - Multiline text input
//! - Integer input with min/max constraints
//! - Float input with min/max constraints
//! - Boolean confirmation
//! - Enum selection (OneOf) with unit, newtype, tuple, and struct variants
//! - Multi-select (AnyOf) with validation
//! - Nested structs (AllOf)
//! - Deeply nested structures
//! - Field-level validation
//! - Builder API with suggestions and assumptions
//! - PathBuf support
//!
//! Run with: cargo run -p derive-requestty-wizard --example spooky_forest

use derive_requestty_wizard::RequesttyBackend;
use example_surveys::SpookyForest;

fn main() -> anyhow::Result<()> {
    let backend = RequesttyBackend::new();

    let result = SpookyForest::builder()
        // Simple field suggestions
        .suggest_name("Gandalf".to_string())
        .suggest_age(500) // Wizards live long, but within the 1000 year max
        .suggest_email("gandalf@middleearth.org".to_string())
        .suggest_lucky_number(7)
        .suggest_gold_multiplier_raw(5)
        .suggest_hardcore_mode(false)
        // Nested struct suggestion using closure API
        .suggest_home(|home| {
            home.realm("Middle-earth")
                .village("Hobbiton")
                .distance_leagues(500.0)
        })
        // Nested struct for character stats (total must be <= 75)
        .suggest_stats(|stats| {
            stats
                .strength(8)
                .dexterity(10)
                .intelligence(18)
                .wisdom(16)
                .charisma(12)
                .constitution(10)
            // Total: 8+10+18+16+12+10 = 74, within the 75 point limit
        })
        // Enum suggestion with variant selection and nested fields
        .suggest_role(|role| {
            // Pre-select Mage as the default class
            role.suggest_mage()
        })
        // Enum with struct variant fields
        .suggest_background(|bg| {
            // Pre-select Hermit and suggest its fields (years max is 50)
            bg.suggest_hermit().hermit(|h| {
                h.years(42)
                    .wisdom("A wizard is never late, nor is he early.")
            })
        })
        // Enum with various variant types
        .suggest_companion(|comp| {
            // Pre-select Familiar variant and configure its fields
            comp.suggest_familiar()
                .familiar(|f| f.name("Shadowfax").form(|form| form.suggest_other()))
                // Also suggest values for Friend variant (in case user picks it)
                .friend(|details| details.name("Hynix"))
        })
        .run(backend)?;

    println!("{result:#?}");
    Ok(())
}
