use derive_survey::{ResponseValue, Responses, Survey};
use std::path::PathBuf;

/// Validates that a name is between 3 and 50 characters
pub fn validate_name(value: &ResponseValue, _responses: &Responses) -> Result<(), String> {
    let ResponseValue::String(name) = value else {
        return Ok(());
    };
    if name.trim().is_empty() {
        return Err("Name cannot be empty".to_string());
    }
    if name.len() < 3 {
        return Err("Name must be at least 3 characters".to_string());
    }
    if name.len() > 50 {
        return Err("Name must be at most 50 characters".to_string());
    }
    if !name.chars().all(|c| c.is_alphabetic() || c.is_whitespace()) {
        return Err("Name can only contain letters and spaces".to_string());
    }
    Ok(())
}

/// Validates email format
pub fn validate_email(value: &ResponseValue, _responses: &Responses) -> Result<(), String> {
    let ResponseValue::String(email) = value else {
        return Ok(());
    };
    if !email.contains('@') || !email.contains('.') {
        return Err("Please enter a valid email address".to_string());
    }
    let parts: Vec<&str> = email.split('@').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        return Err("Email must be in format 'user@domain.com'".to_string());
    }
    Ok(())
}

/// Validates the secret passphrase
pub fn validate_passphrase(value: &ResponseValue, _responses: &Responses) -> Result<(), String> {
    let ResponseValue::String(pass) = value else {
        return Ok(());
    };
    if pass.len() < 8 {
        return Err("Passphrase must be at least 8 characters".to_string());
    }
    if !pass.chars().any(|c| c.is_uppercase()) {
        return Err("Passphrase must contain at least one uppercase letter".to_string());
    }
    if !pass.chars().any(|c| c.is_numeric()) {
        return Err("Passphrase must contain at least one number".to_string());
    }
    Ok(())
}

/// Validates biography length
pub fn validate_bio(value: &ResponseValue, _responses: &Responses) -> Result<(), String> {
    let ResponseValue::String(bio) = value else {
        return Ok(());
    };
    if bio.len() > 500 {
        return Err("Biography must be at most 500 characters".to_string());
    }
    Ok(())
}

/// Validates inventory budget (called on multi-select)
pub fn validate_inventory_budget(
    value: &ResponseValue,
    _responses: &Responses,
) -> Result<(), String> {
    let ResponseValue::ChosenVariants(selections) = value else {
        return Ok(());
    };

    const STARTING_BUDGET: u32 = 200;
    let mut total_cost: u32 = 0;

    for &variant_idx in selections {
        let item_cost = match variant_idx {
            0 => 80,  // Sword
            1 => 50,  // Shield
            2 => 20,  // Potion
            3 => 10,  // Scroll
            4 => 15,  // ChewingGum base
            5 => 100, // MagicWand
            _ => 0,
        };
        total_cost += item_cost;
    }

    if total_cost > STARTING_BUDGET {
        Err(format!(
            "Over budget! Total: {} gold, limit: {} gold. Remove some items.",
            total_cost, STARTING_BUDGET
        ))
    } else {
        Ok(())
    }
}

/// Validates that at least one skill is selected
pub fn validate_skills(value: &ResponseValue, _responses: &Responses) -> Result<(), String> {
    let ResponseValue::ChosenVariants(selections) = value else {
        return Ok(());
    };
    if selections.is_empty() {
        return Err("You must select at least one skill".to_string());
    }
    if selections.len() > 3 {
        return Err("You can select at most 3 skills".to_string());
    }
    Ok(())
}

/// Validates character stats - total points cannot exceed threshold
/// This validator is called each time a stat value is entered, checking the running total.
pub const MAX_STAT_POINTS: i64 = 75;

