use derive_wizard::Wizard;

#[derive(Debug, Wizard)]
#[allow(unused)]
struct ComprehensiveConfig {
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
    age_i32: i32,

    #[prompt("Enter your age (i64):")]
    age_i64: i64,

    #[prompt("Enter a small number (u8):")]
    small_num: u8,

    #[prompt("Enter a medium number (u16):")]
    medium_num: u16,

    // Float types - defaults to 'float'
    #[prompt("Enter your height in meters (f64):")]
    height: f64,

    #[prompt("Enter a decimal number (f32):")]
    decimal: f32,
}

fn main() {
    let config = ComprehensiveConfig::wizard();
    println!("Config: {config:#?}");
}
