//! Masked input example - generate an HTML form with password fields.
//!
//! Run with: cargo run -p derive-html-document --example masked_input

use derive_html_document::to_html;
use example_surveys::Login;

fn main() {
    let html = to_html::<Login>(Some("Login"));

    std::fs::write("masked_input.html", &html).expect("Failed to write HTML file");

    println!("Generated masked_input.html");
}
