//! Multiline text example - generate an HTML form with textarea fields.
//!
//! Run with: cargo run -p derive-html-document --example multiline_text

use derive_html_document::to_html;
use example_surveys::BlogPost;

fn main() {
    let html = to_html::<BlogPost>(Some("Blog Post"));

    std::fs::write("multiline_text.html", &html).expect("Failed to write HTML file");

    println!("Generated multiline_text.html");
}
