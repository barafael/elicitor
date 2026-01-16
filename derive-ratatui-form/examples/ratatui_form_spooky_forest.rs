//! Magic Forest - Form backend example demonstrating derive-survey features
//!
//! This example showcases the form-based UI where all fields are visible at once,
//! similar to the egui backend but in the terminal.

use derive_ratatui_form::{RatatuiFormBackend, Theme};
use example_surveys::SpookyForest;
use ratatui::style::Color;

fn main() -> anyhow::Result<()> {
    let fantasy_theme = Theme {
        primary: Color::Magenta,
        secondary: Color::LightMagenta,
        background: Color::Reset,
        text: Color::White,
        highlight: Color::Yellow,
        error: Color::LightRed,
        success: Color::LightGreen,
        border: Color::DarkGray,
        selected_bg: Color::DarkGray,
    };

    let backend = RatatuiFormBackend::new()
        .with_title("Magic Forest - Character Creation (Form View)")
        .with_theme(fantasy_theme);

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
