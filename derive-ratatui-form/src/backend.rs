//! Ratatui form backend implementation for SurveyBackend trait.
//!
//! Displays all fields at once in a scrollable form with keyboard navigation.

use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use derive_survey::{
    DefaultValue, ListElementKind, Question, QuestionKind, ResponsePath, ResponseValue, Responses,
    SELECTED_VARIANT_KEY, SELECTED_VARIANTS_KEY, SurveyBackend, SurveyDefinition, Variant,
};
use ratatui::{
    Frame, Terminal,
    layout::{Constraint, Direction, Layout, Rect},
    prelude::CrosstermBackend,
    style::{Color, Modifier, Style},
    widgets::{
        Block, Borders, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
    },
};
use std::io::{self, Stdout};
use thiserror::Error;

/// Error type for the Ratatui form backend.
#[derive(Debug, Error)]
pub enum RatatuiFormError {
    /// User cancelled the form (e.g., pressed Esc).
    #[error("Form cancelled by user")]
    Cancelled,

    /// An I/O error occurred.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Terminal setup/restore error.
    #[error("Terminal error: {0}")]
    Terminal(String),
}

/// Color theme for the TUI form.
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
    pub selected_bg: Color,
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
            selected_bg: Color::DarkGray,
        }
    }
}

/// Ratatui form backend that displays all fields at once.
#[derive(Debug, Clone)]
pub struct RatatuiFormBackend {
    /// Title shown at the top of the form.
    title: String,
    /// Color theme for the UI.
    theme: Theme,
}

impl Default for RatatuiFormBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl RatatuiFormBackend {
    /// Create a new Ratatui form backend with default settings.
    pub fn new() -> Self {
        Self {
            title: "Form".to_string(),
            theme: Theme::default(),
        }
    }

