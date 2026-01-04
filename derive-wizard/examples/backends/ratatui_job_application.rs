//! Job Application Wizard 💼
//!
//! A professional form demonstrating various input types and validation.
//!
//! Run with: cargo run --example ratatui_job_application --features ratatui-backend

use derive_wizard::Wizard;

#[derive(Debug, Wizard)]
#[allow(dead_code)]
enum EmploymentType {
    FullTime,
    PartTime,
    Contract,
    Internship,
}

#[derive(Debug, Wizard)]
#[allow(dead_code)]
enum ExperienceLevel {
    Entry,
    Junior,
    Mid,
    Senior,
    Lead,
    Principal,
}

#[derive(Debug, Wizard)]
#[allow(dead_code)]
enum Department {
    Engineering,
    Design,
    Marketing,
    Sales,
    HumanResources,
    Finance,
    Operations,
}

#[derive(Debug, Wizard)]
#[allow(dead_code)]
#[prelude(
    "Welcome to TechCorp Careers!\n\nPlease fill out this application form.\nAll information will be kept confidential."
)]
#[epilogue(
    "Thank you for applying!\n\nWe will review your application and contact you within 5 business days."
)]
struct JobApplication {
    // Personal Information
    #[prompt("Full legal name:")]
    full_name: String,

    #[prompt("Email address:")]
    email: String,

    #[prompt("Phone number:")]
    phone: String,

    #[prompt("City of residence:")]
    city: String,

    // Position Details
    #[prompt("Which department are you applying for?")]
    department: Department,

    #[prompt("Desired employment type:")]
    employment_type: EmploymentType,

    #[prompt("Your experience level:")]
    experience_level: ExperienceLevel,

    #[prompt("Years of professional experience:")]
    #[min(0)]
    #[max(50)]
    years_experience: i64,

    #[prompt("Expected annual salary (USD):")]
    #[min(0.0)]
    expected_salary: f64,

    // Additional Information
    #[prompt("Are you authorized to work in this country?")]
    work_authorized: bool,

    #[prompt("Can you start within 2 weeks if selected?")]
    available_soon: bool,

    #[prompt("How did you hear about us?")]
    referral_source: String,

    #[prompt("Any additional comments for the hiring team?")]
    comments: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use derive_wizard::{InterviewBackend, RatatuiBackend, RatatuiColor, RatatuiTheme};

    let theme = RatatuiTheme {
        primary: RatatuiColor::Rgb(0, 122, 204), // Professional blue
        secondary: RatatuiColor::Rgb(100, 100, 100), // Gray
        highlight: RatatuiColor::Rgb(0, 200, 150), // Teal accent
        success: RatatuiColor::Rgb(40, 167, 69), // Green
        error: RatatuiColor::Rgb(220, 53, 69),   // Red
        text: RatatuiColor::White,
        background: RatatuiColor::Reset,
        border: RatatuiColor::Rgb(80, 80, 80),
    };

    let interview = JobApplication::interview();
    let backend = RatatuiBackend::new()
        .with_title("💼 TechCorp Job Application Portal")
        .with_theme(theme);

    let answers = backend.execute(&interview)?;
    let application = JobApplication::from_answers(&answers)?;

    println!("\n📄 Application Submitted:");
    println!("═══════════════════════════════════════════════════");
    println!("Applicant: {}", application.full_name);
    println!("Department: {:?}", application.department);
    println!("Position Type: {:?}", application.employment_type);
    println!(
        "Experience: {:?} ({} years)",
        application.experience_level, application.years_experience
    );
    println!("Expected Salary: ${:.2}", application.expected_salary);
    println!("═══════════════════════════════════════════════════");

    Ok(())
}
