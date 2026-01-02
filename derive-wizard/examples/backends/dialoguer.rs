use derive_wizard::Wizard;

#[derive(Debug, Wizard)]
#[allow(dead_code)]
enum Gender {
    Male,
    Female,
    Other(#[prompt("Please specify:")] String),
}

#[derive(Debug, Wizard)]
#[allow(dead_code)]
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
    println!("=== User Profile Wizard - dialoguer Demo ===");

    // Use the dialoguer backend with builder API
    let backend = derive_wizard::DialoguerBackend::new();
    let profile = UserProfile::wizard_builder().with_backend(backend).build();

    println!("=== Profile Created ===");
    println!("{:#?}", profile);
}
