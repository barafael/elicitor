//! Simple example demonstrating the ratatui wizard backend for derive-survey.
//!
//! Run with: cargo run -p derive-ratatui-wizard --example simple

use derive_ratatui_wizard::RatatuiBackend;
use example_surveys::UserProfile;

fn main() -> anyhow::Result<()> {
    let backend = RatatuiBackend::new().with_title("User Profile Survey");

    let profile: UserProfile = UserProfile::builder().run(backend)?;

    println!("\n=== Profile Created ===");
    println!("Name: {}", profile.name);
    println!("Age: {}", profile.age);
    println!("Email: {}", profile.email);
    println!(
        "Newsletter: {}",
        if profile.newsletter { "Yes" } else { "No" }
    );

    Ok(())
}
