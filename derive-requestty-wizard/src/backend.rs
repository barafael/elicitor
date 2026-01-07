//! Requestty backend implementation for SurveyBackend trait.

use derive_survey::{
    DefaultValue, Question, QuestionKind, ResponsePath, ResponseValue, Responses,
    SELECTED_VARIANT_KEY, SELECTED_VARIANTS_KEY, SurveyBackend, SurveyDefinition,
};
use thiserror::Error;

/// Error type for the Requestty backend.
#[derive(Debug, Error)]
pub enum RequesttyError {
    /// User cancelled the survey (e.g., pressed Ctrl+C).
    #[error("Survey cancelled by user")]
    Cancelled,

    /// An error occurred during prompting.
    #[error("Prompt error: {0}")]
    PromptError(String),

    /// Unexpected answer type received.
    #[error("Unexpected answer type: expected {expected}, got {got}")]
    UnexpectedAnswerType { expected: String, got: String },
}

impl From<requestty::ErrorKind> for RequesttyError {
    fn from(err: requestty::ErrorKind) -> Self {
        match err {
            requestty::ErrorKind::Interrupted => Self::Cancelled,
            _ => Self::PromptError(err.to_string()),
        }
    }
}

/// Requestty backend for interactive CLI prompts.
///
/// This backend uses the `requestty` library to present questions
/// to the user in a command-line interface.
#[derive(Debug, Default, Clone)]
pub struct RequesttyBackend;

impl RequesttyBackend {
    /// Create a new Requestty backend.
    pub const fn new() -> Self {
        Self
    }

