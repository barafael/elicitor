//! Job Application - A compact example showcasing ALL derive-survey features
//!
//! Features demonstrated:
//! - Prelude/epilogue messages
//! - All primitives: String, bool, integers (i32, u8, u32)
//! - Text input with validation (#[validate])
//! - Password/masked input (#[mask])
//! - Multiline text input (#[multiline])
//! - Numeric bounds (#[min], #[max])
//! - Enum selection with unit, newtype, tuple, and struct variants
//! - Multi-select with validation (#[multiselect])
//! - List input for Vec<String> (comma-separated)
//! - Nested structs with propagated field validation (#[validate_fields])
//! - PathBuf support
//! - Builder API: suggestions, closures for nested types

use derive_ratatui_wizard::{RatatuiBackend, Theme};
use example_surveys::JobApplication;
use ratatui::style::Color;

fn main() {
    let theme = Theme {
        primary: Color::Blue,
        secondary: Color::LightBlue,
        background: Color::Reset,
        text: Color::White,
        highlight: Color::Cyan,
        error: Color::Red,
        success: Color::Green,
        border: Color::DarkGray,
    };

    let backend = RatatuiBackend::new()
        .with_title("Acme Corp - Job Application")
        .with_theme(theme);

    let result = JobApplication::builder()
        // Simple suggestions
        .suggest_name("Jane Doe".to_string())
        .suggest_email("jane@example.com".to_string())
        .suggest_timezone(-5) // EST
        .suggest_relocate(false)
        // Nested struct suggestions via closure
        .suggest_experience(|exp| exp.company("Previous Corp").months(30).remote(true))
        .suggest_salary(|sal| sal.base(120).bonus(20))
        // Enum variant selection
        .suggest_position(|pos| pos.suggest_senior())
        .suggest_work_style(|ws| ws.suggest_remote())
        // Enum with nested fields
        .suggest_referral(|r| {
            r.suggest_linked_in()
                // Also pre-fill Conference in case they switch
                .conference(|c| c.name("RustConf").year(2025))
        })
        .run(backend);

    match result {
        Ok(app) => {
            println!("\n=== Application Received ===\n");
            println!("Name: {}", app.name);
            println!("Email: {}", app.email);
            println!("Position: {:?}", app.position);
            println!("Work style: {:?}", app.work_style);
            println!("Referral: {:?}", app.referral);
            println!(
                "Experience: {} months at {}",
                app.experience.months, app.experience.company
            );
            println!(
                "Salary: ${}k base + ${}k bonus",
                app.salary.base, app.salary.bonus
            );
            println!("Skills: {:?}", app.skills);
            println!(
                "Schools attended: {}",
                if app.schools_attended.is_empty() {
                    "None".to_string()
                } else {
                    app.schools_attended.join(", ")
                }
            );
            println!("Resume: {:?}", app.resume);
            println!("Relocate: {}", if app.relocate { "Yes" } else { "No" });
            println!("Timezone: UTC{:+}", app.timezone);
            println!("\n{:#?}", app);
        }
        Err(e) => {
            eprintln!("Application cancelled: {e}");
            std::process::exit(1);
        }
    }
}
