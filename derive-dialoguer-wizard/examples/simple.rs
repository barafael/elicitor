//! Simple example demonstrating the dialoguer backend for derive-survey.
//!
//! Run with: cargo run -p derive-dialoguer-wizard --example simple

use derive_dialoguer_wizard::DialoguerBackend;
use example_surveys::UserProfile;

fn main() -> anyhow::Result<()> {
    let backend = DialoguerBackend::new();
    let profile = UserProfile::builder().run(backend)?;
    println!("{profile:#?}");
    Ok(())
}
