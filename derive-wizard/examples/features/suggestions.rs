use derive_wizard::Wizard;

#[derive(Debug, Wizard)]
#[allow(unused)]
struct UserProfile {
    #[prompt("Enter your name:")]
    name: String,

    #[prompt("Enter your age:")]
    age: u16,

    #[prompt("Are you a developer?")]
    is_developer: bool,
}

fn main() {
    println!("=== Creating a new user profile ===");
    let profile = UserProfile::wizard_builder().build();
    println!("Created profile: {profile:#?}");

    println!("=== Editing the existing profile ===");
    println!("The current values will be pre-filled as suggestions.");
    let updated_profile = UserProfile::wizard_builder()
        .with_suggestions(profile)
        .build();
    println!("Updated profile: {updated_profile:#?}");
}
