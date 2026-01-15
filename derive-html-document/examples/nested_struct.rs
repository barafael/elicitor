//! Nested struct example - generate an HTML form with nested structures.
//!
//! Run with: cargo run -p derive-html-document --example nested_struct

use derive_html_document::to_html;
use example_surveys::UserRegistration;

fn main() {
    let html = to_html::<UserRegistration>(Some("User Registration"));

    std::fs::write("nested_struct.html", &html).expect("Failed to write HTML file");

    println!("Generated nested_struct.html");
}
