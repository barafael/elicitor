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

    // Long text with editor
    #[prompt("Enter a bio:")]
    #[editor]
    bio: String,

    // Bool type - defaults to 'confirm'
    #[prompt("Do you agree to the terms?")]
    agree: bool,

    // Integer types - defaults to 'int'
    #[prompt("Enter your age (i32):")]
    #[min(0)]
    #[max(150)]
    age: i32,

    // Float types - defaults to 'float'
    #[prompt("Enter your height in meters (f64):")]
    #[min(0.3)]
    #[max(3.0)]
    height: f64,

    #[prompt("Enter a decimal number (f32):")]
    #[min(0.0)]
    #[max(100.0)]
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
    println!("=== Derive Wizard Showcase ===");
    println!("Demonstrating all major field types and attributes");
    println!();

    #[cfg(feature = "egui-backend")]
    {
        println!("Using egui GUI backend");
        let backend = derive_wizard::EguiBackend::new()
            .with_title("Derive Wizard Showcase")
            .with_window_size([600.0, 700.0]);

        let magic = ShowCase::wizard_builder().with_backend(backend).build();
        println!("=== Configuration Created ===");
        println!("{magic:#?}");
    }

    #[cfg(not(feature = "egui-backend"))]
    {
        println!("Using default (requestty) backend");
        println!("Run with --features egui-backend to use the GUI version");
        println!();

        let magic = ShowCase::wizard_builder().build();
        println!("=== Configuration Created ===");
        println!("{magic:#?}");
    }
}
