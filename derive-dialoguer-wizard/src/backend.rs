//! Dialoguer backend implementation for SurveyBackend trait.

use derive_survey::{
    DefaultValue, ListElementKind, Question, QuestionKind, ResponsePath, ResponseValue, Responses,
    SELECTED_VARIANT_KEY, SELECTED_VARIANTS_KEY, SurveyBackend, SurveyDefinition,
};
use dialoguer::{Confirm, Editor, Input, MultiSelect, Password, Select, theme::ColorfulTheme};
use thiserror::Error;

/// Error type for the Dialoguer backend.
#[derive(Debug, Error)]
pub enum DialoguerError {
    /// User cancelled the survey (e.g., pressed Ctrl+C or Escape).
    #[error("Survey cancelled by user")]
    Cancelled,

    /// An I/O error occurred during prompting.
    #[error("Dialoguer error: {0}")]
    Dialoguer(#[from] dialoguer::Error),

    /// Validation error.
    #[error("Validation error: {0}")]
    ValidationError(String),
}

/// Helper to check if a dialoguer error is a cancellation (Ctrl+C / Escape)
fn is_cancelled(err: &dialoguer::Error) -> bool {
    matches!(err, dialoguer::Error::IO(io_err) if io_err.kind() == std::io::ErrorKind::Interrupted)
}

/// Dialoguer backend for interactive CLI prompts.
///
/// This backend uses the `dialoguer` library to present questions
/// to the user in a command-line interface with colorful themes.
#[derive(Debug, Default, Clone)]
pub struct DialoguerBackend {
    /// Use colorful theme for prompts.
    colorful: bool,
}

impl DialoguerBackend {
    /// Create a new Dialoguer backend with default (colorful) theme.
    pub fn new() -> Self {
        Self { colorful: true }
    }

    /// Create a backend with plain (no color) theme.
    pub fn plain() -> Self {
        Self { colorful: false }
    }

