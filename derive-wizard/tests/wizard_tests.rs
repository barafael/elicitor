use derive_wizard::{TestBackend, Wizard};

#[derive(Debug, PartialEq, Wizard)]
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
    payment: PaymentMethod,
}

#[test]
fn test_basic_struct_with_string_int_bool() {
    let backend = TestBackend::new()
        .with_string("name", "Alice")
        .with_int("age", 30)
        .with_bool("active", true);

    let result = BasicUser::wizard_builder().with_backend(backend).build();

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
        .build();

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
        .with_string("street", "123 Main St")
        .with_string("city", "Springfield");

    let result = UserWithAddress::wizard_builder()
        .with_backend(backend)
        .build();

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
        .with_string("selected_alternative", "Cash");

    let result = Order::wizard_builder().with_backend(backend).build();

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
        .with_string("selected_alternative", "CreditCard")
        .with_string("number", "1234-5678-9012-3456")
        .with_string("cvv", "123");

    let result = Order::wizard_builder().with_backend(backend).build();

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
        .with_string("selected_alternative", "BankTransfer")
        .with_string("account", "DE89370400440532013000");

    let result = Order::wizard_builder().with_backend(backend).build();

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

    let result = BasicUser::wizard_builder().with_backend(backend).build();

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

    let result1 = BasicUser::wizard_builder().with_backend(backend1).build();

    let backend2 = TestBackend::new()
        .with_string("name", "User2")
        .with_int("age", 40)
        .with_bool("active", false);

    let result2 = BasicUser::wizard_builder().with_backend(backend2).build();

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

    let result = NumericTypes::wizard_builder().with_backend(backend).build();

    assert_eq!(result.int_val, 100);
    assert_eq!(result.float_val, 3.15);
    assert_eq!(result.small_int, 42);
    assert!((result.small_float - 2.72).abs() < 0.001);
}

#[derive(Debug, PartialEq, Wizard)]
struct Config {
    debug_mode: bool,
    verbose: bool,
}

#[test]
fn test_boolean_fields() {
    let backend = TestBackend::new()
        .with_bool("debug_mode", true)
        .with_bool("verbose", false);

    let result = Config::wizard_builder().with_backend(backend).build();

    assert!(result.debug_mode);
    assert!(!result.verbose);
}

#[derive(Debug, PartialEq, Wizard)]
struct Person {
    name: String,
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
        .with_string("email", "john@example.com")
        .with_string("phone", "+1-555-0100");

    let result = Person::wizard_builder().with_backend(backend).build();

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
    status: Status,
}

#[test]
fn test_struct_with_enum_field() {
    let backend = TestBackend::new()
        .with_string("username", "alice")
        .with_string("selected_alternative", "Pending")
        .with_string("reason", "Awaiting verification");

    let result = Account::wizard_builder().with_backend(backend).build();

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

    let result = BasicUser::wizard_builder().with_backend(backend).build();

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

    let result = BasicUser::wizard_builder().with_backend(backend).build();

    assert_eq!(result.age, -5);
}

#[test]
fn test_large_numbers() {
    let backend = TestBackend::new()
        .with_int("int_val", i64::MAX)
        .with_float("float_val", f64::MAX)
        .with_int("small_int", i32::MAX as i64)
        .with_float("small_float", f32::MAX as f64);

    let result = NumericTypes::wizard_builder().with_backend(backend).build();

    assert_eq!(result.int_val, i64::MAX);
    assert_eq!(result.float_val, f64::MAX);
    assert_eq!(result.small_int, i32::MAX);
    assert!((result.small_float - f32::MAX).abs() < 1.0);
}
