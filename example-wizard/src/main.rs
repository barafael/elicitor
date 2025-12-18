use derive_wizard::Wizard;

#[derive(Debug, Wizard)]
#[allow(unused)]
struct Config {
    #[prompt("Enter the server address:")]
    #[validate_on_key("is_valid_address")]
    #[validate_on_submit("is_valid_address")]
    server: String,

    #[prompt("Enter the user ID:")]
    user_id: u16,
}

fn is_valid_address(input: &str, _answers: &derive_wizard::Answers) -> Result<(), String> {
    if input.contains(':') && input.len() >= 3 && !input.starts_with(':') && !input.ends_with(':') {
        Ok(())
    } else {
        Err("Address must be in format 'host:port' (e.g., 'localhost:8080')".to_string())
    }
}

fn main() {
    let config = Config::wizard();
    println!("Config: {config:#?}");
}
