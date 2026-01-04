use crate::backend::{BackendError, InterviewBackend};
use crate::interview::{Question, QuestionKind};
use crate::{AnswerValue, Answers};

/// Requestty backend for interactive CLI prompts
pub struct RequesttyBackend;

impl RequesttyBackend {
    pub fn new() -> Self {
        Self
    }

    fn execute_question_with_validator(
        &self,
        question: &Question,
        answers: &mut Answers,
        validator: &(dyn Fn(&str, &str, &Answers) -> Result<(), String> + Send + Sync),
    ) -> Result<(), BackendError> {
        let id = question.id().unwrap_or_else(|| question.name());

        match question.kind() {
            QuestionKind::Input(input_q) => {
                let build_question = || {
                    let mut q = requestty::Question::input(id).message(question.prompt());

                    if let Some(default) = &input_q.default {
                        q = q.default(default.clone());
                    }

                    let answers_for_submit = answers.clone();
                    let answers_for_key = answers.clone();
                    q = q
                        .validate(move |value: &str, _prev_answers| -> Result<(), String> {
                            validator(id, value, &answers_for_submit)
                        })
                        .validate_on_key(move |value: &str, _prev_answers| {
                            validator(id, value, &answers_for_key).is_ok()
                        });

                    q.build()
                };

                loop {
                    match requestty::prompt_one(build_question()) {
                        Ok(requestty::Answer::String(s)) => {
                            // Double-check with validator in case requestty didn't intercept
                            if let Err(msg) = validator(id, &s, answers) {
                                println!("{msg}");
                                continue;
                            }
                            answers.insert(id.to_string(), AnswerValue::String(s));
                            break;
                        }
                        Ok(_) => {
                            return Err(BackendError::ExecutionError(
                                "Expected string answer".to_string(),
                            ));
                        }
                        Err(e) => {
                            println!("{e}");
                            continue;
                        }
                    }
                }
            }
            QuestionKind::Multiline(multiline_q) => {
                let build_question = || {
                    let mut q = requestty::Question::editor(id).message(question.prompt());

                    if let Some(default) = &multiline_q.default {
                        q = q.default(default.clone());
                    }

                    let answers_for_submit = answers.clone();
                    q = q.validate(move |value: &str, _prev_answers| -> Result<(), String> {
                        validator(id, value, &answers_for_submit)
                    });

                    q.build()
                };

                loop {
                    match requestty::prompt_one(build_question()) {
                        Ok(requestty::Answer::String(s)) => {
                            answers.insert(id.to_string(), AnswerValue::String(s));
                            break;
                        }
                        Ok(_) => {
                            return Err(BackendError::ExecutionError(
                                "Expected string answer".to_string(),
                            ));
                        }
                        Err(e) => {
                            println!("{e}");
                            continue;
                        }
                    }
                }
            }
            QuestionKind::Masked(masked_q) => {
                let build_question = || {
                    let mut q = requestty::Question::password(id).message(question.prompt());

                    if let Some(mask) = masked_q.mask {
                        q = q.mask(mask);
                    }

                    let answers_for_submit = answers.clone();
                    q = q.validate(move |value: &str, _prev_answers| -> Result<(), String> {
                        validator(id, value, &answers_for_submit)
                    });

                    q.build()
                };

                loop {
                    match requestty::prompt_one(build_question()) {
                        Ok(requestty::Answer::String(s)) => {
                            answers.insert(id.to_string(), AnswerValue::String(s));
                            break;
                        }
                        Ok(_) => {
                            return Err(BackendError::ExecutionError(
                                "Expected string answer".to_string(),
                            ));
                        }
                        Err(e) => {
                            println!("{e}");
                            continue;
                        }
                    }
                }
            }
            QuestionKind::Sequence(questions) => {
                // Keep enum handling consistent with execute_question
                let is_enum_alternatives = !questions.is_empty()
                    && questions
                        .iter()
                        .all(|q| matches!(q.kind(), QuestionKind::Alternative(_, _)));

                if is_enum_alternatives {
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

                    let parent_prefix = id.strip_suffix(".alternatives");
                    let answer_key = if let Some(prefix) = parent_prefix {
                        format!("{}.{}", prefix, crate::SELECTED_ALTERNATIVE_KEY)
                    } else if id == "alternatives" {
                        crate::SELECTED_ALTERNATIVE_KEY.to_string()
                    } else {
                        format!("{}.{}", id, crate::SELECTED_ALTERNATIVE_KEY)
                    };

                    answers.insert(answer_key, AnswerValue::Int(selection as i64));

                    let selected_variant = &questions[selection];
                    if let QuestionKind::Alternative(_, fields) = selected_variant.kind() {
                        for field_q in fields {
                            if let Some(prefix) = parent_prefix {
                                let field_id = field_q.id().unwrap_or(field_q.name());
                                let prefixed_id = format!("{}.{}", prefix, field_id);
                                let prefixed_question = Question::new(
                                    Some(prefixed_id.clone()),
                                    prefixed_id,
                                    field_q.prompt().to_string(),
                                    field_q.kind().clone(),
                                );
                                self.execute_question_with_validator(
                                    &prefixed_question,
                                    answers,
                                    validator,
                                )?;
                            } else {
                                self.execute_question_with_validator(field_q, answers, validator)?;
                            }
                        }
                    }
                } else {
                    for q in questions {
                        self.execute_question_with_validator(q, answers, validator)?;
                    }
                }
            }
            _ => self.execute_question(question, answers)?,
        }

        Ok(())
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
            QuestionKind::MultiSelect(multi_q) => {
                // Build choices with default selections marked
                let choices: Vec<_> = multi_q.options.iter().enumerate()
                    .map(|(idx, opt)| {
                        let selected = multi_q.defaults.contains(&idx);
                        (opt.clone(), selected)
                    })
                    .collect();

                let q = requestty::Question::multi_select(id)
                    .message(question.prompt())
                    .choices_with_default(choices)
                    .build();

                let answer = requestty::prompt_one(q)
                    .map_err(|e| BackendError::ExecutionError(format!("Failed to prompt: {e}")))?;

                if let requestty::Answer::ListItems(items) = answer {
                    let indices: Vec<i64> = items.into_iter().map(|item| item.index as i64).collect();
                    answers.insert(id.to_string(), AnswerValue::IntList(indices));
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

                    // Store the selected variant index
                    // The question name/id for enum alternatives is "alternatives"
                    // When nested in a struct field, it becomes "fieldname.alternatives"
                    // We need to replace ".alternatives" with ".SELECTED_ALTERNATIVE_KEY"
                    // or just use SELECTED_ALTERNATIVE_KEY for standalone enums
                    let parent_prefix = id.strip_suffix(".alternatives");

                    let answer_key = if let Some(prefix) = parent_prefix {
                        format!("{}.{}", prefix, crate::SELECTED_ALTERNATIVE_KEY)
                    } else if id == "alternatives" {
                        crate::SELECTED_ALTERNATIVE_KEY.to_string()
                    } else {
                        // Fallback: shouldn't happen but handle it
                        format!("{}.{}", id, crate::SELECTED_ALTERNATIVE_KEY)
                    };

                    answers.insert(answer_key, AnswerValue::Int(selection as i64));

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

                // Store the selected alternative index
                answers.insert(
                    crate::SELECTED_ALTERNATIVE_KEY.to_string(),
                    AnswerValue::Int(selected_idx as i64),
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
        use derive_wizard_types::AssumedAnswer;

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

    fn execute_with_validator(
        &self,
        interview: &crate::interview::Interview,
        validator: &(dyn Fn(&str, &str, &Answers) -> Result<(), String> + Send + Sync),
    ) -> Result<Answers, BackendError> {
        use derive_wizard_types::AssumedAnswer;

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

            // Execute with validation support for nested questions
            self.execute_question_with_validator(question, &mut answers, validator)?;
        }

        // Display epilogue if present
        if let Some(epilogue) = &interview.epilogue {
            println!();
            println!("{}", epilogue);
        }

        Ok(answers)
    }
}
