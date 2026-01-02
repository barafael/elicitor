use derive_wizard::Wizard;

#[derive(Debug, Wizard)]
#[allow(unused)]
struct Order {
    #[prompt("Enter product name:")]
    name: String,

    #[prompt("Select payment method:")]
    payment: PaymentMethod,
}

#[derive(Debug, Wizard)]
#[allow(unused)]
enum PaymentMethod {
    Cash,

    CreditCard {
        #[prompt("Card number:")]
        card_number: String,

        #[prompt("Expiry:")]
        expiry: String,

        #[prompt("CVV:")]
        #[mask]
        cvv: String,
    },

    BankTransfer {
        #[prompt("Account number:")]
        account: String,

        #[prompt("Routing number:")]
        routing: String,
    },
}

fn main() {
    let order = Order::wizard_builder().build();
    println!("{:#?}", order);
}
