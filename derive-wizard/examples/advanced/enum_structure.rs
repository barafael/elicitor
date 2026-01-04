use derive_wizard::Wizard;

#[derive(Wizard, Debug)]
#[allow(unused)]
enum AccountType {
    Free,
    Premium {
        #[prompt("Select payment method")]
        payment: PaymentMethod,
    },
}

#[derive(Wizard, Debug)]
#[allow(unused)]
enum PaymentMethod {
    CreditCard,
    PayPal,
}

fn main() {
    let interview = AccountType::interview();
    println!("{:#?}", interview);
}
