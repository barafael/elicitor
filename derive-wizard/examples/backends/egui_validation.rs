//! Example demonstrating real-time validation in the egui backend.
//!
//! This example shows how validation errors appear as you type,
//! providing immediate feedback to the user.
//!
//! Run with: cargo run --example egui_validation --features egui-backend

use derive_wizard::Wizard;

#[derive(Debug, Wizard)]
#[prelude("Please fill in your server configuration.\nValidation happens as you type!")]
#[allow(dead_code)]
struct ServerConfig {
    #[prompt("Server address (host:port):")]
    #[validate("validate_address")]
    address: String,

    #[prompt("Admin username:")]
    #[validate("validate_username")]
    username: String,

    #[prompt("Admin email:")]
    #[validate("validate_email")]
    email: String,

    #[prompt("Server port:")]
    #[validate("validate_port")]
    #[min(1)]
    #[max(65535)]
    port: i32,
}

/// Validates that the address is in host:port format
pub fn validate_address(input: &str, _answers: &derive_wizard::Answers) -> Result<(), String> {
    if input.is_empty() {
        return Err("Address cannot be empty".to_string());
    }

    if !input.contains(':') {
        return Err("Missing port - use format 'host:port'".to_string());
    }

    let parts: Vec<&str> = input.split(':').collect();
    if parts.len() != 2 {
        return Err("Invalid format - use 'host:port'".to_string());
    }

    if parts[0].is_empty() {
        return Err("Host cannot be empty".to_string());
    }

    match parts[1].parse::<u16>() {
        Ok(port) if port > 0 => Ok(()),
        Ok(_) => Err("Port must be greater than 0".to_string()),
        Err(_) => Err(format!("'{}' is not a valid port number", parts[1])),
    }
}

/// Validates username: 3-20 chars, alphanumeric and underscores only
pub fn validate_username(input: &str, _answers: &derive_wizard::Answers) -> Result<(), String> {
    if input.is_empty() {
        return Err("Username cannot be empty".to_string());
    }

    if input.len() < 3 {
        return Err(format!("Too short ({}/3 characters minimum)", input.len()));
    }

    if input.len() > 20 {
        return Err(format!("Too long ({}/20 characters maximum)", input.len()));
    }

    if let Some(invalid_char) = input.chars().find(|c| !c.is_alphanumeric() && *c != '_') {
        return Err(format!("Invalid character: '{}'", invalid_char));
    }

    Ok(())
}

/// Validates email address format
pub fn validate_email(input: &str, _answers: &derive_wizard::Answers) -> Result<(), String> {
    if input.is_empty() {
        return Err("Email cannot be empty".to_string());
    }

    if !input.contains('@') {
        return Err("Missing @ symbol".to_string());
    }

    let parts: Vec<&str> = input.split('@').collect();
    if parts.len() != 2 {
        return Err("Multiple @ symbols not allowed".to_string());
    }

    if parts[0].is_empty() {
        return Err("Missing username before @".to_string());
    }

    if parts[1].is_empty() {
        return Err("Missing domain after @".to_string());
    }

    if !parts[1].contains('.') {
        return Err("Domain must contain a dot (e.g., example.com)".to_string());
    }

    Ok(())
}

/// Validates port number is in a reasonable range
pub fn validate_port(input: &str, _answers: &derive_wizard::Answers) -> Result<(), String> {
    match input.parse::<i32>() {
        Ok(port) if port < 1024 => {
            Err("Ports below 1024 are reserved (use 1024-65535)".to_string())
        }
        Ok(_) => Ok(()),
        Err(_) => Err("Invalid port number".to_string()),
    }
}

fn main() {
    println!("=== Server Configuration Wizard ===");
    println!("This example demonstrates real-time validation in the egui backend.");
    println!();

    let backend = derive_wizard::EguiBackend::new()
        .with_title("Server Configuration")
        .with_window_size([450.0, 350.0]);

    let config = ServerConfig::wizard_builder().with_backend(backend).build().unwrap();

    println!();
    println!("=== Configuration Created ===");
    println!("{:#?}", config);
}