    /// Set the title shown at the top of the form.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Set a custom color theme.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    fn setup_terminal(&self) -> Result<Terminal<CrosstermBackend<Stdout>>, RatatuiFormError> {
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
    ) -> Result<(), RatatuiFormError> {
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

/// Type of field for rendering and input handling.
#[derive(Debug, Clone)]
enum FieldKind {
    Text {
        multiline: bool,
        masked: bool,
    },
    Int {
        min: Option<i64>,
        max: Option<i64>,
    },
    Float {
        min: Option<f64>,
        max: Option<f64>,
    },
    Bool,
    List {
        element_kind: ListElementKind,
    },
    OneOf {
        variants: Vec<Variant>,
        selected: Option<usize>,
        /// Currently highlighted option (for keyboard navigation)
        highlight: usize,
    },
    AnyOf {
        variants: Vec<Variant>,
        selected: Vec<bool>,
        /// Currently highlighted option (for keyboard navigation)
        highlight: usize,
    },
}

/// Condition for field visibility based on parent OneOf/AnyOf selection.
#[derive(Debug, Clone)]
enum VisibilityCondition {
    /// Always visible
    Always,
    /// Visible when a specific variant is selected in a OneOf field
    OneOfVariant {
        /// Path of the parent OneOf field
        parent_path: ResponsePath,
        /// Index of the variant that must be selected
        variant_idx: usize,
    },
    /// Visible when a specific variant is selected in an AnyOf field
    AnyOfVariant {
        /// Path of the parent AnyOf field
        parent_path: ResponsePath,
        /// Index of the variant that must be selected
        variant_idx: usize,
    },
}

/// A field in the form.
#[derive(Debug, Clone)]
struct FormField {
    path: ResponsePath,
    prompt: String,
    kind: FieldKind,
    value: String,
    cursor_pos: usize,
    error: Option<String>,
    assumed: bool,
    /// Condition for this field to be visible
    visibility: VisibilityCondition,
    /// Whether this is a top-level field (for spacing between sections)
    is_top_level: bool,
}

/// State for the entire form.
struct FormState {
    fields: Vec<FormField>,
    focused_idx: usize,
    /// Scroll offset in pixels (vertical)
    scroll_offset: u16,
    /// Whether the submit button is focused
    submit_focused: bool,
    submitted: bool,
    cancelled: bool,
    theme: Theme,
    title: String,
    prelude: Option<String>,
    #[allow(dead_code)]
    epilogue: Option<String>,
}

impl FormState {
    fn new(definition: &SurveyDefinition, theme: Theme, title: String) -> Self {
        let mut fields = Vec::new();
        Self::flatten_questions(&definition.questions, &mut fields, None);

        Self {
            fields,
            focused_idx: 0,
            scroll_offset: 0,
            submit_focused: false,
            submitted: false,
            cancelled: false,
            theme,
            title,
            prelude: definition.prelude.clone(),
            epilogue: definition.epilogue.clone(),
        }
    }

    fn flatten_questions(
        questions: &[Question],
        fields: &mut Vec<FormField>,
        prefix: Option<&ResponsePath>,
    ) {
        // Top-level fields have no prefix
        let is_top_level = prefix.is_none();

        for question in questions {
            let path = match prefix {
                Some(p) => p.child(question.path().as_str()),
                None => question.path().clone(),
            };

            let assumed = matches!(question.default(), DefaultValue::Assumed(_));

            let prompt = if question.ask().is_empty() {
                // Create readable label from path
                path.as_str()
                    .split('.')
                    .next_back()
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
                QuestionKind::Input(input_q) => {
                    let default = match question.default() {
                        DefaultValue::Suggested(ResponseValue::String(s)) => s.clone(),
                        _ => input_q.default.clone().unwrap_or_default(),
                    };
                    fields.push(FormField {
                        path,
                        prompt,
                        kind: FieldKind::Text {
                            multiline: false,
                            masked: false,
                        },
                        value: default.clone(),
                        cursor_pos: default.len(),
                        error: None,
                        assumed,
                        visibility: VisibilityCondition::Always,
                        is_top_level,
                    });
                }
                QuestionKind::Multiline(ml_q) => {
                    let default = match question.default() {
                        DefaultValue::Suggested(ResponseValue::String(s)) => s.clone(),
                        _ => ml_q.default.clone().unwrap_or_default(),
                    };
                    fields.push(FormField {
                        path,
                        prompt,
                        kind: FieldKind::Text {
                            multiline: true,
                            masked: false,
                        },
                        value: default.clone(),
                        cursor_pos: default.len(),
                        error: None,
                        assumed,
                        visibility: VisibilityCondition::Always,
                        is_top_level,
                    });
                }
                QuestionKind::Masked(_) => {
                    fields.push(FormField {
                        path,
                        prompt,
                        kind: FieldKind::Text {
                            multiline: false,
                            masked: true,
                        },
                        value: String::new(),
                        cursor_pos: 0,
                        error: None,
                        assumed,
                        visibility: VisibilityCondition::Always,
                        is_top_level,
                    });
                }
                QuestionKind::Int(int_q) => {
                    let default = match question.default() {
                        DefaultValue::Suggested(ResponseValue::Int(i)) => i.to_string(),
                        _ => int_q.default.map(|i| i.to_string()).unwrap_or_default(),
                    };
                    fields.push(FormField {
                        path,
                        prompt,
                        kind: FieldKind::Int {
                            min: int_q.min,
                            max: int_q.max,
                        },
                        value: default.clone(),
                        cursor_pos: default.len(),
                        error: None,
                        assumed,
                        visibility: VisibilityCondition::Always,
                        is_top_level,
                    });
                }
                QuestionKind::Float(float_q) => {
                    let default = match question.default() {
                        DefaultValue::Suggested(ResponseValue::Float(f)) => f.to_string(),
                        _ => float_q.default.map(|f| f.to_string()).unwrap_or_default(),
                    };
                    fields.push(FormField {
                        path,
                        prompt,
                        kind: FieldKind::Float {
                            min: float_q.min,
                            max: float_q.max,
                        },
                        value: default.clone(),
                        cursor_pos: default.len(),
                        error: None,
                        assumed,
                        visibility: VisibilityCondition::Always,
                        is_top_level,
                    });
                }
                QuestionKind::Confirm(confirm_q) => {
                    let default = match question.default() {
                        DefaultValue::Suggested(ResponseValue::Bool(b)) => *b,
                        _ => confirm_q.default,
                    };
                    fields.push(FormField {
                        path,
                        prompt,
                        kind: FieldKind::Bool,
                        value: if default { "true" } else { "false" }.to_string(),
                        cursor_pos: 0,
                        error: None,
                        assumed,
                        visibility: VisibilityCondition::Always,
                        is_top_level,
                    });
                }
                QuestionKind::List(list_q) => {
                    fields.push(FormField {
                        path,
                        prompt,
                        kind: FieldKind::List {
                            element_kind: list_q.element_kind.clone(),
                        },
                        value: String::new(),
                        cursor_pos: 0,
                        error: None,
                        assumed,
                        visibility: VisibilityCondition::Always,
                        is_top_level,
                    });
                }
                QuestionKind::OneOf(one_of) => {
                    let default_idx = match question.default() {
                        DefaultValue::Suggested(ResponseValue::ChosenVariant(idx)) => Some(*idx),
                        _ => one_of.default,
                    };
                    fields.push(FormField {
                        path: path.clone(),
                        prompt,
                        kind: FieldKind::OneOf {
                            variants: one_of.variants.clone(),
                            selected: default_idx,
                            highlight: default_idx.unwrap_or(0),
                        },
                        value: String::new(),
                        cursor_pos: 0,
                        error: None,
                        assumed,
                        visibility: VisibilityCondition::Always,
                        is_top_level,
                    });

                    // Add nested fields for all variants
                    for (idx, variant) in one_of.variants.iter().enumerate() {
                        Self::add_variant_fields(
                            variant,
                            &path,
                            fields,
                            VisibilityCondition::OneOfVariant {
                                parent_path: path.clone(),
                                variant_idx: idx,
                            },
                        );
                    }
                }
                QuestionKind::AnyOf(any_of) => {
                    let default_selected = match question.default() {
                        DefaultValue::Suggested(ResponseValue::ChosenVariants(indices)) => {
                            let mut sel = vec![false; any_of.variants.len()];
                            for &idx in indices {
                                if idx < sel.len() {
                                    sel[idx] = true;
                                }
                            }
                            sel
                        }
                        _ => {
                            let mut sel = vec![false; any_of.variants.len()];
                            for &idx in &any_of.defaults {
                                if idx < sel.len() {
                                    sel[idx] = true;
                                }
                            }
                            sel
                        }
                    };
                    fields.push(FormField {
                        path: path.clone(),
                        prompt,
                        kind: FieldKind::AnyOf {
                            variants: any_of.variants.clone(),
                            selected: default_selected,
                            highlight: 0,
                        },
                        value: String::new(),
                        cursor_pos: 0,
                        error: None,
                        assumed,
                        visibility: VisibilityCondition::Always,
                        is_top_level,
                    });

                    // Add nested fields for all variants
                    for (idx, variant) in any_of.variants.iter().enumerate() {
                        let item_path = path.child(&idx.to_string());
                        Self::add_variant_fields(
                            variant,
                            &item_path,
                            fields,
                            VisibilityCondition::AnyOfVariant {
                                parent_path: path.clone(),
                                variant_idx: idx,
                            },
                        );
                    }
                }
                QuestionKind::AllOf(all_of) => {
                    Self::flatten_questions(all_of.questions(), fields, Some(&path));
                }
            }
        }
    }

    fn add_variant_fields(
        variant: &Variant,
        parent_path: &ResponsePath,
        fields: &mut Vec<FormField>,
        visibility: VisibilityCondition,
    ) {
        match &variant.kind {
            QuestionKind::Unit => {}
            QuestionKind::AllOf(all_of) => {
                // For AllOf inside a variant, all nested fields inherit the same visibility
                for q in all_of.questions() {
                    Self::add_question_with_visibility(
                        q,
                        fields,
                        Some(parent_path),
                        visibility.clone(),
                    );
                }
            }
            QuestionKind::Input(input_q) => {
                let path = parent_path.child(&variant.name);
                fields.push(FormField {
                    path,
                    prompt: variant.name.clone(),
                    kind: FieldKind::Text {
                        multiline: false,
                        masked: false,
                    },
                    value: input_q.default.clone().unwrap_or_default(),
                    cursor_pos: 0,
                    error: None,
                    assumed: false,
                    visibility: visibility.clone(),
                    is_top_level: false,
                });
            }
            QuestionKind::Int(int_q) => {
                let path = parent_path.child(&variant.name);
                let default = int_q.default.map(|i| i.to_string()).unwrap_or_default();
                fields.push(FormField {
                    path,
                    prompt: variant.name.clone(),
                    kind: FieldKind::Int {
                        min: int_q.min,
                        max: int_q.max,
                    },
                    value: default,
                    cursor_pos: 0,
                    error: None,
                    assumed: false,
                    visibility: visibility.clone(),
                    is_top_level: false,
                });
            }
            QuestionKind::Float(float_q) => {
                let path = parent_path.child(&variant.name);
                let default = float_q.default.map(|f| f.to_string()).unwrap_or_default();
                fields.push(FormField {
                    path,
                    prompt: variant.name.clone(),
                    kind: FieldKind::Float {
                        min: float_q.min,
                        max: float_q.max,
                    },
                    value: default,
                    cursor_pos: 0,
                    error: None,
                    assumed: false,
                    visibility: visibility.clone(),
                    is_top_level: false,
                });
            }
            QuestionKind::Confirm(confirm_q) => {
                let path = parent_path.child(&variant.name);
                fields.push(FormField {
                    path,
                    prompt: variant.name.clone(),
                    kind: FieldKind::Bool,
                    value: if confirm_q.default { "true" } else { "false" }.to_string(),
                    cursor_pos: 0,
                    error: None,
                    assumed: false,
                    visibility: visibility.clone(),
                    is_top_level: false,
                });
            }
            QuestionKind::OneOf(one_of) => {
                let path = parent_path.child(&variant.name);
                fields.push(FormField {
                    path: path.clone(),
                    prompt: variant.name.clone(),
                    kind: FieldKind::OneOf {
                        variants: one_of.variants.clone(),
                        selected: one_of.default,
                        highlight: one_of.default.unwrap_or(0),
                    },
                    value: String::new(),
                    cursor_pos: 0,
                    error: None,
                    assumed: false,
                    visibility: visibility.clone(),
                    is_top_level: false,
                });
                for (idx, v) in one_of.variants.iter().enumerate() {
                    Self::add_variant_fields(
                        v,
                        &path,
                        fields,
                        VisibilityCondition::OneOfVariant {
                            parent_path: path.clone(),
                            variant_idx: idx,
                        },
                    );
                }
            }
            QuestionKind::AnyOf(any_of) => {
                let path = parent_path.child(&variant.name);
                let mut selected = vec![false; any_of.variants.len()];
                for &idx in &any_of.defaults {
                    if idx < selected.len() {
                        selected[idx] = true;
                    }
                }
                fields.push(FormField {
                    path: path.clone(),
                    prompt: variant.name.clone(),
                    kind: FieldKind::AnyOf {
                        variants: any_of.variants.clone(),
                        selected,
                        highlight: 0,
                    },
                    value: String::new(),
                    cursor_pos: 0,
                    error: None,
                    assumed: false,
                    visibility: visibility.clone(),
                    is_top_level: false,
                });
                for (idx, v) in any_of.variants.iter().enumerate() {
                    let item_path = path.child(&idx.to_string());
                    Self::add_variant_fields(
                        v,
                        &item_path,
                        fields,
                        VisibilityCondition::AnyOfVariant {
                            parent_path: path.clone(),
                            variant_idx: idx,
                        },
                    );
                }
            }
            _ => {}
        }
    }

    /// Add a question with a specific visibility condition
    fn add_question_with_visibility(
        question: &Question,
        fields: &mut Vec<FormField>,
        prefix: Option<&ResponsePath>,
        visibility: VisibilityCondition,
    ) {
        let path = match prefix {
            Some(p) => p.child(question.path().as_str()),
            None => question.path().clone(),
        };

        let assumed = matches!(question.default(), DefaultValue::Assumed(_));

        let prompt = if question.ask().is_empty() {
            path.as_str()
                .split('.')
                .next_back()
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
            QuestionKind::Input(input_q) => {
                let default = match question.default() {
                    DefaultValue::Suggested(ResponseValue::String(s)) => s.clone(),
                    _ => input_q.default.clone().unwrap_or_default(),
                };
                fields.push(FormField {
                    path,
                    prompt,
                    kind: FieldKind::Text {
                        multiline: false,
                        masked: false,
                    },
                    value: default.clone(),
                    cursor_pos: default.len(),
                    error: None,
                    assumed,
                    visibility: visibility.clone(),
                    is_top_level: false,
                });
            }
            QuestionKind::Multiline(ml_q) => {
                let default = match question.default() {
                    DefaultValue::Suggested(ResponseValue::String(s)) => s.clone(),
                    _ => ml_q.default.clone().unwrap_or_default(),
                };
                fields.push(FormField {
                    path,
                    prompt,
                    kind: FieldKind::Text {
                        multiline: true,
                        masked: false,
                    },
                    value: default.clone(),
                    cursor_pos: default.len(),
                    error: None,
                    assumed,
                    visibility: visibility.clone(),
                    is_top_level: false,
                });
            }
            QuestionKind::Masked(_) => {
                fields.push(FormField {
                    path,
                    prompt,
                    kind: FieldKind::Text {
                        multiline: false,
                        masked: true,
                    },
                    value: String::new(),
                    cursor_pos: 0,
                    error: None,
                    assumed,
                    visibility: visibility.clone(),
                    is_top_level: false,
                });
            }
            QuestionKind::Int(int_q) => {
                let default = match question.default() {
                    DefaultValue::Suggested(ResponseValue::Int(i)) => i.to_string(),
                    _ => int_q.default.map(|i| i.to_string()).unwrap_or_default(),
                };
                fields.push(FormField {
                    path,
                    prompt,
                    kind: FieldKind::Int {
                        min: int_q.min,
                        max: int_q.max,
                    },
                    value: default.clone(),
                    cursor_pos: default.len(),
                    error: None,
                    assumed,
                    visibility: visibility.clone(),
                    is_top_level: false,
                });
            }
            QuestionKind::Float(float_q) => {
                let default = match question.default() {
                    DefaultValue::Suggested(ResponseValue::Float(f)) => f.to_string(),
                    _ => float_q.default.map(|f| f.to_string()).unwrap_or_default(),
                };
                fields.push(FormField {
                    path,
                    prompt,
                    kind: FieldKind::Float {
                        min: float_q.min,
                        max: float_q.max,
                    },
                    value: default.clone(),
                    cursor_pos: default.len(),
                    error: None,
                    assumed,
                    visibility: visibility.clone(),
                    is_top_level: false,
                });
            }
            QuestionKind::Confirm(confirm_q) => {
                let default = match question.default() {
                    DefaultValue::Suggested(ResponseValue::Bool(b)) => *b,
                    _ => confirm_q.default,
                };
                fields.push(FormField {
                    path,
                    prompt,
                    kind: FieldKind::Bool,
                    value: if default { "true" } else { "false" }.to_string(),
                    cursor_pos: 0,
                    error: None,
                    assumed,
                    visibility: visibility.clone(),
                    is_top_level: false,
                });
            }
            QuestionKind::List(list_q) => {
                fields.push(FormField {
                    path,
                    prompt,
                    kind: FieldKind::List {
                        element_kind: list_q.element_kind.clone(),
                    },
                    value: String::new(),
                    cursor_pos: 0,
                    error: None,
                    assumed,
                    visibility: visibility.clone(),
                    is_top_level: false,
                });
            }
            QuestionKind::OneOf(one_of) => {
                let default_idx = match question.default() {
                    DefaultValue::Suggested(ResponseValue::ChosenVariant(idx)) => Some(*idx),
                    _ => one_of.default,
                };
                fields.push(FormField {
                    path: path.clone(),
                    prompt,
                    kind: FieldKind::OneOf {
                        variants: one_of.variants.clone(),
                        selected: default_idx,
                        highlight: default_idx.unwrap_or(0),
                    },
                    value: String::new(),
                    cursor_pos: 0,
                    error: None,
                    assumed,
                    visibility: visibility.clone(),
                    is_top_level: false,
                });

                // Add nested fields for all variants
                for (idx, variant) in one_of.variants.iter().enumerate() {
                    Self::add_variant_fields(
                        variant,
                        &path,
                        fields,
                        VisibilityCondition::OneOfVariant {
                            parent_path: path.clone(),
                            variant_idx: idx,
                        },
                    );
                }
            }
            QuestionKind::AnyOf(any_of) => {
                let default_selected = match question.default() {
                    DefaultValue::Suggested(ResponseValue::ChosenVariants(indices)) => {
                        let mut sel = vec![false; any_of.variants.len()];
                        for &idx in indices {
                            if idx < sel.len() {
                                sel[idx] = true;
                            }
                        }
                        sel
                    }
                    _ => {
                        let mut sel = vec![false; any_of.variants.len()];
                        for &idx in &any_of.defaults {
                            if idx < sel.len() {
                                sel[idx] = true;
                            }
                        }
                        sel
                    }
                };
                fields.push(FormField {
                    path: path.clone(),
                    prompt,
                    kind: FieldKind::AnyOf {
                        variants: any_of.variants.clone(),
                        selected: default_selected,
                        highlight: 0,
                    },
                    value: String::new(),
                    cursor_pos: 0,
                    error: None,
                    assumed,
                    visibility: visibility.clone(),
                    is_top_level: false,
                });

                // Add nested fields for all variants
                for (idx, variant) in any_of.variants.iter().enumerate() {
                    let item_path = path.child(&idx.to_string());
                    Self::add_variant_fields(
                        variant,
                        &item_path,
                        fields,
                        VisibilityCondition::AnyOfVariant {
                            parent_path: path.clone(),
                            variant_idx: idx,
                        },
                    );
                }
            }
            QuestionKind::AllOf(all_of) => {
                // Recursively add all nested questions with the same visibility
                for q in all_of.questions() {
                    Self::add_question_with_visibility(q, fields, Some(&path), visibility.clone());
                }
            }
        }
    }

    fn focused_field(&self) -> Option<&FormField> {
        self.fields.get(self.focused_idx)
    }

    fn focused_field_mut(&mut self) -> Option<&mut FormField> {
        self.fields.get_mut(self.focused_idx)
    }

    /// Check if a field is currently visible based on its visibility condition
    fn is_field_visible(&self, field: &FormField) -> bool {
        if field.assumed {
            return false;
        }
        match &field.visibility {
            VisibilityCondition::Always => true,
            VisibilityCondition::OneOfVariant {
                parent_path,
                variant_idx,
            } => {
                // Find the parent OneOf field and check if this variant is selected
                self.fields.iter().any(|f| {
                    f.path == *parent_path
                        && matches!(&f.kind, FieldKind::OneOf { selected, .. } if *selected == Some(*variant_idx))
                })
            }
            VisibilityCondition::AnyOfVariant {
                parent_path,
                variant_idx,
            } => {
                // Find the parent AnyOf field and check if this variant is selected
                self.fields.iter().any(|f| {
                    f.path == *parent_path
                        && matches!(&f.kind, FieldKind::AnyOf { selected, .. } if selected.get(*variant_idx).copied().unwrap_or(false))
                })
            }
        }
    }

    /// Check if a field at the given index is visible
    fn is_field_visible_by_idx(&self, idx: usize) -> bool {
        self.fields
            .get(idx)
            .map(|f| self.is_field_visible(f))
            .unwrap_or(false)
    }

    /// Calculate the Y position of a field (by index) in the virtual scroll area
    fn field_y_position(&self, target_idx: usize) -> u16 {
        let mut y: u16 = 0;
        let mut is_first = true;
        for (idx, field) in self.fields.iter().enumerate() {
            if !self.is_field_visible(field) {
                continue;
            }
            if idx == target_idx {
                return y;
            }
            // Include spacing for all except the first visible field
            y += get_field_height(field, !is_first);
            is_first = false;
        }
        y
    }

    /// Calculate total content height
    fn total_content_height(&self) -> u16 {
        let mut total: u16 = 0;
        let mut is_first = true;
        for field in &self.fields {
            if !self.is_field_visible(field) {
                continue;
            }
            total += get_field_height(field, !is_first);
            is_first = false;
        }
        total
    }

    /// Adjust scroll offset to ensure focused field is visible
    fn adjust_scroll(&mut self, viewport_height: u16) {
        let field_y = self.field_y_position(self.focused_idx);

        // Check if this field has spacing before it
        let is_first_visible = self
            .fields
            .iter()
            .filter(|f| self.is_field_visible(f))
            .next()
            .map(|f| std::ptr::eq(f, &self.fields[self.focused_idx]))
            .unwrap_or(false);

        let focused = self.focused_field();
        let spacing = if !is_first_visible && focused.map(|f| f.is_top_level).unwrap_or(false) {
            TOP_LEVEL_SPACING
        } else {
            0
        };

        let field_height = focused.map(|f| get_field_height(f, false)).unwrap_or(3);

        // If field is above viewport, scroll up (to show spacing too)
        if field_y < self.scroll_offset {
            self.scroll_offset = field_y;
        }

        // If field is below viewport, scroll down
        // field_y is start of space (including spacing), actual content is at field_y + spacing
        let field_content_bottom = field_y + spacing + field_height;
        let viewport_bottom = self.scroll_offset + viewport_height;
        if field_content_bottom > viewport_bottom {
            self.scroll_offset = field_content_bottom.saturating_sub(viewport_height);
        }
    }

    fn next_field(&mut self) {
        if self.submit_focused {
            // Already on submit button, can't go further
            return;
        }

        // Find next visible field
        let mut next = self.focused_idx + 1;
        while next < self.fields.len() && !self.is_field_visible_by_idx(next) {
            next += 1;
        }
        if next < self.fields.len() {
            self.focused_idx = next;
        } else {
            // No more fields, focus the submit button
            self.submit_focused = true;
        }
    }

    fn prev_field(&mut self) {
        if self.submit_focused {
            // Move from submit button back to last visible field
            self.submit_focused = false;
            // Find the last visible field
            let mut last_visible = self.fields.len();
            for i in (0..self.fields.len()).rev() {
                if self.is_field_visible_by_idx(i) {
                    last_visible = i;
                    break;
                }
            }
            if last_visible < self.fields.len() {
                self.focused_idx = last_visible;
            }
            return;
        }

        // Find previous visible field
        if self.focused_idx > 0 {
            let mut prev = self.focused_idx - 1;
            while prev > 0 && !self.is_field_visible_by_idx(prev) {
                prev -= 1;
            }
            if self.is_field_visible_by_idx(prev) {
                self.focused_idx = prev;
            }
        }
    }

    fn handle_text_input(&mut self, c: char) {
        if let Some(field) = self.focused_field_mut() {
            field.value.insert(field.cursor_pos, c);
            field.cursor_pos += 1;
            field.error = None;
        }
    }

    fn handle_backspace(&mut self) {
        if let Some(field) = self.focused_field_mut()
            && field.cursor_pos > 0
        {
            field.cursor_pos -= 1;
            field.value.remove(field.cursor_pos);
            field.error = None;
        }
    }

    fn handle_delete(&mut self) {
        if let Some(field) = self.focused_field_mut()
            && field.cursor_pos < field.value.len()
        {
            field.value.remove(field.cursor_pos);
            field.error = None;
        }
    }

    fn cursor_left(&mut self) {
        if let Some(field) = self.focused_field_mut()
            && field.cursor_pos > 0
        {
            field.cursor_pos -= 1;
        }
    }

    fn cursor_right(&mut self) {
        if let Some(field) = self.focused_field_mut()
            && field.cursor_pos < field.value.len()
        {
            field.cursor_pos += 1;
        }
    }

    fn toggle_bool(&mut self) {
        if let Some(field) = self.focused_field_mut()
            && matches!(field.kind, FieldKind::Bool)
        {
            field.value = if field.value == "true" {
                "false"
            } else {
                "true"
            }
            .to_string();
        }
    }

    /// Move highlight up within OneOf/AnyOf options
    fn option_up(&mut self) {
        if let Some(field) = self.focused_field_mut() {
            match &mut field.kind {
                FieldKind::OneOf {
                    variants,
                    highlight,
                    ..
                } => {
                    if !variants.is_empty() {
                        *highlight = (*highlight + variants.len() - 1) % variants.len();
                    }
                }
                FieldKind::AnyOf {
                    variants,
                    highlight,
                    ..
                } => {
                    if !variants.is_empty() {
                        *highlight = (*highlight + variants.len() - 1) % variants.len();
                    }
                }
                _ => {}
            }
        }
    }

    /// Move highlight down within OneOf/AnyOf options
    fn option_down(&mut self) {
        if let Some(field) = self.focused_field_mut() {
            match &mut field.kind {
                FieldKind::OneOf {
                    variants,
                    highlight,
                    ..
                } => {
                    if !variants.is_empty() {
                        *highlight = (*highlight + 1) % variants.len();
                    }
                }
                FieldKind::AnyOf {
                    variants,
                    highlight,
                    ..
                } => {
                    if !variants.is_empty() {
                        *highlight = (*highlight + 1) % variants.len();
                    }
                }
                _ => {}
            }
        }
    }

    /// Select the currently highlighted option (for OneOf) or toggle it (for AnyOf)
    fn select_option(&mut self) {
        if let Some(field) = self.focused_field_mut() {
            match &mut field.kind {
                FieldKind::OneOf {
                    highlight,
                    selected,
                    ..
                } => {
                    *selected = Some(*highlight);
                }
                FieldKind::AnyOf {
                    highlight,
                    selected,
                    ..
                } => {
                    if *highlight < selected.len() {
                        selected[*highlight] = !selected[*highlight];
                    }
                }
                _ => {}
            }
        }
    }

    /// Check if current field is a selection type (OneOf/AnyOf)
    fn is_selection_field(&self) -> bool {
        if self.submit_focused {
            return false;
        }
        self.focused_field()
            .map(|f| matches!(f.kind, FieldKind::OneOf { .. } | FieldKind::AnyOf { .. }))
            .unwrap_or(false)
    }

    fn toggle_anyof(&mut self, idx: usize) {
        if let Some(field) = self.focused_field_mut()
            && let FieldKind::AnyOf { selected, .. } = &mut field.kind
            && idx < selected.len()
        {
            selected[idx] = !selected[idx];
        }
    }

    fn collect_responses(&self) -> Responses {
        let mut responses = Responses::new();

        for field in &self.fields {
            // Skip assumed fields and fields that are not visible
            // (except OneOf/AnyOf which always need their selection recorded)
            let dominated_by_variant = !matches!(field.visibility, VisibilityCondition::Always);
            if field.assumed || (dominated_by_variant && !self.is_field_visible(field)) {
                continue;
            }

            match &field.kind {
                FieldKind::Text { .. } => {
                    responses.insert(
                        field.path.clone(),
                        ResponseValue::String(field.value.clone()),
                    );
                }
                FieldKind::Int { .. } => {
                    if let Ok(n) = field.value.parse::<i64>() {
                        responses.insert(field.path.clone(), ResponseValue::Int(n));
                    }
                }
                FieldKind::Float { .. } => {
                    if let Ok(n) = field.value.parse::<f64>() {
                        responses.insert(field.path.clone(), ResponseValue::Float(n));
                    }
                }
                FieldKind::Bool => {
                    let b = field.value == "true";
                    responses.insert(field.path.clone(), ResponseValue::Bool(b));
                }
                FieldKind::List { element_kind } => {
                    let items: Vec<&str> = field
                        .value
                        .split(',')
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty())
                        .collect();

                    let rv = match element_kind {
                        ListElementKind::String => {
                            ResponseValue::StringList(items.iter().map(|s| s.to_string()).collect())
                        }
                        ListElementKind::Int { .. } => {
                            let ints: Result<Vec<i64>, _> =
                                items.iter().map(|s| s.parse()).collect();
                            if let Ok(list) = ints {
                                ResponseValue::IntList(list)
                            } else {
                                continue;
                            }
                        }
                        ListElementKind::Float { .. } => {
                            let floats: Result<Vec<f64>, _> =
                                items.iter().map(|s| s.parse()).collect();
                            if let Ok(list) = floats {
                                ResponseValue::FloatList(list)
                            } else {
                                continue;
                            }
                        }
                    };
                    responses.insert(field.path.clone(), rv);
                }
                FieldKind::OneOf { selected, .. } => {
                    if let Some(idx) = selected {
                        let variant_path = field.path.child(SELECTED_VARIANT_KEY);
                        responses.insert(variant_path, ResponseValue::ChosenVariant(*idx));
                    }
                }
                FieldKind::AnyOf { selected, .. } => {
                    let indices: Vec<usize> = selected
                        .iter()
                        .enumerate()
                        .filter_map(|(i, &s)| if s { Some(i) } else { None })
                        .collect();
                    let variants_path = field.path.child(SELECTED_VARIANTS_KEY);
                    responses.insert(
                        variants_path,
                        ResponseValue::ChosenVariants(indices.clone()),
                    );

                    // Store variant index for each selected item
                    for (item_idx, &variant_idx) in indices.iter().enumerate() {
                        let item_path = field.path.child(&item_idx.to_string());
                        let item_variant_path = item_path.child(SELECTED_VARIANT_KEY);
                        responses
                            .insert(item_variant_path, ResponseValue::ChosenVariant(variant_idx));
                    }
                }
            }
        }

        responses
    }

    fn validate_all(
        &mut self,
        validate: &dyn Fn(&ResponseValue, &Responses, &ResponsePath) -> Result<(), String>,
    ) -> bool {
        let responses = self.collect_responses();
        let mut has_errors = false;

        // Collect visible field indices first (before mutable borrow)
        let visible_indices: Vec<usize> = self
            .fields
            .iter()
            .enumerate()
            .filter(|(_, f)| self.is_field_visible(f))
            .map(|(i, _)| i)
            .collect();

        // First pass: clear all errors and do basic type validation
        for idx in &visible_indices {
            let field = &mut self.fields[*idx];

            field.error = None;

            // Basic validation (min/max, type parsing, required selections)
            match &field.kind {
                FieldKind::OneOf { selected, .. } => {
                    if selected.is_none() {
                        field.error = Some("Please select an option".to_string());
                        has_errors = true;
                    }
                }
                FieldKind::Int { min, max } => match field.value.parse::<i64>() {
                    Ok(n) => {
                        if let Some(m) = min
                            && n < *m
                        {
                            field.error = Some(format!("Must be at least {}", m));
                            has_errors = true;
                        } else if let Some(m) = max
                            && n > *m
                        {
                            field.error = Some(format!("Must be at most {}", m));
                            has_errors = true;
                        }
                    }
                    Err(_) if !field.value.is_empty() => {
                        field.error = Some("Invalid integer".to_string());
                        has_errors = true;
                    }
                    _ => {}
                },
                FieldKind::Float { min, max } => match field.value.parse::<f64>() {
                    Ok(n) => {
                        if let Some(m) = min
                            && n < *m
                        {
                            field.error = Some(format!("Must be at least {}", m));
                            has_errors = true;
                        } else if let Some(m) = max
                            && n > *m
                        {
                            field.error = Some(format!("Must be at most {}", m));
                            has_errors = true;
                        }
                    }
                    Err(_) if !field.value.is_empty() => {
                        field.error = Some("Invalid number".to_string());
                        has_errors = true;
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        // Second pass: collect custom validation errors into a map by path
        // This ensures each error is associated with its correct field
        let mut validation_errors: std::collections::HashMap<ResponsePath, String> =
            std::collections::HashMap::new();

        for (path, value) in responses.iter() {
            if let Err(e) = validate(value, &responses, path) {
                validation_errors.insert(path.clone(), e);
            }
        }

        // Third pass: apply custom validation errors to visible fields
        for idx in &visible_indices {
            let field = &mut self.fields[*idx];
            if field.error.is_some() {
                continue; // Skip if already has a basic validation error
            }

            if let Some(error) = validation_errors.get(&field.path) {
                field.error = Some(error.clone());
                has_errors = true;
            }
        }

        // If there are errors, focus the first field with an error
        if has_errors {
            for idx in &visible_indices {
                if self.fields[*idx].error.is_some() {
                    self.focused_idx = *idx;
                    self.submit_focused = false;
                    break;
                }
            }
        }

        !has_errors
    }
}

fn draw_form(frame: &mut Frame, state: &mut FormState) {
    let area = frame.area();
    let theme = state.theme.clone();

    // Main layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(10),   // Form content
            Constraint::Length(3), // Submit button
            Constraint::Length(1), // Help bar
        ])
        .split(area);

    // Title
    let title_text = if let Some(prelude) = &state.prelude {
        format!("{}\n{}", state.title, prelude)
    } else {
        state.title.clone()
    };
    let title = Paragraph::new(title_text)
        .style(
            Style::default()
                .fg(theme.primary)
                .add_modifier(Modifier::BOLD),
        )
        .block(
            Block::default()
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(theme.border)),
        );
    frame.render_widget(title, chunks[0]);

    // Form content area - reserve space for scrollbar on the right
    let form_area = chunks[1];
    let content_width = form_area.width.saturating_sub(2); // 1 for left margin, 1 for scrollbar
    let viewport_height = form_area.height;

    // Adjust scroll to keep focused field visible
    state.adjust_scroll(viewport_height);

    let total_height = state.total_content_height();
    let scroll_offset = state.scroll_offset;

    // Collect visible fields (based on visibility conditions, not just assumed)
    let visible_fields: Vec<(usize, &FormField)> = state
        .fields
        .iter()
        .enumerate()
        .filter(|(_, f)| state.is_field_visible(f))
        .collect();

    // Render fields with scroll offset
    let mut virtual_y: u16 = 0;
    let mut is_first_visible = true;
    for (field_idx, field) in visible_fields.iter() {
        // Field is only focused if it's the focused index AND the submit button is not focused
        let is_focused = !state.submit_focused && *field_idx == state.focused_idx;

        // Include spacing for top-level fields (except the first visible one)
        let include_spacing = !is_first_visible;
        let field_height_with_spacing = get_field_height(field, include_spacing);
        let field_height_base = get_field_height(field, false);
        let spacing = if include_spacing && field.is_top_level {
            TOP_LEVEL_SPACING
        } else {
            0
        };

        // Calculate if this field is visible in the viewport
        let field_top = virtual_y;
        let field_bottom = virtual_y + field_height_with_spacing;

        // Skip fields completely above the viewport
        if field_bottom <= scroll_offset {
            virtual_y += field_height_with_spacing;
            is_first_visible = false;
            continue;
        }

        // Stop if we're completely below the viewport
        if field_top >= scroll_offset + viewport_height {
            break;
        }

        // Calculate the visible portion of this field (accounting for spacing)
        let visible_top = field_top.saturating_sub(scroll_offset) + spacing;
        let clip_top = scroll_offset.saturating_sub(field_top + spacing);
        let available_height = viewport_height.saturating_sub(visible_top);
        let visible_height = (field_height_base - clip_top).min(available_height);

        if visible_height > 0 {
            let field_area = Rect {
                x: form_area.x + 1,
                y: form_area.y + visible_top,
                width: content_width,
                height: visible_height,
            };

            // Only draw if we have the full field height (to avoid partial rendering issues)
            if clip_top == 0 && visible_height >= field_height_base {
                draw_field(frame, field, field_area, is_focused, &theme);
            } else if clip_top == 0 {
                // Field is partially visible at the bottom - draw what we can
                draw_field(frame, field, field_area, is_focused, &theme);
            }
            // Skip fields that are clipped at the top (they look weird)
        }

        is_first_visible = false;

        virtual_y += field_height_with_spacing;
    }

    // Draw scrollbar if content exceeds viewport
    if total_height > viewport_height {
        let scrollbar_area = Rect {
            x: form_area.x + form_area.width - 1,
            y: form_area.y,
            width: 1,
            height: viewport_height,
        };

        let mut scrollbar_state = ScrollbarState::new(total_height as usize)
            .position(scroll_offset as usize)
            .viewport_content_length(viewport_height as usize);

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("▲"))
            .end_symbol(Some("▼"))
            .track_symbol(Some("│"))
            .thumb_symbol("█");

        frame.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
    }

    // Submit button
    let submit_style = if state.submit_focused {
        Style::default()
            .fg(theme.text)
            .bg(theme.primary)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default()
            .fg(theme.primary)
            .add_modifier(Modifier::BOLD)
    };
    let submit_text = if state.submit_focused {
        "  [ Submit ]  "
    } else {
        "    Submit    "
    };
    let submit_button = Paragraph::new(submit_text)
        .style(submit_style)
        .alignment(ratatui::layout::Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(if state.submit_focused {
                    theme.primary
                } else {
                    theme.border
                })),
        );
    frame.render_widget(submit_button, chunks[2]);

    // Help bar
    let help_text = "Tab: Next  ↑/↓: Navigate  Space/Enter: Select  Esc: Cancel";
    let help = Paragraph::new(help_text).style(Style::default().fg(theme.border));
    frame.render_widget(help, chunks[3]);
}

