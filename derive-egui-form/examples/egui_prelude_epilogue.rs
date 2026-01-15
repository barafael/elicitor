//! Prelude and epilogue example
//!
//! Demonstrates:
//! - #[prelude("...")] for showing a message before the survey starts
//! - #[epilogue("...")] for showing a message after the survey completes
//!
//! Run with: cargo run --example prelude_epilogue

use derive_egui_form::EguiBackend;
use example_surveys::FitnessProfile;

fn main() -> anyhow::Result<()> {
    let backend = EguiBackend::new();
    let result = FitnessProfile::builder().run(backend)?;
    println!("{result:#?}");
    Ok(())
}
