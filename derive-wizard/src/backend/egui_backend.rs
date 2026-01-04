#[cfg(feature = "egui-backend")]
use crate::backend::{AnswerValue, Answers, BackendError, InterviewBackend};
use crate::interview::{Interview, Question, QuestionKind};
use itertools::Itertools;
use std::sync::Arc;

/// egui-based interview backend
pub struct EguiBackend {
    title: String,
    window_size: [f32; 2],
    options: Option<eframe::NativeOptions>,
}

impl EguiBackend {
    /// Create a new egui backend with default settings
    pub fn new() -> Self {
        Self {
            title: "Interview Wizard".to_string(),
            window_size: [400.0, 300.0],
            options: None,
        }
    }

    /// Set the window title
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Set the window size
    pub fn with_window_size(mut self, size: [f32; 2]) -> Self {
        self.window_size = size;
        self
    }

    /// Set custom eframe options
    pub fn with_options(mut self, options: eframe::NativeOptions) -> Self {
        self.options = Some(options);
        self
    }
}

impl Default for EguiBackend {
    fn default() -> Self {
        Self::new()
    }
}

/// Type alias for the validator function
type ValidatorFn = Arc<dyn Fn(&str, &str, &Answers) -> Result<(), String> + Send + Sync>;

/// A no-op validator that always succeeds
fn noop_validator(_field: &str, _value: &str, _answers: &Answers) -> Result<(), String> {
    Ok(())
}

impl InterviewBackend for EguiBackend {
    fn execute(&self, interview: &Interview) -> Result<Answers, BackendError> {
        self.execute_with_validator(interview, &noop_validator)
    }

    fn execute_with_validator(
        &self,
        interview: &Interview,
        validator: &(dyn Fn(&str, &str, &Answers) -> Result<(), String> + Send + Sync),
    ) -> Result<Answers, BackendError> {
        use derive_wizard_types::AssumedAnswer;

        let mut answers = Answers::new();

        // First, collect all assumed answers
        for question in &interview.sections {
            if let Some(assumed) = question.assumed() {
                let value = match assumed {
                    AssumedAnswer::String(s) => AnswerValue::String(s.clone()),
                    AssumedAnswer::Int(i) => AnswerValue::Int(*i),
                    AssumedAnswer::Float(f) => AnswerValue::Float(*f),
                    AssumedAnswer::Bool(b) => AnswerValue::Bool(*b),
                };
                answers.insert(question.name().to_string(), value);
            }
        }

        // Check if all questions have assumptions - if so, skip the GUI
        let all_assumed = interview.sections.iter().all(|q| q.assumed().is_some());
        if all_assumed {
            return Ok(answers);
        }

        // Create a channel to get the result back from the GUI
        let (tx, rx) = std::sync::mpsc::channel();

        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size(self.window_size),
            ..Default::default()
        };

        let title = self.title.clone();
        let interview = interview.clone();

        // Use channels for validation requests/responses to avoid lifetime issues
        // The GUI sends validation requests through a channel, and we respond synchronously
        let (validate_tx, validate_rx) = std::sync::mpsc::channel::<(String, String, Answers)>();
        let (validate_result_tx, validate_result_rx) =
            std::sync::mpsc::channel::<Result<(), String>>();

        // Wrap the receiver in a Mutex so it can be shared (Sync)
        let validate_result_rx = std::sync::Mutex::new(validate_result_rx);

        // Wrap the validation channels into the validator function for the GUI
        let gui_validator: ValidatorFn =
            Arc::new(move |field: &str, value: &str, answers: &Answers| {
                // Send validation request
                if validate_tx
                    .send((field.to_string(), value.to_string(), answers.clone()))
                    .is_err()
                {
                    return Ok(()); // Channel closed, assume valid
                }

                // Wait for response without panicking on lock/channel errors
                match validate_result_rx.lock() {
                    Ok(rx) => rx.recv().unwrap_or(Ok(())),
                    Err(_) => Ok(()), // Poisoned lock: assume valid instead of panic
                }
            });

