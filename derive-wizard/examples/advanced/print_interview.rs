use derive_wizard::Wizard;

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
#[allow(unused)]
enum PaymentMethod {
    CreditCard,
    PayPal,
    BankTransfer,
}

fn main() {
    let interview = AccountType::interview();
    println!("{:#?}", interview);
}
