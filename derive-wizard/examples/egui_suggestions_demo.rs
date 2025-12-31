use derive_wizard::Wizard;

/// This example demonstrates how suggestions work with the egui backend.
/// The egui backend shows suggested values as placeholder text (hint text) in text fields.
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
    println!("=== Application Settings - egui Suggestions Demo ===\n");
    println!("This demo shows how to use the builder API with suggestions.");
    println!("First, create initial settings, then edit them with suggestions.\n");

    // First run: Create initial settings
    println!("--- First Run: Create New Settings ---");
    let backend = derive_wizard::EguiBackend::new()
        .with_title("Application Settings - New")
        .with_window_size([500.0, 450.0]);

    let settings = AppSettings::wizard_builder().with_backend(backend).build();

    println!("\n=== Settings Created ===");
    println!("{:#?}\n", settings);

    // Second run: Edit existing settings with suggestions
    println!("--- Second Run: Edit Existing Settings ---");
    println!("The current values will be shown as suggestions (placeholders).\n");

    let backend = derive_wizard::EguiBackend::new()
        .with_title("Application Settings - Edit")
        .with_window_size([500.0, 450.0]);

    let updated_settings = AppSettings::wizard_builder()
        .with_suggestions(settings)
        .with_backend(backend)
        .build();

    println!("\n=== Updated Settings ===");
    println!("{:#?}", updated_settings);
}
