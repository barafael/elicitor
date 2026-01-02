use crate::backend::{BackendError, InterviewBackend};
use crate::interview::{Question, QuestionKind};
use crate::{AnswerValue, Answers};

/// Requestty backend for interactive CLI prompts
pub struct RequesttyBackend;

impl RequesttyBackend {
    pub fn new() -> Self {
        Self
    }

    fn execute_question(
        &self,
        question: &Question,
        answers: &mut Answers,
    ) -> Result<(), BackendError> {
        let id = question.id().unwrap_or_else(|| question.name());

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
            QuestionKind::Sequence(questions) => {
                // Check if this is an enum alternatives sequence
                // (all items are Alternative questions)
                let is_enum_alternatives = !questions.is_empty()
                    && questions
                        .iter()
                        .all(|q| matches!(q.kind(), QuestionKind::Alternative(_, _)));

                if is_enum_alternatives {
                    // This is an enum - present a selection menu
                    let choices: Vec<String> =
                        questions.iter().map(|q| q.name().to_string()).collect();

                    let q = requestty::Question::select(id)
                        .message(question.prompt())
                        .choices(choices.clone())
                        .default(0)
                        .build();

                    let answer = requestty::prompt_one(q).map_err(|e| {
                        BackendError::ExecutionError(format!("Failed to prompt: {e}"))
                    })?;

                    let selection = match answer {
                        requestty::Answer::ListItem(item) => item.index,
                        _ => return Err(BackendError::ExecutionError("Expected list item".into())),
                    };

                    // Store the selected variant name
                    // The question name/id for enum alternatives is "alternatives"
                    // When nested in a struct field, it becomes "fieldname.alternatives"
                    // We need to replace ".alternatives" with ".selected_alternative"
                    // or just use "selected_alternative" for standalone enums
                    let parent_prefix = id.strip_suffix(".alternatives");

                    let answer_key = if let Some(prefix) = parent_prefix {
                        format!("{}.selected_alternative", prefix)
                    } else if id == "alternatives" {
                        "selected_alternative".to_string()
                    } else {
                        // Fallback: shouldn't happen but handle it
                        format!("{}.selected_alternative", id)
                    };

                    answers.insert(answer_key, AnswerValue::String(choices[selection].clone()));

                    // Execute the selected variant's fields
                    // Need to prefix them if this enum is a field in a struct
                    let selected_variant = &questions[selection];
                    if let QuestionKind::Alternative(_, fields) = selected_variant.kind() {
                        for field_q in fields {
                            // If there's a parent prefix (e.g., "payment"), prefix the field questions
                            if let Some(prefix) = parent_prefix {
                                let field_id = field_q.id().unwrap_or(field_q.name());
                                let prefixed_id = format!("{}.{}", prefix, field_id);
                                let prefixed_question = Question::new(
                                    Some(prefixed_id.clone()),
                                    prefixed_id,
                                    field_q.prompt().to_string(),
                                    field_q.kind().clone(),
                                );
                                self.execute_question(&prefixed_question, answers)?;
                            } else {
                                self.execute_question(field_q, answers)?;
                            }
                        }
                    }
                } else {
                    // Regular sequence - execute all questions
                    for q in questions {
                        self.execute_question(q, answers)?;
                    }
                }
            }
            QuestionKind::Alternative(default_idx, alternatives) => {
                // Build the select question for alternatives
                let choices: Vec<String> = alternatives
                    .iter()
                    .map(|alt| alt.name().to_string())
                    .collect();

                let q = requestty::Question::select(id)
                    .message(question.prompt())
                    .choices(choices.clone())
                    .default(*default_idx)
                    .build();

                let answer = requestty::prompt_one(q)
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

                // Execute the selected alternative's questions
                if let QuestionKind::Alternative(_, alts) = alternatives[selected_idx].kind() {
                    for q in alts {
                        self.execute_question(q, answers)?;
                    }
                }
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
        use derive_wizard_types::default::AssumedAnswer;

        // Display prelude if present
        if let Some(prelude) = &interview.prelude {
            println!("{}", prelude);
            println!();
        }

        let mut answers = Answers::new();

        for question in &interview.sections {
            // Check if question has an assumption - if so, use it and skip prompting
            if let Some(assumed) = question.assumed() {
                let value = match assumed {
                    AssumedAnswer::String(s) => AnswerValue::String(s.clone()),
                    AssumedAnswer::Int(i) => AnswerValue::Int(*i),
                    AssumedAnswer::Float(f) => AnswerValue::Float(*f),
                    AssumedAnswer::Bool(b) => AnswerValue::Bool(*b),
                };
                answers.insert(question.name().to_string(), value);
                continue;
            }

            self.execute_question(question, &mut answers)?;
        }

        // Display epilogue if present
        if let Some(epilogue) = &interview.epilogue {
            println!();
            println!("{}", epilogue);
        }

        Ok(answers)
    }
}
