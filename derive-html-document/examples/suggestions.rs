//! Suggestions example - generate an HTML form for application settings.
//!
//! Note: HTML forms don't support dynamic suggestions like interactive backends,
//! but this example generates the same form structure.
//!
//! Run with: cargo run -p derive-html-document --example suggestions

use derive_html_document::{HtmlOptions, to_html_with_options};
use example_surveys::AppSettings;

fn main() {
    let options = HtmlOptions::new()
        .with_title("Application Settings")
        .with_styles(true)
        .full_document(true);

    let html = to_html_with_options::<AppSettings>(options);

    std::fs::write("suggestions.html", &html).expect("Failed to write HTML file");

    println!("Generated suggestions.html");
}
