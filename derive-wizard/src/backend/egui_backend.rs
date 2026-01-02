#[cfg(feature = "egui-backend")]
use crate::backend::{AnswerValue, Answers, BackendError, InterviewBackend};
use crate::interview::{Interview, Question, QuestionKind};

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

impl InterviewBackend for EguiBackend {
    fn execute(&self, interview: &Interview) -> Result<Answers, BackendError> {
        use derive_wizard_types::default::AssumedAnswer;

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

        // Run the GUI - this blocks until the window is closed
        let _ = eframe::run_native(
            &title,
            options,
            Box::new(move |_cc| Ok(Box::new(EguiWizardApp::new(interview, tx)))),
        );

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
}

impl EguiWizardApp {
    fn new(
        interview: Interview,
        tx: std::sync::mpsc::Sender<Result<Answers, BackendError>>,
    ) -> Self {
        Self {
            interview,
            state: InterviewState::new(),
            completed: false,
            result_sender: Some(tx),
        }
    }

    fn show_wizard(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Interview Wizard");
            ui.separator();

            if self.completed {
                ui.label("Interview completed!");
                return;
            }

            // Show all questions in a scrollable area
            egui::ScrollArea::vertical().show(ui, |ui| {
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
                if ui.button("Submit").clicked()
                    && let Some(answers) = self.validate_and_collect()
                    && let Some(tx) = self.result_sender.take()
                {
                    let _ = tx.send(Ok(answers));
                    self.completed = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }

                // Show validation errors
                if !self.state.validation_errors.is_empty() {
                    ui.add_space(10.0);
                    ui.colored_label(egui::Color32::RED, "Please fix the following errors:");
                    for (field, error) in &self.state.validation_errors {
                        ui.colored_label(egui::Color32::RED, format!("  • {}: {}", field, error));
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

        ui.horizontal(|ui| {
            ui.label(question.prompt());

            if self.state.validation_errors.contains_key(id) {
                ui.colored_label(egui::Color32::RED, "⚠");
            }
        });
        ui.add_space(3.0);

        match question.kind() {
            QuestionKind::Input(input_q) => {
                let buffer = self.state.get_or_init_buffer(id);
                let mut text_edit = egui::TextEdit::singleline(buffer);

                if let Some(default) = &input_q.default {
                    text_edit = text_edit.hint_text(default);
                }

                ui.add(text_edit);
            }
            QuestionKind::Multiline(multiline_q) => {
                let buffer = self.state.get_or_init_buffer(id);
                let mut text_edit = egui::TextEdit::multiline(buffer);

                if let Some(default) = &multiline_q.default {
                    text_edit = text_edit.hint_text(default);
                }

                ui.add(text_edit);
            }
            QuestionKind::Masked(_) => {
                let buffer = self.state.get_or_init_buffer(id);
                ui.add(egui::TextEdit::singleline(buffer).password(true));
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

                if ui.add(drag).changed() {
                    *buffer = value.to_string();
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

                if ui.add(drag).changed() {
                    *buffer = value.to_string();
                }
            }
            QuestionKind::Confirm(confirm_q) => {
                let buffer = self.state.get_or_init_buffer(id);

                if buffer.is_empty() {
                    *buffer = confirm_q.default.to_string();
                }

                let mut value = buffer == "true";
                ui.checkbox(&mut value, "Yes");
                *buffer = value.to_string();
            }
            QuestionKind::Sequence(_) | QuestionKind::Alternative(_, _) => {
                // These are handled in show_question_recursive
                ui.colored_label(
                    egui::Color32::RED,
                    "Error: Sequence/Alternative should be handled recursively",
                );
            }
        }
    }

    fn validate_and_collect(&mut self) -> Option<Answers> {
        self.state.validation_errors.clear();
        let mut answers = Answers::new();
        let mut all_valid = true;

        let questions = self.interview.sections.clone();
        for (question_idx, question) in questions.iter().enumerate() {
            if !self.validate_question_recursive(question, question_idx, &mut answers) {
                all_valid = false;
            }
        }

        if all_valid { Some(answers) } else { None }
    }

    fn validate_question_recursive(
        &mut self,
        question: &Question,
        question_idx: usize,
        answers: &mut Answers,
    ) -> bool {
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

                        // Validate the selected variant's fields
                        if let QuestionKind::Alternative(_, fields) = selected_variant.kind() {
                            let mut valid = true;
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
                                    if !self.validate_question_recursive(
                                        &prefixed_question,
                                        question_idx * 1000 + selected * 100 + idx,
                                        answers,
                                    ) {
                                        valid = false;
                                    }
                                } else {
                                    if !self.validate_question_recursive(
                                        field_q,
                                        question_idx * 1000 + selected * 100 + idx,
                                        answers,
                                    ) {
                                        valid = false;
                                    }
                                }
                            }
                            valid
                        } else {
                            true
                        }
                    } else {
                        true
                    }
                } else {
                    // Regular sequence - validate all questions
                    let mut valid = true;
                    for (idx, q) in questions.iter().enumerate() {
                        if !self.validate_question_recursive(q, question_idx * 1000 + idx, answers)
                        {
                            valid = false;
                        }
                    }
                    valid
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
                        let mut valid = true;
                        for (idx, q) in alts.iter().enumerate() {
                            if !self.validate_question_recursive(
                                q,
                                question_idx * 1000 + selected * 100 + idx,
                                answers,
                            ) {
                                valid = false;
                            }
                        }
                        valid
                    } else {
                        true
                    }
                } else {
                    true
                }
            }
            _ => self.validate_question(question, answers),
        }
    }

    fn validate_question(&mut self, question: &Question, answers: &mut Answers) -> bool {
        let id = question.id().unwrap_or(question.name());
        let buffer = self.state.get_or_init_buffer(id).clone();

        match question.kind() {
            QuestionKind::Input(input_q) => {
                let value = if buffer.is_empty() {
                    input_q.default.clone().unwrap_or_default()
                } else {
                    buffer
                };
                answers.insert(id.to_string(), AnswerValue::String(value));
                true
            }
            QuestionKind::Multiline(multiline_q) => {
                let value = if buffer.is_empty() {
                    multiline_q.default.clone().unwrap_or_default()
                } else {
                    buffer
                };
                answers.insert(id.to_string(), AnswerValue::String(value));
                true
            }
            QuestionKind::Masked(_) => {
                answers.insert(id.to_string(), AnswerValue::String(buffer));
                true
            }
            QuestionKind::Int(int_q) => {
                if buffer.is_empty() {
                    let val = int_q.default.unwrap_or(0);
                    answers.insert(id.to_string(), AnswerValue::Int(val));
                    self.state.validation_errors.remove(id);
                    true
                } else {
                    match buffer.parse::<i64>() {
                        Ok(val) => {
                            answers.insert(id.to_string(), AnswerValue::Int(val));
                            self.state.validation_errors.remove(id);
                            true
                        }
                        Err(_) => {
                            self.state
                                .validation_errors
                                .insert(id.to_string(), "Please enter a valid integer".to_string());
                            false
                        }
                    }
                }
            }
            QuestionKind::Float(float_q) => {
                if buffer.is_empty() {
                    let val = float_q.default.unwrap_or(0.0);
                    answers.insert(id.to_string(), AnswerValue::Float(val));
                    self.state.validation_errors.remove(id);
                    true
                } else {
                    match buffer.parse::<f64>() {
                        Ok(val) => {
                            answers.insert(id.to_string(), AnswerValue::Float(val));
                            self.state.validation_errors.remove(id);
                            true
                        }
                        Err(_) => {
                            self.state.validation_errors.insert(
                                id.to_string(),
                                "Please enter a valid decimal number".to_string(),
                            );
                            false
                        }
                    }
                }
            }
            QuestionKind::Confirm(_) => {
                let val = buffer == "true";
                answers.insert(id.to_string(), AnswerValue::Bool(val));
                true
            }
            QuestionKind::Sequence(_) | QuestionKind::Alternative(_, _) => {
                // These are handled in validate_question_recursive
                false
            }
        }
    }
}

impl eframe::App for EguiWizardApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.show_wizard(ctx);
    }
}
