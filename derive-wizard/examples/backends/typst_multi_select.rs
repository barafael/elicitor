//! Typst Multi-Select Example
//!
//! Generates a Typst form with multi-select checkbox groups.
//!
//! Run with: cargo run --example typst_multi_select --features typst-form

use derive_wizard::Wizard;

#[derive(Debug, Clone, Copy, Wizard)]
pub enum Skill {
    Rust,
    Python,
    JavaScript,
    Go,
    SQL,
}

#[derive(Debug, Clone, Copy, Wizard)]
pub enum Availability {
    Morning,
    Afternoon,
    Evening,
    Weekends,
}

#[derive(Debug, Clone, Default, Wizard)]
pub enum Role {
    #[default]
    Developer,
    Designer,
    Manager,
    Other(#[prompt("Description:")] String),
}

#[allow(dead_code)] // Fields are used for typst form generation, not runtime
#[derive(Debug, Wizard)]
#[prelude("Please complete this job application form.")]
struct JobApplication {
    #[prompt("Full name:")]
    name: String,

    #[prompt("Email address:")]
    email: String,

    #[prompt("Preferred role:")]
    role: Role,

    #[prompt("Technical skills (select all that apply):")]
    skills: Vec<Skill>,

    #[prompt("Available times:")]
    availability: Vec<Availability>,

    #[prompt("Open to remote work?")]
    remote: bool,
}

fn main() {
    let typst = JobApplication::to_typst_form(Some("Job Application"));
    println!("{typst}");
}
