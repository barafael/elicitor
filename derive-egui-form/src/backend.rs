//! Egui backend implementation for SurveyBackend trait.

use derive_survey::{
    AllOfQuestion, AnyOfQuestion, DefaultValue, FloatQuestion, IntQuestion, ListElementKind,
    ListQuestion, OneOfQuestion, Question, QuestionKind, ResponsePath, ResponseValue, Responses,
    SELECTED_VARIANT_KEY, SELECTED_VARIANTS_KEY, SurveyBackend, SurveyDefinition, Variant,
};
use eframe::egui;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use thiserror::Error;

/// Error type for the Egui backend.
#[derive(Debug, Error)]
pub enum EguiError {
    /// User cancelled the survey (closed the window).
    #[error("Survey cancelled by user")]
    Cancelled,

    /// An error occurred in the egui/eframe backend.
    #[error("Egui error: {0}")]
    EguiError(String),
}

/// Builder/configuration for the Egui backend.
#[derive(Debug, Clone)]
pub struct EguiBackend {
    /// Window title.
    title: String,
    /// Window size [width, height].
    window_size: [f32; 2],
}

impl Default for EguiBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl EguiBackend {
    /// Create a new Egui backend with default settings.
    pub fn new() -> Self {
        Self {
            title: "Survey".to_string(),
            window_size: [500.0, 600.0],
        }
    }

    /// Set the window title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Set the window size.
    pub fn with_window_size(mut self, size: [f32; 2]) -> Self {
        self.window_size = size;
        self
    }
}

/// State for a single field in the form.
#[derive(Debug, Clone)]
enum FieldState {
    /// String input (for Input, Multiline, Masked).
    Text {
        value: String,
        is_password: bool,
        is_multiline: bool,
    },
    /// Integer input.
    Int { value: String, parsed: Option<i64> },
    /// Float input.
    Float { value: String, parsed: Option<f64> },
    /// Boolean toggle.
    Bool { value: bool },
    /// List of values (comma-separated input).
    List {
        value: String,
        element_kind: ListElementKind,
    },
    /// Single selection from options (OneOf).
    OneOf {
        selected: Option<usize>,
        #[allow(dead_code)]
        variants: Vec<String>,
    },
    /// Multiple selection (AnyOf).
    AnyOf {
        selected: Vec<bool>,
        #[allow(dead_code)]
        variants: Vec<String>,
    },
}

impl FieldState {
    /// Extract the ResponseValue from this field state.
    fn to_response_value(&self) -> Option<ResponseValue> {
        match self {
            FieldState::Text { value, .. } => Some(ResponseValue::String(value.clone())),
            FieldState::Int { parsed, .. } => parsed.map(ResponseValue::Int),
            FieldState::Float { parsed, .. } => parsed.map(ResponseValue::Float),
            FieldState::Bool { value } => Some(ResponseValue::Bool(*value)),
            FieldState::List {
                value,
                element_kind,
            } => {
                let items: Vec<&str> = value
                    .split(',')
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .collect();

                match element_kind {
                    ListElementKind::String => Some(ResponseValue::StringList(
                        items.iter().map(|s| s.to_string()).collect(),
                    )),
                    ListElementKind::Int { .. } => {
                        let ints: Result<Vec<i64>, _> =
                            items.iter().map(|s| s.parse::<i64>()).collect();
                        ints.ok().map(ResponseValue::IntList)
                    }
                    ListElementKind::Float { .. } => {
                        let floats: Result<Vec<f64>, _> =
                            items.iter().map(|s| s.parse::<f64>()).collect();
                        floats.ok().map(ResponseValue::FloatList)
                    }
                }
            }
            FieldState::OneOf { selected, .. } => selected.map(ResponseValue::ChosenVariant),
            FieldState::AnyOf { selected, .. } => {
                let indices: Vec<usize> = selected
                    .iter()
                    .enumerate()
                    .filter_map(|(i, &s)| if s { Some(i) } else { None })
                    .collect();
                Some(ResponseValue::ChosenVariants(indices))
            }
        }
    }
}

/// The form state for the entire survey.
struct FormState {
    /// Map from response path to field state.
    fields: HashMap<ResponsePath, FieldState>,
    /// Validation errors for each field.
    errors: HashMap<ResponsePath, String>,
    /// Whether the form has been submitted.
    submitted: bool,
    /// Whether the window was closed (cancelled).
    cancelled: bool,
    /// Prelude message.
    prelude: Option<String>,
    /// Epilogue message.
    epilogue: Option<String>,
    /// The survey definition for rendering.
    definition: SurveyDefinition,
}

impl FormState {
    fn new(definition: SurveyDefinition) -> Self {
        let mut state = Self {
            fields: HashMap::new(),
            errors: HashMap::new(),
            submitted: false,
            cancelled: false,
            prelude: definition.prelude.clone(),
            epilogue: definition.epilogue.clone(),
            definition,
        };

        // Initialize field states from the survey definition
        for question in state.definition.questions.clone() {
            state.init_question_state(&question, None);
        }

        state
    }

