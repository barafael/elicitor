use derive_wizard::Wizard;

#[derive(Debug, Clone, Wizard)]
struct UserProfile {
    #[prompt("Enter your name:")]
    name: String,

    #[prompt("Enter your age:")]
    #[min(0)]
    #[max(150)]
    age: i32,

    #[prompt("Enter your email:")]
    email: String,

    #[prompt("Subscribe to newsletter:")]
    subscribe: bool,
}

/// Choose which backend to demonstrate
#[derive(Debug, Clone, Wizard)]
enum BackendChoice {
    /// CLI prompts using requestty
    #[cfg(feature = "requestty-backend")]
    Requestty,

    /// CLI prompts using dialoguer
    #[cfg(feature = "dialoguer-backend")]
    Dialoguer,

    /// TUI interface using ratatui
    #[cfg(feature = "ratatui-backend")]
    Ratatui,

    /// GUI form using egui
    #[cfg(feature = "egui-backend")]
    Egui,

    /// Demo with pre-filled suggestions
    #[cfg(feature = "requestty-backend")]
    Suggestions,
}

fn main() {
    println!("=== Comprehensive Builder API Demo ===\n");

    // Use requestty to let the user choose which backend to demo
    let choice = BackendChoice::wizard_builder().build().unwrap();

    match choice {
        #[cfg(feature = "requestty-backend")]
        BackendChoice::Requestty => run_requestty_demo(),

        #[cfg(feature = "dialoguer-backend")]
        BackendChoice::Dialoguer => run_dialoguer_demo(),

        #[cfg(feature = "ratatui-backend")]
        BackendChoice::Ratatui => run_ratatui_demo(),

        #[cfg(feature = "egui-backend")]
        BackendChoice::Egui => run_egui_demo(),

        #[cfg(feature = "requestty-backend")]
        BackendChoice::Suggestions => run_suggestions_demo(),
    }

    println!("\n=== Demo Complete ===");
}

#[cfg(feature = "requestty-backend")]
fn run_requestty_demo() {
    println!("\n--- Requestty Backend Demo ---");
    let profile = UserProfile::wizard_builder().build().unwrap();
    println!("Profile: {:#?}", profile);
}

#[cfg(feature = "dialoguer-backend")]
fn run_dialoguer_demo() {
    println!("\n--- Dialoguer Backend Demo ---");
    let backend = derive_wizard::DialoguerBackend::new();
    let profile = UserProfile::wizard_builder()
        .with_backend(backend)
        .build()
        .unwrap();
    println!("Profile: {:#?}", profile);
}

#[cfg(feature = "ratatui-backend")]
fn run_ratatui_demo() {
    println!("\n--- Ratatui Backend Demo ---");
    let backend = derive_wizard::RatatuiBackend::new()
        .with_title("User Profile");

    let profile = UserProfile::wizard_builder()
        .with_backend(backend)
        .build()
        .unwrap();
    println!("Profile: {:#?}", profile);
}

#[cfg(feature = "egui-backend")]
fn run_egui_demo() {
    println!("\n--- Egui Backend Demo ---");
    let backend = derive_wizard::EguiBackend::new()
        .with_title("User Profile")
        .with_window_size([450.0, 350.0]);

    let profile = UserProfile::wizard_builder()
        .with_backend(backend)
        .build()
        .unwrap();
    println!("Profile: {:#?}", profile);
}

#[cfg(feature = "requestty-backend")]
fn run_suggestions_demo() {
    println!("\n--- Demo with Suggestions ---");
    let suggestions = UserProfile {
        name: "John Doe".to_string(),
        age: 30,
        email: "john@example.com".to_string(),
        subscribe: true,
    };
    let profile = UserProfile::wizard_builder()
        .with_suggestions(suggestions)
        .build()
        .unwrap();
    println!("Profile: {:#?}", profile);
}
