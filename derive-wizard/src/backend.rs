use crate::{AnswerError, AnswerValue, Answers, interview::Interview};

#[cfg(feature = "requestty-backend")]
pub mod requestty_backend;

#[cfg(feature = "dialoguer-backend")]
pub mod dialoguer_backend;

#[cfg(feature = "egui-backend")]
pub mod egui_backend;

#[derive(Debug, thiserror::Error)]
pub enum BackendError {
    #[error("Answer error: {0}")]
    Answer(#[from] AnswerError),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Execution error: {0}")]
    ExecutionError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Backend-specific error: {0}")]
    Custom(String),
}

/// Trait for interview execution backends
pub trait InterviewBackend {
    /// Execute an interview and return the collected answers
    fn execute(&self, interview: &Interview) -> Result<Answers, BackendError>;
}

/// Test backend that returns predefined answers
#[derive(Debug, Default)]
pub struct TestBackend {
    answers: Answers,
}

impl TestBackend {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_string(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.answers
            .insert(key.into(), AnswerValue::String(value.into()));
        self
    }

    pub fn with_int(mut self, key: impl Into<String>, value: i64) -> Self {
        self.answers.insert(key.into(), AnswerValue::Int(value));
        self
    }

    pub fn with_float(mut self, key: impl Into<String>, value: f64) -> Self {
        self.answers.insert(key.into(), AnswerValue::Float(value));
        self
    }

    pub fn with_bool(mut self, key: impl Into<String>, value: bool) -> Self {
        self.answers.insert(key.into(), AnswerValue::Bool(value));
        self
    }
}

impl InterviewBackend for TestBackend {
    fn execute(&self, _interview: &Interview) -> Result<Answers, BackendError> {
        // Simply return the predefined answers
        Ok(self.answers.clone())
    }
}