    fn init_question_state(&mut self, question: &Question, prefix: Option<&ResponsePath>) {
        let path = match prefix {
            Some(p) => p.child(question.path().as_str()),
            None => question.path().clone(),
        };

        // Apply default values if present
        let default_value = question.default().value();

        match question.kind() {
            QuestionKind::Unit => {
                // No state needed for unit types
            }
            QuestionKind::Input(input_q) => {
                let default = default_value
                    .and_then(|v| v.as_str().map(String::from))
                    .or_else(|| input_q.default.clone())
                    .unwrap_or_default();
                self.fields.insert(
                    path,
                    FieldState::Text {
                        value: default,
                        is_password: false,
                        is_multiline: false,
                    },
                );
            }
            QuestionKind::Multiline(multiline_q) => {
                let default = default_value
                    .and_then(|v| v.as_str().map(String::from))
                    .or_else(|| multiline_q.default.clone())
                    .unwrap_or_default();
                self.fields.insert(
                    path,
                    FieldState::Text {
                        value: default,
                        is_password: false,
                        is_multiline: true,
                    },
                );
            }
            QuestionKind::Masked(_) => {
                self.fields.insert(
                    path,
                    FieldState::Text {
                        value: String::new(),
                        is_password: true,
                        is_multiline: false,
                    },
                );
            }
            QuestionKind::Int(int_q) => {
                let default = default_value
                    .and_then(|v| v.as_int())
                    .or(int_q.default)
                    .map(|i| i.to_string())
                    .unwrap_or_default();
                let parsed = default.parse().ok();
                self.fields.insert(
                    path,
                    FieldState::Int {
                        value: default,
                        parsed,
                    },
                );
            }
            QuestionKind::Float(float_q) => {
                let default = default_value
                    .and_then(|v| v.as_float())
                    .or(float_q.default)
                    .map(|f| f.to_string())
                    .unwrap_or_default();
                let parsed = default.parse().ok();
                self.fields.insert(
                    path,
                    FieldState::Float {
                        value: default,
                        parsed,
                    },
                );
            }
            QuestionKind::Confirm(confirm_q) => {
                let default = default_value
                    .and_then(|v| v.as_bool())
                    .unwrap_or(confirm_q.default);
                self.fields
                    .insert(path, FieldState::Bool { value: default });
            }
            QuestionKind::List(list_q) => {
                self.fields.insert(
                    path,
                    FieldState::List {
                        value: String::new(),
                        element_kind: list_q.element_kind.clone(),
                    },
                );
            }
            QuestionKind::OneOf(one_of) => {
                let variants: Vec<String> =
                    one_of.variants.iter().map(|v| v.name.clone()).collect();
                let selected = default_value
                    .and_then(|v| v.as_chosen_variant())
                    .or(one_of.default);
                self.fields
                    .insert(path.clone(), FieldState::OneOf { selected, variants });

                // Initialize nested fields for all variants
                for variant in &one_of.variants {
                    self.init_variant_state(&variant, &path);
                }
            }
            QuestionKind::AnyOf(any_of) => {
                let variants: Vec<String> =
                    any_of.variants.iter().map(|v| v.name.clone()).collect();
                let selected = if let Some(ResponseValue::ChosenVariants(indices)) = default_value {
                    let mut sel = vec![false; variants.len()];
                    for &idx in indices {
                        if idx < sel.len() {
                            sel[idx] = true;
                        }
                    }
                    sel
                } else {
                    let mut sel = vec![false; variants.len()];
                    for &idx in &any_of.defaults {
                        if idx < sel.len() {
                            sel[idx] = true;
                        }
                    }
                    sel
                };
                self.fields
                    .insert(path.clone(), FieldState::AnyOf { selected, variants });

                // Initialize nested fields for all variants (for struct variants)
                for variant in &any_of.variants {
                    self.init_variant_state(&variant, &path);
                }
            }
            QuestionKind::AllOf(all_of) => {
                // Recursively initialize nested questions
                for nested_q in all_of.questions() {
                    self.init_question_state(nested_q, Some(&path));
                }
            }
        }
    }

    fn init_variant_state(&mut self, variant: &Variant, parent_path: &ResponsePath) {
        match &variant.kind {
            QuestionKind::AllOf(all_of) => {
                for nested_q in all_of.questions() {
                    self.init_question_state(nested_q, Some(parent_path));
                }
            }
            QuestionKind::Input(input_q) => {
                let path = parent_path.child(&variant.name);
                self.fields.insert(
                    path,
                    FieldState::Text {
                        value: input_q.default.clone().unwrap_or_default(),
                        is_password: false,
                        is_multiline: false,
                    },
                );
            }
            QuestionKind::Int(int_q) => {
                let path = parent_path.child(&variant.name);
                let default = int_q.default.map(|i| i.to_string()).unwrap_or_default();
                let parsed = default.parse().ok();
                self.fields.insert(
                    path,
                    FieldState::Int {
                        value: default,
                        parsed,
                    },
                );
            }
            QuestionKind::Float(float_q) => {
                let path = parent_path.child(&variant.name);
                let default = float_q.default.map(|f| f.to_string()).unwrap_or_default();
                let parsed = default.parse().ok();
                self.fields.insert(
                    path,
                    FieldState::Float {
                        value: default,
                        parsed,
                    },
                );
            }
            _ => {}
        }
    }

