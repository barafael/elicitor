//! Magic Forest example - generate a comprehensive HTML form.
//!
//! Run with: cargo run -p derive-html-document --example spooky_forest

use derive_html_document::{HtmlOptions, to_html_with_options};
use example_surveys::SpookyForest;

fn main() {
    let options = HtmlOptions::new()
        .with_title("Magic Forest Adventure")
        .with_styles(true)
        .full_document(true);

    let html = to_html_with_options::<SpookyForest>(options);

    // Write to file
    std::fs::write("spooky_forest.html", &html).expect("Failed to write HTML file");
}
