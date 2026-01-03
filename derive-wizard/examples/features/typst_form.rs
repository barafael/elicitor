//! Example of generating a Typst form (.typ file) from a wizard
//!
//! This example demonstrates how to use the `to_typst_form()` method
//! to generate a Typst markup file that can be compiled to a PDF form.

use derive_wizard::Wizard;

#[derive(Wizard, Debug)]
#[prelude("Please fill out this registration form carefully.")]
#[epilogue("Thank you for your submission!")]
#[allow(unused)]
struct RegistrationForm {
    #[prompt("What is your full name?")]
    name: String,

    #[prompt("What is your age?")]
    #[min(18)]
    age: i64,

    #[prompt("What is your email address?")]
    email: String,

    #[prompt("Would you like to subscribe to our newsletter?")]
    newsletter: bool,

    #[prompt("Select your account type")]
    account_type: AccountType,
}

#[derive(Wizard, Debug)]
#[allow(unused)]
enum AccountType {
    Free,

    Premium {
        #[prompt("Select payment method")]
        payment: PaymentMethod,
    },

    Enterprise {
        #[prompt("Company name")]
        company: String,

        #[prompt("Number of seats")]
        #[min(5)]
        seats: i64,
    },
}

#[derive(Wizard, Debug)]
enum PaymentMethod {
    CreditCard,
    PayPal,
    BankTransfer,
}

fn main() {
    let typst_markup = RegistrationForm::to_typst_form(Some("Registration Form"));
    println!("{typst_markup}");
}