    /// Ensure fields exist for a variant at the given path.
    /// This is called dynamically when AnyOf items are selected.
    fn ensure_variant_fields(&mut self, variant: &Variant, parent_path: &ResponsePath) {
        match &variant.kind {
            QuestionKind::AllOf(all_of) => {
                for nested_q in all_of.questions() {
                    self.ensure_question_fields(nested_q, Some(parent_path));
                }
            }
            QuestionKind::Input(input_q) => {
                let path = parent_path.child(&variant.name);
                if !self.fields.contains_key(&path) {
                    self.fields.insert(
                        path,
                        FieldState::Text {
                            value: input_q.default.clone().unwrap_or_default(),
                            is_password: false,
                            is_multiline: false,
                        },
                    );
                }
            }
            QuestionKind::Int(int_q) => {
                let path = parent_path.child(&variant.name);
                if !self.fields.contains_key(&path) {
                    let default = int_q.default.map(|i| i.to_string()).unwrap_or_default();
                    let parsed = default.parse().ok();
                    self.fields.insert(
                        path,
                        FieldState::Int {
                            value: default,
                            parsed,
                        },
                    );
                }
            }
            QuestionKind::Float(float_q) => {
                let path = parent_path.child(&variant.name);
                if !self.fields.contains_key(&path) {
                    let default = float_q.default.map(|f| f.to_string()).unwrap_or_default();
                    let parsed = default.parse().ok();
                    self.fields.insert(
                        path,
                        FieldState::Float {
                            value: default,
                            parsed,
                        },
                    );
                }
            }
            _ => {}
        }
    }

    /// Ensure fields exist for a question at the given path.
    fn ensure_question_fields(&mut self, question: &Question, prefix: Option<&ResponsePath>) {
        let path = match prefix {
            Some(p) => p.child(question.path().as_str()),
            None => question.path().clone(),
        };

        match question.kind() {
            QuestionKind::Unit => {}
            QuestionKind::Input(input_q) => {
                if !self.fields.contains_key(&path) {
                    self.fields.insert(
                        path,
                        FieldState::Text {
                            value: input_q.default.clone().unwrap_or_default(),
                            is_password: false,
                            is_multiline: false,
                        },
                    );
                }
            }
            QuestionKind::Multiline(multiline_q) => {
                if !self.fields.contains_key(&path) {
                    self.fields.insert(
                        path,
                        FieldState::Text {
                            value: multiline_q.default.clone().unwrap_or_default(),
                            is_password: false,
                            is_multiline: true,
                        },
                    );
                }
            }
            QuestionKind::Masked(_) => {
                if !self.fields.contains_key(&path) {
                    self.fields.insert(
                        path,
                        FieldState::Text {
                            value: String::new(),
                            is_password: true,
                            is_multiline: false,
                        },
                    );
                }
            }
            QuestionKind::Int(int_q) => {
                if !self.fields.contains_key(&path) {
                    let default = int_q.default.map(|i| i.to_string()).unwrap_or_default();
                    let parsed = default.parse().ok();
                    self.fields.insert(
                        path,
                        FieldState::Int {
                            value: default,
                            parsed,
                        },
                    );
                }
            }
            QuestionKind::Float(float_q) => {
                if !self.fields.contains_key(&path) {
                    let default = float_q.default.map(|f| f.to_string()).unwrap_or_default();
                    let parsed = default.parse().ok();
                    self.fields.insert(
                        path,
                        FieldState::Float {
                            value: default,
                            parsed,
                        },
                    );
                }
            }
            QuestionKind::Confirm(confirm_q) => {
                if !self.fields.contains_key(&path) {
                    self.fields.insert(
                        path,
                        FieldState::Bool {
                            value: confirm_q.default,
                        },
                    );
                }
            }
            QuestionKind::List(list_q) => {
                if !self.fields.contains_key(&path) {
                    self.fields.insert(
                        path,
                        FieldState::List {
                            value: String::new(),
                            element_kind: list_q.element_kind.clone(),
                        },
                    );
                }
            }
            QuestionKind::AllOf(all_of) => {
                for nested_q in all_of.questions() {
                    self.ensure_question_fields(nested_q, Some(&path));
                }
            }
            _ => {}
        }
    }

    fn collect_responses(&self) -> Responses {
        let mut responses = Responses::new();

        for question in &self.definition.questions {
            self.collect_question_responses(question, &mut responses, None);
        }

        responses
    }

