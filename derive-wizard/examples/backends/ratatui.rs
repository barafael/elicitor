//! Example demonstrating the ratatui TUI backend
//!
//! Run with: cargo run --example ratatui --features ratatui-backend

use derive_wizard::Wizard;

#[derive(Debug, Wizard)]
#[allow(dead_code)]
enum Subscription {
    Free,
    Basic,
    Premium,
}

/// A user profile form with various field types
#[derive(Debug, Wizard)]
#[allow(dead_code)]
struct UserProfile {
    #[prompt("What is your name?")]
    name: String,

    #[prompt("How old are you?")]
    #[min(0)]
    #[max(150)]
    age: i64,

    #[prompt("Enter your email:")]
    email: String,

    #[prompt("What is your monthly income?")]
    #[min(0.0)]
    income: f64,

    #[prompt("Subscribe to newsletter?")]
    subscribe: bool,

    #[prompt("Select subscription tier:")]
    tier: Subscription,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use derive_wizard::{InterviewBackend, RatatuiBackend};

    println!("Starting ratatui wizard...\n");

    let interview = UserProfile::interview();
    let backend = RatatuiBackend::new().with_title("✨ User Profile Setup");

    let answers = backend.execute(&interview)?;

    println!("\n📋 Collected Answers:");
    println!("━━━━━━━━━━━━━━━━━━━━━");
    for (key, value) in answers.iter() {
        println!("  {}: {:?}", key, value);
    }

    let profile = UserProfile::from_answers(&answers)?;
    println!("\n👤 Profile Created:");
    println!("{:#?}", profile);

    Ok(())
}
