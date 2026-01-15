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
//!
//! Run with: cargo run -p derive-dialoguer-wizard --example job_application

use derive_dialoguer_wizard::DialoguerBackend;
use example_surveys::JobApplication;

fn main() -> anyhow::Result<()> {
    let backend = DialoguerBackend::new();

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
        .run(backend)?;

    println!("{result:#?}");
    Ok(())
}
