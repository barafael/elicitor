//! Test backend for testing surveys without user interaction.
//!
//! `TestBackend` allows you to run surveys programmatically by providing
//! pre-defined responses. This is useful for testing survey-enabled types.
//!
//! # Example
//!
//! ```rust,ignore
//! use derive_survey::{Survey, TestBackend};
//!
//! #[derive(Survey, Debug, PartialEq)]
//! struct Config {
//!     #[ask("Host:")]
//!     host: String,
//!     #[ask("Port:")]
//!     port: u16,
//! }
//!
//! let config: Config = Config::builder()
//!     .run(
//!         TestBackend::new()
//!             .with_response("host", "localhost")
//!             .with_response("port", 8080)
//!     )
//!     .unwrap();
//!
//! assert_eq!(config.host, "localhost");
//! assert_eq!(config.port, 8080);
//! ```

use std::collections::HashMap;

use crate::{ResponsePath, ResponseValue, Responses, SurveyBackend, SurveyDefinition};

/// A test backend that returns pre-configured responses.
///
/// This backend is useful for testing survey-enabled types without
/// requiring user interaction.
#[derive(Debug, Clone, Default)]
pub struct TestBackend {
    responses: HashMap<String, ResponseValue>,
}

/// Error type for TestBackend.
#[derive(Debug, thiserror::Error)]
pub enum TestBackendError {
    #[error("Missing response for path: {0}")]
    MissingResponse(String),

    #[error("Validation failed for '{path}': {message}")]
    ValidationFailed { path: String, message: String },
}

impl TestBackend {
    /// Create a new empty test backend.
    pub fn new() -> Self {
        Self {
            responses: HashMap::new(),
        }
    }

    /// Add a response for a given path.
    ///
    /// The path should match the field name or dot-separated path for nested fields.
    pub fn with_response(
        mut self,
        path: impl Into<String>,
        value: impl Into<ResponseValue>,
    ) -> Self {
        self.responses.insert(path.into(), value.into());
        self
    }

    /// Add a string response.
    pub fn with_string(self, path: impl Into<String>, value: impl Into<String>) -> Self {
        self.with_response(path, ResponseValue::String(value.into()))
    }

    /// Add an integer response.
    pub fn with_int(self, path: impl Into<String>, value: i64) -> Self {
        self.with_response(path, ResponseValue::Int(value))
    }

    /// Add a float response.
    pub fn with_float(self, path: impl Into<String>, value: f64) -> Self {
        self.with_response(path, ResponseValue::Float(value))
    }

    /// Add a boolean response.
    pub fn with_bool(self, path: impl Into<String>, value: bool) -> Self {
        self.with_response(path, ResponseValue::Bool(value))
    }

    /// Add a chosen variant response (for OneOf questions).
    pub fn with_variant(self, path: impl Into<String>, index: usize) -> Self {
        self.with_response(path, ResponseValue::ChosenVariant(index))
    }

    /// Add chosen variants response (for AnyOf questions).
    pub fn with_variants(self, path: impl Into<String>, indices: Vec<usize>) -> Self {
        self.with_response(path, ResponseValue::ChosenVariants(indices))
    }
}

impl SurveyBackend for TestBackend {
    type Error = TestBackendError;

    fn collect(
        &self,
        definition: &SurveyDefinition,
        validate: &dyn Fn(&ResponseValue, &Responses) -> Result<(), String>,
    ) -> Result<Responses, Self::Error> {
        let mut responses = Responses::new();

        // Recursively collect responses for all questions
        collect_question_responses(
            &definition.questions,
            &ResponsePath::empty(),
            &self.responses,
            &mut responses,
            validate,
        )?;

        Ok(responses)
    }
}

