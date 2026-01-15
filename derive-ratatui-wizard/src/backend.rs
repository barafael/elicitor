//! Ratatui backend implementation for SurveyBackend trait.
//!
//! Provides a rich terminal UI with panels, progress indicators,
//! and keyboard navigation for wizard-style surveys.

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use derive_survey::{
    DefaultValue, ListElementKind, Question, QuestionKind, ResponsePath, ResponseValue, Responses,
    SELECTED_VARIANT_KEY, SELECTED_VARIANTS_KEY, SurveyBackend, SurveyDefinition,
};

/// Helper function to get the parent path by stripping the last segment.
fn parent_path(path: &ResponsePath) -> ResponsePath {
    let path_str = path.as_str();
    if let Some(last_dot) = path_str.rfind('.') {
        ResponsePath::new(&path_str[..last_dot])
    } else {
        ResponsePath::empty()
    }
}
use ratatui::{
    Frame, Terminal,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::CrosstermBackend,
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};
use std::io::{self, Stdout};
use thiserror::Error;

/// Error type for the Ratatui backend.
#[derive(Debug, Error)]
pub enum RatatuiError {
    /// User cancelled the survey (e.g., pressed Esc).
    #[error("Survey cancelled by user")]
    Cancelled,

    /// An I/O error occurred.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Terminal setup/restore error.
    #[error("Terminal error: {0}")]
    Terminal(String),
}

/// Color theme for the TUI.
#[derive(Debug, Clone)]
pub struct Theme {
    pub primary: Color,
    pub secondary: Color,
    pub background: Color,
    pub text: Color,
    pub highlight: Color,
    pub error: Color,
    pub success: Color,
    pub border: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            primary: Color::Cyan,
            secondary: Color::Blue,
            background: Color::Reset,
            text: Color::White,
            highlight: Color::Yellow,
            error: Color::Red,
            success: Color::Green,
            border: Color::Gray,
        }
    }
}

/// Ratatui-based TUI backend with rich visual interface.
///
/// This backend presents questions one at a time in a styled terminal UI
/// with progress tracking, keyboard navigation, and visual feedback.
#[derive(Debug, Clone)]
pub struct RatatuiBackend {
    /// Title shown at the top of the wizard.
    title: String,
    /// Color theme for the UI.
    theme: Theme,
}

impl Default for RatatuiBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl RatatuiBackend {
    /// Create a new Ratatui backend with default settings.
    pub fn new() -> Self {
        Self {
            title: "Survey".to_string(),
            theme: Theme::default(),
        }
    }

    /// Set the title shown at the top of the wizard.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Set a custom color theme.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    fn setup_terminal(&self) -> Result<Terminal<CrosstermBackend<Stdout>>, RatatuiError> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(terminal)
    }

    fn restore_terminal(
        &self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<(), RatatuiError> {
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;
        Ok(())
    }
}

/// State for the entire wizard.
struct WizardState {
    /// All flattened questions.
    questions: Vec<FlatQuestion>,
    /// Current question index.
    current_index: usize,
    /// Collected responses.
    responses: Responses,
    /// Current input buffer.
    input: String,
    /// Cursor position in input.
    cursor_pos: usize,
    /// For select/confirm questions: selected option index.
    selected_option: usize,
    /// For multi-select questions: which options are selected.
    multi_selected: Vec<bool>,
    /// Current validation error message.
    error_message: Option<String>,
    /// Whether wizard is complete.
    complete: bool,
    /// Whether user cancelled.
    cancelled: bool,
    /// Theme.
    theme: Theme,
    /// Title.
    title: String,
    /// Epilogue text.
    epilogue: Option<String>,
}

/// A flattened question for easier processing.
#[derive(Clone)]
struct FlatQuestion {
    /// Response path for this question.
    path: ResponsePath,
    /// Prompt text.
    prompt: String,
    /// Question kind.
    kind: FlatQuestionKind,
    /// Default value (string representation).
    default_value: Option<String>,
    /// Whether this question should be skipped (assumed value).
    assumed: Option<ResponseValue>,
    /// Whether this field has custom validation.
    has_validation: bool,
}

#[derive(Clone)]
enum FlatQuestionKind {
    Input,
    Multiline,
    Masked,
    Int {
        min: Option<i64>,
        max: Option<i64>,
    },
    Float {
        min: Option<f64>,
        max: Option<f64>,
    },
    Confirm {
        default: bool,
    },
    List {
        element_kind: ListElementKind,
    },
    Select {
        options: Vec<String>,
        default_idx: usize,
        /// For enum variants: the variants with their nested questions.
        variants: Option<Vec<derive_survey::Variant>>,
    },
    MultiSelect {
        options: Vec<String>,
        defaults: Vec<usize>,
        /// For AnyOf: the variants.
        variants: Option<Vec<derive_survey::Variant>>,
    },
}

impl WizardState {
    fn new(definition: &SurveyDefinition, theme: Theme, title: String) -> Self {
        let questions = Self::flatten_questions(definition.questions(), &ResponsePath::empty());

        // If there's a prelude, include it in the title
        let display_title = if let Some(ref prelude) = definition.prelude {
            format!("{}\n{}", title, prelude)
        } else {
            title
        };

        // Initialize state for the first question
        let (selected_option, multi_selected) = if let Some(first) = questions.first() {
            match &first.kind {
                FlatQuestionKind::MultiSelect {
                    options, defaults, ..
                } => {
                    let mut selected = vec![false; options.len()];
                    for &idx in defaults {
                        if idx < selected.len() {
                            selected[idx] = true;
                        }
                    }
                    (0, selected)
                }
                FlatQuestionKind::Select { default_idx, .. } => (*default_idx, Vec::new()),
                FlatQuestionKind::Confirm { default } => (if *default { 0 } else { 1 }, Vec::new()),
                _ => (0, Vec::new()),
            }
        } else {
            (0, Vec::new())
        };

        Self {
            questions,
            current_index: 0,
            responses: Responses::new(),
            input: String::new(),
            cursor_pos: 0,
            selected_option,
            multi_selected,
            error_message: None,
            complete: false,
            cancelled: false,
            theme,
            title: display_title,
            epilogue: definition.epilogue.clone(),
        }
    }

