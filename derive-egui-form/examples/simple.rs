//! Simple example demonstrating the egui backend for derive-survey.
//!
//! This example shows basic field types: strings, integers, floats, and booleans.
//!
//! Run with: cargo run -p derive-egui-form --example simple

use derive_survey::Survey;
use derive_egui_form::EguiBackend;

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

    /// User's height in centimeters.
    #[ask("What is your height (in cm)?")]
    #[min(30)]
    #[max(300)]
    height_cm: i64,

    /// User's email address.
    #[ask("What is your email?")]
    email: String,

    /// Whether the user wants to receive the newsletter.
    #[ask("Would you like to receive our newsletter?")]
    newsletter: bool,
}

fn main() -> anyhow::Result<()> {
    println!("=== User Profile Survey - egui Demo ===\n");

    let backend = EguiBackend::new()
        .with_title("User Profile Survey")
        .with_window_size([450.0, 400.0]);

    let profile: UserProfile = UserProfile::builder().run(backend)?;

    println!("\n=== Profile Created ===");
    println!("Name: {}", profile.name);
    println!("Age: {}", profile.age);
    println!("Height: {} cm", profile.height_cm);
    println!("Email: {}", profile.email);
    println!(
        "Newsletter: {}",
        if profile.newsletter { "Yes" } else { "No" }
    );

    Ok(())
}
