use derive_wizard::Wizard;

/// This example demonstrates how suggestions work in the dialoguer backend.
/// The dialoguer backend shows suggested values in square brackets [default].
/// Simply press Enter to accept the suggested value.
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
    println!("=== Application Settings - dialoguer Suggestions Demo ===\n");
    println!("This demo showcases how dialoguer displays suggested values.");
    println!("Suggested values appear in [square brackets].");
    println!("Press Enter to accept a suggestion, or type a new value.\n");

    // Create initial settings with builder API
    let backend = derive_wizard::DialoguerBackend::new();
    println!("--- First Run: Create New Settings ---\n");
    let settings = AppSettings::wizard_builder().with_backend(backend).build();

    println!("\n=== Settings Created ===");
    println!("{:#?}\n", settings);

    // Edit existing settings with suggestions using builder API
    println!("--- Second Run: Edit Existing Settings ---");
    println!("The current values will be shown as suggestions.\n");
    let backend = derive_wizard::DialoguerBackend::new();
    let updated_settings = AppSettings::wizard_builder()
        .with_suggestions(settings)
        .with_backend(backend)
        .build();

    println!("\n=== Updated Settings ===");
    println!("{:#?}", updated_settings);
}
