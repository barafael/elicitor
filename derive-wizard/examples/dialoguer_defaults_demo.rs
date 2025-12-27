use derive_wizard::Wizard;

/// This example demonstrates how defaults work in the dialoguer backend.
/// The dialoguer backend shows default values in square brackets [default].
/// Simply press Enter to accept the default value.
#[derive(Debug, Clone, Wizard)]
struct AppSettings {
    #[prompt("Application name:")]
    app_name: String,

    #[prompt("Port number:")]
    #[min(1024)]
    #[max(65535)]
    port: i32,

    #[prompt("Max connections:")]
    #[min(1)]
    #[max(10000)]
    max_connections: i32,

    #[prompt("Timeout in seconds:")]
    #[min(0.1)]
    #[max(300.0)]
    timeout: f64,

    #[prompt("Enable debug mode:")]
    debug_mode: bool,

    #[prompt("Log level:")]
    log_level: String,
}

fn main() {
    println!("=== Application Settings - dialoguer Defaults Demo ===\n");
    println!("This demo showcases how dialoguer displays default values.");
    println!("Default values appear in [square brackets].");
    println!("Press Enter to accept a default, or type a new value.\n");

    // Create initial settings
    let backend = derive_wizard::DialoguerBackend::new();
    println!("--- First Run: Create New Settings ---\n");
    let settings = AppSettings::wizard_with_backend(&backend);

    println!("\n=== Settings Created ===");
    println!("{:#?}\n", settings);

    // Edit existing settings with wizard_with_defaults
    println!("--- Second Run: Edit Existing Settings ---");
    println!("The current values will be shown as defaults.\n");
    let updated_settings = settings.wizard_with_defaults();

    println!("\n=== Updated Settings ===");
    println!("{:#?}", updated_settings);
}
