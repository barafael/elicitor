//! To validate a value during input, either
//! * specify a 'validate' attribute (like in the 'address' member), or
//! * specify one of or both of:
//!   - 'validate_on_key' (validate on each input for user feedback), and/or
//!   - 'validate_on_submit' (validate on the value submission, possibly displaying an error message)

use derive_wizard::Wizard;

#[derive(Debug, Wizard)]
#[allow(dead_code)]
struct ServerConfig {
    #[prompt("Enter server address (host:port):")]
    #[validate("validate_address")]
    address: String,

    #[prompt("Enter admin username:")]
    #[validate_on_key("validate_username")]
    username: String,

    #[prompt("Enter admin email:")]
    #[validate_on_submit("validate_email")]
    email: String,
}

/// Validates that the address is in host:port format
pub fn validate_address(input: &str, _answers: &derive_wizard::Answers) -> Result<(), String> {
    if input.contains(':') && input.len() >= 3 && !input.starts_with(':') && !input.ends_with(':') {
        let parts: Vec<&str> = input.split(':').collect();
        if parts.len() == 2 && !parts[0].is_empty() && parts[1].parse::<u16>().is_ok() {
            return Ok(());
        }
    }
    Err("Address must be in format 'host:port' (e.g., 'localhost:8080')".to_string())
}

/// Validates username: 3-20 chars, alphanumeric and underscores only
pub fn validate_username(input: &str, _answers: &derive_wizard::Answers) -> Result<(), String> {
    if input.len() < 3 {
        return Err("Username must be at least 3 characters long".to_string());
    }
    if input.len() > 20 {
        return Err("Username must be at most 20 characters long".to_string());
    }
    if !input.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err("Username can only contain letters, numbers, and underscores".to_string());
    }
    Ok(())
}

/// Validates email address format
pub fn validate_email(input: &str, _answers: &derive_wizard::Answers) -> Result<(), String> {
    if !input.contains('@') {
        return Err("Email must contain an @ symbol".to_string());
    }
    let parts: Vec<&str> = input.split('@').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        return Err("Email must be in format 'user@domain'".to_string());
    }
    if !parts[1].contains('.') {
        return Err("Email domain must contain a dot (e.g., example.com)".to_string());
    }
    Ok(())
}

fn main() {
    let config = ServerConfig::wizard_builder().build();
    println!("{config:#?}");
}
