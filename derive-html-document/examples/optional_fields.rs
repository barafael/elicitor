//! Optional fields example - generate an HTML form with optional fields.
//!
//! Run with: cargo run -p derive-html-document --example optional_fields

use derive_html_document::to_html;
use example_surveys::ProjectConfig;

fn main() {
    let html = to_html::<ProjectConfig>(Some("Project Configuration"));

    std::fs::write("optional_fields.html", &html).expect("Failed to write HTML file");

    println!("Generated optional_fields.html");
}
