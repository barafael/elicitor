use derive_wizard::Wizard;

#[derive(Debug, Wizard)]
#[allow(dead_code)]
enum AuthMethod {
    #[prompt("Username and password")]
    UsernamePassword {
        #[prompt("Username:")]
        username: String,

        #[prompt("Password:")]
        #[mask('*')]
        password: String,
    },

    #[prompt("API key")]
    ApiKey {
        #[prompt("API key:")]
        #[mask('*')]
        api_key: String,
    },

    #[prompt("OAuth token")]
    OAuth {
        #[prompt("OAuth token:")]
        #[mask('*')]
        token: String,
    },
}

#[derive(Debug, Wizard)]
#[allow(dead_code)]
struct ServiceConfig {
    #[prompt("Service name:")]
    service_name: String,

    #[prompt("Service URL:")]
    url: String,

    #[prompt("Select authentication method:")]
    auth: AuthMethod,
}

fn main() {
    println!("=== Service Configuration - dialoguer Password Demo ===");
    println!("This demo showcases password masking and alternatives in dialoguer.");
    println!("Passwords and sensitive fields are masked with asterisks.");

    let backend = derive_wizard::DialoguerBackend::new();
    let config = ServiceConfig::wizard_builder()
        .with_backend(backend)
        .build();

    println!("=== Configuration Created ===");
    println!("{:#?}", config);
}