    fn collect_question_responses(
        &self,
        question: &Question,
        responses: &mut Responses,
        prefix: Option<&ResponsePath>,
    ) {
        let path = match prefix {
            Some(p) => p.child(question.path().as_str()),
            None => question.path().clone(),
        };

        // Check for assumed values
        if let DefaultValue::Assumed(value) = question.default() {
            responses.insert(path.clone(), value.clone());
            return;
        }

        match question.kind() {
            QuestionKind::Unit => {
                // Nothing to collect
            }
            QuestionKind::Input(_) | QuestionKind::Multiline(_) | QuestionKind::Masked(_) => {
                if let Some(field) = self.fields.get(&path) {
                    if let Some(value) = field.to_response_value() {
                        responses.insert(path, value);
                    }
                }
            }
            QuestionKind::Int(_) => {
                if let Some(field) = self.fields.get(&path) {
                    if let Some(value) = field.to_response_value() {
                        responses.insert(path, value);
                    }
                }
            }
            QuestionKind::Float(_) => {
                if let Some(field) = self.fields.get(&path) {
                    if let Some(value) = field.to_response_value() {
                        responses.insert(path, value);
                    }
                }
            }
            QuestionKind::Confirm(_) => {
                if let Some(field) = self.fields.get(&path) {
                    if let Some(value) = field.to_response_value() {
                        responses.insert(path, value);
                    }
                }
            }
            QuestionKind::List(_) => {
                if let Some(field) = self.fields.get(&path) {
                    if let Some(value) = field.to_response_value() {
                        responses.insert(path, value);
                    }
                }
            }
            QuestionKind::OneOf(one_of) => {
                if let Some(FieldState::OneOf {
                    selected: Some(selected),
                    ..
                }) = self.fields.get(&path)
                {
                    let variant_path = path.child(SELECTED_VARIANT_KEY);
                    responses.insert(variant_path, ResponseValue::ChosenVariant(*selected));

                    // Collect nested data for the selected variant
                    let variant = &one_of.variants[*selected];
                    self.collect_variant_responses(variant, &path, responses);
                }
            }
            QuestionKind::AnyOf(any_of) => {
                if let Some(FieldState::AnyOf { selected, .. }) = self.fields.get(&path) {
                    let indices: Vec<usize> = selected
                        .iter()
                        .enumerate()
                        .filter_map(|(i, &s)| if s { Some(i) } else { None })
                        .collect();

                    let variants_path = path.child(SELECTED_VARIANTS_KEY);
                    responses.insert(
                        variants_path,
                        ResponseValue::ChosenVariants(indices.clone()),
                    );

                    // Collect nested data for each selected variant
                    for (item_idx, &variant_idx) in indices.iter().enumerate() {
                        let variant = &any_of.variants[variant_idx];
                        let item_path = path.child(&item_idx.to_string());

                        // Store which variant this item is
                        let item_variant_path = item_path.child(SELECTED_VARIANT_KEY);
                        responses
                            .insert(item_variant_path, ResponseValue::ChosenVariant(variant_idx));

                        // Collect variant data
                        self.collect_variant_responses(variant, &item_path, responses);
                    }
                }
            }
            QuestionKind::AllOf(all_of) => {
                for nested_q in all_of.questions() {
                    self.collect_question_responses(nested_q, responses, Some(&path));
                }
            }
        }
    }

    fn collect_variant_responses(
        &self,
        variant: &Variant,
        parent_path: &ResponsePath,
        responses: &mut Responses,
    ) {
        match &variant.kind {
            QuestionKind::Unit => {}
            QuestionKind::AllOf(all_of) => {
                for nested_q in all_of.questions() {
                    self.collect_question_responses(nested_q, responses, Some(parent_path));
                }
            }
            QuestionKind::Input(_) | QuestionKind::Int(_) | QuestionKind::Float(_) => {
                let path = parent_path.child(&variant.name);
                if let Some(field) = self.fields.get(&path) {
                    if let Some(value) = field.to_response_value() {
                        responses.insert(path, value);
                    }
                }
            }
            _ => {}
        }
    }

    /// Validate that all required fields have values.
    /// Adds errors for empty Int/Float fields.
    fn validate_required_fields(&mut self) {
        for question in self.definition.questions.clone() {
            self.validate_question_required(&question, None);
        }
    }