fn collect_question_responses(
    questions: &[crate::Question],
    prefix: &ResponsePath,
    test_responses: &HashMap<String, ResponseValue>,
    responses: &mut Responses,
    validate: &dyn Fn(&ResponseValue, &Responses) -> Result<(), String>,
) -> Result<(), TestBackendError> {
    use crate::QuestionKind;

    for question in questions {
        let full_path = if prefix.is_empty() {
            question.path().clone()
        } else {
            prefix.child(question.path().as_str())
        };

        let path_str = full_path.as_str().to_string();

        match question.kind() {
            QuestionKind::Unit => {
                // No response needed for unit types
            }
            QuestionKind::Input(_) | QuestionKind::Multiline(_) | QuestionKind::Masked(_) => {
                if let Some(value) = test_responses.get(&path_str) {
                    // Validate before inserting
                    if let Err(msg) = validate(value, responses) {
                        return Err(TestBackendError::ValidationFailed {
                            path: path_str,
                            message: msg,
                        });
                    }
                    responses.insert(full_path.clone(), value.clone());
                } else if !question.is_assumed() {
                    return Err(TestBackendError::MissingResponse(path_str));
                }
            }
            QuestionKind::Int(_) => {
                if let Some(value) = test_responses.get(&path_str) {
                    // Validate before inserting
                    if let Err(msg) = validate(value, responses) {
                        return Err(TestBackendError::ValidationFailed {
                            path: path_str,
                            message: msg,
                        });
                    }
                    responses.insert(full_path.clone(), value.clone());
                } else if !question.is_assumed() {
                    return Err(TestBackendError::MissingResponse(path_str));
                }
            }
            QuestionKind::Float(_) => {
                if let Some(value) = test_responses.get(&path_str) {
                    // Validate before inserting
                    if let Err(msg) = validate(value, responses) {
                        return Err(TestBackendError::ValidationFailed {
                            path: path_str,
                            message: msg,
                        });
                    }
                    responses.insert(full_path.clone(), value.clone());
                } else if !question.is_assumed() {
                    return Err(TestBackendError::MissingResponse(path_str));
                }
            }
            QuestionKind::Confirm(_) => {
                if let Some(value) = test_responses.get(&path_str) {
                    responses.insert(full_path, value.clone());
                } else if !question.is_assumed() {
                    return Err(TestBackendError::MissingResponse(path_str));
                }
            }
            QuestionKind::OneOf(one_of) => {
                let variant_key = format!("{}.{}", path_str, crate::SELECTED_VARIANT_KEY);
                if let Some(ResponseValue::ChosenVariant(idx)) = test_responses.get(&variant_key) {
                    responses.insert(
                        full_path.child(crate::SELECTED_VARIANT_KEY),
                        ResponseValue::ChosenVariant(*idx),
                    );

                    // Recursively collect responses for the selected variant
                    if let Some(variant) = one_of.variants.get(*idx)
                        && let QuestionKind::AllOf(all_of) = &variant.kind
                    {
                        collect_question_responses(
                            all_of.questions(),
                            &full_path,
                            test_responses,
                            responses,
                            validate,
                        )?;
                    }
                } else if !question.is_assumed() {
                    return Err(TestBackendError::MissingResponse(variant_key));
                }
            }
            QuestionKind::AnyOf(any_of) => {
                let variants_key = format!("{}.{}", path_str, crate::SELECTED_VARIANTS_KEY);
                if let Some(ResponseValue::ChosenVariants(indices)) =
                    test_responses.get(&variants_key)
                {
                    responses.insert(
                        full_path.child(crate::SELECTED_VARIANTS_KEY),
                        ResponseValue::ChosenVariants(indices.clone()),
                    );

                    // Recursively collect responses for each selected variant
                    for &idx in indices {
                        if let Some(variant) = any_of.variants.get(idx)
                            && let QuestionKind::AllOf(all_of) = &variant.kind
                        {
                            let variant_prefix = full_path.child(&idx.to_string());
                            collect_question_responses(
                                all_of.questions(),
                                &variant_prefix,
                                test_responses,
                                responses,
                                validate,
                            )?;
                        }
                    }
                } else if !question.is_assumed() {
                    return Err(TestBackendError::MissingResponse(variants_key));
                }
            }
            QuestionKind::AllOf(all_of) => {
                collect_question_responses(
                    all_of.questions(),
                    &full_path,
                    test_responses,
                    responses,
                    validate,
                )?;
            }
        }
    }

    Ok(())
}
