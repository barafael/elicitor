use derive_survey::{ResponseValue, Responses, Survey};
use std::path::PathBuf;

/// Single validator: toppings budget (each topping = $0.50, max $3 = 6 toppings)
pub fn validate_toppings(value: &ResponseValue, _: &Responses) -> Result<(), String> {
    let ResponseValue::ChosenVariants(picks) = value else {
        return Ok(());
    };
    if picks.len() > 6 {
        return Err(format!(
            "Max 6 toppings ($3 budget) - you picked {}",
            picks.len()
        ));
    }
    Ok(())
}

/// Propagated validator for nutrition info
pub fn validate_nutrition(value: &ResponseValue, responses: &Responses) -> Result<(), String> {
    let ResponseValue::Int(current) = value else {
        return Ok(());
    };
    let cals = Nutrition::get_calories(responses).unwrap_or(0) as i64;
    let protein = Nutrition::get_protein(responses).unwrap_or(0) as i64;
    let total = cals + protein * 4 + current; // rough check
    if total > 1500 {
        return Err("That's a lot of food! Consider a lighter option.".into());
    }
    Ok(())
}

/// Bread choice
#[derive(Survey, Debug)]
pub enum Bread {
    Italian,
    Wheat,
    HoneyOat,
    Flatbread,
    Wrap,
}

/// Filling - demonstrates unit, newtype, and struct variants
#[derive(Survey, Debug)]
pub enum Filling {
    Turkey,
    Ham,
    RoastBeef,
    Tuna,
    Meatball,
    VeggiePatty,
    /// Double portion
    Double(#[ask("Which filling to double?")] FillingType),
    /// Custom combo
    Combo {
        #[ask("First filling:")]
        first: FillingType,
        #[ask("Second filling:")]
        second: FillingType,
    },
}

#[derive(Survey, Debug)]
pub enum FillingType {
    Turkey,
    Ham,
    Bacon,
    Chicken,
    Falafel,
}

/// Cheese selection
#[derive(Survey, Debug)]
pub enum Cheese {
    American,
    Provolone,
    Swiss,
    Cheddar,
    Pepper,
    None,
}

/// Toppings for multi-select
#[derive(Survey, Debug)]
pub enum Topping {
    Lettuce,
    Tomato,
    Onion,
    Pickle,
    Olive,
    Jalapeno,
    Banana,
    Spinach,
    Avocado,
    Bacon,
}

/// Sauce choice
#[derive(Survey, Debug)]
pub enum Sauce {
    Mayo,
    Mustard,
    Ranch,
    Chipotle,
    OilVinegar,
    None,
}

/// Size options
#[derive(Survey, Debug)]
pub enum Size {
    #[ask("6 inch ($7)")]
    Six,
    #[ask("Footlong ($12)")]
    Footlong,
}

/// Nested struct for nutrition tracking
#[derive(Survey, Debug)]
#[validate_fields(validate_nutrition)]
pub struct Nutrition {
    #[ask("Calorie limit:")]
    #[min(200)]
    #[max(1200)]
    pub calories: u32,

    #[ask("Protein goal (g):")]
    #[min(10)]
    #[max(100)]
    pub protein: u32,
}

/// Main sandwich order
#[derive(Survey, Debug)]
#[prelude("Welcome to Rusty's Subs!\nLet's build your perfect sandwich.\n")]
#[epilogue("Order placed! Your sandwich will be ready in 5 minutes.")]
pub struct SandwichOrder {
    #[ask("Name for the order:")]
    pub name: String,

    #[ask("Rewards PIN (4 digits):")]
    #[mask]
    pub pin: String,

    #[ask("Choose your bread:")]
    pub bread: Bread,

    #[ask("Select your filling:")]
    pub filling: Filling,

    #[ask("What cheese?")]
    pub cheese: Cheese,

    #[ask("Pick your toppings (max 6, $0.50 each):")]
    #[multiselect]
    #[validate(validate_toppings)]
    pub toppings: Vec<Topping>,

    #[ask("Choose a sauce:")]
    pub sauce: Sauce,

    #[ask("What size?")]
    pub size: Size,

    #[ask("Toast it?")]
    pub toasted: bool,

    #[ask("Nutrition preferences:")]
    pub nutrition: Nutrition,

    #[ask("Tip amount (-$5 to +$20):")]
    #[min(-5)]
    #[max(20)]
    pub tip: i32,

    #[ask("Special instructions:")]
    #[multiline]
    pub notes: String,

    #[ask("Receipt file (optional):")]
    pub receipt_path: Option<PathBuf>,
}