    fn validate_question_required(&mut self, question: &Question, prefix: Option<&ResponsePath>) {
        let path = match prefix {
            Some(p) => p.child(question.path().as_str()),
            None => question.path().clone(),
        };

        // Skip assumed fields
        if question.is_assumed() {
            return;
        }

        match question.kind() {
            QuestionKind::Int(_) => {
                if let Some(FieldState::Int { parsed, .. }) = self.fields.get(&path) {
                    if parsed.is_none() {
                        self.errors
                            .insert(path, "This field is required".to_string());
                    }
                }
            }
            QuestionKind::Float(_) => {
                if let Some(FieldState::Float { parsed, .. }) = self.fields.get(&path) {
                    if parsed.is_none() {
                        self.errors
                            .insert(path, "This field is required".to_string());
                    }
                }
            }
            QuestionKind::OneOf(one_of) => {
                // Validate that a variant is selected
                if let Some(FieldState::OneOf { selected, .. }) = self.fields.get(&path) {
                    if let Some(idx) = *selected {
                        // Validate nested fields of the selected variant
                        let variant = &one_of.variants[idx];
                        self.validate_variant_required(variant, &path);
                    } else {
                        self.errors
                            .insert(path.clone(), "Please select an option".to_string());
                    }
                }
            }
            QuestionKind::AnyOf(any_of) => {
                // Validate nested fields of selected variants
                if let Some(FieldState::AnyOf { selected, .. }) = self.fields.get(&path) {
                    let indices: Vec<usize> = selected
                        .iter()
                        .enumerate()
                        .filter_map(|(i, &s)| if s { Some(i) } else { None })
                        .collect();
                    for (item_idx, &variant_idx) in indices.iter().enumerate() {
                        let variant = &any_of.variants[variant_idx];
                        let item_path = path.child(&item_idx.to_string());
                        self.validate_variant_required(variant, &item_path);
                    }
                }
            }
            QuestionKind::AllOf(all_of) => {
                for nested_q in all_of.questions() {
                    self.validate_question_required(nested_q, Some(&path));
                }
            }
            _ => {}
        }
    }

    fn validate_variant_required(&mut self, variant: &Variant, parent_path: &ResponsePath) {
        match &variant.kind {
            QuestionKind::AllOf(all_of) => {
                for nested_q in all_of.questions() {
                    self.validate_question_required(nested_q, Some(parent_path));
                }
            }
            QuestionKind::Int(_) => {
                let path = parent_path.child(&variant.name);
                if let Some(FieldState::Int { parsed, .. }) = self.fields.get(&path) {
                    if parsed.is_none() {
                        self.errors
                            .insert(path, "This field is required".to_string());
                    }
                }
            }
            QuestionKind::Float(_) => {
                let path = parent_path.child(&variant.name);
                if let Some(FieldState::Float { parsed, .. }) = self.fields.get(&path) {
                    if parsed.is_none() {
                        self.errors
                            .insert(path, "This field is required".to_string());
                    }
                }
            }
            _ => {}
        }
    }
}

/// The egui application that renders the survey form.
struct SurveyApp {
    state: Arc<Mutex<FormState>>,
    validate: Box<dyn Fn(&ResponseValue, &Responses) -> Result<(), String> + Send>,
}

impl SurveyApp {
    /// Format a prompt as a label, adding a colon only if the prompt doesn't end with punctuation.
    fn format_label(prompt: &str) -> String {
        let trimmed = prompt.trim();
        if trimmed.is_empty() {
            return String::new();
        }
        // If already ends with punctuation, don't add a colon
        let last_char = trimmed.chars().last().unwrap();
        if matches!(last_char, ':' | '?' | '!' | '.') {
            trimmed.to_string()
        } else {
            format!("{trimmed}:")
        }
    }

