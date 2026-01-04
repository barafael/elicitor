use derive_wizard::{TestBackend, Wizard};

#[derive(Debug, PartialEq, Clone, Wizard)]
struct BasicUser {
    name: String,
    age: i64,
    active: bool,
}

#[derive(Debug, PartialEq, Wizard)]
struct UserWithFloat {
    name: String,
    rating: f64,
}

#[derive(Debug, PartialEq, Wizard)]
struct Address {
    street: String,
    city: String,
}

#[derive(Debug, PartialEq, Wizard)]
struct UserWithAddress {
    name: String,
    #[prompt("Address:")]
    address: Address,
}

#[derive(Debug, PartialEq, Wizard)]
enum PaymentMethod {
    Cash,
    CreditCard { number: String, cvv: String },
    BankTransfer { account: String },
}

#[derive(Debug, PartialEq, Wizard)]
struct Order {
    item: String,
    quantity: i64,
    #[prompt("Payment method:")]
    payment: PaymentMethod,
}

#[test]
fn test_basic_struct_with_string_int_bool() {
    let backend = TestBackend::new()
        .with_string("name", "Alice")
        .with_int("age", 30)
        .with_bool("active", true);

    let result = BasicUser::wizard_builder()
        .with_backend(backend)
        .build()
        .unwrap();

    assert_eq!(
        result,
        BasicUser {
            name: "Alice".to_string(),
            age: 30,
            active: true,
        }
    );
}

#[test]
fn test_struct_with_float() {
    let backend = TestBackend::new()
        .with_string("name", "Bob")
        .with_float("rating", 4.5);

    let result = UserWithFloat::wizard_builder()
        .with_backend(backend)
        .build()
        .unwrap();

    assert_eq!(
        result,
        UserWithFloat {
            name: "Bob".to_string(),
            rating: 4.5,
        }
    );
}

#[test]
fn test_nested_struct() {
    let backend = TestBackend::new()
        .with_string("name", "Charlie")
        .with_string("address.street", "123 Main St")
        .with_string("address.city", "Springfield");

    let result = UserWithAddress::wizard_builder()
        .with_backend(backend)
        .build()
        .unwrap();

    assert_eq!(
        result,
        UserWithAddress {
            name: "Charlie".to_string(),
            address: Address {
                street: "123 Main St".to_string(),
                city: "Springfield".to_string(),
            },
        }
    );
}

#[test]
fn test_enum_simple_variant() {
    let backend = TestBackend::new()
        .with_string("item", "Laptop")
        .with_int("quantity", 2)
        .with_int(
            format!("payment.{}", derive_wizard::SELECTED_ALTERNATIVE_KEY),
            0,
        ); // Cash

    let result = Order::wizard_builder()
        .with_backend(backend)
        .build()
        .unwrap();

    assert_eq!(
        result,
        Order {
            item: "Laptop".to_string(),
            quantity: 2,
            payment: PaymentMethod::Cash,
        }
    );
}

#[test]
fn test_enum_variant_with_fields() {
    let backend = TestBackend::new()
        .with_string("item", "Laptop")
        .with_int("quantity", 2)
        .with_int(
            format!("payment.{}", derive_wizard::SELECTED_ALTERNATIVE_KEY),
            1,
        ) // CreditCard
        .with_string("payment.number", "1234-5678-9012-3456")
        .with_string("payment.cvv", "123");

    let result = Order::wizard_builder()
        .with_backend(backend)
        .build()
        .unwrap();

    assert_eq!(
        result,
        Order {
            item: "Laptop".to_string(),
            quantity: 2,
            payment: PaymentMethod::CreditCard {
                number: "1234-5678-9012-3456".to_string(),
                cvv: "123".to_string(),
            },
        }
    );
}

