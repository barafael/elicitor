use std::slice;

use crate::{AnswerError, AnswerValue, Answers, interview::Interview};

#[cfg(feature = "requestty-backend")]
pub mod requestty_backend;

#[cfg(feature = "dialoguer-backend")]
pub mod dialoguer_backend;

#[cfg(feature = "egui-backend")]
pub mod egui_backend;

#[cfg(feature = "ratatui-backend")]
pub mod ratatui_backend;

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

    /// Execute an interview with validation support
    /// The validator function takes (field_name, value, answers) and returns validation result
    fn execute_with_validator(
        &self,
        interview: &Interview,
        validator: &(dyn Fn(&str, &str, &Answers) -> Result<(), String> + Send + Sync),
    ) -> Result<Answers, BackendError> {
        // Default implementation: just execute without validation
        let _ = validator;
        self.execute(interview)
    }
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
    fn execute(&self, interview: &Interview) -> Result<Answers, BackendError> {
        use crate::interview::{Question, QuestionKind};
        use derive_wizard_types::AssumedAnswer;

        let mut answers = self.answers.clone();

        // Recursively add assumed answers from the interview
        fn collect_assumptions(questions: &[Question], answers: &mut Answers) {
            for question in questions {
                if let Some(assumed) = question.assumed() {
                    let value = match assumed {
                        AssumedAnswer::String(s) => AnswerValue::String(s.clone()),
                        AssumedAnswer::Int(i) => AnswerValue::Int(*i),
                        AssumedAnswer::Float(f) => AnswerValue::Float(*f),
                        AssumedAnswer::Bool(b) => AnswerValue::Bool(*b),
                    };
                    answers.insert(question.name().to_string(), value);
                }

                // Recursively handle nested questions
                match question.kind() {
                    QuestionKind::Sequence(nested_questions) => {
                        collect_assumptions(nested_questions, answers);
                    }
                    QuestionKind::Alternative(_, alternatives) => {
                        for alt in alternatives {
                            let alt = slice::from_ref(alt);
                            collect_assumptions(alt, answers);
                        }
                    }
                    _ => {}
                }
            }
        }

        collect_assumptions(&interview.sections, &mut answers);

        Ok(answers)
    }
}