    fn render_question(
        &self,
        ui: &mut egui::Ui,
        question: &Question,
        state: &mut FormState,
        prefix: Option<&ResponsePath>,
    ) {
        let path = match prefix {
            Some(p) => p.child(question.path().as_str()),
            None => question.path().clone(),
        };

        // Skip assumed questions
        if question.is_assumed() {
            return;
        }

        let prompt = if question.ask().is_empty() {
            // Create a readable label from the path
            path.as_str()
                .split('.')
                .last()
                .unwrap_or("")
                .split('_')
                .map(|word| {
                    let mut chars = word.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(first) => first.to_uppercase().chain(chars).collect(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
        } else {
            question.ask().to_string()
        };

        match question.kind() {
            QuestionKind::Unit => {}
            QuestionKind::Input(_) | QuestionKind::Multiline(_) | QuestionKind::Masked(_) => {
                self.render_text_field(ui, &path, &prompt, question.kind(), state);
            }
            QuestionKind::Int(int_q) => {
                self.render_int_field(ui, &path, &prompt, int_q, state);
            }
            QuestionKind::Float(float_q) => {
                self.render_float_field(ui, &path, &prompt, float_q, state);
            }
            QuestionKind::Confirm(_) => {
                self.render_bool_field(ui, &path, &prompt, state);
            }
            QuestionKind::List(list_q) => {
                self.render_list_field(ui, &path, &prompt, list_q, state);
            }
            QuestionKind::OneOf(one_of) => {
                self.render_one_of(ui, &path, &prompt, one_of, state);
            }
            QuestionKind::AnyOf(any_of) => {
                self.render_any_of(ui, &path, &prompt, any_of, state);
            }
            QuestionKind::AllOf(all_of) => {
                self.render_all_of(ui, &path, &prompt, all_of, state);
            }
        }
    }

    fn render_text_field(
        &self,
        ui: &mut egui::Ui,
        path: &ResponsePath,
        prompt: &str,
        _kind: &QuestionKind,
        state: &mut FormState,
    ) {
        ui.horizontal(|ui| {
            ui.label(Self::format_label(prompt));
        });

        if let Some(FieldState::Text {
            value,
            is_password,
            is_multiline,
        }) = state.fields.get_mut(path)
        {
            let changed;

            if *is_multiline {
                let response = ui.add(
                    egui::TextEdit::multiline(value)
                        .desired_width(f32::INFINITY)
                        .desired_rows(3),
                );
                changed = response.changed();
            } else if *is_password {
                let response = ui.add(egui::TextEdit::singleline(value).password(true));
                changed = response.changed();
            } else {
                let response =
                    ui.add(egui::TextEdit::singleline(value).desired_width(f32::INFINITY));
                changed = response.changed();
            }

            if changed {
                // Validate on change
                let rv = ResponseValue::String(value.clone());
                let responses = state.collect_responses();
                if let Err(msg) = (self.validate)(&rv, &responses) {
                    state.errors.insert(path.clone(), msg);
                } else {
                    state.errors.remove(path);
                }
            }
        }

        // Show error if any
        if let Some(error) = state.errors.get(path) {
            ui.colored_label(egui::Color32::RED, format!("⚠ {error}"));
        }

        ui.add_space(8.0);
    }

    fn render_int_field(
        &self,
        ui: &mut egui::Ui,
        path: &ResponsePath,
        prompt: &str,
        int_q: &IntQuestion,
        state: &mut FormState,
    ) {
        ui.horizontal(|ui| {
            ui.label(Self::format_label(prompt));
            if let (Some(min), Some(max)) = (int_q.min, int_q.max) {
                ui.label(format!("({min} - {max})"));
            } else if let Some(min) = int_q.min {
                ui.label(format!("(min: {min})"));
            } else if let Some(max) = int_q.max {
                ui.label(format!("(max: {max})"));
            }
        });

        if let Some(FieldState::Int { value, parsed }) = state.fields.get_mut(path) {
            let response = ui.add(egui::TextEdit::singleline(value).desired_width(f32::INFINITY));

            if response.changed() {
                *parsed = value.parse().ok();

                if let Some(i) = *parsed {
                    // Clear any previous errors (like "required" or parse errors)
                    state.errors.remove(path);

                    // Check bounds
                    if let Some(min) = int_q.min {
                        if i < min {
                            state
                                .errors
                                .insert(path.clone(), format!("Value must be at least {min}"));
                        }
                    }
                    if let Some(max) = int_q.max {
                        if i > max {
                            state
                                .errors
                                .insert(path.clone(), format!("Value must be at most {max}"));
                        }
                    }

                    // Custom validation
                    if state.errors.get(path).is_none() {
                        let rv = ResponseValue::Int(i);
                        let responses = state.collect_responses();
                        if let Err(msg) = (self.validate)(&rv, &responses) {
                            state.errors.insert(path.clone(), msg);
                        }
                    }
                } else if !value.is_empty() {
                    state
                        .errors
                        .insert(path.clone(), "Please enter a valid integer".to_string());
                }
            }
        }

        if let Some(error) = state.errors.get(path) {
            ui.colored_label(egui::Color32::RED, format!("⚠ {error}"));
        }

        ui.add_space(8.0);
    }

    fn render_float_field(
        &self,
        ui: &mut egui::Ui,
        path: &ResponsePath,
        prompt: &str,
        float_q: &FloatQuestion,
        state: &mut FormState,
    ) {
        ui.horizontal(|ui| {
            ui.label(Self::format_label(prompt));
            if let (Some(min), Some(max)) = (float_q.min, float_q.max) {
                ui.label(format!("({min} - {max})"));
            } else if let Some(min) = float_q.min {
                ui.label(format!("(min: {min})"));
            } else if let Some(max) = float_q.max {
                ui.label(format!("(max: {max})"));
            }
        });

        if let Some(FieldState::Float { value, parsed }) = state.fields.get_mut(path) {
            let response = ui.add(egui::TextEdit::singleline(value).desired_width(f32::INFINITY));

            if response.changed() {
                *parsed = value.parse().ok();

                if let Some(f) = *parsed {
                    // Clear any previous errors (like "required" or parse errors)
                    state.errors.remove(path);

                    if let Some(min) = float_q.min {
                        if f < min {
                            state
                                .errors
                                .insert(path.clone(), format!("Value must be at least {min}"));
                        }
                    }
                    if let Some(max) = float_q.max {
                        if f > max {
                            state
                                .errors
                                .insert(path.clone(), format!("Value must be at most {max}"));
                        }
                    }

                    if state.errors.get(path).is_none() {
                        let rv = ResponseValue::Float(f);
                        let responses = state.collect_responses();
                        if let Err(msg) = (self.validate)(&rv, &responses) {
                            state.errors.insert(path.clone(), msg);
                        }
                    }
                } else if !value.is_empty() {
                    state
                        .errors
                        .insert(path.clone(), "Please enter a valid number".to_string());
                }
            }
        }

        if let Some(error) = state.errors.get(path) {
            ui.colored_label(egui::Color32::RED, format!("⚠ {error}"));
        }

        ui.add_space(8.0);
    }

    fn render_bool_field(
        &self,
        ui: &mut egui::Ui,
        path: &ResponsePath,
        prompt: &str,
        state: &mut FormState,
    ) {
        if let Some(FieldState::Bool { value }) = state.fields.get_mut(path) {
            ui.checkbox(value, prompt);
        }
        ui.add_space(8.0);
    }

    fn render_list_field(
        &self,
        ui: &mut egui::Ui,
        path: &ResponsePath,
        prompt: &str,
        list_q: &ListQuestion,
        state: &mut FormState,
    ) {
        let type_hint = match &list_q.element_kind {
            ListElementKind::String => "strings",
            ListElementKind::Int { .. } => "integers",
            ListElementKind::Float { .. } => "numbers",
        };

        ui.label(Self::format_label(&format!(
            "{} (comma-separated {})",
            prompt, type_hint
        )));

        if let Some(FieldState::List { value, .. }) = state.fields.get_mut(path) {
            let response = ui.add(egui::TextEdit::singleline(value).desired_width(300.0));
            if response.changed() {
                state.errors.remove(path);
            }
        }

        // Show error if any
        if let Some(error) = state.errors.get(path) {
            ui.colored_label(egui::Color32::RED, error);
        }

        ui.add_space(8.0);
    }

    fn render_one_of(
        &self,
        ui: &mut egui::Ui,
        path: &ResponsePath,
        prompt: &str,
        one_of: &OneOfQuestion,
        state: &mut FormState,
    ) {
        ui.label(Self::format_label(prompt));

        let selected = if let Some(FieldState::OneOf { selected, .. }) = state.fields.get(path) {
            *selected
        } else {
            None
        };

        // Render radio buttons
        let mut new_selected = selected;
        for (idx, variant) in one_of.variants.iter().enumerate() {
            if ui.radio(selected == Some(idx), &variant.name).clicked() {
                new_selected = Some(idx);
                // Clear any "required" error when user makes a selection
                state.errors.remove(path);
            }
        }

        if new_selected != selected {
            if let Some(FieldState::OneOf { selected, .. }) = state.fields.get_mut(path) {
                *selected = new_selected;
            }
        }

        // Show nested fields for the selected variant (if any)
        if let Some(idx) = new_selected {
            let variant = &one_of.variants[idx];
            self.render_variant_fields(ui, variant, path, state);
        }

        // Show error if no selection
        if let Some(error) = state.errors.get(path) {
            ui.colored_label(egui::Color32::RED, format!("⚠ {error}"));
        }

        ui.add_space(8.0);
    }

    fn render_any_of(
        &self,
        ui: &mut egui::Ui,
        path: &ResponsePath,
        prompt: &str,
        any_of: &AnyOfQuestion,
        state: &mut FormState,
    ) {
        ui.label(Self::format_label(prompt));

        // Get current selection state
        let selections = if let Some(FieldState::AnyOf { selected, .. }) = state.fields.get(path) {
            selected.clone()
        } else {
            vec![false; any_of.variants.len()]
        };

        // Render checkboxes
        let mut new_selections = selections.clone();
        for (idx, variant) in any_of.variants.iter().enumerate() {
            let mut checked = selections.get(idx).copied().unwrap_or(false);
            if ui.checkbox(&mut checked, &variant.name).changed() {
                if idx < new_selections.len() {
                    new_selections[idx] = checked;
                }
            }
        }

        // Update state if changed
        if new_selections != selections {
            if let Some(FieldState::AnyOf { selected, .. }) = state.fields.get_mut(path) {
                *selected = new_selections.clone();
            }

            // Validate selection
            let indices: Vec<usize> = new_selections
                .iter()
                .enumerate()
                .filter_map(|(i, &s)| if s { Some(i) } else { None })
                .collect();
            let rv = ResponseValue::ChosenVariants(indices);
            let responses = state.collect_responses();
            if let Err(msg) = (self.validate)(&rv, &responses) {
                state.errors.insert(path.clone(), msg);
            } else {
                state.errors.remove(path);
            }
        }

        // Show error if any
        if let Some(error) = state.errors.get(path) {
            ui.colored_label(egui::Color32::RED, format!("⚠ {error}"));
        }

        // Show nested fields for selected variants with data
        let mut item_idx = 0;
        for (variant_idx, variant) in any_of.variants.iter().enumerate() {
            if new_selections.get(variant_idx).copied().unwrap_or(false) {
                if !matches!(variant.kind, QuestionKind::Unit) {
                    let item_path = path.child(&item_idx.to_string());
                    ui.separator();
                    ui.label(format!("{}:", variant.name));
                    ui.indent(format!("anyof_{item_idx}"), |ui| {
                        // Ensure fields exist for this item
                        state.ensure_variant_fields(variant, &item_path);
                        self.render_variant_fields(ui, variant, &item_path, state);
                    });
                }
                item_idx += 1;
            }
        }

        ui.add_space(8.0);
    }

    fn render_all_of(
        &self,
        ui: &mut egui::Ui,
        path: &ResponsePath,
        prompt: &str,
        all_of: &AllOfQuestion,
        state: &mut FormState,
    ) {
        if !prompt.is_empty() {
            ui.separator();
            ui.strong(prompt);
        }

        ui.indent(path.as_str(), |ui| {
            for nested_q in all_of.questions() {
                self.render_question(ui, nested_q, state, Some(path));
            }
        });
    }

    fn render_variant_fields(
        &self,
        ui: &mut egui::Ui,
        variant: &Variant,
        parent_path: &ResponsePath,
        state: &mut FormState,
    ) {
        match &variant.kind {
            QuestionKind::Unit => {}
            QuestionKind::AllOf(all_of) => {
                ui.indent(format!("variant_{}", variant.name), |ui| {
                    for nested_q in all_of.questions() {
                        self.render_question(ui, nested_q, state, Some(parent_path));
                    }
                });
            }
            QuestionKind::Input(_) => {
                let path = parent_path.child(&variant.name);
                self.render_text_field(ui, &path, "", &variant.kind, state);
            }
            QuestionKind::Int(int_q) => {
                let path = parent_path.child(&variant.name);
                self.render_int_field(ui, &path, "", int_q, state);
            }
            QuestionKind::Float(float_q) => {
                let path = parent_path.child(&variant.name);
                self.render_float_field(ui, &path, "", float_q, state);
            }
            _ => {}
        }
    }
}

impl eframe::App for SurveyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let mut state = self.state.lock().unwrap();

            // Show prelude if present
            if let Some(prelude) = &state.prelude {
                ui.label(prelude);
                ui.separator();
            }

            egui::ScrollArea::vertical().show(ui, |ui| {
                // Render all questions
                for question in state.definition.questions.clone() {
                    self.render_question(ui, &question, &mut state, None);
                }

                ui.separator();

                // Show epilogue if present
                if let Some(epilogue) = &state.epilogue {
                    ui.label(epilogue);
                    ui.add_space(8.0);
                }

                // Submit button
                ui.horizontal(|ui| {
                    let has_errors = !state.errors.is_empty();

                    if ui
                        .add_enabled(!has_errors, egui::Button::new("Submit"))
                        .clicked()
                    {
                        // Check for empty required fields first
                        state.validate_required_fields();

                        if state.errors.is_empty() {
                            // Final validation of all fields
                            let responses = state.collect_responses();
                            let mut all_valid = true;

                            for (path, value) in responses.iter() {
                                if let Err(msg) = (self.validate)(value, &responses) {
                                    state.errors.insert(path.clone(), msg);
                                    all_valid = false;
                                }
                            }

                            if all_valid {
                                state.submitted = true;
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                        }
                    }

                    if ui.button("Cancel").clicked() {
                        state.cancelled = true;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }

                    if has_errors || !state.errors.is_empty() {
                        ui.colored_label(
                            egui::Color32::RED,
                            format!("{} validation error(s)", state.errors.len()),
                        );
                    }
                });
            });
        });
    }
}

