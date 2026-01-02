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
    let form = LoginForm::wizard_builder().build();
    println!("LoginForm: {form:#?}");
}
