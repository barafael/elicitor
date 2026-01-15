//! Simple example demonstrating the requestty backend for derive-survey.
//!
//! Run with: cargo run -p derive-requestty-wizard --example simple

use derive_requestty_wizard::RequesttyBackend;
use example_surveys::UserProfile;

fn main() -> anyhow::Result<()> {
    let backend = RequesttyBackend::new();

    let profile = UserProfile::builder().run(backend)?;

    println!("{profile:#?}");

    Ok(())
}