    /// Ask a single question and store the response.
    fn ask_question(
        &self,
        question: &Question,
        responses: &mut Responses,
        validate: &dyn Fn(&ResponseValue, &Responses) -> Result<(), String>,
        path_prefix: Option<&ResponsePath>,
    ) -> Result<(), RequesttyError> {
        let path = match path_prefix {
            Some(prefix) => prefix.child(question.path().as_str()),
            None => question.path().clone(),
        };

        // Use the question's prompt, or fall back to a title-cased version of the path
        let prompt = if question.ask().is_empty() {
            // Convert path like "role" or "user_name" to "Role" or "User Name"
            path.as_str()
                .split('.')
                .last()
                .unwrap_or("")
                .split('_')
                .map(|word| {
                    let mut chars = word.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(first) => first.to_uppercase().chain(chars).collect(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
        } else {
            question.ask().to_string()
        };

        // Check for assumed values - skip the question entirely
        if let DefaultValue::Assumed(value) = question.default() {
            responses.insert(path, value.clone());
            return Ok(());
        }

        match question.kind() {
            QuestionKind::Unit => {
                // Nothing to collect for unit types
                Ok(())
            }

            QuestionKind::Input(input_q) => self.ask_input(
                &path,
                &prompt,
                input_q,
                question.default(),
                responses,
                validate,
            ),

            QuestionKind::Multiline(multiline_q) => self.ask_multiline(
                &path,
                &prompt,
                multiline_q,
                question.default(),
                responses,
                validate,
            ),

            QuestionKind::Masked(masked_q) => self.ask_masked(
                &path,
                &prompt,
                masked_q,
                question.default(),
                responses,
                validate,
            ),

            QuestionKind::Int(int_q) => self.ask_int(
                &path,
                &prompt,
                int_q,
                question.default(),
                responses,
                validate,
            ),

            QuestionKind::Float(float_q) => self.ask_float(
                &path,
                &prompt,
                float_q,
                question.default(),
                responses,
                validate,
            ),

            QuestionKind::Confirm(confirm_q) => {
                self.ask_confirm(&path, &prompt, confirm_q, question.default(), responses)
            }

            QuestionKind::OneOf(one_of) => {
                self.ask_one_of(&path, &prompt, one_of, responses, validate)
            }

            QuestionKind::AnyOf(any_of) => {
                self.ask_any_of(&path, &prompt, any_of, responses, validate)
            }

            QuestionKind::AllOf(all_of) => {
                // Recursively ask all nested questions
                for nested_q in all_of.questions() {
                    self.ask_question(nested_q, responses, validate, Some(&path))?;
                }
                Ok(())
            }
        }
    }

    fn ask_input(
        &self,
        path: &ResponsePath,
        prompt: &str,
        input_q: &derive_survey::InputQuestion,
        default: &DefaultValue,
        responses: &mut Responses,
        validate: &dyn Fn(&ResponseValue, &Responses) -> Result<(), String>,
    ) -> Result<(), RequesttyError> {
        loop {
            let mut q = requestty::Question::input(path.as_str()).message(prompt);

            // Apply default value
            if let Some(default_val) = default.value() {
                if let ResponseValue::String(s) = default_val {
                    q = q.default(s.clone());
                }
            } else if let Some(ref def) = input_q.default {
                q = q.default(def.clone());
            }

            // Set up validation - pass the value directly
            let responses_clone = responses.clone();
            let validate_fn = move |value: &str, _: &requestty::Answers| -> Result<(), String> {
                let rv = ResponseValue::String(value.to_string());
                validate(&rv, &responses_clone)
            };

            let result = requestty::prompt_one(q.validate(validate_fn).build());

            match result {
                Ok(requestty::Answer::String(s)) => {
                    responses.insert(path.clone(), ResponseValue::String(s));
                    return Ok(());
                }
                Ok(other) => {
                    return Err(RequesttyError::UnexpectedAnswerType {
                        expected: "String".to_string(),
                        got: format!("{other:?}"),
                    });
                }
                Err(e) => {
                    if matches!(e, requestty::ErrorKind::Interrupted) {
                        return Err(RequesttyError::Cancelled);
                    }
                    // For other errors, the validation message was shown, retry
                    eprintln!("Error: {e}");
                    continue;
                }
            }
        }
    }

    fn ask_multiline(
        &self,
        path: &ResponsePath,
        prompt: &str,
        multiline_q: &derive_survey::MultilineQuestion,
        default: &DefaultValue,
        responses: &mut Responses,
        validate: &dyn Fn(&ResponseValue, &Responses) -> Result<(), String>,
    ) -> Result<(), RequesttyError> {
        loop {
            let mut q = requestty::Question::editor(path.as_str()).message(prompt);

            if let Some(default_val) = default.value() {
                if let ResponseValue::String(s) = default_val {
                    q = q.default(s.clone());
                }
            } else if let Some(ref def) = multiline_q.default {
                q = q.default(def.clone());
            }

            let responses_clone = responses.clone();
            let validate_fn = move |value: &str, _: &requestty::Answers| -> Result<(), String> {
                let rv = ResponseValue::String(value.to_string());
                validate(&rv, &responses_clone)
            };

            let result = requestty::prompt_one(q.validate(validate_fn).build());

            match result {
                Ok(requestty::Answer::String(s)) => {
                    responses.insert(path.clone(), ResponseValue::String(s));
                    return Ok(());
                }
                Ok(other) => {
                    return Err(RequesttyError::UnexpectedAnswerType {
                        expected: "String".to_string(),
                        got: format!("{other:?}"),
                    });
                }
                Err(e) => {
                    if matches!(e, requestty::ErrorKind::Interrupted) {
                        return Err(RequesttyError::Cancelled);
                    }
                    eprintln!("Error: {e}");
                    continue;
                }
            }
        }
    }

    fn ask_masked(
        &self,
        path: &ResponsePath,
        prompt: &str,
        masked_q: &derive_survey::MaskedQuestion,
        default: &DefaultValue,
        responses: &mut Responses,
        validate: &dyn Fn(&ResponseValue, &Responses) -> Result<(), String>,
    ) -> Result<(), RequesttyError> {
        // Note: requestty password doesn't support default values for security
        let _ = default;

        loop {
            let mut q = requestty::Question::password(path.as_str()).message(prompt);

            if let Some(mask) = masked_q.mask {
                q = q.mask(mask);
            }

            let responses_clone = responses.clone();
            let validate_fn = move |value: &str, _: &requestty::Answers| -> Result<(), String> {
                let rv = ResponseValue::String(value.to_string());
                validate(&rv, &responses_clone)
            };

            let result = requestty::prompt_one(q.validate(validate_fn).build());

            match result {
                Ok(requestty::Answer::String(s)) => {
                    responses.insert(path.clone(), ResponseValue::String(s));
                    return Ok(());
                }
                Ok(other) => {
                    return Err(RequesttyError::UnexpectedAnswerType {
                        expected: "String".to_string(),
                        got: format!("{other:?}"),
                    });
                }
                Err(e) => {
                    if matches!(e, requestty::ErrorKind::Interrupted) {
                        return Err(RequesttyError::Cancelled);
                    }
                    eprintln!("Error: {e}");
                    continue;
                }
            }
        }
    }

    fn ask_int(
        &self,
        path: &ResponsePath,
        prompt: &str,
        int_q: &derive_survey::IntQuestion,
        default: &DefaultValue,
        responses: &mut Responses,
        validate: &dyn Fn(&ResponseValue, &Responses) -> Result<(), String>,
    ) -> Result<(), RequesttyError> {
        loop {
            let mut q = requestty::Question::int(path.as_str()).message(prompt);

            if let Some(default_val) = default.value() {
                if let ResponseValue::Int(i) = default_val {
                    q = q.default(*i);
                }
            } else if let Some(def) = int_q.default {
                q = q.default(def);
            }

            // Add min/max validation
            let min = int_q.min;
            let max = int_q.max;
            let responses_clone = responses.clone();

            let validate_fn = move |value: i64, _: &requestty::Answers| -> Result<(), String> {
                // Check bounds first
                if let Some(min_val) = min
                    && value < min_val
                {
                    return Err(format!("Value must be at least {min_val}"));
                }
                if let Some(max_val) = max
                    && value > max_val
                {
                    return Err(format!("Value must be at most {max_val}"));
                }
                // Then run custom validation
                let rv = ResponseValue::Int(value);
                validate(&rv, &responses_clone)
            };

            let result = requestty::prompt_one(q.validate(validate_fn).build());

            match result {
                Ok(requestty::Answer::Int(i)) => {
                    responses.insert(path.clone(), ResponseValue::Int(i));
                    return Ok(());
                }
                Ok(other) => {
                    return Err(RequesttyError::UnexpectedAnswerType {
                        expected: "Int".to_string(),
                        got: format!("{other:?}"),
                    });
                }
                Err(e) => {
                    if matches!(e, requestty::ErrorKind::Interrupted) {
                        return Err(RequesttyError::Cancelled);
                    }
                    eprintln!("Error: {e}");
                    continue;
                }
            }
        }
    }

    fn ask_float(
        &self,
        path: &ResponsePath,
        prompt: &str,
        float_q: &derive_survey::FloatQuestion,
        default: &DefaultValue,
        responses: &mut Responses,
        validate: &dyn Fn(&ResponseValue, &Responses) -> Result<(), String>,
    ) -> Result<(), RequesttyError> {
        loop {
            let mut q = requestty::Question::float(path.as_str()).message(prompt);

            if let Some(default_val) = default.value() {
                if let ResponseValue::Float(f) = default_val {
                    q = q.default(*f);
                }
            } else if let Some(def) = float_q.default {
                q = q.default(def);
            }

            // Add min/max validation
            let min = float_q.min;
            let max = float_q.max;
            let responses_clone = responses.clone();

            let validate_fn = move |value: f64, _: &requestty::Answers| -> Result<(), String> {
                if let Some(min_val) = min
                    && value < min_val
                {
                    return Err(format!("Value must be at least {min_val}"));
                }
                if let Some(max_val) = max
                    && value > max_val
                {
                    return Err(format!("Value must be at most {max_val}"));
                }
                let rv = ResponseValue::Float(value);
                validate(&rv, &responses_clone)
            };

            let result = requestty::prompt_one(q.validate(validate_fn).build());

            match result {
                Ok(requestty::Answer::Float(f)) => {
                    responses.insert(path.clone(), ResponseValue::Float(f));
                    return Ok(());
                }
                Ok(other) => {
                    return Err(RequesttyError::UnexpectedAnswerType {
                        expected: "Float".to_string(),
                        got: format!("{other:?}"),
                    });
                }
                Err(e) => {
                    if matches!(e, requestty::ErrorKind::Interrupted) {
                        return Err(RequesttyError::Cancelled);
                    }
                    eprintln!("Error: {e}");
                    continue;
                }
            }
        }
    }

    fn ask_confirm(
        &self,
        path: &ResponsePath,
        prompt: &str,
        confirm_q: &derive_survey::ConfirmQuestion,
        default: &DefaultValue,
        responses: &mut Responses,
    ) -> Result<(), RequesttyError> {
        let default_val = if let Some(ResponseValue::Bool(b)) = default.value() {
            *b
        } else {
            confirm_q.default
        };

        let q = requestty::Question::confirm(path.as_str())
            .message(prompt)
            .default(default_val)
            .build();

        let result = requestty::prompt_one(q)?;

        match result {
            requestty::Answer::Bool(b) => {
                responses.insert(path.clone(), ResponseValue::Bool(b));
                Ok(())
            }
            other => Err(RequesttyError::UnexpectedAnswerType {
                expected: "Bool".to_string(),
                got: format!("{other:?}"),
            }),
        }
    }

    fn ask_one_of(
        &self,
        path: &ResponsePath,
        prompt: &str,
        one_of: &derive_survey::OneOfQuestion,
        responses: &mut Responses,
        validate: &dyn Fn(&ResponseValue, &Responses) -> Result<(), String>,
    ) -> Result<(), RequesttyError> {
        // Build choices from variant names
        let choices: Vec<String> = one_of.variants.iter().map(|v| v.name.clone()).collect();

        let mut q = requestty::Question::select(path.as_str())
            .message(prompt)
            .choices(choices);

        if let Some(default_idx) = one_of.default {
            q = q.default(default_idx);
        }

        let result = requestty::prompt_one(q.build())?;

        let selection = match result {
            requestty::Answer::ListItem(item) => item.index,
            other => {
                return Err(RequesttyError::UnexpectedAnswerType {
                    expected: "ListItem".to_string(),
                    got: format!("{other:?}"),
                });
            }
        };

        // Store the selected variant index
        let variant_path = path.child(SELECTED_VARIANT_KEY);
        responses.insert(variant_path, ResponseValue::ChosenVariant(selection));

        // Ask follow-up questions for the selected variant
        let selected_variant = &one_of.variants[selection];
        match &selected_variant.kind {
            QuestionKind::Unit => {
                // No follow-up questions needed
            }
            QuestionKind::AllOf(all_of) => {
                for nested_q in all_of.questions() {
                    self.ask_question(nested_q, responses, validate, Some(path))?;
                }
            }
            QuestionKind::Input(_)
            | QuestionKind::Int(_)
            | QuestionKind::Float(_)
            | QuestionKind::Confirm(_)
            | QuestionKind::Masked(_)
            | QuestionKind::Multiline(_) => {
                // Create a synthetic question for the variant's data
                let variant_q = Question::new(
                    selected_variant.name.clone(),
                    format!("Enter {} value:", selected_variant.name),
                    selected_variant.kind.clone(),
                );
                self.ask_question(&variant_q, responses, validate, Some(path))?;
            }
            QuestionKind::OneOf(nested_one_of) => {
                // Nested enum
                let variant_q = Question::new(
                    selected_variant.name.clone(),
                    format!("Select {}:", selected_variant.name),
                    QuestionKind::OneOf(nested_one_of.clone()),
                );
                self.ask_question(&variant_q, responses, validate, Some(path))?;
            }
            QuestionKind::AnyOf(nested_any_of) => {
                let variant_q = Question::new(
                    selected_variant.name.clone(),
                    format!("Select {} options:", selected_variant.name),
                    QuestionKind::AnyOf(nested_any_of.clone()),
                );
                self.ask_question(&variant_q, responses, validate, Some(path))?;
            }
        }

        Ok(())
    }

    fn ask_any_of(
        &self,
        path: &ResponsePath,
        prompt: &str,
        any_of: &derive_survey::AnyOfQuestion,
        responses: &mut Responses,
        validate: &dyn Fn(&ResponseValue, &Responses) -> Result<(), String>,
    ) -> Result<(), RequesttyError> {
        // Loop until valid selection or user cancels
        let selections = loop {
            // Build choices with default selections
            let choices: Vec<_> = any_of
                .variants
                .iter()
                .enumerate()
                .map(|(idx, v)| {
                    let selected = any_of.defaults.contains(&idx);
                    (v.name.clone(), selected)
                })
                .collect();

            let q = requestty::Question::multi_select(path.as_str())
                .message(prompt)
                .choices_with_default(choices)
                .build();

            let result = requestty::prompt_one(q)?;

            let selections = match result {
                requestty::Answer::ListItems(items) => {
                    items.iter().map(|item| item.index).collect::<Vec<_>>()
                }
                other => {
                    return Err(RequesttyError::UnexpectedAnswerType {
                        expected: "ListItems".to_string(),
                        got: format!("{other:?}"),
                    });
                }
            };

            // Validate the selection before asking follow-up questions
            let selection_value = ResponseValue::ChosenVariants(selections.clone());
            if let Err(msg) = validate(&selection_value, responses) {
                // Show error and let user re-select
                println!("Error: {msg}");
                continue;
            }

            break selections;
        };

        // Store the selected variant indices
        let variants_path = path.child(SELECTED_VARIANTS_KEY);
        responses.insert(
            variants_path,
            ResponseValue::ChosenVariants(selections.clone()),
        );

        // Ask follow-up questions for each selected variant
        // Each item is indexed: inventory.0.field, inventory.1.field, etc.
        for (item_idx, &variant_idx) in selections.iter().enumerate() {
            let variant = &any_of.variants[variant_idx];
            let item_path = path.child(&item_idx.to_string());

            // Store which variant this item is
            let item_variant_path = item_path.child(SELECTED_VARIANT_KEY);
            responses.insert(item_variant_path, ResponseValue::ChosenVariant(variant_idx));

            match &variant.kind {
                QuestionKind::Unit => {
                    // No follow-up questions needed
                }
                QuestionKind::AllOf(all_of) => {
                    for nested_q in all_of.questions() {
                        self.ask_question(nested_q, responses, validate, Some(&item_path))?;
                    }
                }
                _ => {
                    // Handle other variant types if needed
                }
            }
        }

        Ok(())
    }
}

impl SurveyBackend for RequesttyBackend {
    type Error = RequesttyError;

    fn collect(
        &self,
        definition: &SurveyDefinition,
        validate: &dyn Fn(&ResponseValue, &Responses) -> Result<(), String>,
    ) -> Result<Responses, Self::Error> {
        let mut responses = Responses::new();

        // Show prelude if present
        if let Some(prelude) = &definition.prelude {
            println!("{prelude}");
            println!();
        }

        // Ask all questions
        for question in definition.questions() {
            self.ask_question(question, &mut responses, validate, None)?;
        }

        // Show epilogue if present
        if let Some(epilogue) = &definition.epilogue {
            println!();
            println!("{epilogue}");
        }

        Ok(responses)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_creation() {
        let _backend = RequesttyBackend::new();
    }

    #[test]
    fn test_error_types() {
        let err = RequesttyError::Cancelled;
        assert_eq!(err.to_string(), "Survey cancelled by user");

        let err = RequesttyError::PromptError("test error".to_string());
        assert_eq!(err.to_string(), "Prompt error: test error");

        let err = RequesttyError::UnexpectedAnswerType {
            expected: "String".to_string(),
            got: "Int".to_string(),
        };
        assert_eq!(
            err.to_string(),
            "Unexpected answer type: expected String, got Int"
        );
    }
}