        // Use thread::scope to allow borrowing the validator reference
        std::thread::scope(|scope| {
            // Spawn a thread to handle validation requests using the original validator
            // Move validate_rx and validate_result_tx into the closure
            scope.spawn(move || {
                // The thread will exit when the channel is closed (GUI exits)
                while let Ok((field, value, answers)) = validate_rx.recv() {
                    let result = validator(&field, &value, &answers);
                    if validate_result_tx.send(result).is_err() {
                        break;
                    }
                }
            });

            // Run the GUI - this blocks until the window is closed
            let _ = eframe::run_native(
                &title,
                options,
                Box::new(move |_cc| Ok(Box::new(EguiWizardApp::new(interview, tx, gui_validator)))),
            );
        });

        // Get the result from the channel
        let mut gui_answers = rx.recv().map_err(|e| {
            BackendError::ExecutionError(format!("GUI closed without result: {}", e))
        })??;

        // Merge assumed answers with GUI answers (assumptions take precedence)
        gui_answers.merge(answers);

        Ok(gui_answers)
    }
}

/// State for the interview
#[derive(Debug, Clone)]
struct InterviewState {
    input_buffers: std::collections::HashMap<String, String>,
    selected_alternatives: std::collections::HashMap<String, usize>,
    validation_errors: std::collections::HashMap<String, String>,
}

impl InterviewState {
    fn new() -> Self {
        Self {
            input_buffers: std::collections::HashMap::new(),
            selected_alternatives: std::collections::HashMap::new(),
            validation_errors: std::collections::HashMap::new(),
        }
    }

    fn get_or_init_buffer(&mut self, key: &str) -> &mut String {
        self.input_buffers.entry(key.to_string()).or_default()
    }
}

/// Internal GUI app
struct EguiWizardApp {
    interview: Interview,
    state: InterviewState,
    completed: bool,
    result_sender: Option<std::sync::mpsc::Sender<Result<Answers, BackendError>>>,
    validator: ValidatorFn,
}

impl EguiWizardApp {
    fn new(
        interview: Interview,
        tx: std::sync::mpsc::Sender<Result<Answers, BackendError>>,
        validator: ValidatorFn,
    ) -> Self {
        Self {
            interview,
            state: InterviewState::new(),
            completed: false,
            result_sender: Some(tx),
            validator,
        }
    }

