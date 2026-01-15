//! Magic Forest example - comprehensive demo of all features in egui.
//!
//! Run with: cargo run -p derive-egui-form --example spooky_forest

use derive_egui_form::EguiBackend;
use example_surveys::SpookyForest;

fn main() -> anyhow::Result<()> {
    let backend = EguiBackend::new()
        .with_title("Magic Forest Adventure")
        .with_window_size([550.0, 700.0]);
    let result = SpookyForest::builder().run(backend)?;
    println!("{result:#?}");
    Ok(())
}