#[test]
fn test_different_enum_variant() {
    let backend = TestBackend::new()
        .with_string("item", "Phone")
        .with_int("quantity", 1)
        .with_int(
            format!("payment.{}", derive_wizard::SELECTED_ALTERNATIVE_KEY),
            2,
        ) // BankTransfer
        .with_string("payment.account", "DE89370400440532013000");

    let result = Order::wizard_builder()
        .with_backend(backend)
        .build()
        .unwrap();

    assert_eq!(
        result,
        Order {
            item: "Phone".to_string(),
            quantity: 1,
            payment: PaymentMethod::BankTransfer {
                account: "DE89370400440532013000".to_string(),
            },
        }
    );
}

#[test]
fn test_builder_pattern_chaining() {
    let backend = TestBackend::new()
        .with_string("name", "Dave")
        .with_int("age", 25)
        .with_bool("active", false);

    let result = BasicUser::wizard_builder()
        .with_backend(backend)
        .build()
        .unwrap();

    assert_eq!(result.name, "Dave");
    assert_eq!(result.age, 25);
    assert!(!result.active);
}

#[test]
fn test_multiple_test_backends() {
    let backend1 = TestBackend::new()
        .with_string("name", "User1")
        .with_int("age", 20)
        .with_bool("active", true);

    let result1 = BasicUser::wizard_builder()
        .with_backend(backend1)
        .build()
        .unwrap();

    let backend2 = TestBackend::new()
        .with_string("name", "User2")
        .with_int("age", 40)
        .with_bool("active", false);

    let result2 = BasicUser::wizard_builder()
        .with_backend(backend2)
        .build()
        .unwrap();

    assert_eq!(result1.name, "User1");
    assert_eq!(result2.name, "User2");
    assert_ne!(result1.age, result2.age);
}

#[derive(Debug, PartialEq, Wizard)]
struct NumericTypes {
    int_val: i64,
    float_val: f64,
    small_int: i32,
    small_float: f32,
}

#[test]
fn test_multiple_numeric_types() {
    let backend = TestBackend::new()
        .with_int("int_val", 100)
        .with_float("float_val", 3.15)
        .with_int("small_int", 42)
        .with_float("small_float", 2.72);

    let result = NumericTypes::wizard_builder()
        .with_backend(backend)
        .build()
        .unwrap();

    assert_eq!(result.int_val, 100);
    assert_eq!(result.float_val, 3.15);
    assert_eq!(result.small_int, 42);
    assert!((result.small_float - 2.72).abs() < 0.001);
}

#[derive(Debug, PartialEq, Clone, Wizard)]
struct Config {
    debug_mode: bool,
    port: i64,
    host: String,
}

#[test]
fn test_boolean_fields() {
    let backend = TestBackend::new()
        .with_bool("debug_mode", true)
        .with_int("port", 3000)
        .with_string("host", "0.0.0.0");

    let result = Config::wizard_builder()
        .with_backend(backend)
        .build()
        .unwrap();

    assert!(result.debug_mode);
    assert_eq!(result.port, 3000);
    assert_eq!(result.host, "0.0.0.0");
}

#[derive(Debug, PartialEq, Wizard)]
struct Person {
    name: String,
    #[prompt("Contact info:")]
    contact: ContactInfo,
}

#[derive(Debug, PartialEq, Wizard)]
struct ContactInfo {
    email: String,
    phone: String,
}

#[test]
fn test_deeply_nested_structs() {
    let backend = TestBackend::new()
        .with_string("name", "John Doe")
        .with_string("contact.email", "john@example.com")
        .with_string("contact.phone", "+1-555-0100");

    let result = Person::wizard_builder()
        .with_backend(backend)
        .build()
        .unwrap();

    assert_eq!(result.name, "John Doe");
    assert_eq!(result.contact.email, "john@example.com");
    assert_eq!(result.contact.phone, "+1-555-0100");
}

#[derive(Debug, PartialEq, Wizard)]
enum Status {
    Active,
    Inactive,
    Pending { reason: String },
}

#[derive(Debug, PartialEq, Wizard)]
struct Account {
    username: String,
    #[prompt("Account status:")]
    status: Status,
}

