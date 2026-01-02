use derive_wizard::Wizard;

#[derive(Debug, Wizard)]
#[prelude(
    "Welcome to the Server Configuration Wizard!\nThis wizard will help you configure your server settings."
)]
#[epilogue("Configuration complete! Your server is ready to start.")]
struct ServerConfig {
    #[prompt("Server name:")]
    name: String,

    #[prompt("Port number:")]
    #[min(1024)]
    #[max(65535)]
    port: i32,

    #[prompt("Enable SSL:")]
    ssl: bool,
}

fn main() {
    let config = ServerConfig::wizard_builder().build();
    println!("\n=== Server Configuration ===");
    println!("{:#?}", config);
}
