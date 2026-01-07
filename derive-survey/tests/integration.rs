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
fn test_simple_survey_definition() {
    let survey = SimpleConfig::survey();

    assert_eq!(survey.questions.len(), 3);
    assert_eq!(survey.questions[0].ask(), "What is your name?");
    assert_eq!(survey.questions[1].ask(), "What is your age?");
    assert_eq!(survey.questions[2].ask(), "Are you a developer?");
}

#[test]
fn test_simple_survey_with_test_backend() {
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
    assert_eq!(config.developer, true);
}

#[test]
fn test_builder_with_suggestions() {
    // Just verify the builder methods exist and compile
    let _builder = SimpleConfig::builder()
        .suggest_name("Bob")
        .suggest_age(25)
        .suggest_developer(false);
}

#[test]
fn test_builder_with_assumptions() {
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
fn test_prelude_and_epilogue() {
    let survey = ServerConfig::survey();

    assert_eq!(
        survey.prelude,
        Some("Welcome to the server configuration!".to_string())
    );
    assert_eq!(survey.epilogue, Some("Configuration complete.".to_string()));
}

#[test]
fn test_min_max_bounds() {
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
fn test_mask_and_multiline() {
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
