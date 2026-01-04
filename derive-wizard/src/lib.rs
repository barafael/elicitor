#![doc = include_str!("../README.md")]

// Require at least one interactive backend feature. This forces users who disable
// the default `requestty-backend` to opt into another backend instead of getting
// a runtime error later.
#[cfg(all(
    not(feature = "requestty-backend"),
    not(feature = "egui-backend"),
    not(feature = "dialoguer-backend"),
    not(feature = "ratatui-backend"),
))]
compile_error!("derive-wizard requires a backend feature. Enable one backend feature.");

pub mod answer;
pub mod backend;
pub mod field_path;

#[cfg(feature = "typst-form")]
pub mod typst_form;

pub use answer::{AnswerError, AnswerValue, Answers};
pub use backend::{BackendError, InterviewBackend, TestBackend};
pub use derive_wizard_macro::*;
pub use derive_wizard_types::SELECTED_ALTERNATIVE_KEY;
pub use derive_wizard_types::interview;
pub use field_path::FieldPath;

#[cfg(feature = "requestty-backend")]
pub use backend::requestty_backend::RequesttyBackend;

#[cfg(feature = "dialoguer-backend")]
pub use backend::dialoguer_backend::DialoguerBackend;

#[cfg(feature = "egui-backend")]
pub use backend::egui_backend::EguiBackend;

#[cfg(feature = "ratatui-backend")]
pub use backend::ratatui_backend::{RatatuiBackend, Theme as RatatuiTheme};

#[cfg(feature = "ratatui-backend")]
pub use ratatui::style::Color as RatatuiColor;

pub trait Wizard: Sized {
    /// Get the interview structure for this type
    fn interview() -> interview::Interview;

    /// Get the interview structure with suggested values from this instance
    fn interview_with_suggestions(&self) -> interview::Interview;

    /// Build this type from collected answers
    fn from_answers(answers: &Answers) -> Result<Self, BackendError>;

    /// Validate a field value
    /// This is called by backends during execution to validate user input
    fn validate_field(field: &str, value: &str, answers: &Answers) -> Result<(), String>;

    /// Create a builder for this wizard
    fn wizard_builder() -> WizardBuilder<Self> {
        WizardBuilder::new()
    }

    /// Generate a Typst form (.typ file) from the interview structure
    ///
    /// This method is only available when the `typst-form` feature is enabled.
    /// It returns a String containing valid Typst markup that can be saved as a .typ file
    /// and compiled to a professional-looking PDF form.
    ///
    /// Note: Typst does not currently support interactive fillable PDF forms. The generated
    /// PDF is a static, print-ready form that can be filled out by hand or with PDF annotation tools.
    ///
    /// # Example
    /// ```ignore
    /// use derive_wizard::Wizard;
    ///
    /// #[derive(Wizard)]
    /// struct Person {
    ///     name: String,
    ///     age: i64,
    /// }
    ///
    /// let typst_markup = Person::to_typst_form(Some("Registration Form"));
    /// std::fs::write("form.typ", &typst_markup).unwrap();
    ///
    /// // Compile to PDF using: typst compile form.typ
    /// ```
    #[cfg(feature = "typst-form")]
    fn to_typst_form(title: Option<&str>) -> String {
        let interview = Self::interview();
        crate::typst_form::generate_typst_form(&interview, title)
    }
}

/// Helper function to recursively find a question by field path.
///
/// This searches through the interview hierarchy, navigating into
/// nested `QuestionKind::Sequence` to find the target question.
/// Questions can be named either hierarchically (separate questions)
/// or with dot-separated paths (e.g., "address.street").
fn find_question_by_path<'a>(
    questions: &'a mut [interview::Question],
    path: &FieldPath,
) -> Option<&'a mut interview::Question> {
    let segments = path.segments();

    if segments.is_empty() {
        return None;
    }

    // Try to find by full dot-separated path first (flat question naming)
    let full_path = path.to_path();

    // Check if any question matches the full path
    let full_path_idx = questions.iter().position(|q| q.name() == full_path);
    if let Some(idx) = full_path_idx {
        return Some(&mut questions[idx]);
    }

    // If it's a single segment, search at this level
    if segments.len() == 1 {
        let idx = questions.iter().position(|q| q.name() == segments[0])?;
        return Some(&mut questions[idx]);
    }

    // Multi-segment path: find the first segment at this level
    let first = &segments[0];
    let rest = FieldPath::new(segments[1..].to_vec());

    for question in questions.iter_mut() {
        // If this question's name matches the first segment
        if question.name() == first {
            // Check if it's a Sequence kind (nested struct)
            if let interview::QuestionKind::Sequence(nested_questions) = question.kind_mut() {
                // Recursively search in the nested questions
                return find_question_by_path(nested_questions, &rest);
            }
        }
    }

    None
}

