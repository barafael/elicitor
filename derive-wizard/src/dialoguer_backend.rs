#[cfg(feature = "dialoguer-backend")]
use crate::backend::{AnswerValue, Answers, BackendError, InterviewBackend};
use crate::interview::{Interview, Section};
use crate::question::{Question, QuestionKind};

/// dialoguer-based interview backend
pub struct DialoguerBackend;

impl DialoguerBackend {
    pub fn new() -> Self {
        Self
    }

    fn execute_section(
        &self,
        section: &Section,
        answers: &mut Answers,
    ) -> Result<(), BackendError> {
        use Section;

        match section {
            Section::Empty => Ok(()),
            Section::Sequence(seq) => {
                for question in &seq.sequence {
                    self.execute_question(question, answers)?;
                }
                Ok(())
            }
            Section::Alternatives(default_idx, alternatives) => {
                let choices: Vec<&str> = alternatives.iter().map(|alt| alt.name.as_str()).collect();

                let selection = dialoguer::Select::new()
                    .with_prompt("Select an option")
                    .items(&choices)
                    .default(*default_idx)
                    .interact()
                    .map_err(|e| {
                        BackendError::ExecutionError(format!("Failed to prompt: {}", e))
                    })?;

                // Store the selected alternative name
                answers.insert(
                    "selected_alternative".to_string(),
                    AnswerValue::String(alternatives[selection].name.clone()),
                );

                // Execute the follow-up section for the selected alternative
                self.execute_section(&alternatives[selection].section, answers)?;
                Ok(())
            }
        }
    }

    fn execute_question(
        &self,
        question: &Question,
        answers: &mut Answers,
    ) -> Result<(), BackendError> {
        let id = question.id().unwrap_or(question.name());

        match question.kind() {
            QuestionKind::Input(input_q) => {
                let mut input = dialoguer::Input::<String>::new().with_prompt(question.prompt());

                if let Some(default) = &input_q.default {
                    input = input.default(default.clone());
                }

                let answer = input.interact_text().map_err(|e| {
                    BackendError::ExecutionError(format!("Failed to prompt: {}", e))
                })?;

                answers.insert(id.to_string(), AnswerValue::String(answer));
            }
            QuestionKind::Multiline(_multiline_q) => {
                // dialoguer doesn't have built-in multiline support, use editor
                let answer = dialoguer::Editor::new()
                    .require_save(true)
                    .edit("")
                    .map_err(|e| {
                        BackendError::ExecutionError(format!("Failed to open editor: {}", e))
                    })?
                    .unwrap_or_default();

                answers.insert(id.to_string(), AnswerValue::String(answer));
            }
            QuestionKind::Masked(_masked_q) => {
                let answer = dialoguer::Password::new()
                    .with_prompt(question.prompt())
                    .interact()
                    .map_err(|e| {
                        BackendError::ExecutionError(format!("Failed to prompt: {}", e))
                    })?;

                answers.insert(id.to_string(), AnswerValue::String(answer));
            }
            QuestionKind::Int(int_q) => {
                let mut input = dialoguer::Input::<i64>::new().with_prompt(question.prompt());

                if let Some(default) = int_q.default {
                    input = input.default(default);
                }

                // Add validation for min/max
                if int_q.min.is_some() || int_q.max.is_some() {
                    let min = int_q.min;
                    let max = int_q.max;
                    input = input.validate_with(move |value: &i64| -> Result<(), String> {
                        if let Some(min_val) = min {
                            if *value < min_val {
                                return Err(format!("Value must be at least {}", min_val));
                            }
                        }
                        if let Some(max_val) = max {
                            if *value > max_val {
                                return Err(format!("Value must be at most {}", max_val));
                            }
                        }
                        Ok(())
                    });
                }

                let answer = input.interact_text().map_err(|e| {
                    BackendError::ExecutionError(format!("Failed to prompt: {}", e))
                })?;

                answers.insert(id.to_string(), AnswerValue::Int(answer));
            }
            QuestionKind::Float(float_q) => {
                let mut input = dialoguer::Input::<f64>::new().with_prompt(question.prompt());

                if let Some(default) = float_q.default {
                    input = input.default(default);
                }

                // Add validation for min/max
                if float_q.min.is_some() || float_q.max.is_some() {
                    let min = float_q.min;
                    let max = float_q.max;
                    input = input.validate_with(move |value: &f64| -> Result<(), String> {
                        if let Some(min_val) = min {
                            if *value < min_val {
                                return Err(format!("Value must be at least {}", min_val));
                            }
                        }
                        if let Some(max_val) = max {
                            if *value > max_val {
                                return Err(format!("Value must be at most {}", max_val));
                            }
                        }
                        Ok(())
                    });
                }

                let answer = input.interact_text().map_err(|e| {
                    BackendError::ExecutionError(format!("Failed to prompt: {}", e))
                })?;

                answers.insert(id.to_string(), AnswerValue::Float(answer));
            }
            QuestionKind::Confirm(confirm_q) => {
                let answer = dialoguer::Confirm::new()
                    .with_prompt(question.prompt())
                    .default(confirm_q.default)
                    .interact()
                    .map_err(|e| {
                        BackendError::ExecutionError(format!("Failed to prompt: {}", e))
                    })?;

                answers.insert(id.to_string(), AnswerValue::Bool(answer));
            }
            QuestionKind::Nested(_) => {
                return Err(BackendError::ExecutionError(
                    "Nested questions should be inlined".into(),
                ));
            }
        }

        Ok(())
    }
}

impl Default for DialoguerBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl InterviewBackend for DialoguerBackend {
    fn execute(&self, interview: &Interview) -> Result<Answers, BackendError> {
        let mut answers = Answers::new();

        for section in &interview.sections {
            self.execute_section(section, &mut answers)?;
        }

        Ok(answers)
    }
}