#[test]
fn test_struct_with_enum_field() {
    let backend = TestBackend::new()
        .with_string("username", "alice")
        .with_int(
            format!("status.{}", derive_wizard::SELECTED_ALTERNATIVE_KEY),
            2,
        ) // Pending
        .with_string("status.reason", "Awaiting verification");

    let result = Account::wizard_builder()
        .with_backend(backend)
        .build()
        .unwrap();

    assert_eq!(result.username, "alice");
    assert_eq!(
        result.status,
        Status::Pending {
            reason: "Awaiting verification".to_string()
        }
    );
}

#[test]
fn test_edge_case_empty_strings() {
    let backend = TestBackend::new()
        .with_string("name", "")
        .with_int("age", 0)
        .with_bool("active", false);

    let result = BasicUser::wizard_builder()
        .with_backend(backend)
        .build()
        .unwrap();

    assert_eq!(result.name, "");
    assert_eq!(result.age, 0);
    assert!(!result.active);
}

#[test]
fn test_negative_numbers() {
    let backend = TestBackend::new()
        .with_string("name", "Test")
        .with_int("age", -5)
        .with_bool("active", true);

    let result = BasicUser::wizard_builder()
        .with_backend(backend)
        .build()
        .unwrap();

    assert_eq!(result.age, -5);
}

#[test]
fn test_large_numbers() {
    let backend = TestBackend::new()
        .with_int("int_val", i64::MAX)
        .with_float("float_val", f64::MAX)
        .with_int("small_int", i32::MAX as i64)
        .with_float("small_float", f32::MAX as f64);

    let result = NumericTypes::wizard_builder()
        .with_backend(backend)
        .build()
        .unwrap();

    assert_eq!(result.int_val, i64::MAX);
    assert_eq!(result.float_val, f64::MAX);
    assert_eq!(result.small_int, i32::MAX);
    assert!((result.small_float - f32::MAX).abs() < 1.0);
}

#[test]
fn test_assumptions_skip_questions() {
    // Create a config with some values
    let _initial_config = Config {
        debug_mode: true,
        port: 8080,
        host: "localhost".to_string(),
    };

    // Use assumptions - should not require any user input
    // The TestBackend doesn't provide any answers, but assumptions should fill them in
    let backend = TestBackend::new();

    let result = Config::wizard_builder()
        .assume_field("debug_mode", true)
        .assume_field("port", 8080)
        .assume_field("host", "localhost".to_string())
        .with_backend(backend)
        .build()
        .unwrap();

    // Verify the result matches the assumptions
    assert!(result.debug_mode);
    assert_eq!(result.port, 8080);
    assert_eq!(result.host, "localhost");
}

#[test]
fn test_suggest_field() {
    // Test the suggest_field API - questions are asked but with pre-filled defaults
    let backend = TestBackend::new()
        .with_string("name", "Bob") // User can override the suggestion
        .with_int("age", 30)
        .with_bool("active", false);

    let result = BasicUser::wizard_builder()
        .suggest_field("name", "Alice".to_string()) // Suggest but don't assume
        .suggest_field("age", 25)
        .with_backend(backend)
        .build()
        .unwrap();

    // The backend's answers should win over suggestions
    assert_eq!(result.name, "Bob");
    assert_eq!(result.age, 30);
    assert!(!result.active);
}

#[test]
fn test_multiple_suggest_fields() {
    // Test suggesting multiple individual fields
    let backend = TestBackend::new()
        .with_string("host", "production.example.com") // Override suggestion
        .with_int("port", 443) // Override suggestion
        .with_bool("debug_mode", false); // Override suggestion

    let result = Config::wizard_builder()
        .suggest_field("host", "localhost".to_string())
        .suggest_field("port", 8080)
        .suggest_field("debug_mode", true)
        .with_backend(backend)
        .build()
        .unwrap();

    // Backend answers should override suggestions
    assert_eq!(result.host, "production.example.com");
    assert_eq!(result.port, 443);
    assert!(!result.debug_mode);
}

