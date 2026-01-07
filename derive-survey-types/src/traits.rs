use std::collections::HashMap;

use crate::{ResponsePath, ResponseValue, Responses, SurveyDefinition};

/// Trait for types that can be collected via a survey.
///
/// This trait is typically derived using `#[derive(Survey)]`.
/// It provides the survey structure, response reconstruction, and validation.
pub trait Survey: Sized {
    /// Returns the survey structure (questions, prompts, validation metadata).
    fn survey() -> SurveyDefinition;

    /// Reconstructs an instance from collected responses.
    ///
    /// This is infallible — the macro generates both `survey()` and `from_responses()`,
    /// guaranteeing they are consistent. If all questions are answered, reconstruction succeeds.
    fn from_responses(responses: &Responses) -> Self;

    /// Validates a field's value.
    ///
    /// Called by backends during input collection to provide immediate feedback.
    ///
    /// # Arguments
    /// * `value` - The value to validate
    /// * `responses` - All responses collected so far (for inter-field validation)
    ///
    /// # Returns
    /// * `Ok(())` if validation passes
    /// * `Err(message)` with an error message if validation fails
    fn validate_field(value: &ResponseValue, responses: &Responses) -> Result<(), String>;

    /// Validates the entire survey (composite validators, inter-field conditions).
    ///
    /// Called by form backends to validate all fields at once, typically on submit.
    /// Returns a map of path -> error message for all validation failures.
    ///
    /// The default implementation returns an empty map (no composite validation).
    fn validate_all(_responses: &Responses) -> HashMap<ResponsePath, String> {
        HashMap::new()
    }
}

/// Trait for backend implementations that collect survey responses.
///
/// Backends receive a `SurveyDefinition` and return `Responses`.
/// They decide how to present the survey (wizard-style, form-style, etc.)
/// and handle validation internally in retry loops.
pub trait SurveyBackend {
    /// The error type for this backend.
    type Error: Into<anyhow::Error>;

    /// Collect responses for a survey.
    ///
    /// # Arguments
    /// * `definition` - The survey structure to collect responses for
    /// * `validate` - A function to validate field values. Receives the value being
    ///   validated and all responses collected so far (for inter-field validation).
    ///
    /// # Returns
    /// * `Ok(responses)` on success
    /// * `Err` on cancellation or backend failure
    ///
    /// Validation is handled internally — this only returns when all fields are valid
    /// (or on error/cancellation).
    fn collect(
        &self,
        definition: &SurveyDefinition,
        validate: &dyn Fn(&ResponseValue, &Responses) -> Result<(), String>,
    ) -> Result<Responses, Self::Error>;
}
