//! Nested structs and enums example for the egui backend.
//!
//! This example demonstrates:
//! - OneOf (enum selection with radio buttons)
//! - AllOf (nested structs)
//! - Variants with data
//!
//! Run with: cargo run -p derive-egui-form --example nested_structs

use derive_survey::Survey;
use derive_egui_form::EguiBackend;

/// Payment method selection (OneOf example).
#[allow(dead_code)]
#[derive(Debug, Survey)]
enum PaymentMethod {
    #[ask("Credit Card")]
    CreditCard {
        #[ask("Card number:")]
        card_number: String,
        #[ask("Expiry (MM/YY):")]
        expiry: String,
        #[ask("CVV:")]
        #[mask]
        cvv: String,
    },

    #[ask("PayPal")]
    PayPal {
        #[ask("PayPal email:")]
        email: String,
    },

    #[ask("Bank Transfer")]
    BankTransfer {
        #[ask("Account number:")]
        account_number: String,
        #[ask("Routing number:")]
        routing_number: String,
    },

    #[ask("Cash on Delivery")]
    CashOnDelivery,
}

/// Shipping address (AllOf example - nested struct).
#[derive(Debug, Survey)]
struct Address {
    #[ask("Street address:")]
    street: String,

    #[ask("City:")]
    city: String,

    #[ask("State/Province:")]
    state: String,

    #[ask("Postal code:")]
    postal_code: String,

    #[ask("Country:")]
    country: String,
}

/// Shipping speed options.
#[derive(Debug, Survey)]
enum ShippingSpeed {
    #[ask("Standard (5-7 business days)")]
    Standard,

    #[ask("Express (2-3 business days)")]
    Express,

    #[ask("Overnight (next business day)")]
    Overnight,
}

/// Complete order form with nested structures.
#[allow(dead_code)]
#[derive(Debug, Survey)]
#[prelude("Complete your order by filling in the details below.")]
#[epilogue("Thank you for your order! We'll process it shortly.")]
struct OrderForm {
    #[ask("Your name:")]
    customer_name: String,

    #[ask("Email for order confirmation:")]
    email: String,

    #[ask("Phone number:")]
    phone: String,

    #[ask("Shipping Address")]
    shipping_address: Address,

    #[ask("Shipping speed:")]
    shipping_speed: ShippingSpeed,

    #[ask("Payment method:")]
    payment_method: PaymentMethod,

    #[ask("Order notes (optional):")]
    #[multiline]
    notes: String,

    #[ask("Save for future orders?")]
    save_details: bool,
}

fn main() -> anyhow::Result<()> {
    println!("=== Order Form - egui Nested Structs Demo ===\n");

    let backend = EguiBackend::new()
        .with_title("Order Form")
        .with_window_size([550.0, 700.0]);

    let order: OrderForm = OrderForm::builder().run(backend)?;

    println!("\n=== Order Submitted ===");
    println!("Customer: {}", order.customer_name);
    println!("Email: {}", order.email);
    println!("Phone: {}", order.phone);
    println!("\nShipping Address:");
    println!("  {}", order.shipping_address.street);
    println!(
        "  {}, {} {}",
        order.shipping_address.city,
        order.shipping_address.state,
        order.shipping_address.postal_code
    );
    println!("  {}", order.shipping_address.country);
    println!("\nShipping Speed: {:?}", order.shipping_speed);
    println!("Payment Method: {:?}", order.payment_method);
    if !order.notes.is_empty() {
        println!("\nNotes: {}", order.notes);
    }

    Ok(())
}
