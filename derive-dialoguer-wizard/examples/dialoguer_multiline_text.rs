//! Multiline text input example. Run with: cargo run --example multiline_text

use derive_dialoguer_wizard::DialoguerBackend;
use example_surveys::BlogPost;

fn main() -> anyhow::Result<()> {
    let backend = DialoguerBackend::new();
    let result = BlogPost::builder().run(backend)?;
    println!("{result:#?}");
    Ok(())
}
