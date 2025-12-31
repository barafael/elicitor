#![doc = include_str!("../README.md")]

pub mod answer;
pub mod backend;

pub use answer::{AnswerError, AnswerValue, Answers};
pub use backend::{BackendError, InterviewBackend, TestBackend};
pub use derive_wizard_macro::*;
pub use derive_wizard_types::interview;

#[cfg(feature = "requestty-backend")]
pub use backend::requestty_backend::RequesttyBackend;

#[cfg(feature = "dialoguer-backend")]
pub use backend::dialoguer_backend::DialoguerBackend;

#[cfg(feature = "egui-backend")]
pub use backend::egui_backend::EguiBackend;

pub trait Wizard: Sized {
    /// Get the interview structure for this type
    fn interview() -> interview::Interview;

    /// Get the interview structure with suggested values from this instance
    fn interview_with_suggestions(&self) -> interview::Interview;

    /// Build this type from collected answers
    fn from_answers(answers: &Answers) -> Result<Self, BackendError>;

    /// Create a builder for this wizard
    fn wizard_builder() -> WizardBuilder<Self> {
        WizardBuilder::new()
    }
}

/// Builder for configuring and executing a wizard
#[derive(Default)]
pub struct WizardBuilder<T: Wizard> {
    suggestions: Option<T>,
    backend: Option<Box<dyn InterviewBackend>>,
}

impl<T: Wizard> WizardBuilder<T> {
    /// Create a new wizard builder
    pub fn new() -> Self {
        Self {
            suggestions: None,
            backend: None,
        }
    }

    /// Set suggested values for the wizard
    pub fn with_suggestions(mut self, suggestions: T) -> Self {
        self.suggestions = Some(suggestions);
        self
    }

    /// Set a custom backend
    pub fn with_backend<B: InterviewBackend + 'static>(mut self, backend: B) -> Self {
        self.backend = Some(Box::new(backend));
        self
    }

    /// Execute the wizard and return the result
    #[cfg(feature = "requestty-backend")]
    pub fn build(self) -> T {
        use crate::backend::requestty_backend::RequesttyBackend;

        let backend = self.backend.unwrap_or_else(|| Box::new(RequesttyBackend));

        let interview = self
            .suggestions
            .as_ref()
            .map_or_else(T::interview, |suggestions| {
                suggestions.interview_with_suggestions()
            });

        let answers = backend
            .execute(&interview)
            .expect("Failed to execute interview");
        T::from_answers(&answers).expect("Failed to build from answers")
    }

    /// Execute the wizard and return the result (no default backend required)
    #[cfg(not(feature = "requestty-backend"))]
    pub fn build(self) -> T {
        let backend = self
            .backend
            .expect("No backend specified and requestty-backend feature is not enabled");

        let interview = if let Some(ref suggestions) = self.suggestions {
            suggestions.interview_with_suggestions()
        } else {
            T::interview()
        };

        let answers = backend
            .execute(&interview)
            .expect("Failed to execute interview");
        T::from_answers(&answers).expect("Failed to build from answers")
    }
}
