//! Integration tests for derive-survey

use derive_survey::{Survey, TestBackend};

#[derive(Survey, Debug, PartialEq)]
struct SimpleConfig {
    #[ask("What is your name?")]
    name: String,

    #[ask("What is your age?")]
    age: u32,

    #[ask("Are you a developer?")]
    developer: bool,
}

#[test]
fn simple_survey_definition() {
    let survey = SimpleConfig::survey();

    assert_eq!(survey.questions.len(), 3);
    assert_eq!(survey.questions[0].ask(), "What is your name?");
    assert_eq!(survey.questions[1].ask(), "What is your age?");
    assert_eq!(survey.questions[2].ask(), "Are you a developer?");
}

#[test]
fn simple_survey_with_test_backend() {
    let config: SimpleConfig = SimpleConfig::builder()
        .run(
            TestBackend::new()
                .with_string("name", "Alice")
                .with_int("age", 30)
                .with_bool("developer", true),
        )
        .unwrap();

    assert_eq!(config.name, "Alice");
    assert_eq!(config.age, 30);
    assert!(config.developer);
}

#[test]
fn builder_with_suggestions() {
    // Just verify the builder methods exist and compile
    let _builder = SimpleConfig::builder()
        .suggest_name("Bob")
        .suggest_age(25)
        .suggest_developer(false);
}

#[test]
fn builder_with_assumptions() {
    // Just verify the builder methods exist and compile
    let _builder = SimpleConfig::builder()
        .assume_name("Charlie")
        .assume_age(35);
}

#[derive(Survey, Debug, PartialEq)]
#[prelude("Welcome to the server configuration!")]
#[epilogue("Configuration complete.")]
struct ServerConfig {
    #[ask("Server host:")]
    host: String,

    #[ask("Server port:")]
    #[min(1)]
    #[max(65535)]
    port: u16,
}

#[test]
fn prelude_and_epilogue() {
    let survey = ServerConfig::survey();

    assert_eq!(
        survey.prelude,
        Some("Welcome to the server configuration!".to_string())
    );
    assert_eq!(survey.epilogue, Some("Configuration complete.".to_string()));
}

#[test]
fn min_max_bounds() {
    use derive_survey::QuestionKind;

    let survey = ServerConfig::survey();
    let port_question = &survey.questions[1];

    match port_question.kind() {
        QuestionKind::Int(int_q) => {
            assert_eq!(int_q.min, Some(1));
            assert_eq!(int_q.max, Some(65535));
        }
        _ => panic!("Expected Int question kind"),
    }
}

#[derive(Survey, Debug, PartialEq)]
struct PasswordForm {
    #[ask("Enter password:")]
    #[mask]
    password: String,

    #[ask("Enter your biography:")]
    #[multiline]
    bio: String,
}

#[test]
fn mask_and_multiline() {
    use derive_survey::QuestionKind;

    let survey = PasswordForm::survey();

    assert!(matches!(
        survey.questions[0].kind(),
        QuestionKind::Masked(_)
    ));
    assert!(matches!(
        survey.questions[1].kind(),
        QuestionKind::Multiline(_)
    ));
}

// ============================================================================
// Nested Builder Tests
// ============================================================================

#[derive(Survey, Debug, PartialEq)]
struct Address {
    #[ask("Street address:")]
    street: String,

    #[ask("City:")]
    city: String,

    #[ask("Zip code:")]
    zip: String,
}

#[derive(Survey, Debug, PartialEq)]
enum PaymentMethod {
    #[ask("Cash payment")]
    Cash,

    #[ask("Credit card")]
    CreditCard {
        #[ask("Card number:")]
        number: String,
        #[ask("CVV:")]
        cvv: String,
    },

    #[ask("Bank transfer")]
    BankTransfer {
        #[ask("IBAN:")]
        iban: String,
    },
}

#[derive(Survey, Debug, PartialEq)]
struct OrderForm {
    #[ask("Customer name:")]
    customer_name: String,

    #[ask("Shipping address:")]
    shipping_address: Address,

    #[ask("Payment method:")]
    payment: PaymentMethod,

    #[ask("Nickname (optional):")]
    nickname: Option<String>,
}

#[test]
fn nested_struct_suggest_builder() {
    // Test that closure-based suggest methods work for nested structs
    let _builder = OrderForm::builder()
        .suggest_customer_name("John Doe")
        .suggest_shipping_address(|addr| {
            addr.street("123 Main St").city("Springfield").zip("12345")
        });
}

#[test]
fn enum_suggest_builder_suggest_variant() {
    // Test that suggest_* methods work for enums
    let _builder = OrderForm::builder().suggest_payment(|p| p.suggest_cash());

    let _builder2 = OrderForm::builder().suggest_payment(|p| p.suggest_credit_card());

    let _builder3 = OrderForm::builder().suggest_payment(|p| p.suggest_bank_transfer());
}

#[test]
fn enum_suggest_builder_variant_fields() {
    // Test that variant field methods work
    let _builder = OrderForm::builder().suggest_payment(|p| {
        p.suggest_credit_card()
            .credit_card(|cc| cc.number("4111111111111111").cvv("123"))
    });
}

#[test]
fn enum_suggest_multiple_variants() {
    // Test that we can suggest values for multiple variants
    // (only the selected one will be used, but all can have suggestions)
    let _builder = OrderForm::builder().suggest_payment(|p| {
        p.suggest_credit_card()
            .credit_card(|cc| cc.number("4111111111111111").cvv("123"))
            .bank_transfer(|bt| bt.iban("DE89370400440532013000"))
    });
}

#[test]
fn option_suggest_some() {
    // Test that Option fields can be suggested with some()
    let _builder = OrderForm::builder().suggest_nickname(|opt| opt.some("Johnny"));
}

#[test]
fn option_suggest_none() {
    // Test that Option fields can be suggested with none()
    let _builder = OrderForm::builder().suggest_nickname(|opt| opt.none());
}

#[test]
fn assume_nested_struct() {
    // Test that assume works the same as suggest for nested structs
    let _builder = OrderForm::builder().assume_shipping_address(|addr| {
        addr.street("456 Oak Ave").city("Shelbyville").zip("67890")
    });
}

#[test]
fn assume_enum_with_fields() {
    // Test that assume works for enums with variant fields
    let _builder = OrderForm::builder().assume_payment(|p| {
        p.suggest_bank_transfer()
            .bank_transfer(|bt| bt.iban("DE89370400440532013000"))
    });
}

#[test]
fn combined_suggest_and_assume() {
    // Test combining suggest and assume in one builder
    let _builder = OrderForm::builder()
        .suggest_customer_name("John Doe")
        .assume_shipping_address(|addr| addr.street("123 Main St").city("Springfield").zip("12345"))
        .suggest_payment(|p| p.suggest_cash())
        .assume_nickname(|opt| opt.none());
}