    fn flatten_questions(questions: &[Question], prefix: &ResponsePath) -> Vec<FlatQuestion> {
        let mut flat = Vec::new();

        for question in questions {
            let path = if prefix.is_empty() {
                question.path().clone()
            } else {
                prefix.child(question.path().as_str())
            };

            // Check for assumed value
            let assumed = if let DefaultValue::Assumed(val) = question.default() {
                Some(val.clone())
            } else {
                None
            };

            match question.kind() {
                QuestionKind::Unit => {
                    // No data to collect for unit types
                }
                QuestionKind::Input(input_q) => {
                    let default_value = match question.default() {
                        DefaultValue::Suggested(ResponseValue::String(s)) => Some(s.clone()),
                        _ => input_q.default.clone(),
                    };
                    flat.push(FlatQuestion {
                        path,
                        prompt: question.ask().to_string(),
                        kind: FlatQuestionKind::Input,
                        default_value,
                        assumed,
                        has_validation: input_q.validate.is_some(),
                    });
                }
                QuestionKind::Multiline(ml_q) => {
                    let default_value = match question.default() {
                        DefaultValue::Suggested(ResponseValue::String(s)) => Some(s.clone()),
                        _ => ml_q.default.clone(),
                    };
                    flat.push(FlatQuestion {
                        path,
                        prompt: question.ask().to_string(),
                        kind: FlatQuestionKind::Multiline,
                        default_value,
                        assumed,
                        has_validation: ml_q.validate.is_some(),
                    });
                }
                QuestionKind::Masked(masked_q) => {
                    flat.push(FlatQuestion {
                        path,
                        prompt: question.ask().to_string(),
                        kind: FlatQuestionKind::Masked,
                        default_value: None,
                        assumed,
                        has_validation: masked_q.validate.is_some(),
                    });
                }
                QuestionKind::Int(int_q) => {
                    let default_value = match question.default() {
                        DefaultValue::Suggested(ResponseValue::Int(i)) => Some(i.to_string()),
                        _ => int_q.default.map(|d| d.to_string()),
                    };
                    flat.push(FlatQuestion {
                        path,
                        prompt: question.ask().to_string(),
                        kind: FlatQuestionKind::Int {
                            min: int_q.min,
                            max: int_q.max,
                        },
                        default_value,
                        assumed,
                        has_validation: int_q.validate.is_some(),
                    });
                }
                QuestionKind::Float(float_q) => {
                    let default_value = match question.default() {
                        DefaultValue::Suggested(ResponseValue::Float(f)) => Some(f.to_string()),
                        _ => float_q.default.map(|d| d.to_string()),
                    };
                    flat.push(FlatQuestion {
                        path,
                        prompt: question.ask().to_string(),
                        kind: FlatQuestionKind::Float {
                            min: float_q.min,
                            max: float_q.max,
                        },
                        default_value,
                        assumed,
                        has_validation: float_q.validate.is_some(),
                    });
                }
                QuestionKind::Confirm(confirm_q) => {
                    let default = match question.default() {
                        DefaultValue::Suggested(ResponseValue::Bool(b)) => *b,
                        _ => confirm_q.default,
                    };
                    flat.push(FlatQuestion {
                        path,
                        prompt: question.ask().to_string(),
                        kind: FlatQuestionKind::Confirm { default },
                        default_value: Some(if default { "yes" } else { "no" }.to_string()),
                        assumed,
                        has_validation: false,
                    });
                }
                QuestionKind::List(list_q) => {
                    flat.push(FlatQuestion {
                        path,
                        prompt: question.ask().to_string(),
                        kind: FlatQuestionKind::List {
                            element_kind: list_q.element_kind.clone(),
                        },
                        default_value: None,
                        assumed,
                        has_validation: list_q.validate.is_some(),
                    });
                }
                QuestionKind::OneOf(one_of) => {
                    let options: Vec<String> =
                        one_of.variants.iter().map(|v| v.name.clone()).collect();
                    let default_idx = one_of.default.unwrap_or(0);

                    flat.push(FlatQuestion {
                        path: path.child(SELECTED_VARIANT_KEY),
                        prompt: question.ask().to_string(),
                        kind: FlatQuestionKind::Select {
                            options,
                            default_idx,
                            variants: Some(one_of.variants.clone()),
                        },
                        default_value: None,
                        assumed,
                        has_validation: false,
                    });
                }
                QuestionKind::AnyOf(any_of) => {
                    let options: Vec<String> =
                        any_of.variants.iter().map(|v| v.name.clone()).collect();

                    flat.push(FlatQuestion {
                        path: path.child(SELECTED_VARIANTS_KEY),
                        prompt: question.ask().to_string(),
                        kind: FlatQuestionKind::MultiSelect {
                            options,
                            defaults: any_of.defaults.clone(),
                            variants: Some(any_of.variants.clone()),
                        },
                        default_value: None,
                        assumed,
                        has_validation: false,
                    });
                }
                QuestionKind::AllOf(all_of) => {
                    // Recursively flatten nested questions
                    let mut nested = Self::flatten_questions(all_of.questions(), &path);

                    // If parent has a prompt and the first nested question has an empty prompt,
                    // propagate the parent's prompt to the first nested question.
                    // This handles the case of enum fields where the #[ask(...)] is on the
                    // struct field but the enum generates a OneOf with an empty prompt.
                    let parent_prompt = question.ask();
                    if !parent_prompt.is_empty() {
                        if let Some(first) = nested.first_mut() {
                            if first.prompt.is_empty() {
                                first.prompt = parent_prompt.to_string();
                            }
                        }
                    }

                    flat.extend(nested);
                }
            }
        }

        flat
    }

