//! Min/Max bounds example - generate an HTML form with numeric constraints.
//!
//! Run with: cargo run -p derive-html-document --example min_max_bounds

use derive_html_document::to_html;
use example_surveys::GameSettings;

fn main() {
    let html = to_html::<GameSettings>(Some("Game Settings"));

    std::fs::write("min_max_bounds.html", &html).expect("Failed to write HTML file");

    println!("Generated min_max_bounds.html");
}
