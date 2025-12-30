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

    /// Get the interview structure with default values from this instance
    fn interview_with_defaults(&self) -> interview::Interview;

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
    defaults: Option<T>,
    backend: Option<Box<dyn InterviewBackend>>,
}

impl<T: Wizard> WizardBuilder<T> {
    /// Create a new wizard builder
    pub fn new() -> Self {
        Self {
            defaults: None,
            backend: None,
        }
    }

    /// Set default values for the wizard
    pub fn with_defaults(mut self, defaults: T) -> Self {
        self.defaults = Some(defaults);
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

        let interview = if let Some(ref defaults) = self.defaults {
            defaults.interview_with_defaults()
        } else {
            T::interview()
        };

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

        let interview = if let Some(ref defaults) = self.defaults {
            defaults.interview_with_defaults()
        } else {
            T::interview()
        };

        let answers = backend
            .execute(&interview)
            .expect("Failed to execute interview");
        T::from_answers(&answers).expect("Failed to build from answers")
    }
}
