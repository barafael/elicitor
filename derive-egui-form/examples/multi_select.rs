//! Multi-select example demonstrating AnyOf (Vec<Enum>) in the egui backend.
//!
//! This example shows how to use multi-select checkboxes for enum vectors.
//!
//! Run with: cargo run -p derive-egui-form --example multi_select

use derive_survey::{ResponseValue, Responses, Survey};
use derive_egui_form::EguiBackend;

/// Programming languages for the survey.
#[derive(Debug, Survey)]
enum Language {
    #[ask("Rust 🦀")]
    Rust,
    #[ask("Python 🐍")]
    Python,
    #[ask("JavaScript")]
    JavaScript,
    #[ask("TypeScript")]
    TypeScript,
    #[ask("Go")]
    Go,
    #[ask("Java ☕")]
    Java,
    #[ask("C++")]
    Cpp,
}

/// Development tools.
#[derive(Debug, Survey)]
enum Tool {
    #[ask("Git")]
    Git,
    #[ask("Docker 🐳")]
    Docker,
    #[ask("Kubernetes ☸️")]
    Kubernetes,
    #[ask("VS Code")]
    VSCode,
    #[ask("Vim/Neovim")]
    Vim,
}

/// Areas of interest.
#[derive(Debug, Survey)]
enum Interest {
    #[ask("Web Development")]
    WebDev,
    #[ask("Backend Systems")]
    Backend,
    #[ask("Mobile Development")]
    Mobile,
    #[ask("Game Development 🎮")]
    GameDev,
    #[ask("Machine Learning 🤖")]
    MachineLearning,
    #[ask("DevOps & Infrastructure")]
    DevOps,
}

/// Developer profile survey with multi-select fields.
#[derive(Debug, Survey)]
#[prelude(
    "Welcome! Tell us about your developer profile.\nThis helps us understand our community better."
)]
#[epilogue("Thank you for sharing your profile!")]
struct DeveloperProfile {
    #[ask("What's your name?")]
    name: String,

    #[ask("Years of programming experience:")]
    #[min(0)]
    #[max(50)]
    years_experience: i64,

    #[ask("Which programming languages do you use regularly?")]
    #[multiselect]
    #[validate(at_least_one_language)]
    languages: Vec<Language>,

    #[ask("Which tools are part of your daily workflow?")]
    #[multiselect]
    tools: Vec<Tool>,

    #[ask("What areas of development interest you most?")]
    #[multiselect]
    interests: Vec<Interest>,

    #[ask("Do you contribute to open source projects?")]
    open_source: bool,
}

fn at_least_one_language(value: &ResponseValue, _responses: &Responses) -> Result<(), String> {
    let ResponseValue::ChosenVariants(selections) = value else {
        return Ok(());
    };

    if selections.is_empty() {
        return Err("Please select at least one programming language".to_string());
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    println!("=== Developer Profile Survey - egui Multi-Select Demo ===\n");

    let backend = EguiBackend::new()
        .with_title("Developer Profile Survey")
        .with_window_size([550.0, 650.0]);

    let profile: DeveloperProfile = DeveloperProfile::builder().run(backend)?;

    println!("\n=== Profile Created ===");
    println!("Name: {}", profile.name);
    println!("Experience: {} years", profile.years_experience);
    println!("Languages: {:?}", profile.languages);
    println!("Tools: {:?}", profile.tools);
    println!("Interests: {:?}", profile.interests);
    println!(
        "Open Source Contributor: {}",
        if profile.open_source { "Yes" } else { "No" }
    );

    Ok(())
}
