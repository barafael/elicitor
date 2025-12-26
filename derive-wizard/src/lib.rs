#![doc = include_str!("../../README.md")]

pub mod backend;

#[cfg(feature = "egui-backend")]
pub mod egui_backend;

pub use backend::{AnswerValue, Answers, BackendError, InterviewBackend, TestBackend};
pub use derive_wizard_macro::*;
pub use derive_wizard_types::{interview, question};
pub use requestty::{ExpandItem, ListItem, Question, prompt_one};

#[cfg(feature = "egui-backend")]
pub use egui_backend::EguiBackend;

pub trait Wizard: Sized {
    /// Get the interview structure for this type
    fn interview() -> interview::Interview;

    /// Build this type from collected answers
    fn from_answers(answers: &Answers) -> Result<Self, BackendError>;

    /// Execute the interview with the default backend
    fn wizard() -> Self {
        Self::wizard_with_backend(&DefaultBackend)
    }

    /// Execute the interview with a custom backend
    fn wizard_with_backend<B: InterviewBackend>(backend: &B) -> Self {
        let interview = Self::interview();
        let answers = backend
            .execute(&interview)
            .expect("Failed to execute interview");
        Self::from_answers(&answers).expect("Failed to build from answers")
    }

    fn wizard_with_message(message: &str) -> Self {
        let _ = message;
        Self::wizard()
    }

    fn wizard_with_defaults(self) -> Self {
        self
    }
}

/// Default backend using requestty for interactive CLI prompts
pub struct DefaultBackend;

impl DefaultBackend {
    fn execute_section(
        &self,
        section: &interview::Section,
        answers: &mut Answers,
    ) -> Result<(), BackendError> {
        use interview::Section;

        match section {
            Section::Empty => Ok(()),
            Section::Sequence(seq) => {
                for question in &seq.sequence {
                    self.execute_question(question, answers)?;
                }
                Ok(())
            }
            Section::Alternatives(default_idx, alternatives) => {
                // Build the select question
                let choices: Vec<String> =
                    alternatives.iter().map(|alt| alt.name.clone()).collect();

                let question = requestty::Question::select("choice")
                    .message("Select an option")
                    .choices(choices.clone())
                    .default(*default_idx)
                    .build();

                let answer = requestty::prompt_one(question).map_err(|e| {
                    BackendError::ExecutionError(format!("Failed to prompt: {}", e))
                })?;

                let selected_idx = match answer {
                    requestty::Answer::ListItem(item) => item.index,
                    _ => return Err(BackendError::ExecutionError("Expected list item".into())),
                };

                // Store the selected alternative name
                answers.insert(
                    "selected_alternative".to_string(),
                    AnswerValue::String(choices[selected_idx].clone()),
                );

                // Execute the follow-up section for the selected alternative
                self.execute_section(&alternatives[selected_idx].section, answers)?;
                Ok(())
            }
        }
    }

    fn execute_question(
        &self,
        question: &question::Question,
        answers: &mut Answers,
    ) -> Result<(), BackendError> {
        use question::QuestionKind;

        let id = question.id().unwrap_or(question.name());

        match question.kind() {
            QuestionKind::Input(input_q) => {
                let mut q = requestty::Question::input(id).message(question.prompt());

                if let Some(default) = &input_q.default {
                    q = q.default(default.clone());
                }

                let answer = requestty::prompt_one(q.build()).map_err(|e| {
                    BackendError::ExecutionError(format!("Failed to prompt: {}", e))
                })?;

                if let requestty::Answer::String(s) = answer {
                    answers.insert(id.to_string(), AnswerValue::String(s));
                }
            }
            QuestionKind::Multiline(multiline_q) => {
                let mut q = requestty::Question::editor(id).message(question.prompt());

                if let Some(default) = &multiline_q.default {
                    q = q.default(default.clone());
                }

                let answer = requestty::prompt_one(q.build()).map_err(|e| {
                    BackendError::ExecutionError(format!("Failed to prompt: {}", e))
                })?;

                if let requestty::Answer::String(s) = answer {
                    answers.insert(id.to_string(), AnswerValue::String(s));
                }
            }
            QuestionKind::Masked(masked_q) => {
                let mut q = requestty::Question::password(id).message(question.prompt());

                if let Some(mask) = masked_q.mask {
                    q = q.mask(mask);
                }

                let answer = requestty::prompt_one(q.build()).map_err(|e| {
                    BackendError::ExecutionError(format!("Failed to prompt: {}", e))
                })?;

                if let requestty::Answer::String(s) = answer {
                    answers.insert(id.to_string(), AnswerValue::String(s));
                }
            }
            QuestionKind::Int(int_q) => {
                let mut q = requestty::Question::int(id).message(question.prompt());

                if let Some(default) = int_q.default {
                    q = q.default(default);
                }

                // Add validation for min/max
                let min = int_q.min;
                let max = int_q.max;
                if min.is_some() || max.is_some() {
                    q = q.validate(move |value, _| {
                        if let Some(min_val) = min
                            && value < min_val
                        {
                            return Err(format!("Value must be at least {}", min_val));
                        }
                        if let Some(max_val) = max
                            && value > max_val
                        {
                            return Err(format!("Value must be at most {}", max_val));
                        }
                        Ok(())
                    });
                }

                let answer = requestty::prompt_one(q.build()).map_err(|e| {
                    BackendError::ExecutionError(format!("Failed to prompt: {}", e))
                })?;

                if let requestty::Answer::Int(i) = answer {
                    answers.insert(id.to_string(), AnswerValue::Int(i));
                }
            }
            QuestionKind::Float(float_q) => {
                let mut q = requestty::Question::float(id).message(question.prompt());

                if let Some(default) = float_q.default {
                    q = q.default(default);
                }

                // Add validation for min/max
                let min = float_q.min;
                let max = float_q.max;
                if min.is_some() || max.is_some() {
                    q = q.validate(move |value, _| {
                        if let Some(min_val) = min
                            && value < min_val
                        {
                            return Err(format!("Value must be at least {}", min_val));
                        }
                        if let Some(max_val) = max
                            && value > max_val
                        {
                            return Err(format!("Value must be at most {}", max_val));
                        }
                        Ok(())
                    });
                }

                let answer = requestty::prompt_one(q.build()).map_err(|e| {
                    BackendError::ExecutionError(format!("Failed to prompt: {}", e))
                })?;

                if let requestty::Answer::Float(f) = answer {
                    answers.insert(id.to_string(), AnswerValue::Float(f));
                }
            }
            QuestionKind::Confirm(confirm_q) => {
                let q = requestty::Question::confirm(id)
                    .message(question.prompt())
                    .default(confirm_q.default)
                    .build();

                let answer = requestty::prompt_one(q).map_err(|e| {
                    BackendError::ExecutionError(format!("Failed to prompt: {}", e))
                })?;

                if let requestty::Answer::Bool(b) = answer {
                    answers.insert(id.to_string(), AnswerValue::Bool(b));
                }
            }
            QuestionKind::Nested(_) => {
                // Nested questions should have been inlined by the macro
                return Err(BackendError::ExecutionError(
                    "Nested questions should be inlined".into(),
                ));
            }
        }

        Ok(())
    }
}

impl InterviewBackend for DefaultBackend {
    fn execute(&self, interview: &interview::Interview) -> Result<Answers, BackendError> {
        let mut answers = Answers::new();

        for section in &interview.sections {
            self.execute_section(section, &mut answers)?;
        }

        Ok(answers)
    }
}
