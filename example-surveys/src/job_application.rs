use derive_survey::{ResponseValue, Responses, Survey};
use std::path::PathBuf;

pub fn validate_email(value: &ResponseValue, _: &Responses) -> Result<(), String> {
    let ResponseValue::String(email) = value else {
        return Ok(());
    };
    if !email.contains('@') || !email.split('@').last().is_some_and(|d| d.contains('.')) {
        return Err("Enter a valid email (e.g., you@example.com)".into());
    }
    Ok(())
}

pub fn validate_password(value: &ResponseValue, _: &Responses) -> Result<(), String> {
    let ResponseValue::String(pw) = value else {
        return Ok(());
    };
    if pw.len() < 6 {
        return Err("Password must be at least 6 characters".into());
    }
    Ok(())
}

pub fn validate_cover_letter(value: &ResponseValue, _: &Responses) -> Result<(), String> {
    let ResponseValue::String(text) = value else {
        return Ok(());
    };
    let words: Vec<_> = text.split_whitespace().collect();
    if words.len() < 10 {
        return Err(format!("Write at least 10 words ({} so far)", words.len()));
    }
    Ok(())
}

/// Skills: must pick 1-5
pub fn validate_skills(value: &ResponseValue, _: &Responses) -> Result<(), String> {
    let ResponseValue::ChosenVariants(picks) = value else {
        return Ok(());
    };
    if picks.is_empty() {
        return Err("Select at least one skill".into());
    }
    if picks.len() > 5 {
        return Err("Select at most 5 skills".into());
    }
    Ok(())
}

/// Salary expectations must total <= $250k (base + bonus)
pub const MAX_TOTAL_COMP: i64 = 250_000;

pub fn validate_salary(value: &ResponseValue, responses: &Responses) -> Result<(), String> {
    let ResponseValue::Int(current) = value else {
        return Ok(());
    };
    let base = Salary::get_base(responses).unwrap_or(0) as i64;
    let bonus = Salary::get_bonus(responses).unwrap_or(0) as i64;
    let total = base + bonus + current;

    if total > MAX_TOTAL_COMP {
        return Err(format!(
            "Total comp ${total}k exceeds ${MAX_TOTAL_COMP}k limit"
        ));
    }
    Ok(())
}

/// Salary expectations with cross-field validation
#[derive(Survey, Debug)]
#[validate_fields(validate_salary)]
pub struct Salary {
    #[ask("Base salary ($k/year):")]
    #[min(30)]
    #[max(200)]
    pub base: u32,

    #[ask("Expected bonus ($k/year):")]
    #[min(0)]
    #[max(100)]
    pub bonus: u32,
}

/// Work experience entry
#[derive(Survey, Debug)]
pub struct Experience {
    #[ask("Company name:")]
    pub company: String,

    #[ask("Months at company:")]
    #[min(1)]
    #[max(600)]
    pub months: u32,

    #[ask("Was this a remote position?")]
    pub remote: bool,
}

/// Position applying for - demonstrates unit, newtype, tuple, and struct variants
#[derive(Survey, Debug)]
pub enum Position {
    /// Junior developer
    Junior,
    /// Senior developer
    Senior,
    /// Tech lead with team size
    TechLead(#[ask("Team size you'd manage:")] u8),
    /// Staff engineer
    Staff {
        #[ask("Primary focus area:")]
        focus: FocusArea,
        #[ask("Years of staff+ experience:")]
        #[min(0)]
        #[max(30)]
        years_at_level: u32,
    },
    /// Other role with custom title
    Other(#[ask("Role title:")] String, #[ask("Level (1-10):")] u8),
}

/// Engineering focus area
#[derive(Survey, Debug)]
pub enum FocusArea {
    Backend,
    Frontend,
    Fullstack,
    Infrastructure,
    Security,
    Data,
}

/// Work preference
#[derive(Survey, Debug)]
pub enum WorkStyle {
    Remote,
    Hybrid,
    OnSite,
}

/// Available skills for multi-select
#[derive(Survey, Debug)]
pub enum JobSkill {
    Rust,
    Python,
    TypeScript,
    Go,
    SQL,
    Docker,
    Kubernetes,
    AWS,
    Leadership,
    Communication,
}

/// How did you hear about us?
#[derive(Survey, Debug)]
pub enum Referral {
    LinkedIn,
    JobBoard,
    Referral(#[ask("Who referred you?")] String),
    Conference {
        #[ask("Conference name:")]
        name: String,
        #[ask("Year:")]
        #[min(2020)]
        #[max(2030)]
        year: u32,
    },
    Other(String),
}

/// Main job application survey
#[derive(Survey, Debug)]
#[prelude("Welcome to Acme Corp!\nLet's get your application started.\n")]
#[epilogue("Application submitted! We'll be in touch within 5 business days.")]
pub struct JobApplication {
    // Basic info with validation
    #[ask("Full name:")]
    pub name: String,

    #[ask("Email address:")]
    #[validate(validate_email)]
    pub email: String,

    #[ask("Create a portal password:")]
    #[mask]
    #[validate(validate_password)]
    pub password: String,

    // Enum selections
    #[ask("Position applying for:")]
    pub position: Position,

    #[ask("Preferred work style:")]
    pub work_style: WorkStyle,

    #[ask("How did you hear about us?")]
    pub referral: Referral,

    // Nested struct
    #[ask("Most recent experience:")]
    pub experience: Experience,

    // Nested struct with propagated validation
    #[ask("Salary expectations:")]
    pub salary: Salary,

    // Multi-select with validation
    #[ask("Your top skills (1-5):")]
    #[multiselect]
    #[validate(validate_skills)]
    pub skills: Vec<JobSkill>,

    // List of strings - schools attended
    #[ask("Schools attended (comma-separated):")]
    pub schools_attended: Vec<String>,

    // Multiline with validation
    #[ask("Cover letter:")]
    #[multiline]
    #[validate(validate_cover_letter)]
    pub cover_letter: String,

    // PathBuf
    #[ask("Resume file path:")]
    pub resume: PathBuf,

    // Simple bool
    #[ask("Willing to relocate?")]
    pub relocate: bool,

    // Signed integer with bounds
    #[ask("Timezone offset from UTC (-12 to +14):")]
    #[min(-12)]
    #[max(14)]
    pub timezone: i32,
}
