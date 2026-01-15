//! Basic fields example - generate an HTML form with primitive types.
//!
//! Run with: cargo run -p derive-html-document --example basic_fields

use derive_html_document::to_html;
use example_surveys::BasicFields;

fn main() {
    let html = to_html::<BasicFields>(Some("Basic Fields"));

    std::fs::write("basic_fields.html", &html).expect("Failed to write HTML file");

    println!("Generated basic_fields.html");
}
