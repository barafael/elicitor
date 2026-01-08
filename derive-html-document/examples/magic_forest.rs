//! Magic Forest example - generate a comprehensive HTML form.
//!
//! Run with: cargo run -p derive-html-document --example magic_forest

use derive_html_document::{HtmlOptions, to_html_with_options};
use derive_survey::Survey;

/// The magic forest adventure survey.
#[derive(Debug, Survey)]
#[allow(unused)]
#[prelude("🌲 Welcome to the Magic Forest! 🌲\n\nA journey begins...")]
#[epilogue("Good luck on your adventure!")]
struct MagicForest {
    #[ask("What is your name, adventurer?")]
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

    #[ask("Pick your starting inventory")]
    #[multiselect]
    inventory: Vec<Item>,
}

/// Character role selection.
#[derive(Debug, Survey)]
#[allow(unused)]
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
#[derive(Debug, Survey)]
#[allow(unused)]
enum Item {
    #[ask("⚔️ Sword (value: 80 gold)")]
    Sword,

    #[ask("🛡️ Shield (value: 50 gold)")]
    Shield,

    #[ask("🧪 Potion (value: 20 gold)")]
    Potion,

    #[ask("📜 Scroll (value: 10 gold)")]
    Scroll,

    #[ask("🍬 Chewing Gum (value: 2 × quantity)")]
    ChewingGum {
        #[ask("What flavor?")]
        flavor: String,
        #[ask("How many pieces?")]
        #[min(1)]
        #[max(100)]
        quantity: i64,
    },
}

fn main() {
    let options = HtmlOptions::new()
        .with_title("Magic Forest Adventure")
        .with_styles(true)
        .full_document(true);

    let html = to_html_with_options::<MagicForest>(options);

    // Write to file
    std::fs::write("magic_forest.html", &html).expect("Failed to write HTML file");

    println!("Generated magic_forest.html");
    println!("\n--- Preview (first 100 lines) ---\n");
    for (i, line) in html.lines().take(100).enumerate() {
        println!("{:3}: {}", i + 1, line);
    }
    println!("\n... (open magic_forest.html in a browser to see the full form)");
}
