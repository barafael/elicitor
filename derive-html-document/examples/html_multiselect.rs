//! Multiselect example - generate an HTML form with multi-select checkboxes.
//!
//! Run with: cargo run -p derive-html-document --example multiselect

use derive_html_document::to_html;
use example_surveys::DeveloperProfile;

fn main() {
    let html = to_html::<DeveloperProfile>(Some("Developer Profile"));

    std::fs::write("multiselect.html", &html).expect("Failed to write HTML file");

    println!("Generated multiselect.html");
}
