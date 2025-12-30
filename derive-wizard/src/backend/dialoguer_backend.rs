#[cfg(feature = "dialoguer-backend")]
use crate::backend::{AnswerValue, Answers, BackendError, InterviewBackend};
use crate::interview::{Interview, Question, QuestionKind};

/// dialoguer-based interview backend
pub struct DialoguerBackend;

impl DialoguerBackend {
    pub fn new() -> Self {
        Self
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

                if let Some(ref default) = input_q.default {
                    input = input.default(default.to_string());
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
            QuestionKind::Sequence(questions) => {
                for q in questions {
                    self.execute_question(q, answers)?;
                }
            }
            QuestionKind::Alternative(default_idx, alternatives) => {
                // Build the select question for alternatives
                let choices: Vec<String> = alternatives
                    .iter()
                    .map(|alt| alt.name().to_string())
                    .collect();
                let choice_refs: Vec<&str> = choices.iter().map(|s| s.as_str()).collect();

                let selection = dialoguer::Select::new()
                    .with_prompt(question.prompt())
                    .items(&choice_refs)
                    .default(*default_idx)
                    .interact()
                    .map_err(|e| {
                        BackendError::ExecutionError(format!("Failed to prompt: {}", e))
                    })?;

                // Store the selected alternative name
                answers.insert(
                    "selected_alternative".to_string(),
                    AnswerValue::String(choices[selection].clone()),
                );

                // Execute the selected alternative's questions
                if let QuestionKind::Alternative(_, alts) = alternatives[selection].kind() {
                    for q in alts {
                        self.execute_question(q, answers)?;
                    }
                }
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

        for question in &interview.sections {
            self.execute_question(question, &mut answers)?;
        }

        Ok(answers)
    }
}
