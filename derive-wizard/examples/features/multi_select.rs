//! Example demonstrating multi-select functionality
//!
//! Run with: cargo run --example multi_select --features dialoguer-backend

use derive_wizard::Wizard;

/// Programming languages the user is familiar with
#[derive(Debug, Clone, Copy, PartialEq, Eq, Wizard)]
pub enum Language {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Go,
    Java,
    CSharp,
    Cpp,
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Language::Rust => write!(f, "Rust"),
            Language::Python => write!(f, "Python"),
            Language::JavaScript => write!(f, "JavaScript"),
            Language::TypeScript => write!(f, "TypeScript"),
            Language::Go => write!(f, "Go"),
            Language::Java => write!(f, "Java"),
            Language::CSharp => write!(f, "C#"),
            Language::Cpp => write!(f, "C++"),
        }
    }
}

/// Interests for the survey
#[derive(Debug, Clone, Copy, PartialEq, Eq, Wizard)]
pub enum Interest {
    WebDevelopment,
    MobileDevelopment,
    GameDevelopment,
    MachineLearning,
    SystemsProgramming,
    DevOps,
    DataEngineering,
    Security,
}

impl std::fmt::Display for Interest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Interest::WebDevelopment => write!(f, "Web Development"),
            Interest::MobileDevelopment => write!(f, "Mobile Development"),
            Interest::GameDevelopment => write!(f, "Game Development"),
            Interest::MachineLearning => write!(f, "Machine Learning"),
            Interest::SystemsProgramming => write!(f, "Systems Programming"),
            Interest::DevOps => write!(f, "DevOps & Infrastructure"),
            Interest::DataEngineering => write!(f, "Data Engineering"),
            Interest::Security => write!(f, "Security"),
        }
    }
}

/// Developer survey with multi-select fields
#[derive(Debug, Wizard)]
pub struct DeveloperSurvey {
    /// Your name
    #[prompt("What is your name?")]
    name: String,

    /// Years of experience
    #[prompt("How many years of programming experience do you have?")]
    years_experience: u32,

    /// Programming languages (select multiple)
    #[prompt("Which programming languages are you proficient in?")]
    languages: Vec<Language>,

    /// Areas of interest (select multiple)
    #[prompt("Which areas of software development interest you?")]
    interests: Vec<Interest>,

    /// Preferred work style
    #[prompt("Do you prefer remote work?")]
    remote_preference: bool,
}

fn main() {
    let backend = derive_wizard::DialoguerBackend::new();
    let survey = DeveloperSurvey::wizard_builder()
        .with_backend(backend)
        .build()
        .unwrap();

    println!("\n=== Survey Results ===\n");
    println!("Name: {}", survey.name);
    println!("Years of Experience: {}", survey.years_experience);

    println!("\nLanguages ({} selected):", survey.languages.len());
    for lang in &survey.languages {
        println!("  • {}", lang);
    }

    println!("\nInterests ({} selected):", survey.interests.len());
    for interest in &survey.interests {
        println!("  • {}", interest);
    }

    println!(
        "\nRemote Preference: {}",
        if survey.remote_preference {
            "Yes"
        } else {
            "No"
        }
    );
}
