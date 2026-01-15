//! Job Application example - generate an HTML form showcasing all features.
//!
//! Run with: cargo run -p derive-html-document --example job_application

use derive_html_document::{HtmlOptions, to_html_with_options};
use example_surveys::JobApplication;

fn main() {
    let options = HtmlOptions::new()
        .with_title("Acme Corp - Job Application")
        .with_styles(true)
        .full_document(true);

    let html = to_html_with_options::<JobApplication>(options);

    std::fs::write("job_application.html", &html).expect("Failed to write HTML file");

    println!("Generated job_application.html");
}
