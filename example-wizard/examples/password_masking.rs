use derive_wizard::Wizard;

#[derive(Debug, Wizard)]
#[allow(unused)]
struct LoginForm {
    #[prompt("Enter username:")]
    username: String,

    #[prompt("Enter password:")]
    #[mask]
    password: String,
}

fn main() {
    println!("Testing #[mask] attribute...");
    println!("This example demonstrates the #[mask] attribute");
    println!("which creates a password field with hidden input.");

    let form = LoginForm::wizard();
    println!("LoginForm: {form:#?}");
}
