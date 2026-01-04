use derive_wizard::Wizard;

#[derive(Debug, Wizard)]
#[allow(unused)]
struct ShowCase {
    // String types - defaults to 'input'
    #[prompt("Enter your name:")]
    name: String,

    // Override with password question type
    #[prompt("Enter your password:")]
    #[mask]
    password: String,

    // Long text with multiline editor
    #[prompt("Enter a bio:")]
    #[multiline]
    bio: String,

    // Bool type - defaults to 'confirm'
    #[prompt("Do you agree to the terms?")]
    agree: bool,

    // Integer types - defaults to 'int'
    #[prompt("Enter your age (i32):")]
    age: i32,

    // Float types - defaults to 'float'
    #[prompt("Enter your height in meters (f64):")]
    height: f64,

    #[prompt("Enter a decimal number (f32):")]
    decimal: f32,

    #[prompt("Enter your gender")]
    gender: Gender,
}

#[derive(Debug, Wizard)]
#[allow(unused)]
enum Gender {
    Male,
    Female,
    Other(#[prompt("Please specify:")] String),
}

fn main() {
    println!("=== Creating a new configuration ===");
    let config = ShowCase::wizard_builder().build().unwrap();
    println!("Config: {config:#?}");

    println!("=== Editing the configuration with suggestions ===");
    println!("The current values will be pre-filled. Press Enter to keep them or type new values.");
    let updated_config = ShowCase::wizard_builder().with_suggestions(config).build().unwrap();
    println!("Updated Config: {updated_config:#?}");
}