    fn current_question(&self) -> Option<&FlatQuestion> {
        self.questions.get(self.current_index)
    }

    fn progress(&self) -> (usize, usize) {
        (self.current_index + 1, self.questions.len())
    }

    fn handle_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char(c) => {
                self.input.insert(self.cursor_pos, c);
                self.cursor_pos += 1;
                self.error_message = None;
            }
            KeyCode::Backspace => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                    self.input.remove(self.cursor_pos);
                    self.error_message = None;
                }
            }
            KeyCode::Delete => {
                if self.cursor_pos < self.input.len() {
                    self.input.remove(self.cursor_pos);
                    self.error_message = None;
                }
            }
            KeyCode::Left => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                }
            }
            KeyCode::Right => {
                if self.cursor_pos < self.input.len() {
                    self.cursor_pos += 1;
                }
            }
            KeyCode::Home => {
                self.cursor_pos = 0;
            }
            KeyCode::End => {
                self.cursor_pos = self.input.len();
            }
            _ => {}
        }
    }

    fn validate_and_submit(
        &mut self,
        validate: &dyn Fn(&ResponseValue, &Responses, &ResponsePath) -> Result<(), String>,
    ) -> bool {
        let Some(question) = self.current_question().cloned() else {
            return false;
        };

        // Use default if input is empty and default exists
        let value = if self.input.is_empty() {
            question.default_value.clone().unwrap_or_default()
        } else {
            self.input.clone()
        };

        // Remove the current field's old value before validation.
        // This is important when going back and re-entering a value,
        // so cross-field validators don't count both old and new values.
        let old_value = self.responses.remove(&question.path);

        match &question.kind {
            FlatQuestionKind::Input | FlatQuestionKind::Multiline | FlatQuestionKind::Masked => {
                let rv = ResponseValue::String(value.clone());
                // Run validation if field has it
                if question.has_validation {
                    if let Err(err) = validate(&rv, &self.responses, &question.path) {
                        self.error_message = Some(err);
                        // Restore old value on validation failure
                        if let Some(old) = old_value {
                            self.responses.insert(question.path.clone(), old);
                        }
                        return false;
                    }
                }
                self.responses.insert(question.path.clone(), rv);
            }
            FlatQuestionKind::Int { min, max } => match value.parse::<i64>() {
                Ok(n) => {
                    if let Some(min_val) = min
                        && n < *min_val
                    {
                        self.error_message = Some(format!("Value must be at least {}", min_val));
                        // Restore old value on validation failure
                        if let Some(old) = old_value {
                            self.responses.insert(question.path.clone(), old);
                        }
                        return false;
                    }
                    if let Some(max_val) = max
                        && n > *max_val
                    {
                        self.error_message = Some(format!("Value must be at most {}", max_val));
                        // Restore old value on validation failure
                        if let Some(old) = old_value {
                            self.responses.insert(question.path.clone(), old);
                        }
                        return false;
                    }
                    let rv = ResponseValue::Int(n);
                    if question.has_validation {
                        if let Err(err) = validate(&rv, &self.responses, &question.path) {
                            self.error_message = Some(err);
                            // Restore old value on validation failure
                            if let Some(old) = old_value {
                                self.responses.insert(question.path.clone(), old);
                            }
                            return false;
                        }
                    }
                    self.responses.insert(question.path.clone(), rv);
                }
                Err(_) => {
                    self.error_message = Some("Please enter a valid integer".to_string());
                    // Restore old value on validation failure
                    if let Some(old) = old_value {
                        self.responses.insert(question.path.clone(), old);
                    }
                    return false;
                }
            },
            FlatQuestionKind::Float { min, max } => match value.parse::<f64>() {
                Ok(n) => {
                    if let Some(min_val) = min
                        && n < *min_val
                    {
                        self.error_message = Some(format!("Value must be at least {}", min_val));
                        // Restore old value on validation failure
                        if let Some(old) = old_value {
                            self.responses.insert(question.path.clone(), old);
                        }
                        return false;
                    }
                    if let Some(max_val) = max
                        && n > *max_val
                    {
                        self.error_message = Some(format!("Value must be at most {}", max_val));
                        // Restore old value on validation failure
                        if let Some(old) = old_value {
                            self.responses.insert(question.path.clone(), old);
                        }
                        return false;
                    }
                    let rv = ResponseValue::Float(n);
                    if question.has_validation {
                        if let Err(err) = validate(&rv, &self.responses, &question.path) {
                            self.error_message = Some(err);
                            // Restore old value on validation failure
                            if let Some(old) = old_value {
                                self.responses.insert(question.path.clone(), old);
                            }
                            return false;
                        }
                    }
                    self.responses.insert(question.path.clone(), rv);
                }
                Err(_) => {
                    self.error_message = Some("Please enter a valid number".to_string());
                    // Restore old value on validation failure
                    if let Some(old) = old_value {
                        self.responses.insert(question.path.clone(), old);
                    }
                    return false;
                }
            },
            FlatQuestionKind::Confirm { .. } => {
                let answer = self.selected_option == 0; // 0 = Yes, 1 = No
                self.responses
                    .insert(question.path.clone(), ResponseValue::Bool(answer));
            }
            FlatQuestionKind::List { element_kind } => {
                // Parse the input as a list (comma or newline separated)
                let items: Vec<&str> = self
                    .input
                    .split(|c| c == ',' || c == '\n')
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .collect();

                let rv = match element_kind {
                    ListElementKind::String => {
                        ResponseValue::StringList(items.iter().map(|s| s.to_string()).collect())
                    }
                    ListElementKind::Int { min, max } => {
                        let mut ints = Vec::new();
                        for item in &items {
                            match item.parse::<i64>() {
                                Ok(n) => {
                                    if let Some(min_val) = min {
                                        if n < *min_val {
                                            self.error_message = Some(format!(
                                                "Value {} must be at least {}",
                                                n, min_val
                                            ));
                                            if let Some(old) = old_value {
                                                self.responses.insert(question.path.clone(), old);
                                            }
                                            return false;
                                        }
                                    }
                                    if let Some(max_val) = max {
                                        if n > *max_val {
                                            self.error_message = Some(format!(
                                                "Value {} must be at most {}",
                                                n, max_val
                                            ));
                                            if let Some(old) = old_value {
                                                self.responses.insert(question.path.clone(), old);
                                            }
                                            return false;
                                        }
                                    }
                                    ints.push(n);
                                }
                                Err(_) => {
                                    self.error_message =
                                        Some(format!("'{}' is not a valid integer", item));
                                    if let Some(old) = old_value {
                                        self.responses.insert(question.path.clone(), old);
                                    }
                                    return false;
                                }
                            }
                        }
                        ResponseValue::IntList(ints)
                    }
                    ListElementKind::Float { min, max } => {
                        let mut floats = Vec::new();
                        for item in &items {
                            match item.parse::<f64>() {
                                Ok(n) => {
                                    if let Some(min_val) = min {
                                        if n < *min_val {
                                            self.error_message = Some(format!(
                                                "Value {} must be at least {}",
                                                n, min_val
                                            ));
                                            if let Some(old) = old_value {
                                                self.responses.insert(question.path.clone(), old);
                                            }
                                            return false;
                                        }
                                    }
                                    if let Some(max_val) = max {
                                        if n > *max_val {
                                            self.error_message = Some(format!(
                                                "Value {} must be at most {}",
                                                n, max_val
                                            ));
                                            if let Some(old) = old_value {
                                                self.responses.insert(question.path.clone(), old);
                                            }
                                            return false;
                                        }
                                    }
                                    floats.push(n);
                                }
                                Err(_) => {
                                    self.error_message =
                                        Some(format!("'{}' is not a valid number", item));
                                    if let Some(old) = old_value {
                                        self.responses.insert(question.path.clone(), old);
                                    }
                                    return false;
                                }
                            }
                        }
                        ResponseValue::FloatList(floats)
                    }
                };

                if question.has_validation {
                    if let Err(err) = validate(&rv, &self.responses, &question.path) {
                        self.error_message = Some(err);
                        if let Some(old) = old_value {
                            self.responses.insert(question.path.clone(), old);
                        }
                        return false;
                    }
                }
                self.responses.insert(question.path.clone(), rv);
            }
            FlatQuestionKind::Select { variants, .. } => {
                // Get the base path (strip the selected_variant suffix)
                let base_path = parent_path(&question.path);

                // Check if we previously selected a different variant
                let old_variant_idx = if let Some(ResponseValue::ChosenVariant(idx)) = old_value {
                    Some(idx)
                } else {
                    None
                };

                // If changing variants, remove old variant's dynamically-inserted questions
                // and clear their responses
                if old_variant_idx.is_some() && old_variant_idx != Some(self.selected_option) {
                    // Remove questions that were dynamically inserted for the old variant
                    // These are questions after current_index whose path starts with base_path
                    let base_path_str = base_path.as_str();
                    let current_path_str = question.path.as_str();

                    // Collect paths of questions to remove and remove the questions
                    let i = self.current_index + 1;
                    while i < self.questions.len() {
                        let q_path = self.questions[i].path.as_str();
                        // Remove if path starts with base_path but is not the select question itself
                        if q_path.starts_with(base_path_str) && q_path != current_path_str {
                            // Also remove the response for this question
                            self.responses.remove(&self.questions[i].path);
                            self.questions.remove(i);
                        } else {
                            // Stop when we hit a question outside this enum's scope
                            break;
                        }
                    }
                }

                // Store the selected variant index
                self.responses.insert(
                    question.path.clone(),
                    ResponseValue::ChosenVariant(self.selected_option),
                );

                // For enum variants: expand the selected variant's fields
                if let Some(vars) = variants
                    && let Some(selected_variant) = vars.get(self.selected_option)
                {
                    // Flatten the variant's nested questions and insert after current
                    match &selected_variant.kind {
                        QuestionKind::AllOf(all_of) => {
                            let variant_questions =
                                Self::flatten_questions(all_of.questions(), &base_path);
                            if !variant_questions.is_empty() {
                                let insert_pos = self.current_index + 1;
                                for (i, q) in variant_questions.into_iter().enumerate() {
                                    self.questions.insert(insert_pos + i, q);
                                }
                            }
                        }
                        QuestionKind::Unit => {
                            // No follow-up questions needed
                        }
                        _ => {
                            // Handle single-value variants (Input, Int, etc.)
                            if !selected_variant.kind.is_unit() {
                                let variant_q = FlatQuestion {
                                    path: base_path.child(&selected_variant.name),
                                    prompt: format!("Enter {} value:", selected_variant.name),
                                    kind: match &selected_variant.kind {
                                        QuestionKind::Input(_) => FlatQuestionKind::Input,
                                        QuestionKind::Int(iq) => FlatQuestionKind::Int {
                                            min: iq.min,
                                            max: iq.max,
                                        },
                                        QuestionKind::Float(fq) => FlatQuestionKind::Float {
                                            min: fq.min,
                                            max: fq.max,
                                        },
                                        QuestionKind::Confirm(cq) => FlatQuestionKind::Confirm {
                                            default: cq.default,
                                        },
                                        _ => FlatQuestionKind::Input,
                                    },
                                    default_value: None,
                                    assumed: None,
                                    has_validation: false,
                                };
                                self.questions.insert(self.current_index + 1, variant_q);
                            }
                        }
                    }
                }
            }
            FlatQuestionKind::MultiSelect { variants, .. } => {
                // Collect indices of all selected options
                let selected_indices: Vec<usize> = self
                    .multi_selected
                    .iter()
                    .enumerate()
                    .filter_map(|(i, &selected)| if selected { Some(i) } else { None })
                    .collect();

                self.responses.insert(
                    question.path.clone(),
                    ResponseValue::ChosenVariants(selected_indices.clone()),
                );

                // Get base path (strip selected_variants suffix)
                let base_path = parent_path(&question.path);

                // For each selected variant, add follow-up questions
                if let Some(vars) = variants {
                    let mut insert_offset = 1;
                    for (item_idx, &variant_idx) in selected_indices.iter().enumerate() {
                        if let Some(variant) = vars.get(variant_idx) {
                            let item_path = base_path.child(&item_idx.to_string());

                            // Store which variant this item is
                            self.responses.insert(
                                item_path.child(SELECTED_VARIANT_KEY),
                                ResponseValue::ChosenVariant(variant_idx),
                            );

                            // Add follow-up questions for this item
                            if let QuestionKind::AllOf(all_of) = &variant.kind {
                                let variant_questions =
                                    Self::flatten_questions(all_of.questions(), &item_path);
                                for q in variant_questions {
                                    self.questions.insert(self.current_index + insert_offset, q);
                                    insert_offset += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        true
    }

    fn next_question(
        &mut self,
        validate: &dyn Fn(&ResponseValue, &Responses, &ResponsePath) -> Result<(), String>,
    ) {
        if self.validate_and_submit(validate) {
            self.current_index += 1;
            self.input.clear();
            self.cursor_pos = 0;
            self.selected_option = 0;
            self.multi_selected.clear();
            self.error_message = None;

            // Skip assumed questions
            while self.current_index < self.questions.len() {
                if let Some(assumed) = &self.questions[self.current_index].assumed {
                    self.responses.insert(
                        self.questions[self.current_index].path.clone(),
                        assumed.clone(),
                    );
                    self.current_index += 1;
                } else {
                    // Set default selection for select/confirm/multiselect questions
                    if let Some(q) = self.current_question() {
                        match &q.kind {
                            FlatQuestionKind::Confirm { default } => {
                                self.selected_option = if *default { 0 } else { 1 };
                            }
                            FlatQuestionKind::Select { default_idx, .. } => {
                                self.selected_option = *default_idx;
                            }
                            FlatQuestionKind::MultiSelect {
                                options, defaults, ..
                            } => {
                                let opts_len = options.len();
                                let defs = defaults.clone();
                                self.multi_selected = vec![false; opts_len];
                                for idx in defs {
                                    if idx < self.multi_selected.len() {
                                        self.multi_selected[idx] = true;
                                    }
                                }
                                self.selected_option = 0;
                            }
                            _ => {
                                // Pre-fill with default value if available
                                if let Some(def) = &q.default_value {
                                    self.input = def.clone();
                                    self.cursor_pos = self.input.len();
                                }
                            }
                        }
                    }
                    break;
                }
            }

            if self.current_index >= self.questions.len() {
                self.complete = true;
            }
        }
    }

    fn prev_question(&mut self) {
        if self.current_index > 0 {
            self.current_index -= 1;
            self.input.clear();
            self.cursor_pos = 0;
            self.multi_selected.clear();
            self.error_message = None;

            // Restore previous response as input
            if let Some(q) = self.current_question() {
                if let Some(prev_response) = self.responses.get(&q.path) {
                    match prev_response {
                        ResponseValue::String(s) => {
                            self.input = s.clone();
                            self.cursor_pos = self.input.len();
                        }
                        ResponseValue::Int(n) => {
                            self.input = n.to_string();
                            self.cursor_pos = self.input.len();
                        }
                        ResponseValue::Float(n) => {
                            self.input = n.to_string();
                            self.cursor_pos = self.input.len();
                        }
                        ResponseValue::Bool(b) => {
                            self.selected_option = if *b { 0 } else { 1 };
                        }
                        ResponseValue::ChosenVariant(idx) => {
                            self.selected_option = *idx;
                        }
                        ResponseValue::ChosenVariants(indices) => {
                            if let FlatQuestionKind::MultiSelect { options, .. } = &q.kind {
                                self.multi_selected = vec![false; options.len()];
                                for &idx in indices {
                                    if idx < self.multi_selected.len() {
                                        self.multi_selected[idx] = true;
                                    }
                                }
                            }
                        }
                        ResponseValue::StringList(list) => {
                            self.input = list.join(", ");
                            self.cursor_pos = self.input.len();
                        }
                        ResponseValue::IntList(list) => {
                            self.input = list
                                .iter()
                                .map(|n| n.to_string())
                                .collect::<Vec<_>>()
                                .join(", ");
                            self.cursor_pos = self.input.len();
                        }
                        ResponseValue::FloatList(list) => {
                            self.input = list
                                .iter()
                                .map(|n| n.to_string())
                                .collect::<Vec<_>>()
                                .join(", ");
                            self.cursor_pos = self.input.len();
                        }
                    }
                }
            }
        }
    }
}

fn draw_ui(frame: &mut Frame, state: &WizardState) {
    let area = frame.area();

    // Main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Length(2), // Progress bar
            Constraint::Min(10),   // Content
            Constraint::Length(3), // Help
        ])
        .split(area);

    // Header
    let header = Paragraph::new(state.title.clone())
        .style(Style::default().fg(state.theme.primary).bold())
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(state.theme.border)),
        );
    frame.render_widget(header, chunks[0]);

    // Progress - thin bar with text
    let (current, total) = state.progress();
    let progress_text = format!(" {} / {} ", current, total);

    // Create a horizontal layout for the progress area
    let progress_area = chunks[1];
    let bar_width = progress_area.width.saturating_sub(2); // Leave margin
    let text_width = progress_text.len() as u16;

    // Calculate the filled portion
    let ratio = current as f32 / total as f32;
    let filled_width = (ratio * bar_width as f32) as u16;

    // Draw the thin progress bar (single line)
    let bar_y = progress_area.y;
    let bar_x = progress_area.x + 1;

    // Background track
    let track = "─".repeat(bar_width as usize);
    let track_widget = Paragraph::new(track).style(Style::default().fg(state.theme.border));
    frame.render_widget(track_widget, Rect::new(bar_x, bar_y, bar_width, 1));

    // Filled portion
    if filled_width > 0 {
        let filled = "━".repeat(filled_width as usize);
        let filled_widget = Paragraph::new(filled).style(Style::default().fg(state.theme.primary));
        frame.render_widget(filled_widget, Rect::new(bar_x, bar_y, filled_width, 1));
    }

    // Progress text centered below the bar
    let text_x = bar_x + (bar_width.saturating_sub(text_width)) / 2;
    let text_widget =
        Paragraph::new(progress_text).style(Style::default().fg(state.theme.secondary));
    frame.render_widget(text_widget, Rect::new(text_x, bar_y + 1, text_width, 1));

    // Content area
    let content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Question prompt
            Constraint::Min(5),    // Input area
            Constraint::Length(2), // Error message
        ])
        .split(chunks[2]);

    if let Some(question) = state.current_question() {
        // Question prompt
        let prompt = Paragraph::new(question.prompt.clone())
            .style(Style::default().fg(state.theme.text))
            .wrap(Wrap { trim: true })
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(state.theme.primary))
                    .title(" Question ")
                    .title_style(Style::default().fg(state.theme.highlight)),
            );
        frame.render_widget(prompt, content_chunks[0]);

        // Input area based on question type
        match &question.kind {
            FlatQuestionKind::Input
            | FlatQuestionKind::Multiline
            | FlatQuestionKind::Int { .. }
            | FlatQuestionKind::Float { .. } => {
                let hint = match &question.kind {
                    FlatQuestionKind::Int { min, max } => {
                        let mut hints = vec![];
                        if let Some(m) = min {
                            hints.push(format!("min: {}", m));
                        }
                        if let Some(m) = max {
                            hints.push(format!("max: {}", m));
                        }
                        if hints.is_empty() {
                            "".to_string()
                        } else {
                            format!(" ({})", hints.join(", "))
                        }
                    }
                    FlatQuestionKind::Float { min, max } => {
                        let mut hints = vec![];
                        if let Some(m) = min {
                            hints.push(format!("min: {}", m));
                        }
                        if let Some(m) = max {
                            hints.push(format!("max: {}", m));
                        }
                        if hints.is_empty() {
                            "".to_string()
                        } else {
                            format!(" ({})", hints.join(", "))
                        }
                    }
                    _ => "".to_string(),
                };

                let default_hint = question
                    .default_value
                    .as_ref()
                    .map(|d| format!(" [default: {}]", d))
                    .unwrap_or_default();

                let input_block = Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(state.theme.border))
                    .title(format!(" Input{}{} ", hint, default_hint))
                    .title_style(Style::default().fg(state.theme.secondary));

                // Display input with cursor
                let display_text = if state.input.is_empty() && question.default_value.is_some() {
                    question
                        .default_value
                        .clone()
                        .unwrap_or_default()
                        .dim()
                        .to_string()
                } else {
                    state.input.clone()
                };

                let input_widget = Paragraph::new(display_text)
                    .style(Style::default().fg(state.theme.text))
                    .block(input_block);
                frame.render_widget(input_widget, content_chunks[1]);

                // Show cursor
                let cursor_x = content_chunks[1].x + 1 + state.cursor_pos as u16;
                let cursor_y = content_chunks[1].y + 1;
                frame.set_cursor_position((cursor_x, cursor_y));
            }
            FlatQuestionKind::Masked => {
                let masked_input = "●".repeat(state.input.len());
                let input_block = Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(state.theme.border))
                    .title(" Password ")
                    .title_style(Style::default().fg(state.theme.secondary));
                let input_widget = Paragraph::new(masked_input)
                    .style(Style::default().fg(state.theme.text))
                    .block(input_block);
                frame.render_widget(input_widget, content_chunks[1]);

                let cursor_x = content_chunks[1].x + 1 + state.input.len() as u16;
                let cursor_y = content_chunks[1].y + 1;
                frame.set_cursor_position((cursor_x, cursor_y));
            }
            FlatQuestionKind::Confirm { .. } => {
                let items: Vec<ListItem> = vec![
                    ListItem::new("  Yes").style(if state.selected_option == 0 {
                        Style::default().fg(state.theme.highlight).bold()
                    } else {
                        Style::default().fg(state.theme.text)
                    }),
                    ListItem::new("  No").style(if state.selected_option == 1 {
                        Style::default().fg(state.theme.highlight).bold()
                    } else {
                        Style::default().fg(state.theme.text)
                    }),
                ];

                let list = List::new(items)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(state.theme.border))
                            .title(" Select ")
                            .title_style(Style::default().fg(state.theme.secondary)),
                    )
                    .highlight_symbol("► ");

                let mut list_state = ListState::default();
                list_state.select(Some(state.selected_option));
                frame.render_stateful_widget(list, content_chunks[1], &mut list_state);
            }
            FlatQuestionKind::List { element_kind } => {
                let type_hint = match element_kind {
                    ListElementKind::String => "strings",
                    ListElementKind::Int { .. } => "integers",
                    ListElementKind::Float { .. } => "numbers",
                };

                let input_block = Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(state.theme.border))
                    .title(format!(" List of {} (comma-separated) ", type_hint))
                    .title_style(Style::default().fg(state.theme.secondary));

                let input_widget = Paragraph::new(state.input.clone())
                    .style(Style::default().fg(state.theme.text))
                    .block(input_block);
                frame.render_widget(input_widget, content_chunks[1]);

                // Show cursor
                let cursor_x = content_chunks[1].x + 1 + state.cursor_pos as u16;
                let cursor_y = content_chunks[1].y + 1;
                frame.set_cursor_position((cursor_x, cursor_y));
            }
            FlatQuestionKind::Select { options, .. } => {
                let items: Vec<ListItem> = options
                    .iter()
                    .enumerate()
                    .map(|(i, opt)| {
                        let style = if i == state.selected_option {
                            Style::default().fg(state.theme.highlight).bold()
                        } else {
                            Style::default().fg(state.theme.text)
                        };
                        ListItem::new(format!("  {}", opt)).style(style)
                    })
                    .collect();

                let list = List::new(items)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(state.theme.border))
                            .title(" Select Option ")
                            .title_style(Style::default().fg(state.theme.secondary)),
                    )
                    .highlight_symbol("► ");

                let mut list_state = ListState::default();
                list_state.select(Some(state.selected_option));
                frame.render_stateful_widget(list, content_chunks[1], &mut list_state);
            }
            FlatQuestionKind::MultiSelect { options, .. } => {
                let items: Vec<ListItem> = options
                    .iter()
                    .enumerate()
                    .map(|(i, opt)| {
                        let is_selected = state.multi_selected.get(i).copied().unwrap_or(false);
                        let checkbox = if is_selected { "[✓]" } else { "[ ]" };
                        let style = if i == state.selected_option {
                            Style::default().fg(state.theme.highlight).bold()
                        } else if is_selected {
                            Style::default().fg(state.theme.secondary)
                        } else {
                            Style::default().fg(state.theme.text)
                        };
                        ListItem::new(format!("  {} {}", checkbox, opt)).style(style)
                    })
                    .collect();

                let selected_count = state.multi_selected.iter().filter(|&&x| x).count();
                let list = List::new(items)
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(state.theme.border))
                            .title(format!(" Multi-Select ({} selected) ", selected_count))
                            .title_style(Style::default().fg(state.theme.secondary)),
                    )
                    .highlight_symbol("► ");

                let mut list_state = ListState::default();
                list_state.select(Some(state.selected_option));
                frame.render_stateful_widget(list, content_chunks[1], &mut list_state);
            }
        }

        // Error message
        if let Some(error) = &state.error_message {
            let error_widget = Paragraph::new(error.clone())
                .style(Style::default().fg(state.theme.error).bold())
                .alignment(Alignment::Center);
            frame.render_widget(error_widget, content_chunks[2]);
        }
    }

    // Help bar
    let help_text = match state.current_question().map(|q| &q.kind) {
        Some(FlatQuestionKind::Confirm { .. }) | Some(FlatQuestionKind::Select { .. }) => {
            "↑/↓: Select  Enter: Confirm  Esc: Cancel  Backspace: Back"
        }
        Some(FlatQuestionKind::MultiSelect { .. }) => {
            "↑/↓: Navigate  Space: Toggle  Enter: Confirm  Esc: Cancel  Backspace: Back"
        }
        Some(FlatQuestionKind::List { .. }) => {
            "Enter values separated by commas  Enter: Submit  Esc: Cancel  Backspace: Back"
        }
        _ => "Enter: Submit  Esc: Cancel  Backspace: Back",
    };
    let help = Paragraph::new(help_text)
        .style(Style::default().fg(state.theme.border))
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(state.theme.border)),
        );
    frame.render_widget(help, chunks[3]);
}

