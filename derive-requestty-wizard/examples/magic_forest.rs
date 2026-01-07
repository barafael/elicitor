use derive_survey::{ResponseValue, Responses, Survey};
use derive_requestty_wizard::RequesttyBackend;

#[allow(dead_code)]
#[derive(Survey, Debug)]
#[prelude("A journey begins...!")]
#[epilogue("Good luck.")]
struct MagicForest {
    #[ask("What is your name?")]
    #[validate(is_valid_name)]
    name: String,

    #[ask("What's the secret passphrase?")]
    #[mask]
    passphrase: String,

    #[ask("How old are you?")]
    #[min(18)]
    #[max(233)]
    age: u32,

    #[ask("What is your role?")]
    role: Role,

    #[ask("Pick your inventory:")]
    #[multiselect]
    #[validate(is_within_starting_budget)]
    inventory: Vec<Item>,
}

#[allow(dead_code)]
#[derive(Survey, Debug)]
enum Role {
    Streetfighter,
    Mage,
    Archer,
    Thief,
    Other(#[ask("What then?!")] String),
}

#[allow(dead_code)]
#[derive(Survey, Debug)]
enum Item {
    #[ask("Sword (value: 80)")]
    Sword,

    #[ask("Shield (value: 50)")]
    Shield,

    #[ask("Potion (value: 20)")]
    Potion,

    #[ask("Scroll (value: 10)")]
    Scroll,

    #[ask("Chewing Gum (value: 2 * quantity)")]
    ChewingGum {
        flavor: String,
        #[ask("Quando?")]
        quantity: u32,
    },
}

fn is_valid_name(value: &ResponseValue, _responses: &Responses) -> Result<(), String> {
    let ResponseValue::String(name) = value else {
        return Ok(()); // Not a string, skip validation
    };
    if name.len() > 2 && name.len() < 100 {
        Ok(())
    } else {
        Err("Name must be between 3 and 99 characters".to_string())
    }
}

fn is_within_starting_budget(value: &ResponseValue, _responses: &Responses) -> Result<(), String> {
    // The validator receives ResponseValue::ChosenVariants with the selected indices
    // right after the multiselect, before nested fields are asked.
    let ResponseValue::ChosenVariants(selections) = value else {
        return Ok(()); // Not a selection, skip
    };

    const STARTING_BUDGET: u32 = 150;
    let mut total_cost: u32 = 0;

    for &variant_idx in selections {
        let item_cost = match variant_idx {
            0 => 80, // Sword
            1 => 50, // Shield
            2 => 20, // Potion
            3 => 10, // Scroll
            4 => 2,  // ChewingGum base cost (2 * 1 minimum)
            _ => 0,
        };
        total_cost += item_cost;
    }

    if total_cost <= STARTING_BUDGET {
        Ok(())
    } else {
        Err(format!(
            "Over budget! Total: {} gold, limit: {} gold",
            total_cost, STARTING_BUDGET
        ))
    }
}

fn main() {
    // run the survey with the requestty backend
    let survey_result = MagicForest::builder().run(RequesttyBackend::new()).unwrap();

    println!("{:#?}", survey_result);
}
