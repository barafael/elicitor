//! Validation example demonstrating real-time validation in the egui backend.
//!
//! This example shows how validation errors appear as you type,
//! providing immediate feedback to the user.
//!
//! Run with: cargo run -p derive-egui-form --example validation

use derive_survey::{ResponseValue, Responses, Survey};
use derive_egui_form::EguiBackend;

/// Server configuration with validation.
#[derive(Debug, Survey)]
#[prelude("Please fill in your server configuration.\nValidation happens as you type!")]
struct ServerConfig {
    #[ask("Server hostname:")]
    #[validate(validate_hostname)]
    hostname: String,

    #[ask("Admin username:")]
    #[validate(validate_username)]
    username: String,

    #[ask("Admin email:")]
    #[validate(validate_email)]
    email: String,

    #[ask("Server port:")]
    #[min(1)]
    #[max(65535)]
    port: i64,

    #[ask("Max connections:")]
    #[min(1)]
    #[max(10000)]
    max_connections: i64,

    #[ask("Enable TLS?")]
    enable_tls: bool,
}

/// Validates that the hostname is not empty and contains only valid characters.
fn validate_hostname(value: &ResponseValue, _responses: &Responses) -> Result<(), String> {
    let ResponseValue::String(hostname) = value else {
        return Ok(());
    };

    if hostname.is_empty() {
        return Err("Hostname cannot be empty".to_string());
    }

    if hostname.len() < 3 {
        return Err(format!(
            "Too short ({}/3 characters minimum)",
            hostname.len()
        ));
    }

    // Check for valid hostname characters
    if let Some(invalid_char) = hostname
        .chars()
        .find(|c| !c.is_alphanumeric() && *c != '-' && *c != '.')
    {
        return Err(format!("Invalid character: '{invalid_char}'"));
    }

    Ok(())
}

/// Validates username: 3-20 chars, alphanumeric and underscores only.
fn validate_username(value: &ResponseValue, _responses: &Responses) -> Result<(), String> {
    let ResponseValue::String(username) = value else {
        return Ok(());
    };

    if username.is_empty() {
        return Err("Username cannot be empty".to_string());
    }

    if username.len() < 3 {
        return Err(format!(
            "Too short ({}/3 characters minimum)",
            username.len()
        ));
    }

    if username.len() > 20 {
        return Err(format!(
            "Too long ({}/20 characters maximum)",
            username.len()
        ));
    }

    if let Some(invalid_char) = username.chars().find(|c| !c.is_alphanumeric() && *c != '_') {
        return Err(format!("Invalid character: '{invalid_char}'"));
    }

    Ok(())
}

/// Validates email address format.
fn validate_email(value: &ResponseValue, _responses: &Responses) -> Result<(), String> {
    let ResponseValue::String(email) = value else {
        return Ok(());
    };

    if email.is_empty() {
        return Err("Email cannot be empty".to_string());
    }

    if !email.contains('@') {
        return Err("Missing @ symbol".to_string());
    }

    let parts: Vec<&str> = email.split('@').collect();
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

fn main() -> anyhow::Result<()> {
    println!("=== Server Configuration - egui Validation Demo ===");
    println!("This example demonstrates real-time validation in the egui backend.\n");

    let backend = EguiBackend::new()
        .with_title("Server Configuration")
        .with_window_size([500.0, 450.0]);

    let config: ServerConfig = ServerConfig::builder().run(backend)?;

    println!("\n=== Configuration Created ===");
    println!("Hostname: {}", config.hostname);
    println!("Username: {}", config.username);
    println!("Email: {}", config.email);
    println!("Port: {}", config.port);
    println!("Max Connections: {}", config.max_connections);
    println!("TLS Enabled: {}", config.enable_tls);

    Ok(())
}
