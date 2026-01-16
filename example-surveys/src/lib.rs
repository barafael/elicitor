pub mod app_settings;
pub mod basic_fields;
pub mod enum_oneof;
pub mod job_application;
pub mod masked_input;
pub mod min_max_bounds;
pub mod multiline_text;
pub mod multiselect;
pub mod nested_struct;
pub mod optional_fields;
pub mod order_form;
pub mod prelude_epilogue;
pub mod sandwich;
pub mod simple_spooky_forest;
pub mod spooky_forest;
pub mod user_profile;
pub mod validation;
pub mod vec_lists;

// Re-export app_settings types
pub use app_settings::AppSettings;

// Re-export basic_fields types
pub use basic_fields::BasicFields;

// Re-export enum_oneof types
pub use enum_oneof::{Checkout, PaymentMethod, ShippingMethod};

// Re-export job_application types
pub use job_application::{
    Experience, FocusArea, JobApplication, JobSkill, MAX_TOTAL_COMP, Position, Referral, Salary,
    WorkStyle,
};

// Re-export spooky_forest types
pub use spooky_forest::{
    Cast, CharacterStats, Companion, CompanionDetails, CompanionSpecies, FamiliarForm,
    HomeLocation, Item, Language, MAX_STAT_POINTS, Role, Skill, SpookyForest, WandMaterial,
};

// Re-export masked_input types
pub use masked_input::{Login, Passwords, passwords_match};

// Re-export min_max_bounds types
pub use min_max_bounds::GameSettings;

// Re-export multiline_text types
pub use multiline_text::BlogPost;

// Re-export multiselect types
pub use multiselect::{DeveloperProfile, Hobby, ProgrammingLanguage};

// Re-export nested_struct types
pub use nested_struct::{Address, ContactInfo, UserRegistration};

// Re-export optional_fields types
pub use optional_fields::ProjectConfig;

// Re-export order_form types
pub use order_form::{OrderForm, PaymentMethod2, ShippingAddress, ShippingSpeed};

// Re-export prelude_epilogue types
pub use prelude_epilogue::FitnessProfile;

// Re-export sandwich types
pub use sandwich::{
    Bread, Cheese, Filling, FillingType, Nutrition, SandwichOrder, Sauce, Size, Topping,
    validate_nutrition, validate_toppings,
};

// Re-export simple_spooky_forest types
pub use simple_spooky_forest::{
    SimpleItem, SimpleRole, SimpleSpookyForest, is_valid_name, is_within_starting_budget,
};

// Re-export user_profile types
pub use user_profile::UserProfile;

// Re-export validation types
pub use validation::{AccountCreation, validate_email, validate_password, validate_username};

// Re-export vec_lists types
pub use vec_lists::{ShoppingList, StudentGrades};
