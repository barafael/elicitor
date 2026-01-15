//! Sandwich Builder example - generate an HTML form for ordering.
//!
//! Run with: cargo run -p derive-html-document --example sandwich

use derive_html_document::{HtmlOptions, to_html_with_options};
use example_surveys::SandwichOrder;

fn main() {
    let options = HtmlOptions::new()
        .with_title("Rusty's Subs - Order Form")
        .with_styles(true)
        .full_document(true);

    let html = to_html_with_options::<SandwichOrder>(options);

    std::fs::write("sandwich.html", &html).expect("Failed to write HTML file");

    println!("Generated sandwich.html");
}