pub fn validate_stat_total(value: &ResponseValue, responses: &Responses) -> Result<(), String> {
    let ResponseValue::Int(current_value) = value else {
        return Ok(());
    };

    // Use typed accessors instead of string paths
    let total: i64 = *current_value
        + CharacterStats::get_strength(responses).unwrap_or(0) as i64
        + CharacterStats::get_dexterity(responses).unwrap_or(0) as i64
        + CharacterStats::get_intelligence(responses).unwrap_or(0) as i64
        + CharacterStats::get_wisdom(responses).unwrap_or(0) as i64
        + CharacterStats::get_charisma(responses).unwrap_or(0) as i64
        + CharacterStats::get_constitution(responses).unwrap_or(0) as i64;

    let remaining = MAX_STAT_POINTS - total + current_value; // Points remaining after this

    if total > MAX_STAT_POINTS {
        Err(format!(
            "Total stat points ({}) exceeds maximum of {}! You have {} points remaining.",
            total,
            MAX_STAT_POINTS,
            remaining.max(0)
        ))
    } else {
        Ok(())
    }
}

#[derive(Survey, Debug)]
pub struct HomeLocation {
    #[ask("What realm do you hail from?")]
    pub realm: String,

    #[ask("What is your village name?")]
    pub village: String,

    #[ask("How far is your home from here (in leagues)?")]
    #[min(0)]
    #[max(1000)]
    pub distance_leagues: f64,
}

#[derive(Survey, Debug)]
#[validate_fields(validate_stat_total)]
pub struct CharacterStats {
    #[ask("Strength (1-20, total max 75):")]
    #[min(1)]
    #[max(20)]
    pub strength: u8,

    #[ask("Dexterity (1-20, total max 75):")]
    #[min(1)]
    #[max(20)]
    pub dexterity: u8,

    #[ask("Intelligence (1-20, total max 75):")]
    #[min(1)]
    #[max(20)]
    pub intelligence: u8,

    #[ask("Wisdom (1-20, total max 75):")]
    #[min(1)]
    #[max(20)]
    pub wisdom: u8,

    #[ask("Charisma (1-20, total max 75):")]
    #[min(1)]
    #[max(20)]
    pub charisma: u8,

    #[ask("Constitution (1-20, total max 75):")]
    #[min(1)]
    #[max(20)]
    pub constitution: u8,
}

/// Companion details - demonstrates struct enum variant
#[derive(Survey, Debug)]
pub struct CompanionDetails {
    #[ask("Companion's name:")]
    pub name: String,

    #[ask("Companion's species:")]
    pub species: CompanionSpecies,

    #[ask("Years together:")]
    #[min(0)]
    #[max(100)]
    pub years_together: u32,
}

