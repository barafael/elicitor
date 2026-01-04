//! Pizza Order Wizard 🍕
//!
//! A fun example showing enum selections and nested structures.
//!
//! Run with: cargo run --example ratatui_pizza --features ratatui-backend

use derive_wizard::Wizard;

#[derive(Debug, Wizard)]
#[allow(dead_code)]
enum PizzaSize {
    Small,
    Medium,
    Large,
    ExtraLarge,
}

#[derive(Debug, Wizard)]
#[allow(dead_code)]
enum Crust {
    Thin,
    Regular,
    ThickPan,
    Stuffed,
}

#[derive(Debug, Wizard)]
#[allow(dead_code)]
enum Topping {
    Pepperoni,
    Mushrooms,
    Olives,
    Peppers,
    ExtraCheese,
    Sausage,
    Bacon,
    Pineapple,
}

#[derive(Debug, Wizard)]
#[allow(dead_code)]
struct CustomerInfo {
    #[prompt("Your name:")]
    name: String,

    #[prompt("Phone number:")]
    phone: String,

    #[prompt("Delivery address:")]
    address: String,
}

#[derive(Debug, Wizard)]
#[allow(dead_code)]
#[prelude("🍕 Welcome to Pizza Palace! 🍕\nLet's build your perfect pizza.")]
#[epilogue("Your order has been placed!\nEstimated delivery: 30-45 minutes 🚗")]
struct PizzaOrder {
    #[prompt("Customer information")]
    customer: CustomerInfo,

    #[prompt("Select pizza size:")]
    size: PizzaSize,

    #[prompt("Choose your crust:")]
    crust: Crust,

    #[prompt("Pick a topping:")]
    topping1: Topping,

    #[prompt("Add another topping?")]
    extra_topping: bool,

    #[prompt("How many pizzas?")]
    #[min(1)]
    #[max(10)]
    quantity: i64,

    #[prompt("Any special instructions?")]
    instructions: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use derive_wizard::{InterviewBackend, RatatuiBackend, RatatuiColor, RatatuiTheme};

    let theme = RatatuiTheme {
        primary: RatatuiColor::Red,
        secondary: RatatuiColor::Yellow,
        highlight: RatatuiColor::LightYellow,
        success: RatatuiColor::Green,
        error: RatatuiColor::LightRed,
        ..RatatuiTheme::default()
    };

    let interview = PizzaOrder::interview();
    let backend = RatatuiBackend::new()
        .with_title("🍕 Pizza Palace Order System")
        .with_theme(theme);

    let answers = backend.execute(&interview)?;
    let order = PizzaOrder::from_answers(&answers)?;

    println!("\n🧾 Order Summary:");
    println!("═══════════════════════════════════════");
    println!("{:#?}", order);

    Ok(())
}