#[test]
fn test_suggest_field_vs_assume_field() {
    // Test that assumptions and suggestions work together
    let backend = TestBackend::new()
        .with_string("name", "Charlie") // This will be asked (suggested)
        .with_int("age", 35); // This will be asked (suggested)
    // Note: 'active' is assumed, so it won't be asked

    let result = BasicUser::wizard_builder()
        .suggest_field("name", "Suggested Name".to_string()) // Suggestion - will ask
        .suggest_field("age", 99) // Suggestion - will ask
        .assume_field("active", true) // Assumption - will skip
        .with_backend(backend)
        .build()
        .unwrap();

    assert_eq!(result.name, "Charlie"); // User input overrides suggestion
    assert_eq!(result.age, 35); // User input overrides suggestion
    assert!(result.active); // Assumed value, not asked
}

#[test]
fn test_nested_suggest_field() {
    use derive_wizard::field;

    // Test suggesting nested fields using the field! macro
    let backend = TestBackend::new()
        .with_string("name", "John")
        .with_string("address.street", "456 Oak Ave") // Override suggestion
        .with_string("address.city", "Boston"); // Override suggestion

    let result = UserWithAddress::wizard_builder()
        .suggest_field(field!(name), "Alice".to_string())
        .suggest_field(
            field!(UserWithAddress::address::street),
            "123 Main St".to_string(),
        )
        .suggest_field(
            field!(UserWithAddress::address::city),
            "Springfield".to_string(),
        )
        .with_backend(backend)
        .build()
        .unwrap();

    // User input should override suggestions
    assert_eq!(result.name, "John");
    assert_eq!(result.address.street, "456 Oak Ave");
    assert_eq!(result.address.city, "Boston");
}

#[test]
fn test_nested_assume_field() {
    use derive_wizard::field;

    // Test assuming nested fields - questions should be skipped
    let backend = TestBackend::new().with_string("name", "Jane"); // Only this will be asked

    let result = UserWithAddress::wizard_builder()
        .assume_field(
            field!(UserWithAddress::address::street),
            "789 Elm St".to_string(),
        )
        .assume_field(
            field!(UserWithAddress::address::city),
            "Portland".to_string(),
        )
        .with_backend(backend)
        .build()
        .unwrap();

    assert_eq!(result.name, "Jane");
    assert_eq!(result.address.street, "789 Elm St"); // Assumed, not asked
    assert_eq!(result.address.city, "Portland"); // Assumed, not asked
}

#[test]
fn test_nested_mixed_suggest_and_assume() {
    use derive_wizard::field;

    // Test mixing suggestions and assumptions on nested fields
    let backend = TestBackend::new()
        .with_string("name", "Bob")
        .with_string("address.street", "999 Pine Rd"); // This will be asked with suggestion

    let result = UserWithAddress::wizard_builder()
        .suggest_field(
            field!(UserWithAddress::address::street),
            "100 Default St".to_string(),
        )
        .assume_field(
            field!(UserWithAddress::address::city),
            "Seattle".to_string(),
        )
        .with_backend(backend)
        .build()
        .unwrap();

    assert_eq!(result.name, "Bob");
    assert_eq!(result.address.street, "999 Pine Rd"); // Suggested, user overrode
    assert_eq!(result.address.city, "Seattle"); // Assumed, skipped
}

#[test]
fn test_deeply_nested_assume_field() {
    use derive_wizard::field;

    // Test with deeply nested structure
    let backend = TestBackend::new().with_string("name", "Alice"); // Only this will be asked

    let result = Person::wizard_builder()
        .assume_field(
            field!(Person::contact::email),
            "alice@example.com".to_string(),
        )
        .assume_field(field!(Person::contact::phone), "+1-555-9999".to_string())
        .with_backend(backend)
        .build()
        .unwrap();

    assert_eq!(result.name, "Alice");
    assert_eq!(result.contact.email, "alice@example.com");
    assert_eq!(result.contact.phone, "+1-555-9999");
}