/// Companion species enum
#[derive(Survey, Debug)]
pub enum CompanionSpecies {
    Dog,
    Cat,
    Horse,
    Dragon,
    Phoenix,
    Other(#[ask("What species?")] String),
}

#[derive(Survey, Debug)]
pub enum Role {
    /// A fierce warrior
    Warrior,
    /// A mystical spellcaster
    Mage,
    /// A stealthy shadow
    Rogue,
    /// A holy healer
    Cleric,
    /// A nature guardian
    Ranger,
    /// A musical enchanter
    Bard,
    /// Custom class with description
    Custom(#[ask("Name your custom class:")] String),
}

/// Character background - demonstrates struct variants
#[derive(Survey, Debug)]
pub enum Cast {
    Noble {
        #[ask("Name of your noble house:")]
        house_name: String,
        #[ask("Your title:")]
        title: String,
    },
    Commoner {
        #[ask("Your former trade:")]
        trade: String,
    },
    Outlaw {
        #[ask("What crime were you accused of?")]
        crime: String,
        #[ask("Are you actually guilty?")]
        guilty: bool,
    },
    Hermit {
        #[ask("Years spent in solitude:")]
        #[min(1)]
        #[max(50)]
        years: u32,
        #[ask("What wisdom did you discover?")]
        #[multiline]
        wisdom: String,
    },
    Traveler,
}

/// Companion type - demonstrates tuple and struct variants
#[derive(Survey, Debug)]
pub enum Companion {
    /// Travel alone
    None,
    /// A loyal pet
    Pet(#[ask("Pet's name:")] String),
    /// A trusted friend with full details
    Friend(CompanionDetails),
    /// A magical familiar
    Familiar {
        #[ask("Familiar's name:")]
        name: String,
        #[ask("What form does it take?")]
        form: FamiliarForm,
    },
}

/// Familiar form
#[derive(Survey, Debug)]
pub enum FamiliarForm {
    Cat,
    Owl,
    Raven,
    Toad,
    Imp,
    Sprite,
    Other(#[ask("Describe the form:")] String),
}

/// Inventory items - demonstrates multi-select with budget validation
#[derive(Survey, Debug)]
pub enum Item {
    #[ask("Sword (80 gold)")]
    Sword,
    #[ask("Shield (50 gold)")]
    Shield,
    #[ask("Potion (20 gold)")]
    Potion,
    #[ask("Scroll (10 gold)")]
    Scroll,
    #[ask("Chewing Gum (15 gold)")]
    ChewingGum {
        #[ask("Flavor:")]
        flavor: String,
        #[ask("How many pieces?")]
        #[min(1)]
        #[max(10)]
        quantity: u32,
    },
    #[ask("Magic Wand (100 gold)")]
    MagicWand {
        #[ask("Wand material:")]
        material: WandMaterial,
        #[ask("Core type:")]
        core: String,
    },
}

/// Wand material
#[derive(Survey, Debug)]
pub enum WandMaterial {
    Oak,
    Willow,
    Elder,
    Holly,
    Ebony,
}

/// Character skills - demonstrates simple multi-select
#[derive(Survey, Debug)]
pub enum Skill {
    #[ask("Swordsmanship")]
    Swordsmanship,
    #[ask("Archery")]
    Archery,
    #[ask("Magic")]
    Magic,
    #[ask("Stealth")]
    Stealth,
    #[ask("Persuasion")]
    Persuasion,
    #[ask("Alchemy")]
    Alchemy,
    #[ask("Herbalism")]
    Herbalism,
    #[ask("Lockpicking")]
    Lockpicking,
}

/// Languages known
#[derive(Survey, Debug)]
pub enum Language {
    Common,
    Elvish,
    Dwarvish,
    Orcish,
    Draconic,
    Celestial,
    Infernal,
    Sylvan,
}

/// The complete Magic Forest character creation survey
#[derive(Survey, Debug)]
#[prelude(
    "Welcome, brave adventurer, to the Magic Forest!\nYou stand at the edge of an ancient woodland, ready to begin your journey.\nFirst, tell about yourself...\n\n"
)]
#[epilogue("Your character has been created!\nMay your legend in the Magic Forest be adventurous.")]
pub struct MagicForest {
    #[ask("What is your name?")]
    #[validate(validate_name)]
    pub name: String,

    #[ask("What is your age in years?")]
    #[min(16)]
    #[max(1000)]
    pub age: u32,

    #[ask("What is your contact email? (for the guild records)")]
    #[validate(validate_email)]
    pub email: String,

    #[ask("Create a secret passphrase (8+ chars, uppercase & number required):")]
    #[mask]
    #[validate(validate_passphrase)]
    pub passphrase: String,

    #[ask("Tell us your backstory:")]
    #[multiline]
    #[validate(validate_bio)]
    pub biography: String,

    #[ask("Choose your class:")]
    pub role: Role,

    #[ask("What is your cast?")]
    pub background: Cast,

    #[ask("Allocate your character stats:")]
    pub stats: CharacterStats,

    #[ask("Where do you come from?")]
    pub home: HomeLocation,

    #[ask("Do you travel with a companion?")]
    pub companion: Companion,

    #[ask("Select your skills (1-3):")]
    #[multiselect]
    #[validate(validate_skills)]
    pub skills: Vec<Skill>,

    #[ask("What languages do you speak?")]
    #[multiselect]
    pub languages: Vec<Language>,

    #[ask("Choose your starting inventory (budget: 200 gold):")]
    #[multiselect]
    #[validate(validate_inventory_budget)]
    pub inventory: Vec<Item>,

    #[ask("Your character portrait file:")]
    pub portrait_path: PathBuf,

    #[ask("Enable hardcore mode? (permadeath)")]
    pub hardcore_mode: bool,

    #[ask("Your lucky number:")]
    #[min(-999)]
    #[max(999)]
    pub lucky_number: i32,

    #[ask("Starting gold multiplier (1-10, will be divided by 5):")]
    #[min(1)]
    #[max(10)]
    pub gold_multiplier_raw: i32,

    #[ask("Any additional notes for the Dungeon Master?")]
    #[multiline]
    pub dm_notes: String,
}
