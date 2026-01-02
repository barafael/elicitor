use derive_wizard::Wizard;

#[derive(Debug, Wizard)]
#[allow(unused)]
struct Address {
    #[prompt("Street address:")]
    street: String,

    #[prompt("City:")]
    city: String,

    #[prompt("ZIP code:")]
    zip: String,
}

#[derive(Debug, Wizard)]
#[allow(unused)]
struct ContactInfo {
    #[prompt("Email:")]
    email: String,

    #[prompt("Phone:")]
    phone: String,
}

#[derive(Debug, Wizard)]
#[allow(unused)]
struct User {
    #[prompt("Full name:")]
    name: String,

    #[prompt("Age:")]
    age: u32,

    #[prompt]
    address: Address,

    #[prompt]
    contact: ContactInfo,
}

fn main() {
    let user = User::wizard_builder().build();
    println!("{user:#?}");
}
