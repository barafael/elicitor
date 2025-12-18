use derive_wizard::Wizard;

fn main() {
    let magic = ShowCase::wizard();
    println!("Config: {magic:#?}");
}

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

    // Long text with editor
    #[prompt("Enter a bio:")]
    #[editor]
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
