#[cfg(feature = "dialoguer-backend")]
use crate::backend::{AnswerValue, Answers, BackendError, InterviewBackend};
use crate::interview::{Interview, Question, QuestionKind};

/// dialoguer-based interview backend
pub struct DialoguerBackend;

impl DialoguerBackend {
    pub fn new() -> Self {
        Self
    }

    /// Strip trailing colon from prompt since dialoguer adds one automatically
    fn strip_prompt_colon(prompt: &str) -> &str {
        prompt.strip_suffix(':').unwrap_or(prompt).trim_end()
    }

    fn execute_question(
        &self,
        question: &Question,
        answers: &mut Answers,
    ) -> Result<(), BackendError> {
        let id = question.id().unwrap_or(question.name());

        match question.kind() {
            QuestionKind::Input(input_q) => {
                let mut input = dialoguer::Input::<String>::new()
                    .with_prompt(Self::strip_prompt_colon(question.prompt()));

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
                    .with_prompt(Self::strip_prompt_colon(question.prompt()))
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
                        if let Some(min_val) = min
                            && *value < min_val
                        {
                            return Err(format!("Value must be at least {}", min_val));
                        }
                        if let Some(max_val) = max
                            && *value > max_val
                        {
                            return Err(format!("Value must be at most {}", max_val));
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
                        if let Some(min_val) = min
                            && *value < min_val
                        {
                            return Err(format!("Value must be at least {}", min_val));
                        }
                        if let Some(max_val) = max
                            && *value > max_val
                        {
                            return Err(format!("Value must be at most {}", max_val));
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
                    let choice_refs: Vec<&str> = choices.iter().map(|s| s.as_str()).collect();

                    let selection = dialoguer::Select::new()
                        .with_prompt(question.prompt())
                        .items(&choice_refs)
                        .default(0)
                        .interact()
                        .map_err(|e| {
                            BackendError::ExecutionError(format!("Failed to prompt: {}", e))
                        })?;

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
                            // If there's a parent prefix (e.g., "gender"), prefix the field questions
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
                    id.to_string(),
                    AnswerValue::String(choices[selection].clone()),
                );

                // Execute the selected alternative's nested questions
                let selected_alt = &alternatives[selection];
                match selected_alt.kind() {
                    QuestionKind::Sequence(questions) => {
                        for q in questions {
                            self.execute_question(q, answers)?;
                        }
                    }
                    _ => {
                        // Unit variant with no nested questions
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
