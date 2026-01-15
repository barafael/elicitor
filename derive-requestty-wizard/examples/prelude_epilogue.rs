//! Prelude and epilogue example
//!
//! Demonstrates:
//! - #[prelude("...")] for showing a message before the survey starts
//! - #[epilogue("...")] for showing a message after the survey completes
//!
//! Run with: cargo run --example prelude_epilogue

use derive_requestty_wizard::RequesttyBackend;
use example_surveys::FitnessProfile;

fn main() -> anyhow::Result<()> {
    let backend = RequesttyBackend::new();
    let result = FitnessProfile::builder().run(backend)?;
    println!("{result:#?}");
    Ok(())
}
