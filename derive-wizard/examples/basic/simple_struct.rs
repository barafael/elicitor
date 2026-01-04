use derive_wizard::Wizard;

#[derive(Debug, Wizard)]
struct ServerConfig {
    #[prompt("Server host:")]
    host: String,

    #[prompt("Server port:")]
    #[min(1024)]
    #[max(65535)]
    port: i32,

    #[prompt("Enable SSL:")]
    ssl: bool,
}

fn main() {
    let config = ServerConfig::wizard_builder().build().unwrap();
    println!("Configuration: {:#?}", config);
}
