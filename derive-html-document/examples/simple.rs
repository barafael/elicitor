//! Simple example - generate an HTML form from a basic survey.
//!
//! Run with: cargo run -p derive-html-document --example simple

use derive_html_document::to_html;
use derive_survey::Survey;

#[derive(Debug, Survey)]
struct UserProfile {
    #[ask("What is your name?")]
    name: String,

    #[ask("How old are you?")]
    #[min(0)]
    #[max(150)]
    age: i64,

    #[ask("What is your email address?")]
    email: String,

    #[ask("Tell us about yourself")]
    #[multiline]
    bio: String,

    #[ask("Subscribe to newsletter?")]
    subscribe: bool,
}

fn main() {
    let html = to_html::<UserProfile>(Some("User Profile"));

    // Write to file
    std::fs::write("user_profile.html", &html).expect("Failed to write HTML file");

    println!("Generated user_profile.html");
    println!("\n--- Preview ---\n");
    println!("{html}");
}
