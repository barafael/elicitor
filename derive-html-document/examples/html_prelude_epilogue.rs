//! Prelude/Epilogue example - generate an HTML form with pre/post messages.
//!
//! Run with: cargo run -p derive-html-document --example prelude_epilogue

use derive_html_document::to_html;
use example_surveys::FitnessProfile;

fn main() {
    let html = to_html::<FitnessProfile>(Some("Fitness Profile"));

    std::fs::write("prelude_epilogue.html", &html).expect("Failed to write HTML file");

    println!("Generated prelude_epilogue.html");
}
