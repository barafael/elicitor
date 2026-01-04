//! Validation example using the dialoguer backend.
//! Dialoguer validates input on submission.

use derive_wizard::Wizard;

#[derive(Debug, Wizard)]
#[allow(dead_code)]
struct ServerConfig {
    #[prompt("Enter server address (host:port):")]
    #[validate("validate_address")]
    address: String,

    #[prompt("Enter admin username:")]
    #[validate("validate_username")]
    username: String,

    #[prompt("Enter admin email:")]
    #[validate("validate_email")]
    email: String,

    #[prompt("Enter port number:")]
    #[validate("validate_port")]
    #[min(1)]
    #[max(65535)]
    port: i32,
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

/// Validates port is in commonly used range
pub fn validate_port(input: &str, _answers: &derive_wizard::Answers) -> Result<(), String> {
    let port: i32 = input
        .parse()
        .map_err(|_| "Invalid port number".to_string())?;

    if port < 1024 {
        return Err("Port should be >= 1024 (ports below 1024 are privileged)".to_string());
    }

    Ok(())
}

fn main() {
    println!("=== Server Configuration - dialoguer with Validation ===\n");
    println!("The dialoguer backend validates on submission.");
    println!("Try entering invalid values to see validation in action!\n");

    // Use the dialoguer backend
    let backend = derive_wizard::DialoguerBackend::new();
    let config = ServerConfig::wizard_builder().with_backend(backend).build().unwrap();

    println!("\n=== Configuration Complete ===");
    println!("{config:#?}");
}
