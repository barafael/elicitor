//! Suggestions and assumptions example for the egui backend.
//!
//! This example demonstrates:
//! - Setting default/suggested values that users can modify
//! - Assumptions that skip questions entirely
//! - Editing existing data
//!
//! Run with: cargo run -p derive-egui-form --example suggestions

use derive_egui_form::EguiBackend;
use derive_survey::Survey;

/// Application settings with suggested defaults.
#[derive(Debug, Clone, Survey)]
struct AppSettings {
    #[ask("Application name:")]
    app_name: String,

    #[ask("Port number:")]
    #[min(1024)]
    #[max(65535)]
    port: i64,

    #[ask("Max connections:")]
    #[min(1)]
    #[max(10000)]
    max_connections: i64,

    #[ask("Timeout in seconds:")]
    #[min(1)]
    #[max(300)]
    timeout: i64,

    #[ask("Enable debug mode:")]
    debug_mode: bool,

    #[ask("Log file path:")]
    log_path: String,
}

fn main() -> anyhow::Result<()> {
    println!("=== Application Settings - egui Suggestions Demo ===");
    println!("This demo shows how to use the builder API with suggestions.\n");

    // First run: Create settings with suggested defaults
    println!("--- First Run: Create New Settings with Suggestions ---");

    let backend = EguiBackend::new()
        .with_title("Application Settings - New")
        .with_window_size([500.0, 450.0]);

    let settings: AppSettings = AppSettings::builder()
        .suggest_app_name("my-awesome-app")
        .suggest_port(8080)
        .suggest_max_connections(100)
        .suggest_timeout(30)
        .suggest_debug_mode(false)
        .suggest_log_path("/var/log/app.log")
        .run(backend)?;

    println!("\n=== Settings Created ===");
    println!("{:#?}", settings);

    // Second run: Edit existing settings
    println!("\n--- Second Run: Edit Existing Settings ---");
    println!("The current values will be shown as defaults.");

    let backend = EguiBackend::new()
        .with_title("Application Settings - Edit")
        .with_window_size([500.0, 450.0]);

    // Use with_suggestions to pre-fill from existing instance
    let updated_settings: AppSettings = AppSettings::builder()
        .with_suggestions(&settings)
        .run(backend)?;

    println!("\n=== Updated Settings ===");
    println!("{:#?}", updated_settings);

    // Show what changed
    println!("\n=== Changes ===");
    if settings.app_name != updated_settings.app_name {
        println!(
            "App name: {} -> {}",
            settings.app_name, updated_settings.app_name
        );
    }
    if settings.port != updated_settings.port {
        println!("Port: {} -> {}", settings.port, updated_settings.port);
    }
    if settings.max_connections != updated_settings.max_connections {
        println!(
            "Max connections: {} -> {}",
            settings.max_connections, updated_settings.max_connections
        );
    }
    if settings.timeout != updated_settings.timeout {
        println!(
            "Timeout: {} -> {}",
            settings.timeout, updated_settings.timeout
        );
    }
    if settings.debug_mode != updated_settings.debug_mode {
        println!(
            "Debug mode: {} -> {}",
            settings.debug_mode, updated_settings.debug_mode
        );
    }
    if settings.log_path != updated_settings.log_path {
        println!(
            "Log path: {} -> {}",
            settings.log_path, updated_settings.log_path
        );
    }

    Ok(())
}