/// Extra vertical space before top-level fields (section spacing)
const TOP_LEVEL_SPACING: u16 = 1;

fn get_field_height(field: &FormField, include_spacing: bool) -> u16 {
    let base_height = match &field.kind {
        FieldKind::Text {
            multiline: true, ..
        } => 4,
        FieldKind::OneOf { variants, .. } => 2 + variants.len() as u16,
        FieldKind::AnyOf { variants, .. } => 2 + variants.len() as u16,
        _ => 3,
    };
    // Add spacing before top-level fields (except the first one)
    if include_spacing && field.is_top_level {
        base_height + TOP_LEVEL_SPACING
    } else {
        base_height
    }
}

fn draw_field(frame: &mut Frame, field: &FormField, area: Rect, is_focused: bool, theme: &Theme) {
    let border_color = if field.error.is_some() {
        theme.error
    } else if is_focused {
        theme.primary
    } else {
        theme.border
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(format!(" {} ", field.prompt))
        .title_style(Style::default().fg(if is_focused {
            theme.highlight
        } else {
            theme.text
        }));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    match &field.kind {
        FieldKind::Text { masked, .. } => {
            let display_text = if *masked {
                "●".repeat(field.value.len())
            } else {
                field.value.clone()
            };
            let text = Paragraph::new(display_text).style(Style::default().fg(theme.text));
            frame.render_widget(text, inner);

            if is_focused {
                let cursor_x = inner.x + field.cursor_pos as u16;
                let cursor_y = inner.y;
                if cursor_x < inner.x + inner.width {
                    frame.set_cursor_position((cursor_x, cursor_y));
                }
            }
        }
        FieldKind::Int { .. } | FieldKind::Float { .. } => {
            let text = Paragraph::new(field.value.clone()).style(Style::default().fg(theme.text));
            frame.render_widget(text, inner);

            if is_focused {
                let cursor_x = inner.x + field.cursor_pos as u16;
                let cursor_y = inner.y;
                if cursor_x < inner.x + inner.width {
                    frame.set_cursor_position((cursor_x, cursor_y));
                }
            }
        }
        FieldKind::Bool => {
            let checked = field.value == "true";
            let checkbox = if checked { "[✓]" } else { "[ ]" };
            let text = Paragraph::new(format!("{} Yes", checkbox))
                .style(Style::default().fg(if checked { theme.success } else { theme.text }));
            frame.render_widget(text, inner);
        }
        FieldKind::List { .. } => {
            let text = Paragraph::new(field.value.clone()).style(Style::default().fg(theme.text));
            frame.render_widget(text, inner);

            if is_focused {
                let cursor_x = inner.x + field.cursor_pos as u16;
                let cursor_y = inner.y;
                if cursor_x < inner.x + inner.width {
                    frame.set_cursor_position((cursor_x, cursor_y));
                }
            }
        }
        FieldKind::OneOf {
            variants,
            selected,
            highlight,
        } => {
            let items: Vec<ListItem> = variants
                .iter()
                .enumerate()
                .map(|(idx, v)| {
                    let marker = if *selected == Some(idx) {
                        "(●)"
                    } else {
                        "( )"
                    };
                    let is_highlighted = is_focused && idx == *highlight;
                    let style = if is_highlighted {
                        Style::default()
                            .fg(theme.text)
                            .bg(theme.selected_bg)
                            .add_modifier(Modifier::BOLD)
                    } else if *selected == Some(idx) {
                        Style::default().fg(theme.highlight)
                    } else {
                        Style::default().fg(theme.text)
                    };
                    ListItem::new(format!("{} {}", marker, v.name)).style(style)
                })
                .collect();
            let list = List::new(items);
            frame.render_widget(list, inner);
        }
        FieldKind::AnyOf {
            variants,
            selected,
            highlight,
        } => {
            let items: Vec<ListItem> = variants
                .iter()
                .enumerate()
                .map(|(idx, v)| {
                    let checked = selected.get(idx).copied().unwrap_or(false);
                    let marker = if checked { "[✓]" } else { "[ ]" };
                    let is_highlighted = is_focused && idx == *highlight;
                    let style = if is_highlighted {
                        Style::default()
                            .fg(if checked { theme.success } else { theme.text })
                            .bg(theme.selected_bg)
                            .add_modifier(Modifier::BOLD)
                    } else if checked {
                        Style::default().fg(theme.success)
                    } else {
                        Style::default().fg(theme.text)
                    };
                    ListItem::new(format!("{} {}", marker, v.name)).style(style)
                })
                .collect();
            let list = List::new(items);
            frame.render_widget(list, inner);
        }
    }

    // Show error if any
    if let Some(error) = &field.error {
        let error_y = area.y + area.height - 1;
        if error_y < area.y + area.height {
            let error_text =
                Paragraph::new(format!("⚠ {}", error)).style(Style::default().fg(theme.error));
            let error_area = Rect {
                x: area.x + 1,
                y: error_y,
                width: area.width.saturating_sub(2),
                height: 1,
            };
            frame.render_widget(error_text, error_area);
        }
    }
}

impl SurveyBackend for RatatuiFormBackend {
    type Error = RatatuiFormError;

    fn collect(
        &self,
        definition: &SurveyDefinition,
        validate: &dyn Fn(&ResponseValue, &Responses, &ResponsePath) -> Result<(), String>,
    ) -> Result<Responses, Self::Error> {
        let mut terminal = self.setup_terminal()?;
        let mut state = FormState::new(definition, self.theme.clone(), self.title.clone());

        // Skip to first visible field
        while state.focused_idx < state.fields.len()
            && !state.is_field_visible_by_idx(state.focused_idx)
        {
            state.focused_idx += 1;
        }

        loop {
            terminal.draw(|frame| draw_form(frame, &mut state))?;

            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                match key.code {
                    KeyCode::Esc => {
                        state.cancelled = true;
                        break;
                    }
                    // Ctrl+Enter or F10 to submit the form
                    KeyCode::Enter if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        if state.validate_all(validate) {
                            state.submitted = true;
                            break;
                        }
                    }
                    KeyCode::F(10) => {
                        if state.validate_all(validate) {
                            state.submitted = true;
                            break;
                        }
                    }
                    // Enter: submit if on button, select option, or move to next field
                    KeyCode::Enter => {
                        if state.submit_focused {
                            if state.validate_all(validate) {
                                state.submitted = true;
                                break;
                            }
                        } else if state.is_selection_field() {
                            state.select_option();
                        } else {
                            state.next_field();
                        }
                    }
                    // Shift+Tab: previous field
                    KeyCode::BackTab | KeyCode::Tab
                        if key.modifiers.contains(KeyModifiers::SHIFT) =>
                    {
                        state.prev_field();
                    }
                    // Tab: next field
                    KeyCode::Tab => {
                        state.next_field();
                    }
                    // Up/Down: navigate options or fields
                    KeyCode::Up => {
                        if state.is_selection_field() {
                            state.option_up();
                        } else {
                            state.prev_field();
                        }
                    }
                    KeyCode::Down => {
                        if state.is_selection_field() {
                            state.option_down();
                        } else {
                            state.next_field();
                        }
                    }
                    // Ctrl+arrows: navigate between fields
                    KeyCode::Left if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        state.prev_field();
                    }
                    KeyCode::Right if key.modifiers.contains(KeyModifiers::CONTROL) => {
                        state.next_field();
                    }
                    // Left/Right: cursor movement in text fields
                    KeyCode::Left => {
                        state.cursor_left();
                    }
                    KeyCode::Right => {
                        state.cursor_right();
                    }
                    // Space: toggle bool, select OneOf option, toggle AnyOf option
                    KeyCode::Char(' ') => {
                        if let Some(field) = state.focused_field() {
                            match &field.kind {
                                FieldKind::Bool => state.toggle_bool(),
                                FieldKind::OneOf { .. } | FieldKind::AnyOf { .. } => {
                                    state.select_option();
                                }
                                _ => state.handle_text_input(' '),
                            }
                        }
                    }
                    // Number keys: quick toggle for AnyOf (1-9)
                    KeyCode::Char(c) if c.is_ascii_digit() => {
                        if let Some(field) = state.focused_field() {
                            match &field.kind {
                                FieldKind::AnyOf { .. } => {
                                    let idx = c.to_digit(10).unwrap() as usize;
                                    if idx > 0 {
                                        state.toggle_anyof(idx - 1);
                                    }
                                }
                                _ => state.handle_text_input(c),
                            }
                        } else {
                            state.handle_text_input(c);
                        }
                    }
                    KeyCode::Char(c) => {
                        state.handle_text_input(c);
                    }
                    KeyCode::Backspace => {
                        state.handle_backspace();
                    }
                    KeyCode::Delete => {
                        state.handle_delete();
                    }
                    KeyCode::Home => {
                        if let Some(field) = state.focused_field_mut() {
                            field.cursor_pos = 0;
                        }
                    }
                    KeyCode::End => {
                        if let Some(field) = state.focused_field_mut() {
                            field.cursor_pos = field.value.len();
                        }
                    }
                    KeyCode::PageDown => {
                        // Jump multiple fields down
                        for _ in 0..5 {
                            state.next_field();
                        }
                    }
                    KeyCode::PageUp => {
                        // Jump multiple fields up
                        for _ in 0..5 {
                            state.prev_field();
                        }
                    }
                    _ => {}
                }
            }
        }

        self.restore_terminal(&mut terminal)?;

        if state.cancelled {
            return Err(RatatuiFormError::Cancelled);
        }

        Ok(state.collect_responses())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_creation() {
        let _backend = RatatuiFormBackend::new();
        let _with_title = RatatuiFormBackend::new().with_title("Test");
        let _with_theme = RatatuiFormBackend::new().with_theme(Theme::default());
    }

    #[test]
    fn error_types() {
        let err = RatatuiFormError::Cancelled;
        assert_eq!(err.to_string(), "Form cancelled by user");

        let err = RatatuiFormError::Terminal("test error".to_string());
        assert_eq!(err.to_string(), "Terminal error: test error");
    }

    #[test]
    fn theme_default() {
        let theme = Theme::default();
        assert_eq!(theme.primary, Color::Cyan);
        assert_eq!(theme.error, Color::Red);
    }
}
