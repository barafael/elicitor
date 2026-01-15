//! Suggestions and assumptions example for the requestty backend.
//!
//! This example demonstrates:
//! - Setting default/suggested values that users can modify
//! - Editing existing data with with_suggestions
//!
//! Run with: cargo run -p derive-requestty-wizard --example suggestions

use derive_requestty_wizard::RequesttyBackend;
use example_surveys::AppSettings;

fn main() -> anyhow::Result<()> {
    let backend = RequesttyBackend::new();

    let settings = AppSettings::builder()
        .suggest_app_name("my-awesome-app")
        .suggest_port(8080)
        .suggest_max_connections(100)
        .suggest_timeout(30)
        .suggest_debug_mode(false)
        .suggest_log_path("/var/log/app.log")
        .run(backend)?;

    println!("{settings:#?}");

    // Second run: Edit existing settings using with_suggestions
    let backend = RequesttyBackend::new();

    let updated_settings = AppSettings::builder()
        .with_suggestions(&settings)
        .run(backend)?;

    println!("{updated_settings:#?}");

    Ok(())
}