    /// Ask a single question and store the response.
    fn ask_question(
        &self,
        question: &Question,
        responses: &mut Responses,
        validate: &dyn Fn(&ResponseValue, &Responses, &ResponsePath) -> Result<(), String>,
        path_prefix: Option<&ResponsePath>,
    ) -> Result<(), DialoguerError> {
        let path = match path_prefix {
            Some(prefix) => prefix.child(question.path().as_str()),
            None => question.path().clone(),
        };

        // Use the question's prompt, or fall back to a title-cased version of the path
        let prompt = if question.ask().is_empty() {
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
            QuestionKind::Unit => Ok(()),

            QuestionKind::Input(input_q) => self.ask_input(
                &path,
                &prompt,
                input_q,
                question.default(),
                responses,
                validate,
            ),

            QuestionKind::Multiline(_multiline_q) => {
                self.ask_multiline(&path, &prompt, question.default(), responses, validate)
            }

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

            QuestionKind::List(list_q) => self.ask_list(
                &path,
                &prompt,
                list_q,
                question.default(),
                responses,
                validate,
            ),

            QuestionKind::OneOf(one_of) => {
                self.ask_one_of(&path, &prompt, one_of, responses, validate)
            }

            QuestionKind::AnyOf(any_of) => {
                self.ask_any_of(&path, &prompt, any_of, responses, validate)
            }

            QuestionKind::AllOf(all_of) => {
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
        validate: &dyn Fn(&ResponseValue, &Responses, &ResponsePath) -> Result<(), String>,
    ) -> Result<(), DialoguerError> {
        loop {
            let mut _theme;
            let mut builder: Input<String>;
            if self.colorful {
                _theme = ColorfulTheme::default();
                builder = Input::with_theme(&_theme);
            } else {
                builder = Input::new();
            }

            builder = builder.with_prompt(prompt).allow_empty(false);

            // Apply default value
            if let Some(default_val) = default.value() {
                if let ResponseValue::String(s) = default_val {
                    builder = builder.default(s.clone());
                }
            } else if let Some(ref def) = input_q.default {
                builder = builder.default(def.clone());
            }

            let result = builder.interact_text();

            match result {
                Ok(value) => {
                    let rv = ResponseValue::String(value.clone());
                    if let Err(msg) = validate(&rv, responses, path) {
                        println!("Error: {msg}");
                        continue;
                    }
                    responses.insert(path.clone(), rv);
                    return Ok(());
                }
                Err(e) if is_cancelled(&e) => {
                    return Err(DialoguerError::Cancelled);
                }
                Err(e) => return Err(DialoguerError::Dialoguer(e)),
            }
        }
    }

    fn ask_multiline(
        &self,
        path: &ResponsePath,
        prompt: &str,
        default: &DefaultValue,
        responses: &mut Responses,
        validate: &dyn Fn(&ResponseValue, &Responses, &ResponsePath) -> Result<(), String>,
    ) -> Result<(), DialoguerError> {
        loop {
            println!("{prompt}");

            let editor = Editor::new();

            // Get default text if available
            let default_text = if let Some(default_val) = default.value() {
                if let ResponseValue::String(s) = default_val {
                    s.as_str()
                } else {
                    ""
                }
            } else {
                ""
            };

            let result = editor.edit(default_text);

            match result {
                Ok(Some(value)) => {
                    let rv = ResponseValue::String(value.clone());
                    if let Err(msg) = validate(&rv, responses, path) {
                        println!("Error: {msg}");
                        continue;
                    }
                    responses.insert(path.clone(), rv);
                    return Ok(());
                }
                Ok(None) => {
                    // Editor was aborted or empty, use empty string
                    let rv = ResponseValue::String(String::new());
                    if let Err(msg) = validate(&rv, responses, path) {
                        println!("Error: {msg}");
                        continue;
                    }
                    responses.insert(path.clone(), rv);
                    return Ok(());
                }
                Err(e) if is_cancelled(&e) => {
                    return Err(DialoguerError::Cancelled);
                }
                Err(e) => return Err(DialoguerError::Dialoguer(e)),
            }
        }
    }

    fn ask_masked(
        &self,
        path: &ResponsePath,
        prompt: &str,
        _masked_q: &derive_survey::MaskedQuestion,
        _default: &DefaultValue, // Passwords don't have visible defaults
        responses: &mut Responses,
        validate: &dyn Fn(&ResponseValue, &Responses, &ResponsePath) -> Result<(), String>,
    ) -> Result<(), DialoguerError> {
        loop {
            let mut _theme;
            let mut builder: Password;
            if self.colorful {
                _theme = ColorfulTheme::default();
                builder = Password::with_theme(&_theme);
            } else {
                builder = Password::new();
            }

            builder = builder.with_prompt(prompt);

            let result = builder.interact();

            match result {
                Ok(value) => {
                    let rv = ResponseValue::String(value.clone());
                    if let Err(msg) = validate(&rv, responses, path) {
                        println!("Error: {msg}");
                        continue;
                    }
                    responses.insert(path.clone(), rv);
                    return Ok(());
                }
                Err(e) if is_cancelled(&e) => {
                    return Err(DialoguerError::Cancelled);
                }
                Err(e) => return Err(DialoguerError::Dialoguer(e)),
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
        validate: &dyn Fn(&ResponseValue, &Responses, &ResponsePath) -> Result<(), String>,
    ) -> Result<(), DialoguerError> {
        loop {
            let mut _theme;
            let mut builder: Input<i64>;
            if self.colorful {
                _theme = ColorfulTheme::default();
                builder = Input::with_theme(&_theme);
            } else {
                builder = Input::new();
            }

            builder = builder.with_prompt(prompt);

            // Apply default value
            if let Some(default_val) = default.value() {
                if let ResponseValue::Int(i) = default_val {
                    builder = builder.default(*i);
                }
            } else if let Some(def) = int_q.default {
                builder = builder.default(def);
            }

            let result = builder.interact_text();

            match result {
                Ok(value) => {
                    // Check bounds
                    if let Some(min) = int_q.min {
                        if value < min {
                            println!("Error: Value must be at least {min}");
                            continue;
                        }
                    }
                    if let Some(max) = int_q.max {
                        if value > max {
                            println!("Error: Value must be at most {max}");
                            continue;
                        }
                    }

                    let rv = ResponseValue::Int(value);
                    if let Err(msg) = validate(&rv, responses, path) {
                        println!("Error: {msg}");
                        continue;
                    }
                    responses.insert(path.clone(), rv);
                    return Ok(());
                }
                Err(e) if is_cancelled(&e) => {
                    return Err(DialoguerError::Cancelled);
                }
                Err(e) => return Err(DialoguerError::Dialoguer(e)),
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
        validate: &dyn Fn(&ResponseValue, &Responses, &ResponsePath) -> Result<(), String>,
    ) -> Result<(), DialoguerError> {
        loop {
            let mut _theme;
            let mut builder: Input<f64>;
            if self.colorful {
                _theme = ColorfulTheme::default();
                builder = Input::with_theme(&_theme);
            } else {
                builder = Input::new();
            }

            builder = builder.with_prompt(prompt);

            // Apply default value
            if let Some(default_val) = default.value() {
                if let ResponseValue::Float(f) = default_val {
                    builder = builder.default(*f);
                }
            } else if let Some(def) = float_q.default {
                builder = builder.default(def);
            }

            let result = builder.interact_text();

            match result {
                Ok(value) => {
                    // Check bounds
                    if let Some(min) = float_q.min {
                        if value < min {
                            println!("Error: Value must be at least {min}");
                            continue;
                        }
                    }
                    if let Some(max) = float_q.max {
                        if value > max {
                            println!("Error: Value must be at most {max}");
                            continue;
                        }
                    }

                    let rv = ResponseValue::Float(value);
                    if let Err(msg) = validate(&rv, responses, path) {
                        println!("Error: {msg}");
                        continue;
                    }
                    responses.insert(path.clone(), rv);
                    return Ok(());
                }
                Err(e) if is_cancelled(&e) => {
                    return Err(DialoguerError::Cancelled);
                }
                Err(e) => return Err(DialoguerError::Dialoguer(e)),
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
    ) -> Result<(), DialoguerError> {
        let default_val = if let Some(ResponseValue::Bool(b)) = default.value() {
            *b
        } else {
            confirm_q.default
        };

        let mut builder: Confirm;
        let _theme;
        if self.colorful {
            _theme = ColorfulTheme::default();
            builder = Confirm::with_theme(&_theme);
        } else {
            builder = Confirm::new();
        }

        builder = builder.with_prompt(prompt).default(default_val);

        let result = builder.interact();

        match result {
            Ok(value) => {
                responses.insert(path.clone(), ResponseValue::Bool(value));
                Ok(())
            }
            Err(e) if is_cancelled(&e) => Err(DialoguerError::Cancelled),
            Err(e) => Err(DialoguerError::Dialoguer(e)),
        }
    }

    fn ask_list(
        &self,
        path: &ResponsePath,
        prompt: &str,
        list_q: &derive_survey::ListQuestion,
        _default: &DefaultValue,
        responses: &mut Responses,
        validate: &dyn Fn(&ResponseValue, &Responses, &ResponsePath) -> Result<(), String>,
    ) -> Result<(), DialoguerError> {
        let mut items: Vec<ResponseValue> = Vec::new();

        println!("{}", prompt);
        println!("  (Enter values one per line, empty line to finish)");

        loop {
            let item_prompt = format!("  [{}]", items.len() + 1);

            let value = match &list_q.element_kind {
                ListElementKind::String => {
                    let mut _theme;
                    let mut builder: Input<String>;
                    if self.colorful {
                        _theme = ColorfulTheme::default();
                        builder = Input::with_theme(&_theme);
                    } else {
                        builder = Input::new();
                    }

                    builder = builder.with_prompt(&item_prompt).allow_empty(true);

                    match builder.interact_text() {
                        Ok(s) if s.is_empty() => break,
                        Ok(s) => Some(ResponseValue::String(s)),
                        Err(e) if is_cancelled(&e) => return Err(DialoguerError::Cancelled),
                        Err(e) => return Err(DialoguerError::Dialoguer(e)),
                    }
                }
                ListElementKind::Int { min, max } => {
                    let mut _theme;
                    let mut builder: Input<String>;
                    if self.colorful {
                        _theme = ColorfulTheme::default();
                        builder = Input::with_theme(&_theme);
                    } else {
                        builder = Input::new();
                    }

                    builder = builder.with_prompt(&item_prompt).allow_empty(true);

                    match builder.interact_text() {
                        Ok(s) if s.is_empty() => break,
                        Ok(s) => match s.parse::<i64>() {
                            Ok(n) => {
                                if let Some(min_val) = min {
                                    if n < *min_val {
                                        println!("    Error: Value must be at least {min_val}");
                                        continue;
                                    }
                                }
                                if let Some(max_val) = max {
                                    if n > *max_val {
                                        println!("    Error: Value must be at most {max_val}");
                                        continue;
                                    }
                                }
                                Some(ResponseValue::Int(n))
                            }
                            Err(_) => {
                                println!("    Error: Please enter a valid integer");
                                continue;
                            }
                        },
                        Err(e) if is_cancelled(&e) => return Err(DialoguerError::Cancelled),
                        Err(e) => return Err(DialoguerError::Dialoguer(e)),
                    }
                }
                ListElementKind::Float { min, max } => {
                    let mut _theme;
                    let mut builder: Input<String>;
                    if self.colorful {
                        _theme = ColorfulTheme::default();
                        builder = Input::with_theme(&_theme);
                    } else {
                        builder = Input::new();
                    }

                    builder = builder.with_prompt(&item_prompt).allow_empty(true);

                    match builder.interact_text() {
                        Ok(s) if s.is_empty() => break,
                        Ok(s) => match s.parse::<f64>() {
                            Ok(n) => {
                                if let Some(min_val) = min {
                                    if n < *min_val {
                                        println!("    Error: Value must be at least {min_val}");
                                        continue;
                                    }
                                }
                                if let Some(max_val) = max {
                                    if n > *max_val {
                                        println!("    Error: Value must be at most {max_val}");
                                        continue;
                                    }
                                }
                                Some(ResponseValue::Float(n))
                            }
                            Err(_) => {
                                println!("    Error: Please enter a valid number");
                                continue;
                            }
                        },
                        Err(e) if is_cancelled(&e) => return Err(DialoguerError::Cancelled),
                        Err(e) => return Err(DialoguerError::Dialoguer(e)),
                    }
                }
            };

            if let Some(v) = value {
                items.push(v);
            }
        }

        // Convert to the appropriate list type
        let rv = match &list_q.element_kind {
            ListElementKind::String => {
                let strings: Vec<String> = items
                    .into_iter()
                    .filter_map(|v| {
                        if let ResponseValue::String(s) = v {
                            Some(s)
                        } else {
                            None
                        }
                    })
                    .collect();
                ResponseValue::StringList(strings)
            }
            ListElementKind::Int { .. } => {
                let ints: Vec<i64> = items
                    .into_iter()
                    .filter_map(|v| {
                        if let ResponseValue::Int(n) = v {
                            Some(n)
                        } else {
                            None
                        }
                    })
                    .collect();
                ResponseValue::IntList(ints)
            }
            ListElementKind::Float { .. } => {
                let floats: Vec<f64> = items
                    .into_iter()
                    .filter_map(|v| {
                        if let ResponseValue::Float(n) = v {
                            Some(n)
                        } else {
                            None
                        }
                    })
                    .collect();
                ResponseValue::FloatList(floats)
            }
        };

        // Validate the entire list
        if let Err(msg) = validate(&rv, responses, path) {
            println!("Error: {msg}");
            // For now, just return the error - in a real implementation we might loop
            return Err(DialoguerError::ValidationError(msg));
        }

        responses.insert(path.clone(), rv);
        Ok(())
    }

    fn ask_one_of(
        &self,
        path: &ResponsePath,
        prompt: &str,
        one_of: &derive_survey::OneOfQuestion,
        responses: &mut Responses,
        validate: &dyn Fn(&ResponseValue, &Responses, &ResponsePath) -> Result<(), String>,
    ) -> Result<(), DialoguerError> {
        let items: Vec<&str> = one_of.variants.iter().map(|v| v.name.as_str()).collect();

        let mut builder: Select;
        let _theme;
        if self.colorful {
            _theme = ColorfulTheme::default();
            builder = Select::with_theme(&_theme);
        } else {
            builder = Select::new();
        }

        builder = builder.with_prompt(prompt).items(&items);

        if let Some(default_idx) = one_of.default {
            builder = builder.default(default_idx);
        }

        let result = builder.interact();

        let selection = match result {
            Ok(idx) => idx,
            Err(e) if is_cancelled(&e) => {
                return Err(DialoguerError::Cancelled);
            }
            Err(e) => return Err(DialoguerError::Dialoguer(e)),
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
            | QuestionKind::Multiline(_)
            | QuestionKind::List(_) => {
                let variant_q = Question::new(
                    selected_variant.name.clone(),
                    format!("Enter {} value:", selected_variant.name),
                    selected_variant.kind.clone(),
                );
                self.ask_question(&variant_q, responses, validate, Some(path))?;
            }
            QuestionKind::OneOf(nested_one_of) => {
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
        validate: &dyn Fn(&ResponseValue, &Responses, &ResponsePath) -> Result<(), String>,
    ) -> Result<(), DialoguerError> {
        let selections = loop {
            let items: Vec<&str> = any_of.variants.iter().map(|v| v.name.as_str()).collect();

            // Build defaults array
            let defaults: Vec<bool> = (0..any_of.variants.len())
                .map(|i| any_of.defaults.contains(&i))
                .collect();

            let mut builder: MultiSelect;
            let _theme;
            if self.colorful {
                _theme = ColorfulTheme::default();
                builder = MultiSelect::with_theme(&_theme);
            } else {
                builder = MultiSelect::new();
            }

            builder = builder
                .with_prompt(prompt)
                .items(&items)
                .defaults(&defaults);

            let result = builder.interact();

            let selections = match result {
                Ok(indices) => indices,
                Err(e) if is_cancelled(&e) => {
                    return Err(DialoguerError::Cancelled);
                }
                Err(e) => return Err(DialoguerError::Dialoguer(e)),
            };

            // Validate the selection
            let selection_value = ResponseValue::ChosenVariants(selections.clone());
            if let Err(msg) = validate(&selection_value, responses, path) {
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

impl SurveyBackend for DialoguerBackend {
    type Error = DialoguerError;

    fn collect(
        &self,
        definition: &SurveyDefinition,
        validate: &dyn Fn(&ResponseValue, &Responses, &ResponsePath) -> Result<(), String>,
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
    fn backend_creation() {
        let _backend = DialoguerBackend::new();
        let _plain = DialoguerBackend::plain();
    }

    #[test]
    fn error_types() {
        let err = DialoguerError::Cancelled;
        assert_eq!(err.to_string(), "Survey cancelled by user");

        let err = DialoguerError::ValidationError("test error".to_string());
        assert_eq!(err.to_string(), "Validation error: test error");
    }
}
