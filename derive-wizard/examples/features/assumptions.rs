use derive_wizard::Wizard;

/// This example demonstrates the assumptions feature.
/// Assumptions are values that are pre-filled and skip the questions entirely,
/// unlike suggestions which show as defaults but still ask the user.
#[derive(Debug, Clone, Wizard)]
struct DeploymentConfig {
    #[prompt("Application name:")]
    app_name: String,

    #[prompt("Environment (dev/staging/prod):")]
    environment: String,

    #[prompt("Port number:")]
    #[min(1024)]
    #[max(65535)]
    port: i32,

    #[prompt("Enable debug logging:")]
    debug: bool,

    #[prompt("Database URL:")]
    database_url: String,
}

fn main() {
    println!("=== Deployment Configuration Tool ===");

    // Scenario 1: Partial assumptions - the most common use case
    println!("--- Scenario 1: Partial Assumptions (Recommended) ---");
    println!("We'll assume some security-critical values but ask about others.");

    let config = DeploymentConfig::wizard_builder()
        .assume_field("environment", "production".to_string()) // Fixed: production
        .assume_field("debug", false) // Fixed: no debug in prod
        .assume_field("port", 443) // Fixed: HTTPS port
        .build(); // Will only ask about 'app_name' and 'database_url'

    println!("=== Configuration (with partial assumptions) ===");
    println!("{:#?}", config);
    println!("Notice: Only asked about app_name and database_url!");
    println!("The fields 'environment', 'debug', and 'port' were assumed.");

    // Scenario 2: Full assumptions - for batch processing
    println!("--- Scenario 2: Full Assumptions (for automation) ---");
    println!("Using a complete template - no questions will be asked.");

    let batch_config = DeploymentConfig::wizard_builder()
        .assume_field("app_name", "batch-processor".to_string())
        .assume_field("environment", "production".to_string())
        .assume_field("port", 8080)
        .assume_field("debug", false)
        .assume_field(
            "database_url",
            "postgresql://prod-db:5432/batch".to_string(),
        )
        .build();

    println!("=== Batch Configuration (all assumed, no questions) ===");
    println!("{:#?}", batch_config);

    println!("--- Summary ---");
    println!("Partial assumptions: Fix some fields, ask about others");
    println!("Full assumptions: Fix all fields, no user interaction");
    println!("Suggestions: Pre-fill values, but still ask all questions");
}
