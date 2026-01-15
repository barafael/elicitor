//! Validation example - generate an HTML form with validated fields.
//!
//! Run with: cargo run -p derive-html-document --example validation

use derive_html_document::to_html;
use example_surveys::AccountCreation;

fn main() {
    let html = to_html::<AccountCreation>(Some("Account Creation"));

    std::fs::write("validation.html", &html).expect("Failed to write HTML file");

    println!("Generated validation.html");
}