fn draw_completion(frame: &mut Frame, state: &WizardState) {
    let area = frame.area();

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(state.theme.success))
        .title(" Complete ")
        .title_style(Style::default().fg(state.theme.success).bold());

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let text = if let Some(epilogue) = &state.epilogue {
        epilogue.clone()
    } else {
        "All questions answered!\n\nPress Enter to finish.".to_string()
    };

    let paragraph = Paragraph::new(text)
        .style(Style::default().fg(state.theme.text))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });

    let centered = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Min(3),
            Constraint::Percentage(40),
        ])
        .split(inner);

    frame.render_widget(paragraph, centered[1]);
}

impl SurveyBackend for RatatuiBackend {
    type Error = RatatuiError;

    fn collect(
        &self,
        definition: &SurveyDefinition,
        validate: &dyn Fn(&ResponseValue, &Responses, &ResponsePath) -> Result<(), String>,
    ) -> Result<Responses, Self::Error> {
        let mut terminal = self.setup_terminal()?;
        let mut state = WizardState::new(definition, self.theme.clone(), self.title.clone());

        // Skip initially assumed questions
        while state.current_index < state.questions.len() {
            if let Some(assumed) = &state.questions[state.current_index].assumed {
                state.responses.insert(
                    state.questions[state.current_index].path.clone(),
                    assumed.clone(),
                );
                state.current_index += 1;
            } else {
                // Initialize first question's defaults
                // Extract values first to avoid borrow issues
                let init_data = state.current_question().map(|q| match &q.kind {
                    FlatQuestionKind::Confirm { default } => {
                        (Some(if *default { 0 } else { 1 }), None, None)
                    }
                    FlatQuestionKind::Select { default_idx, .. } => {
                        (Some(*default_idx), None, None)
                    }
                    FlatQuestionKind::MultiSelect {
                        options, defaults, ..
                    } => {
                        let mut selected = vec![false; options.len()];
                        for &idx in defaults {
                            if idx < selected.len() {
                                selected[idx] = true;
                            }
                        }
                        (None, Some(selected), None)
                    }
                    _ => (None, None, q.default_value.clone()),
                });

                if let Some((selected_opt, multi_sel, default_val)) = init_data {
                    if let Some(sel) = selected_opt {
                        state.selected_option = sel;
                    }
                    if let Some(multi) = multi_sel {
                        state.multi_selected = multi;
                    }
                    if let Some(def) = default_val {
                        state.input = def;
                        state.cursor_pos = state.input.len();
                    }
                }
                break;
            }
        }

        if state.current_index >= state.questions.len() {
            state.complete = true;
        }

        loop {
            terminal.draw(|frame| {
                if state.complete {
                    draw_completion(frame, &state);
                } else {
                    draw_ui(frame, &state);
                }
            })?;

            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                if state.complete {
                    match key.code {
                        KeyCode::Enter | KeyCode::Esc => break,
                        _ => {}
                    }
                } else {
                    match key.code {
                        KeyCode::Esc => {
                            state.cancelled = true;
                            break;
                        }
                        KeyCode::Enter => {
                            state.next_question(validate);
                        }
                        KeyCode::Up => {
                            if matches!(
                                state.current_question().map(|q| &q.kind),
                                Some(FlatQuestionKind::Confirm { .. })
                                    | Some(FlatQuestionKind::Select { .. })
                                    | Some(FlatQuestionKind::MultiSelect { .. })
                            ) && state.selected_option > 0
                            {
                                state.selected_option -= 1;
                            }
                        }
                        KeyCode::Down => {
                            if let Some(q) = state.current_question() {
                                match &q.kind {
                                    FlatQuestionKind::Confirm { .. } => {
                                        if state.selected_option < 1 {
                                            state.selected_option += 1;
                                        }
                                    }
                                    FlatQuestionKind::Select { options, .. } => {
                                        if state.selected_option < options.len() - 1 {
                                            state.selected_option += 1;
                                        }
                                    }
                                    FlatQuestionKind::MultiSelect { options, .. } => {
                                        if state.selected_option < options.len() - 1 {
                                            state.selected_option += 1;
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        KeyCode::Char(' ') => {
                            // Space toggles selection in multi-select
                            if let Some(FlatQuestionKind::MultiSelect { options, .. }) =
                                state.current_question().map(|q| &q.kind)
                            {
                                // Ensure multi_selected is properly sized
                                if state.multi_selected.len() != options.len() {
                                    state.multi_selected = vec![false; options.len()];
                                }
                                if state.selected_option < state.multi_selected.len() {
                                    state.multi_selected[state.selected_option] =
                                        !state.multi_selected[state.selected_option];
                                }
                            } else {
                                // For other question types, treat space as regular input
                                state.handle_input(key.code);
                            }
                        }
                        KeyCode::Backspace if state.input.is_empty() && state.current_index > 0 => {
                            state.prev_question();
                        }
                        _ => {
                            if !matches!(
                                state.current_question().map(|q| &q.kind),
                                Some(FlatQuestionKind::Confirm { .. })
                                    | Some(FlatQuestionKind::Select { .. })
                                    | Some(FlatQuestionKind::MultiSelect { .. })
                            ) {
                                state.handle_input(key.code);
                            }
                        }
                    }
                }
            }
        }

        self.restore_terminal(&mut terminal)?;

        if state.cancelled {
            return Err(RatatuiError::Cancelled);
        }

        Ok(state.responses)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_creation() {
        let _backend = RatatuiBackend::new();
        let _with_title = RatatuiBackend::new().with_title("Test");
        let _with_theme = RatatuiBackend::new().with_theme(Theme::default());
    }

    #[test]
    fn error_types() {
        let err = RatatuiError::Cancelled;
        assert_eq!(err.to_string(), "Survey cancelled by user");

        let err = RatatuiError::Terminal("test error".to_string());
        assert_eq!(err.to_string(), "Terminal error: test error");
    }

    #[test]
    fn theme_default() {
        let theme = Theme::default();
        assert_eq!(theme.primary, Color::Cyan);
        assert_eq!(theme.error, Color::Red);
        assert_eq!(theme.success, Color::Green);
    }
}
