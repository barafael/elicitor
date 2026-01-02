use derive_wizard::Wizard;

#[derive(Debug, Wizard)]
#[allow(unused)]
enum Gender {
    Male,
    Female,
    Other(#[prompt("Please specify:")] String),
}

#[derive(Debug, Wizard)]
#[prelude("This profile will be seen by other users.")]
#[epilogue("Welcome! Have fun.")]
#[allow(unused)]
struct UserProfile {
    #[prompt("Enter your name:")]
    name: String,

    #[prompt("Enter your age:")]
    #[min(0)]
    #[max(150)]
    age: i32,

    #[prompt("Enter your height (in meters):")]
    #[min(0.3)]
    #[max(3.0)]
    height: f64,

    #[prompt("Enter your email:")]
    email: String,

    #[prompt("Do you agree to the terms?")]
    agree: bool,

    #[prompt("Select your gender:")]
    gender: Gender,
}

fn main() {
    println!("=== User Profile Wizard - egui Demo ===");

    // Use the egui backend with builder API
    let backend = derive_wizard::EguiBackend::new()
        .with_title("User Profile Wizard")
        .with_window_size([400.0, 300.0]);

    let profile = UserProfile::wizard_builder().with_backend(backend).build();

    println!("=== Profile Created ===");
    println!("{:#?}", profile);
}
