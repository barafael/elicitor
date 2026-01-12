pub mod job_application;
pub mod magic_forest;
pub mod sandwich;
pub mod user_profile;

pub use job_application::{
    Experience, FocusArea, JobApplication, JobSkill, MAX_TOTAL_COMP, Position, Referral, Salary,
    WorkStyle, validate_cover_letter, validate_email as validate_job_email, validate_password,
    validate_salary, validate_skills as validate_job_skills,
};
pub use magic_forest::{
    Cast, CharacterStats, Companion, CompanionDetails, CompanionSpecies, FamiliarForm,
    HomeLocation, Item, Language, MAX_STAT_POINTS, MagicForest, Role, Skill, WandMaterial,
    validate_bio, validate_email as validate_magic_email, validate_inventory_budget, validate_name,
    validate_passphrase, validate_skills as validate_magic_skills, validate_stat_total,
};
pub use sandwich::{
    Bread, Cheese, Filling, FillingType, Nutrition, SandwichOrder, Sauce, Size, Topping,
    validate_nutrition, validate_toppings,
};
pub use user_profile::UserProfile;
