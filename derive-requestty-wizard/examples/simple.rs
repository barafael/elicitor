//! Simple example demonstrating the requestty backend for derive-survey.
//!
//! Run with: cargo run -p derive-requestty-wizard --example simple

use derive_requestty_wizard::RequesttyBackend;
use derive_survey::Survey;

/// A simple user profile survey.
#[derive(Debug, Survey)]
struct UserProfile {
    /// User's full name.
    #[ask("What is your name?")]
    name: String,

    /// User's age.
    #[ask("How old are you?")]
    #[min(0)]
    #[max(150)]
    age: i64,

    /// User's email address.
    #[ask("What is your email?")]
    email: String,

    /// Whether the user wants to receive the newsletter.
    #[ask("Would you like to receive our newsletter?")]
    newsletter: bool,
}

fn main() -> anyhow::Result<()> {
    println!("=== User Profile Survey ===\\n");

    let backend = RequesttyBackend::new();

    let profile: UserProfile = UserProfile::builder().run(backend)?;

    println!("\\n=== Profile Created ===");
    println!("Name: {}", profile.name);
    println!("Age: {}", profile.age);
    println!("Email: {}", profile.email);
    println!(
        "Newsletter: {}",
        if profile.newsletter { "Yes" } else { "No" }
    );

    Ok(())
}
