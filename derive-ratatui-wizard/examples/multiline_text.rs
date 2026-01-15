//! Multiline text input example
//!
//! Demonstrates:
//! - #[multiline] attribute for opening a text editor or textarea
//!
//! Run with: cargo run --example multiline_text

use derive_ratatui_wizard::RatatuiBackend;
use example_surveys::BlogPost;

fn main() -> anyhow::Result<()> {
    let backend = RatatuiBackend::new();
    let result = BlogPost::builder().run(backend)?;
    println!("{result:#?}");
    Ok(())
}
