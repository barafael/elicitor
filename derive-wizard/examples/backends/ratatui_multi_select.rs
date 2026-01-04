//! Example demonstrating multi-select with ratatui backend
//!
//! Run with: cargo run --example ratatui_multi_select --features ratatui-backend

use derive_wizard::{
    Wizard,
    backend::{InterviewBackend, RatatuiBackend, RatatuiTheme},
};

/// Toppings for a custom pizza order
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Topping {
    Pepperoni,
    Mushrooms,
    Onions,
    Sausage,
    Bacon,
    ExtraCheese,
    GreenPeppers,
    BlackOlives,
    Pineapple,
    Jalapenos,
}

impl std::fmt::Display for Topping {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Topping::Pepperoni => write!(f, "Pepperoni"),
            Topping::Mushrooms => write!(f, "Mushrooms"),
            Topping::Onions => write!(f, "Onions"),
            Topping::Sausage => write!(f, "Italian Sausage"),
            Topping::Bacon => write!(f, "Bacon"),
            Topping::ExtraCheese => write!(f, "Extra Cheese"),
            Topping::GreenPeppers => write!(f, "Green Peppers"),
            Topping::BlackOlives => write!(f, "Black Olives"),
            Topping::Pineapple => write!(f, "Pineapple"),
            Topping::Jalapenos => write!(f, "Jalapeños"),
        }
    }
}

/// Dipping sauces
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sauce {
    Marinara,
    Ranch,
    BlueCheeese,
    GarlicButter,
    BuffaloSauce,
}

impl std::fmt::Display for Sauce {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Sauce::Marinara => write!(f, "Marinara"),
            Sauce::Ranch => write!(f, "Ranch"),
            Sauce::BlueCheeese => write!(f, "Blue Cheese"),
            Sauce::GarlicButter => write!(f, "Garlic Butter"),
            Sauce::BuffaloSauce => write!(f, "Buffalo Sauce"),
        }
    }
}

/// Pizza order with multi-select for toppings
#[derive(Debug, Wizard)]
#[wizard(title = "Pizza Order Form", prelude = "Build your perfect pizza!")]
pub struct PizzaOrder {
    /// Customer name
    #[wizard(prompt = "Name for the order:")]
    customer_name: String,

    /// Pizza size
    #[wizard(prompt = "What size pizza would you like?")]
    size: PizzaSize,

    /// Selected toppings (multi-select)
    #[wizard(prompt = "Choose your toppings:")]
    toppings: Vec<Topping>,

    /// Dipping sauces (multi-select)
    #[wizard(prompt = "Any dipping sauces?")]
    sauces: Vec<Sauce>,

    /// Special instructions
    #[wizard(prompt = "Any special instructions?")]
    instructions: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PizzaSize {
    Personal,
    #[default]
    Medium,
    Large,
    ExtraLarge,
}

impl std::fmt::Display for PizzaSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PizzaSize::Personal => write!(f, "Personal (8\")"),
            PizzaSize::Medium => write!(f, "Medium (12\")"),
            PizzaSize::Large => write!(f, "Large (14\")"),
            PizzaSize::ExtraLarge => write!(f, "Extra Large (16\")"),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Use a warm pizza-themed color scheme
    let pizza_theme = RatatuiTheme {
        primary: ratatui::style::Color::Rgb(255, 165, 0), // Orange
        secondary: ratatui::style::Color::Rgb(255, 99, 71), // Tomato red
        highlight: ratatui::style::Color::Rgb(255, 215, 0), // Gold
        text: ratatui::style::Color::White,
        border: ratatui::style::Color::Rgb(139, 69, 19), // Saddle brown
        error: ratatui::style::Color::Red,
        success: ratatui::style::Color::Rgb(50, 205, 50), // Lime green
    };

    let backend = RatatuiBackend::new().with_theme(pizza_theme);

    let interview = PizzaOrder::build_interview();
    let answers = backend.execute(&interview)?;

    let order = PizzaOrder::from_answers(&answers)?;

    println!("\n🍕 Order Summary 🍕\n");
    println!("Customer: {}", order.customer_name);
    println!("Size: {}", order.size);

    if order.toppings.is_empty() {
        println!("Toppings: Plain cheese");
    } else {
        println!("Toppings ({}):", order.toppings.len());
        for topping in &order.toppings {
            println!("  • {}", topping);
        }
    }

    if order.sauces.is_empty() {
        println!("Dipping Sauces: None");
    } else {
        println!("Dipping Sauces ({}):", order.sauces.len());
        for sauce in &order.sauces {
            println!("  • {}", sauce);
        }
    }

    if !order.instructions.is_empty() {
        println!("Special Instructions: {}", order.instructions);
    }

    // Calculate a fun price
    let base_price = match order.size {
        PizzaSize::Personal => 8.99,
        PizzaSize::Medium => 12.99,
        PizzaSize::Large => 15.99,
        PizzaSize::ExtraLarge => 18.99,
    };
    let toppings_cost = order.toppings.len() as f64 * 1.50;
    let sauces_cost = order.sauces.len() as f64 * 0.75;
    let total = base_price + toppings_cost + sauces_cost;

    println!("\n💰 Total: ${:.2}", total);

    Ok(())
}
