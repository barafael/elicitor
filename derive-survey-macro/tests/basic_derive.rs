//! Basic tests for the Survey derive macro

// We need to create a fake derive_survey module for the macro to work
mod derive_survey {
    pub use derive_survey_types::*;
}

use derive_survey::Survey; // Import the trait
use derive_survey_macro::Survey;

#[derive(Survey, Debug, PartialEq)]
struct SimpleStruct {
    #[ask("What is your name?")]
    name: String,

    #[ask("How old are you?")]
    age: u32,

    #[ask("Are you a student?")]
    student: bool,
}

#[test]
fn test_simple_struct_survey() {
    let survey = SimpleStruct::survey();

    assert_eq!(survey.questions.len(), 3);

    // Check first question
    assert_eq!(survey.questions[0].path().as_str(), "name");
    assert_eq!(survey.questions[0].ask(), "What is your name?");

    // Check second question
    assert_eq!(survey.questions[1].path().as_str(), "age");
    assert_eq!(survey.questions[1].ask(), "How old are you?");

    // Check third question
    assert_eq!(survey.questions[2].path().as_str(), "student");
    assert_eq!(survey.questions[2].ask(), "Are you a student?");
}

#[test]
fn test_simple_struct_from_responses() {
    let mut responses = derive_survey::Responses::new();
    responses.insert("name", "Alice");
    responses.insert("age", derive_survey::ResponseValue::Int(25));
    responses.insert("student", derive_survey::ResponseValue::Bool(true));

    let result = SimpleStruct::from_responses(&responses);

    assert_eq!(result.name, "Alice");
    assert_eq!(result.age, 25);
    assert_eq!(result.student, true);
}

#[test]
fn test_builder_methods_exist() {
    // Just verify the builder methods compile
    let _builder = SimpleStruct::builder()
        .suggest_name("Bob")
        .suggest_age(30)
        .assume_student(false);
}

#[derive(Survey, Debug, PartialEq)]
#[prelude("Welcome to the survey!")]
#[epilogue("Thank you!")]
struct WithPreludeEpilogue {
    #[ask("Name:")]
    name: String,
}

#[test]
fn test_prelude_epilogue() {
    let survey = WithPreludeEpilogue::survey();

    assert_eq!(survey.prelude, Some("Welcome to the survey!".to_string()));
    assert_eq!(survey.epilogue, Some("Thank you!".to_string()));
}

#[derive(Survey, Debug, PartialEq)]
struct WithMinMax {
    #[ask("Age:")]
    #[min(18)]
    #[max(120)]
    age: i32,
}

#[test]
fn test_min_max() {
    let survey = WithMinMax::survey();

    match survey.questions[0].kind() {
        derive_survey::QuestionKind::Int(int_q) => {
            assert_eq!(int_q.min, Some(18));
            assert_eq!(int_q.max, Some(120));
        }
        _ => panic!("Expected Int question kind"),
    }
}

#[derive(Survey, Debug, PartialEq)]
struct WithMaskedAndMultiline {
    #[ask("Password:")]
    #[mask]
    password: String,

    #[ask("Bio:")]
    #[multiline]
    bio: String,
}

#[test]
fn test_masked_and_multiline() {
    let survey = WithMaskedAndMultiline::survey();

    assert!(matches!(
        survey.questions[0].kind(),
        derive_survey::QuestionKind::Masked(_)
    ));
    assert!(matches!(
        survey.questions[1].kind(),
        derive_survey::QuestionKind::Multiline(_)
    ));
}
