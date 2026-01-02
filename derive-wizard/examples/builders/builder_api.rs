use derive_wizard::Wizard;

#[derive(Debug, Clone, Wizard)]
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
    println!("=== Builder API Demo ===");

    // Example 1: Simple builder with default backend
    println!("Example 1: Using default backend");
    let config1 = ServerConfig::wizard_builder().build();
    println!("Config: {:#?}", config1);

    // Example 2: Builder with custom backend (dialoguer)
    #[cfg(feature = "dialoguer-backend")]
    {
        println!("Example 2: Using dialoguer backend");
        let backend = derive_wizard::DialoguerBackend::new();
        let config2 = ServerConfig::wizard_builder().with_backend(backend).build();
        println!("Config: {:#?}", config2);
    }

    // Example 3: Builder with suggestions
    println!("Example 3: Using suggestions (re-prompting with existing values)");
    let suggestions = ServerConfig {
        host: "localhost".to_string(),
        port: 8080,
        ssl: true,
    };
    let config3 = ServerConfig::wizard_builder()
        .with_suggestions(suggestions)
        .build();
    println!("Config: {:#?}", config3);
}
