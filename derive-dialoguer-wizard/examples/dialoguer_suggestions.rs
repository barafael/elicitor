//! Suggestions and assumptions example. Run with: cargo run -p derive-dialoguer-wizard --example suggestions

use derive_dialoguer_wizard::DialoguerBackend;
use example_surveys::AppSettings;

fn main() -> anyhow::Result<()> {
    let backend = DialoguerBackend::new();

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
    let backend = DialoguerBackend::new();

    let updated_settings = AppSettings::builder()
        .with_suggestions(&settings)
        .run(backend)?;

    println!("{updated_settings:#?}");

    Ok(())
}
