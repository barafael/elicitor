//! Enum OneOf example - generate an HTML form with enum selection.
//!
//! Run with: cargo run -p derive-html-document --example enum_oneof

use derive_html_document::to_html;
use example_surveys::Checkout;

fn main() {
    let html = to_html::<Checkout>(Some("Checkout"));

    std::fs::write("enum_oneof.html", &html).expect("Failed to write HTML file");

    println!("Generated enum_oneof.html");
}
