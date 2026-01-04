//! Server Configuration Wizard 🖥️
//!
//! A technical example for configuring a server deployment.
//!
//! Run with: cargo run --example ratatui_server_config --features ratatui-backend

use derive_wizard::Wizard;

#[derive(Debug, Wizard)]
#[allow(dead_code)]
enum Environment {
    Development,
    Staging,
    Production,
}

#[derive(Debug, Wizard)]
#[allow(dead_code)]
enum DatabaseType {
    PostgreSQL,
    MySQL,
    SQLite,
    MongoDB,
}

#[derive(Debug, Wizard)]
#[allow(dead_code)]
enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Wizard)]
#[allow(dead_code)]
struct DatabaseConfig {
    #[prompt("Database type:")]
    db_type: DatabaseType,

    #[prompt("Database host (e.g., localhost):")]
    host: String,

    #[prompt("Database port:")]
    #[min(1)]
    #[max(65535)]
    port: i64,

    #[prompt("Database name:")]
    name: String,

    #[prompt("Database username:")]
    username: String,

    #[prompt("Database password:")]
    #[mask]
    password: String,

    #[prompt("Connection pool size:")]
    #[min(1)]
    #[max(100)]
    pool_size: i64,
}

#[derive(Debug, Wizard)]
#[allow(dead_code)]
struct ServerSettings {
    #[prompt("Server bind address (e.g., 0.0.0.0):")]
    bind_address: String,

    #[prompt("HTTP port:")]
    #[min(1)]
    #[max(65535)]
    http_port: i64,

    #[prompt("Enable HTTPS?")]
    enable_https: bool,

    #[prompt("Max concurrent connections:")]
    #[min(10)]
    #[max(10000)]
    max_connections: i64,

    #[prompt("Request timeout (seconds):")]
    #[min(1)]
    #[max(300)]
    timeout_seconds: i64,
}

#[derive(Debug, Wizard)]
#[allow(dead_code)]
#[prelude(
    "🖥️  Server Deployment Configuration\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\nThis wizard will help you configure your server deployment.\nPress Esc at any time to cancel."
)]
#[epilogue(
    "✅ Configuration complete!\n\nYour settings have been saved.\nRun 'deploy.sh' to apply this configuration."
)]
struct DeploymentConfig {
    #[prompt("Application name:")]
    app_name: String,

    #[prompt("Deployment environment:")]
    environment: Environment,

    #[prompt("Server settings")]
    server: ServerSettings,

    #[prompt("Database configuration")]
    database: DatabaseConfig,

    #[prompt("Logging level:")]
    log_level: LogLevel,

    #[prompt("Enable metrics collection?")]
    enable_metrics: bool,

    #[prompt("Enable health check endpoint?")]
    enable_health_check: bool,

    #[prompt("Number of worker threads:")]
    #[min(1)]
    #[max(64)]
    worker_threads: i64,

    #[prompt("Memory limit (MB):")]
    #[min(128)]
    #[max(65536)]
    memory_limit_mb: i64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use derive_wizard::{InterviewBackend, RatatuiBackend, RatatuiColor, RatatuiTheme};

    // Cyberpunk-inspired theme
    let theme = RatatuiTheme {
        primary: RatatuiColor::Rgb(0, 255, 136),    // Neon green
        secondary: RatatuiColor::Rgb(138, 43, 226), // Purple
        highlight: RatatuiColor::Rgb(0, 255, 255),  // Cyan
        success: RatatuiColor::Rgb(0, 255, 136),    // Neon green
        error: RatatuiColor::Rgb(255, 0, 128),      // Hot pink
        text: RatatuiColor::Rgb(200, 200, 200),     // Light gray
        background: RatatuiColor::Reset,
        border: RatatuiColor::Rgb(100, 100, 100),
    };

    let interview = DeploymentConfig::interview();
    let backend = RatatuiBackend::new()
        .with_title("⚙️  Deployment Configuration Wizard")
        .with_theme(theme);

    let answers = backend.execute(&interview)?;
    let config = DeploymentConfig::from_answers(&answers)?;

    println!("\n🔧 Generated Configuration:");
    println!("════════════════════════════════════════════════════════");
    println!();
    println!("[application]");
    println!("name = \"{}\"", config.app_name);
    println!("environment = \"{:?}\"", config.environment);
    println!("workers = {}", config.worker_threads);
    println!("memory_limit = \"{}MB\"", config.memory_limit_mb);
    println!();
    println!("[server]");
    println!("bind = \"{}\"", config.server.bind_address);
    println!("port = {}", config.server.http_port);
    println!("https = {}", config.server.enable_https);
    println!("max_connections = {}", config.server.max_connections);
    println!("timeout = {}", config.server.timeout_seconds);
    println!();
    println!("[database]");
    println!("type = \"{:?}\"", config.database.db_type);
    println!("host = \"{}\"", config.database.host);
    println!("port = {}", config.database.port);
    println!("name = \"{}\"", config.database.name);
    println!("pool_size = {}", config.database.pool_size);
    println!();
    println!("[logging]");
    println!("level = \"{:?}\"", config.log_level);
    println!();
    println!("[monitoring]");
    println!("metrics = {}", config.enable_metrics);
    println!("health_check = {}", config.enable_health_check);
    println!("════════════════════════════════════════════════════════");

    Ok(())
}
