use derive_wizard::Wizard;

#[derive(Debug, Wizard)]
#[allow(unused)]
struct Config {
    #[prompt("Enter the server address:")]
    // #[validate_on_key("is_valid_address")]
    // #[validate_on_submit("is_valid_address")]
    server: String,

    #[prompt("Enter the port number:")]
    port: u16,
}

fn main() {
    let config = Config::wizard();
    println!("Config: {config:#?}",);
}