/// Builder for configuring and executing a wizard
#[derive(Default)]
pub struct WizardBuilder<T: Wizard> {
    suggestions: Option<T>,
    partial_suggestions: std::collections::HashMap<
        FieldPath,
        derive_wizard_types::suggested_answer::SuggestedAnswer,
    >,
    partial_assumptions: std::collections::HashMap<FieldPath, derive_wizard_types::AssumedAnswer>,
    backend: Option<Box<dyn InterviewBackend>>,
}

impl<T: Wizard> WizardBuilder<T> {
    /// Create a new wizard builder
    pub fn new() -> Self {
        Self {
            suggestions: None,
            partial_suggestions: std::collections::HashMap::new(),
            partial_assumptions: std::collections::HashMap::new(),
            backend: None,
        }
    }

    /// Set suggested values for the wizard
    pub fn with_suggestions(mut self, suggestions: T) -> Self {
        self.suggestions = Some(suggestions);
        self
    }

    /// Suggest a specific field value. The question will still be asked but with a pre-filled default.
    ///
    /// For nested fields, use the `field!` macro.
    pub fn suggest_field(
        mut self,
        field: impl Into<FieldPath>,
        value: impl Into<derive_wizard_types::suggested_answer::SuggestedAnswer>,
    ) -> Self {
        self.partial_suggestions.insert(field.into(), value.into());
        self
    }

    /// Assume a specific field value. The question for this field will be skipped.
    ///
    /// For nested fields, use the `field!` macro.
    pub fn assume_field(
        mut self,
        field: impl Into<FieldPath>,
        value: impl Into<derive_wizard_types::AssumedAnswer>,
    ) -> Self {
        self.partial_assumptions.insert(field.into(), value.into());
        self
    }

    /// Set a custom backend
    pub fn with_backend<B: InterviewBackend + 'static>(mut self, backend: B) -> Self {
        self.backend = Some(Box::new(backend));
        self
    }

    /// Execute the wizard and return the result
    #[cfg(feature = "requestty-backend")]
    pub fn build(self) -> Result<T, BackendError> {
        use crate::backend::requestty_backend::RequesttyBackend;

        let backend = self.backend.unwrap_or_else(|| Box::new(RequesttyBackend));

        let mut interview = match &self.suggestions {
            Some(suggestions) => suggestions.interview_with_suggestions(),
            None => T::interview(),
        };

        // Apply partial suggestions
        for (field_path, value) in self.partial_suggestions {
            if let Some(question) = find_question_by_path(&mut interview.sections, &field_path) {
                question.set_suggestion(value);
            }
        }

        // Apply partial assumptions
        for (field_path, value) in self.partial_assumptions {
            if let Some(question) = find_question_by_path(&mut interview.sections, &field_path) {
                question.set_assumption(value);
            }
        }

        let answers = backend.execute_with_validator(&interview, &T::validate_field)?;
        T::from_answers(&answers)
    }

    /// Execute the wizard and return the result (no default backend required)
    #[cfg(not(feature = "requestty-backend"))]
    pub fn build(self) -> Result<T, BackendError> {
        let backend = match self.backend {
            Some(backend) => backend,
            None => {
                return Err(BackendError::Custom(
                    "No backend specified and requestty-backend feature is not enabled".to_string(),
                ));
            }
        };

        let mut interview = match &self.suggestions {
            Some(suggestions) => suggestions.interview_with_suggestions(),
            None => T::interview(),
        };

        // Apply partial suggestions
        for (field_path, value) in self.partial_suggestions {
            if let Some(question) = find_question_by_path(&mut interview.sections, &field_path) {
                question.set_suggestion(value);
            }
        }

        // Apply partial assumptions
        for (field_path, value) in self.partial_assumptions {
            if let Some(question) = find_question_by_path(&mut interview.sections, &field_path) {
                question.set_assumption(value);
            }
        }

        let answers = backend.execute_with_validator(&interview, &T::validate_field)?;
        T::from_answers(&answers)
    }
}