impl SurveyBackend for EguiBackend {
    type Error = EguiError;

    fn collect(
        &self,
        definition: &SurveyDefinition,
        _validate: &dyn Fn(&ResponseValue, &Responses, &ResponsePath) -> Result<(), String>,
    ) -> Result<Responses, Self::Error> {
        let state = Arc::new(Mutex::new(FormState::new(definition.clone())));

        // Create native options
        let options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_title(self.title.clone())
                .with_inner_size(self.window_size),
            ..Default::default()
        };

        // Run the egui app
        // Note: eframe::run_native blocks until the window is closed
        let app_state = Arc::clone(&state);

        // We need to handle the validation in a way that works with eframe
        // Since eframe takes ownership, we'll use a closure that captures what we need
        let title = self.title.clone();

        eframe::run_native(
            &title,
            options,
            Box::new(move |_cc| {
                // Create a validation function that always succeeds for now
                // Real validation happens on submit
                let validate_fn: Box<
                    dyn Fn(&ResponseValue, &Responses) -> Result<(), String> + Send,
                > = Box::new(|_value, _responses| Ok(()));

                Ok(Box::new(SurveyApp {
                    state: app_state,
                    validate: validate_fn,
                }) as Box<dyn eframe::App>)
            }),
        )
        .map_err(|e| EguiError::EguiError(e.to_string()))?;

        // After the window closes, check the result
        let state = state.lock().unwrap();
        if state.cancelled {
            return Err(EguiError::Cancelled);
        }

        if !state.submitted {
            return Err(EguiError::Cancelled);
        }

        Ok(state.collect_responses())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_creation() {
        let _backend = EguiBackend::new();
        let _with_title = EguiBackend::new().with_title("Test");
        let _with_size = EguiBackend::new().with_window_size([800.0, 600.0]);
        let _default = EguiBackend::default();
    }

    #[test]
    fn error_types() {
        let err = EguiError::Cancelled;
        assert_eq!(err.to_string(), "Survey cancelled by user");

        let err = EguiError::EguiError("test error".to_string());
        assert_eq!(err.to_string(), "Egui error: test error");
    }
}
