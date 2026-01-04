use derive_wizard::Wizard;

#[derive(Debug, Clone, Wizard)]
struct AppSettings {
    #[prompt("Application name:")]
    app_name: String,

    #[prompt("Port number:")]
    #[min(1024)]
    #[max(65535)]
    port: i32,

    #[prompt("Enable debug mode:")]
    debug_mode: bool,
}

fn main() {
    println!("=== Builder API with Egui Backend ===");

    // Using builder with egui backend
    let backend = derive_wizard::EguiBackend::new()
        .with_title("Application Settings")
        .with_window_size([500.0, 400.0]);

    let settings = AppSettings::wizard_builder().with_backend(backend).build().unwrap();

    println!("=== Settings Created ===");
    println!("{:#?}", settings);
}