    fn show_wizard(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Interview Wizard");
            ui.separator();

            if self.completed {
                ui.label("Interview completed!");
                // Display epilogue if present
                if let Some(epilogue) = &self.interview.epilogue {
                    ui.add_space(10.0);
                    ui.label(epilogue);
                }
                return;
            }

            // Show all questions in a scrollable area
            egui::ScrollArea::vertical().show(ui, |ui| {
                // Display prelude if present
                if let Some(prelude) = &self.interview.prelude {
                    ui.label(prelude);
                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);
                }

                let questions: Vec<_> = self.interview.sections.clone();
                for (question_idx, question) in questions.iter().enumerate() {
                    // Skip questions that have assumptions
                    if question.assumed().is_some() {
                        continue;
                    }
                    self.show_question_recursive(ui, question, question_idx);
                    ui.add_space(15.0);
                }

                ui.separator();
                ui.add_space(10.0);

                // Submit button at the bottom
                if ui.button("Submit").clicked() {
                    if let Some(answers) = self.validate_and_collect() {
                        if let Some(tx) = self.result_sender.take() {
                            let _ = tx.send(Ok(answers));
                            self.completed = true;
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    }
                }
            });
        });
    }

    fn show_question_recursive(
        &mut self,
        ui: &mut egui::Ui,
        question: &Question,
        question_idx: usize,
    ) {
        match question.kind() {
            QuestionKind::Sequence(questions) => {
                // Check if this is an enum alternatives sequence
                let is_enum_alternatives = !questions.is_empty()
                    && questions
                        .iter()
                        .all(|q| matches!(q.kind(), QuestionKind::Alternative(_, _)));

                if is_enum_alternatives {
                    // This is an enum - render as a selection
                    let alt_key = format!("question_{}", question_idx);

                    ui.label(question.prompt());
                    ui.add_space(5.0);

                    let selected = self
                        .state
                        .selected_alternatives
                        .get(&alt_key)
                        .copied()
                        .unwrap_or(0);

                    for (idx, variant) in questions.iter().enumerate() {
                        if ui.radio(selected == idx, variant.name()).clicked() {
                            self.state
                                .selected_alternatives
                                .insert(alt_key.clone(), idx);
                        }
                    }

                    ui.add_space(10.0);

                    // Show fields for the selected variant
                    if let Some(selected_variant) = questions.get(selected)
                        && let QuestionKind::Alternative(_, fields) = selected_variant.kind()
                        && !fields.is_empty()
                    {
                        ui.group(|ui| {
                            ui.label(format!("Details for '{}':", selected_variant.name()));
                            ui.add_space(5.0);

                            // Determine if we need to prefix field IDs
                            let id = question.id().unwrap_or(question.name());
                            let parent_prefix = id.strip_suffix(".alternatives");

                            for (idx, field_q) in fields.iter().enumerate() {
                                // Prefix field questions if this enum is nested in a struct
                                if let Some(prefix) = parent_prefix {
                                    let field_id = field_q.id().unwrap_or(field_q.name());
                                    let prefixed_id = format!("{}.{}", prefix, field_id);
                                    let prefixed_question = Question::new(
                                        Some(prefixed_id.clone()),
                                        prefixed_id,
                                        field_q.prompt().to_string(),
                                        field_q.kind().clone(),
                                    );
                                    self.show_question_recursive(
                                        ui,
                                        &prefixed_question,
                                        question_idx * 1000 + selected * 100 + idx,
                                    );
                                } else {
                                    self.show_question_recursive(
                                        ui,
                                        field_q,
                                        question_idx * 1000 + selected * 100 + idx,
                                    );
                                }
                                ui.add_space(8.0);
                            }
                        });
                    }
                } else {
                    // Regular sequence - show all questions
                    for (idx, q) in questions.iter().enumerate() {
                        self.show_question_recursive(ui, q, question_idx * 1000 + idx);
                        ui.add_space(8.0);
                    }
                }
            }
            QuestionKind::Alternative(default_idx, alternatives) => {
                let alt_key = format!("question_{}", question_idx);

                ui.label(question.prompt());
                ui.add_space(5.0);

                let selected = self
                    .state
                    .selected_alternatives
                    .get(&alt_key)
                    .copied()
                    .unwrap_or(*default_idx);

                for (idx, alternative) in alternatives.iter().enumerate() {
                    if ui.radio(selected == idx, alternative.name()).clicked() {
                        self.state
                            .selected_alternatives
                            .insert(alt_key.clone(), idx);
                    }
                }

                ui.add_space(10.0);

                // Show follow-up questions for selected alternative
                if let Some(alt) = alternatives.get(selected)
                    && let QuestionKind::Alternative(_, alts) = alt.kind()
                    && !alts.is_empty()
                {
                    ui.group(|ui| {
                        ui.label(format!("Details for '{}':", alt.name()));
                        ui.add_space(5.0);
                        for (idx, q) in alts.iter().enumerate() {
                            self.show_question_recursive(
                                ui,
                                q,
                                question_idx * 1000 + selected * 100 + idx,
                            );
                            ui.add_space(8.0);
                        }
                    });
                }
            }
            _ => {
                self.show_question(ui, question);
            }
        }
    }

    fn show_question(&mut self, ui: &mut egui::Ui, question: &Question) {
        let id = question.id().unwrap_or(question.name());

        ui.label(question.prompt());
        ui.add_space(3.0);

        match question.kind() {
            QuestionKind::Input(input_q) => {
                // Check for validation error before any mutable borrows
                let has_error = self.state.validation_errors.contains_key(id);

                let buffer = self.state.get_or_init_buffer(id);
                let mut text_edit = egui::TextEdit::singleline(buffer);

                if let Some(default) = &input_q.default {
                    text_edit = text_edit.hint_text(default);
                }

                // Add red border if there's a validation error
                if has_error {
                    ui.visuals_mut().widgets.inactive.bg_stroke.color = egui::Color32::RED;
                    ui.visuals_mut().widgets.hovered.bg_stroke.color = egui::Color32::RED;
                    ui.visuals_mut().widgets.active.bg_stroke.color = egui::Color32::RED;
                }

                ui.add(text_edit);

                // Run validation immediately on any change if validator is configured
                if input_q.validate.is_some() {
                    let value = self
                        .state
                        .input_buffers
                        .get(id)
                        .cloned()
                        .unwrap_or_default();
                    let current_answers = self.build_current_answers();
                    match (self.validator)(id, &value, &current_answers) {
                        Ok(()) => {
                            self.state.validation_errors.remove(id);
                        }
                        Err(err) => {
                            self.state.validation_errors.insert(id.to_string(), err);
                        }
                    }
                }
            }
            QuestionKind::Multiline(multiline_q) => {
                // Check for validation error before any mutable borrows
                let has_error = self.state.validation_errors.contains_key(id);

                let buffer = self.state.get_or_init_buffer(id);
                let mut text_edit = egui::TextEdit::multiline(buffer);

                if let Some(default) = &multiline_q.default {
                    text_edit = text_edit.hint_text(default);
                }

                // Add red border if there's a validation error
                if has_error {
                    ui.visuals_mut().widgets.inactive.bg_stroke.color = egui::Color32::RED;
                    ui.visuals_mut().widgets.hovered.bg_stroke.color = egui::Color32::RED;
                    ui.visuals_mut().widgets.active.bg_stroke.color = egui::Color32::RED;
                }

                ui.add(text_edit);

                // Run validation immediately on any change if validator is configured
                if multiline_q.validate.is_some() {
                    let value = self
                        .state
                        .input_buffers
                        .get(id)
                        .cloned()
                        .unwrap_or_default();
                    let current_answers = self.build_current_answers();
                    match (self.validator)(id, &value, &current_answers) {
                        Ok(()) => {
                            self.state.validation_errors.remove(id);
                        }
                        Err(err) => {
                            self.state.validation_errors.insert(id.to_string(), err);
                        }
                    }
                }
            }
            QuestionKind::Masked(masked_q) => {
                // Check for validation error before any mutable borrows
                let has_error = self.state.validation_errors.contains_key(id);

                let buffer = self.state.get_or_init_buffer(id);

                // Add red border if there's a validation error
                if has_error {
                    ui.visuals_mut().widgets.inactive.bg_stroke.color = egui::Color32::RED;
                    ui.visuals_mut().widgets.hovered.bg_stroke.color = egui::Color32::RED;
                    ui.visuals_mut().widgets.active.bg_stroke.color = egui::Color32::RED;
                }

                ui.add(egui::TextEdit::singleline(buffer).password(true));

                // Run validation immediately on any change if validator is configured
                if masked_q.validate.is_some() {
                    let value = self
                        .state
                        .input_buffers
                        .get(id)
                        .cloned()
                        .unwrap_or_default();
                    let current_answers = self.build_current_answers();
                    match (self.validator)(id, &value, &current_answers) {
                        Ok(()) => {
                            self.state.validation_errors.remove(id);
                        }
                        Err(err) => {
                            self.state.validation_errors.insert(id.to_string(), err);
                        }
                    }
                }
            }
            QuestionKind::Int(int_q) => {
                let buffer = self.state.get_or_init_buffer(id);

                let mut value = if buffer.is_empty() {
                    int_q.default.unwrap_or(0)
                } else {
                    buffer.parse::<i64>().unwrap_or(0)
                };

                let mut drag = egui::DragValue::new(&mut value).speed(1.0);

                if let Some(min) = int_q.min {
                    drag = drag.range(min..=int_q.max.unwrap_or(i64::MAX));
                } else if let Some(max) = int_q.max {
                    drag = drag.range(i64::MIN..=max);
                }

                let response = ui.add(drag);
                // Only run real-time validation on actual user interaction, not on auto-clamp
                if response.changed() {
                    *self.state.get_or_init_buffer(id) = value.to_string();
                }
                if response.drag_stopped() || response.lost_focus() {
                    // Run validation when user finishes interacting
                    if int_q.validate.is_some() {
                        let current_answers = self.build_current_answers();
                        match (self.validator)(id, &value.to_string(), &current_answers) {
                            Ok(()) => {
                                self.state.validation_errors.remove(id);
                            }
                            Err(err) => {
                                self.state.validation_errors.insert(id.to_string(), err);
                            }
                        }
                    }
                }
            }
            QuestionKind::Float(float_q) => {
                let buffer = self.state.get_or_init_buffer(id);

                let mut value = if buffer.is_empty() {
                    float_q.default.unwrap_or(0.0)
                } else {
                    buffer.parse::<f64>().unwrap_or(0.0)
                };

                let mut drag = egui::DragValue::new(&mut value).speed(0.1);

                if let Some(min) = float_q.min {
                    drag = drag.range(min..=float_q.max.unwrap_or(f64::MAX));
                } else if let Some(max) = float_q.max {
                    drag = drag.range(f64::MIN..=max);
                }

                let response = ui.add(drag);
                // Only run real-time validation on actual user interaction, not on auto-clamp
                if response.changed() {
                    *self.state.get_or_init_buffer(id) = value.to_string();
                }
                if response.drag_stopped() || response.lost_focus() {
                    // Run validation when user finishes interacting
                    if float_q.validate.is_some() {
                        let current_answers = self.build_current_answers();
                        match (self.validator)(id, &value.to_string(), &current_answers) {
                            Ok(()) => {
                                self.state.validation_errors.remove(id);
                            }
                            Err(err) => {
                                self.state.validation_errors.insert(id.to_string(), err);
                            }
                        }
                    }
                }
            }
            QuestionKind::Confirm(confirm_q) => {
                let buffer = self.state.get_or_init_buffer(id);

                if buffer.is_empty() {
                    *buffer = confirm_q.default.to_string();
                }

                let mut value = buffer == "true";
                ui.checkbox(&mut value, "Yes");
                *self.state.get_or_init_buffer(id) = value.to_string();
            }
            QuestionKind::Sequence(_) | QuestionKind::Alternative(_, _) => {
                // These are handled in show_question_recursive
                ui.colored_label(
                    egui::Color32::RED,
                    "Error: Sequence/Alternative should be handled recursively",
                );
            }
        }

        // Show inline validation error for this field with background
        if let Some(error) = self.state.validation_errors.get(id) {
            ui.add_space(2.0);
            egui::Frame::none()
                .fill(egui::Color32::from_rgb(255, 230, 230))
                .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                .rounding(egui::Rounding::same(4.0))
                .show(ui, |ui| {
                    ui.colored_label(egui::Color32::from_rgb(180, 0, 0), format!("⚠ {}", error));
                });
        }
    }

    /// Build an Answers collection from the current input buffers for validation
    fn build_current_answers(&self) -> Answers {
        let mut answers = Answers::new();
        for (key, value) in &self.state.input_buffers {
            answers.insert(key.clone(), AnswerValue::String(value.clone()));
        }
        answers
    }

    fn validate_and_collect(&mut self) -> Option<Answers> {
        self.state.validation_errors.clear();
        let mut answers = Answers::new();

        let questions = self.interview.sections.clone();

        // Collect all validation results
        let results: Vec<Result<(), (String, String)>> = questions
            .iter()
            .enumerate()
            .map(|(question_idx, question)| {
                self.validate_question_recursive(question, question_idx, &mut answers)
            })
            .collect();

        // Partition into successes and errors
        let (_oks, errs): (Vec<_>, Vec<_>) = results.into_iter().partition_result();

        if errs.is_empty() {
            Some(answers)
        } else {
            // Store all validation errors
            for (id, err) in errs {
                self.state.validation_errors.insert(id, err);
            }
            None
        }
    }

    fn validate_question_recursive(
        &mut self,
        question: &Question,
        question_idx: usize,
        answers: &mut Answers,
    ) -> Result<(), (String, String)> {
        match question.kind() {
            QuestionKind::Sequence(questions) => {
                // Check if this is an enum alternatives sequence
                let is_enum_alternatives = !questions.is_empty()
                    && questions
                        .iter()
                        .all(|q| matches!(q.kind(), QuestionKind::Alternative(_, _)));

                if is_enum_alternatives {
                    // This is an enum - handle variant selection
                    let alt_key = format!("question_{}", question_idx);
                    let selected = self
                        .state
                        .selected_alternatives
                        .get(&alt_key)
                        .copied()
                        .unwrap_or(0);

                    if let Some(selected_variant) = questions.get(selected) {
                        // Store the selected variant name with proper prefixing
                        let id = question.id().unwrap_or(question.name());
                        let parent_prefix = id.strip_suffix(".alternatives");

                        let answer_key = if let Some(prefix) = parent_prefix {
                            format!("{}.selected_alternative", prefix)
                        } else if id == "alternatives" {
                            "selected_alternative".to_string()
                        } else {
                            format!("{}.selected_alternative", id)
                        };

                        answers.insert(
                            answer_key,
                            AnswerValue::String(selected_variant.name().to_string()),
                        );

                        // Validate the selected variant's fields - collect all errors
                        if let QuestionKind::Alternative(_, fields) = selected_variant.kind() {
                            let results: Vec<Result<(), (String, String)>> = fields
                                .iter()
                                .enumerate()
                                .map(|(idx, field_q)| {
                                    // Prefix field questions if this enum is nested in a struct
                                    if let Some(prefix) = parent_prefix {
                                        let field_id = field_q.id().unwrap_or(field_q.name());
                                        let prefixed_id = format!("{}.{}", prefix, field_id);
                                        let prefixed_question = Question::new(
                                            Some(prefixed_id.clone()),
                                            prefixed_id,
                                            field_q.prompt().to_string(),
                                            field_q.kind().clone(),
                                        );
                                        self.validate_question_recursive(
                                            &prefixed_question,
                                            question_idx * 1000 + selected * 100 + idx,
                                            answers,
                                        )
                                    } else {
                                        self.validate_question_recursive(
                                            field_q,
                                            question_idx * 1000 + selected * 100 + idx,
                                            answers,
                                        )
                                    }
                                })
                                .collect();

                            let (_oks, errs): (Vec<_>, Vec<_>) =
                                results.into_iter().partition_result();

                            if errs.is_empty() {
                                Ok(())
                            } else {
                                // Store all errors and return the first one
                                for (id, err) in &errs {
                                    self.state.validation_errors.insert(id.clone(), err.clone());
                                }

                                match errs.into_iter().next() {
                                    Some(first) => Err(first),
                                    None => Ok(()),
                                }
                            }
                        } else {
                            Ok(())
                        }
                    } else {
                        Ok(())
                    }
                } else {
                    // Regular sequence - validate all questions, collect all errors
                    let results: Vec<Result<(), (String, String)>> = questions
                        .iter()
                        .enumerate()
                        .map(|(idx, q)| {
                            self.validate_question_recursive(q, question_idx * 1000 + idx, answers)
                        })
                        .collect();

                    let (_oks, errs): (Vec<_>, Vec<_>) = results.into_iter().partition_result();

                    if errs.is_empty() {
                        Ok(())
                    } else {
                        // Store all errors and return the first one
                        for (id, err) in &errs {
                            self.state.validation_errors.insert(id.clone(), err.clone());
                        }

                        match errs.into_iter().next() {
                            Some(first) => Err(first),
                            None => Ok(()),
                        }
                    }
                }
            }
            QuestionKind::Alternative(default_idx, alternatives) => {
                let alt_key = format!("question_{}", question_idx);
                let selected = self
                    .state
                    .selected_alternatives
                    .get(&alt_key)
                    .copied()
                    .unwrap_or(*default_idx);

                if let Some(alt) = alternatives.get(selected) {
                    answers.insert(
                        "selected_alternative".to_string(),
                        AnswerValue::String(alt.name().to_string()),
                    );

                    if let QuestionKind::Alternative(_, alts) = alt.kind() {
                        let results: Vec<Result<(), (String, String)>> = alts
                            .iter()
                            .enumerate()
                            .map(|(idx, q)| {
                                self.validate_question_recursive(
                                    q,
                                    question_idx * 1000 + selected * 100 + idx,
                                    answers,
                                )
                            })
                            .collect();

                        let (_oks, errs): (Vec<_>, Vec<_>) = results.into_iter().partition_result();

                        if errs.is_empty() {
                            Ok(())
                        } else {
                            // Store all errors and return the first one
                            for (id, err) in &errs {
                                self.state.validation_errors.insert(id.clone(), err.clone());
                            }

                            match errs.into_iter().next() {
                                Some(first) => Err(first),
                                None => Ok(()),
                            }
                        }
                    } else {
                        Ok(())
                    }
                } else {
                    Ok(())
                }
            }
            _ => self.validate_question(question, answers),
        }
    }

    /// Validates a single question and returns Ok(()) if valid, Err((id, message)) if invalid.
    /// Always inserts the answer into the answers map so subsequent validators can access it.
    fn validate_question(
        &mut self,
        question: &Question,
        answers: &mut Answers,
    ) -> Result<(), (String, String)> {
        let id = question.id().unwrap_or(question.name());
        let buffer = self.state.get_or_init_buffer(id).clone();

        match question.kind() {
            QuestionKind::Input(input_q) => {
                let value = if buffer.is_empty() {
                    input_q.default.clone().unwrap_or_default()
                } else {
                    buffer
                };

                // Always insert the answer first
                answers.insert(id.to_string(), AnswerValue::String(value.clone()));

                // Run custom validator if configured
                if input_q.validate.is_some() {
                    if let Err(err) = (self.validator)(id, &value, answers) {
                        return Err((id.to_string(), err));
                    }
                }

                Ok(())
            }
            QuestionKind::Multiline(multiline_q) => {
                let value = if buffer.is_empty() {
                    multiline_q.default.clone().unwrap_or_default()
                } else {
                    buffer
                };

                // Always insert the answer first
                answers.insert(id.to_string(), AnswerValue::String(value.clone()));

                // Run custom validator if configured
                if multiline_q.validate.is_some() {
                    if let Err(err) = (self.validator)(id, &value, answers) {
                        return Err((id.to_string(), err));
                    }
                }

                Ok(())
            }
            QuestionKind::Masked(masked_q) => {
                // Always insert the answer first
                answers.insert(id.to_string(), AnswerValue::String(buffer.clone()));

                // Run custom validator if configured
                if masked_q.validate.is_some() {
                    if let Err(err) = (self.validator)(id, &buffer, answers) {
                        return Err((id.to_string(), err));
                    }
                }

                Ok(())
            }
            QuestionKind::Int(int_q) => {
                let val = if buffer.is_empty() {
                    int_q.default.unwrap_or(0)
                } else {
                    match buffer.parse::<i64>() {
                        Ok(v) => v,
                        Err(_) => {
                            // Still insert a default so other validations can proceed
                            answers.insert(id.to_string(), AnswerValue::Int(0));
                            return Err((
                                id.to_string(),
                                "Please enter a valid integer".to_string(),
                            ));
                        }
                    }
                };

                // Always insert the answer first
                answers.insert(id.to_string(), AnswerValue::Int(val));

                // Run custom validator if configured
                if int_q.validate.is_some() {
                    if let Err(err) = (self.validator)(id, &val.to_string(), answers) {
                        return Err((id.to_string(), err));
                    }
                }

                Ok(())
            }
            QuestionKind::Float(float_q) => {
                let val = if buffer.is_empty() {
                    float_q.default.unwrap_or(0.0)
                } else {
                    match buffer.parse::<f64>() {
                        Ok(v) => v,
                        Err(_) => {
                            // Still insert a default so other validations can proceed
                            answers.insert(id.to_string(), AnswerValue::Float(0.0));
                            return Err((
                                id.to_string(),
                                "Please enter a valid decimal number".to_string(),
                            ));
                        }
                    }
                };

                // Always insert the answer first
                answers.insert(id.to_string(), AnswerValue::Float(val));

                // Run custom validator if configured
                if float_q.validate.is_some() {
                    if let Err(err) = (self.validator)(id, &val.to_string(), answers) {
                        return Err((id.to_string(), err));
                    }
                }

                Ok(())
            }
            QuestionKind::Confirm(_) => {
                let val = buffer == "true";
                answers.insert(id.to_string(), AnswerValue::Bool(val));
                Ok(())
            }
            QuestionKind::Sequence(_) | QuestionKind::Alternative(_, _) => {
                // These are handled in validate_question_recursive
                Ok(())
            }
        }
    }
}

impl eframe::App for EguiWizardApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.show_wizard(ctx);
    }
}
