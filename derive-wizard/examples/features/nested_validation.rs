use derive_wizard::{BackendError, Wizard};

fn validate_zip(value: &str, _answers: &derive_wizard::Answers) -> Result<(), String> {
    if value.len() == 5 && value.chars().all(|c| c.is_ascii_digit()) {
        Ok(())
    } else {
        Err("zip must be 5 digits".into())
    }
}

#[derive(Debug, Wizard)]
#[allow(dead_code)]
struct Address {
    #[validate("validate_zip")]
    zip: String,
}

#[derive(Debug, Wizard)]
#[allow(dead_code)]
struct Person {
    name: String,
    #[prompt("Address:")]
    address: Address,
}

fn main() -> Result<(), BackendError> {
    println!("Nested validation with requestty backend (enter an invalid zip to see the error):");
    let person = Person::wizard_builder().build()?;
    println!("Collected: {:?}", person);
    Ok(())
}