#[test]
fn test_duplicate_field_names_different_paths() {
    use derive_wizard::{Wizard, field};

    // Create a struct with duplicate field names in different nested structs
    #[derive(Debug, PartialEq, Wizard)]
    struct Organization {
        #[prompt("Organization name:")]
        name: String,
        #[prompt("Primary department:")]
        primary: Department,
        #[prompt("Secondary department:")]
        secondary: Department,
    }

    #[derive(Debug, PartialEq, Wizard)]
    struct Department {
        #[prompt("Department name:")]
        name: String,
        #[prompt("Budget:")]
        budget: i32,
    }

    // Both nested structs have a 'name' field - test disambiguation
    let backend = TestBackend::new()
        .with_string("name", "Acme Corp") // Top-level name
        .with_string("secondary.name", "Sales")
        .with_int("secondary.budget", 50000); // secondary department budget (primary assumed)

    let result = Organization::wizard_builder()
        .assume_field(
            field!(Organization::primary::name),
            "Engineering".to_string(),
        )
        .assume_field(field!(Organization::primary::budget), 100000)
        .suggest_field(
            field!(Organization::secondary::name),
            "Marketing".to_string(),
        )
        .with_backend(backend)
        .build()
        .unwrap();

    assert_eq!(result.name, "Acme Corp");
    assert_eq!(result.primary.name, "Engineering"); // Assumed
    assert_eq!(result.primary.budget, 100000); // Assumed
    assert_eq!(result.secondary.budget, 50000); // Asked
}

#[test]
fn test_assumptions_vs_suggestions() {
    // Test that assumptions take precedence over suggestions
    let _assumptions = BasicUser {
        name: "Assumed Name".to_string(),
        age: 100,
        active: true,
    };

    let suggestions = BasicUser {
        name: "Suggested Name".to_string(),
        age: 50,
        active: false,
    };

    let backend = TestBackend::new();

    let result = BasicUser::wizard_builder()
        .with_suggestions(suggestions)
        .assume_field("name", "Assumed Name".to_string())
        .assume_field("age", 100)
        .assume_field("active", true)
        .with_backend(backend)
        .build()
        .unwrap();

    // Assumptions should take precedence
    assert_eq!(result.name, "Assumed Name");
    assert_eq!(result.age, 100);
    assert!(result.active);
}

#[test]
fn test_partial_assumptions() {
    // Test that we can assume some fields and still ask about others
    let _partial = BasicUser {
        name: "Fixed Name".to_string(),
        age: 30,
        active: true,
    };

    // Provide an answer for the field we want to change
    let backend = TestBackend::new().with_string("name", "Override Name"); // This should NOT override the assumption

    let result = BasicUser::wizard_builder()
        .assume_field("name", "Fixed Name".to_string())
        .assume_field("age", 30)
        .assume_field("active", true)
        .with_backend(backend)
        .build()
        .unwrap();

    // The assumption should win
    assert_eq!(result.name, "Fixed Name");
    assert_eq!(result.age, 30);
    assert!(result.active);
}

#[test]
fn test_assume_field() {
    // Test the assume_field API - only assume specific fields
    let backend = TestBackend::new()
        .with_string("name", "Alice") // Will be asked
        .with_int("age", 25); // Will be asked
    // Note: 'active' is assumed, so we don't provide an answer for it

    let result = BasicUser::wizard_builder()
        .assume_field("active", true) // Only assume this field
        .with_backend(backend)
        .build()
        .unwrap();

    assert_eq!(result.name, "Alice");
    assert_eq!(result.age, 25);
    assert!(result.active); // This was assumed, not asked
}

#[test]
fn test_multiple_assume_fields() {
    // Test assuming multiple individual fields
    let backend = TestBackend::new().with_string("host", "localhost"); // Only this will be asked

    let result = Config::wizard_builder()
        .assume_field("debug_mode", true)
        .assume_field("port", 8080)
        .with_backend(backend)
        .build()
        .unwrap();

    assert!(result.debug_mode);
    assert_eq!(result.port, 8080);
    assert_eq!(result.host, "localhost");
}
