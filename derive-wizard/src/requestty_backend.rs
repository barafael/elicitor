#[cfg(feature = "requestty-backend")]
use crate::backend::{AnswerValue, Answers, BackendError, InterviewBackend};
use crate::interview::Section;
use crate::question::{Question, QuestionKind};

/// Requestty backend for interactive CLI prompts
pub struct RequesttyBackend;

impl RequesttyBackend {
    pub fn new() -> Self {
        Self
    }

    fn execute_section(
        &self,
        section: &Section,
        answers: &mut Answers,
    ) -> Result<(), BackendError> {
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

                let answer = requestty::prompt_one(question)
                    .map_err(|e| BackendError::ExecutionError(format!("Failed to prompt: {e}")))?;

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
        question: &Question,
        answers: &mut Answers,
    ) -> Result<(), BackendError> {
        let id = question.id().unwrap_or(question.name());

        match question.kind() {
            QuestionKind::Input(input_q) => {
                let mut q = requestty::Question::input(id).message(question.prompt());

                if let Some(default) = &input_q.default {
                    q = q.default(default.clone());
                }

                let answer = requestty::prompt_one(q.build())
                    .map_err(|e| BackendError::ExecutionError(format!("Failed to prompt: {e}")))?;

                if let requestty::Answer::String(s) = answer {
                    answers.insert(id.to_string(), AnswerValue::String(s));
                }
            }
            QuestionKind::Multiline(multiline_q) => {
                let mut q = requestty::Question::editor(id).message(question.prompt());

                if let Some(default) = &multiline_q.default {
                    q = q.default(default.clone());
                }

                let answer = requestty::prompt_one(q.build())
                    .map_err(|e| BackendError::ExecutionError(format!("Failed to prompt: {e}")))?;

                if let requestty::Answer::String(s) = answer {
                    answers.insert(id.to_string(), AnswerValue::String(s));
                }
            }
            QuestionKind::Masked(masked_q) => {
                let mut q = requestty::Question::password(id).message(question.prompt());

                if let Some(mask) = masked_q.mask {
                    q = q.mask(mask);
                }

                let answer = requestty::prompt_one(q.build())
                    .map_err(|e| BackendError::ExecutionError(format!("Failed to prompt: {e}")))?;

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
                            return Err(format!("Value must be at least {min_val}"));
                        }
                        if let Some(max_val) = max
                            && value > max_val
                        {
                            return Err(format!("Value must be at most {max_val}"));
                        }
                        Ok(())
                    });
                }

                let answer = requestty::prompt_one(q.build())
                    .map_err(|e| BackendError::ExecutionError(format!("Failed to prompt: {e}")))?;

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
                            return Err(format!("Value must be at least {min_val}"));
                        }
                        if let Some(max_val) = max
                            && value > max_val
                        {
                            return Err(format!("Value must be at most {max_val}"));
                        }
                        Ok(())
                    });
                }

                let answer = requestty::prompt_one(q.build())
                    .map_err(|e| BackendError::ExecutionError(format!("Failed to prompt: {e}")))?;

                if let requestty::Answer::Float(f) = answer {
                    answers.insert(id.to_string(), AnswerValue::Float(f));
                }
            }
            QuestionKind::Confirm(confirm_q) => {
                let q = requestty::Question::confirm(id)
                    .message(question.prompt())
                    .default(confirm_q.default)
                    .build();

                let answer = requestty::prompt_one(q)
                    .map_err(|e| BackendError::ExecutionError(format!("Failed to prompt: {e}")))?;

                if let requestty::Answer::Bool(b) = answer {
                    answers.insert(id.to_string(), AnswerValue::Bool(b));
                }
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

impl Default for RequesttyBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl InterviewBackend for RequesttyBackend {
    fn execute(&self, interview: &crate::interview::Interview) -> Result<Answers, BackendError> {
        let mut answers = Answers::new();

        for section in &interview.sections {
            self.execute_section(section, &mut answers)?;
        }

        Ok(answers)
    }
}
