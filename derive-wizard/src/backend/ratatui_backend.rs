//! Ratatui-based TUI backend for derive-wizard
//!
//! Provides a rich terminal UI with panels, progress indicators,
//! and keyboard navigation for wizard interviews.

use crate::backend::{AnswerValue, Answers, BackendError, InterviewBackend, ValidatorFn};
use crate::interview::{Interview, Question, QuestionKind};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use derive_wizard_types::AssumedAnswer;
use ratatui::{
    Frame, Terminal,
    layout::{Alignment, Constraint, Direction, Layout},
    prelude::CrosstermBackend,
    style::{Color, Style, Stylize},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};
use std::io::{self, Stdout};

/// Ratatui-based TUI backend with rich visual interface
pub struct RatatuiBackend {
    /// Title shown at the top of the wizard
    title: String,
    /// Theme colors
    theme: Theme,
}

/// Color theme for the TUI
#[derive(Clone)]
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

impl RatatuiBackend {
    pub fn new() -> Self {
        Self {
            title: "Wizard".to_string(),
            theme: Theme::default(),
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    fn setup_terminal(&self) -> Result<Terminal<CrosstermBackend<Stdout>>, BackendError> {
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
    ) -> Result<(), BackendError> {
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

impl Default for RatatuiBackend {
    fn default() -> Self {
        Self::new()
    }
}

/// State for the entire wizard
struct WizardState {
    /// All flattened questions
    questions: Vec<FlatQuestion>,
    /// Current question index
    current_index: usize,
    /// Collected answers
    answers: Answers,
    /// Current input buffer
    input: String,
    /// Cursor position in input
    cursor_pos: usize,
    /// For select/confirm questions
    selected_option: usize,
    /// For multi-select questions: which options are selected
    multi_selected: Vec<bool>,
    /// Current validation error message
    error_message: Option<String>,
    /// Whether wizard is complete
    complete: bool,
    /// Whether user cancelled
    cancelled: bool,
    /// Theme
    theme: Theme,
    /// Title
    title: String,
    /// Epilogue text
    epilogue: Option<String>,
}

/// A flattened question for easier processing
#[derive(Clone)]
struct FlatQuestion {
    id: String,
    prompt: String,
    kind: FlatQuestionKind,
    default_value: Option<String>,
    assumed: Option<AssumedAnswer>,
    /// Whether this field has custom validation
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
    Select {
        options: Vec<String>,
        default_idx: usize,
        /// For enum variants: the alternatives with their nested questions
        alternatives: Option<Vec<Question>>,
        /// For enum variants: the prefix for nested field IDs
        id_prefix: Option<String>,
    },
    MultiSelect {
        options: Vec<String>,
        defaults: Vec<usize>,
    },
}

impl WizardState {
    fn new(interview: &Interview, theme: Theme, title: String) -> Self {
        let questions = Self::flatten_questions(&interview.sections, "");
        // If there's a prelude, include it in the title
        let display_title = if let Some(ref prelude) = interview.prelude {
            format!("{}\n{}", title, prelude)
        } else {
            title
        };

        // Initialize state for the first question
        let (selected_option, multi_selected) = if let Some(first) = questions.first() {
            match &first.kind {
                FlatQuestionKind::MultiSelect { options, defaults } => {
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
            answers: Answers::new(),
            input: String::new(),
            cursor_pos: 0,
            selected_option,
            multi_selected,
            error_message: None,
            complete: false,
            cancelled: false,
            theme,
            title: display_title,
            epilogue: interview.epilogue.clone(),
        }
    }

    fn flatten_questions(questions: &[Question], prefix: &str) -> Vec<FlatQuestion> {
        let mut flat = Vec::new();

        for question in questions {
            let id = if prefix.is_empty() {
                question.id().unwrap_or(question.name()).to_string()
            } else {
                format!("{}.{}", prefix, question.id().unwrap_or(question.name()))
            };

            match question.kind() {
                QuestionKind::Input(input_q) => {
                    flat.push(FlatQuestion {
                        id,
                        prompt: question.prompt().to_string(),
                        kind: FlatQuestionKind::Input,
                        default_value: input_q.default.clone(),
                        assumed: question.assumed().cloned(),
                        has_validation: input_q.validate.is_some(),
                    });
                }
                QuestionKind::Multiline(ml_q) => {
                    flat.push(FlatQuestion {
                        id,
                        prompt: question.prompt().to_string(),
                        kind: FlatQuestionKind::Multiline,
                        default_value: ml_q.default.clone(),
                        assumed: question.assumed().cloned(),
                        has_validation: ml_q.validate.is_some(),
                    });
                }
                QuestionKind::Masked(masked_q) => {
                    flat.push(FlatQuestion {
                        id,
                        prompt: question.prompt().to_string(),
                        kind: FlatQuestionKind::Masked,
                        default_value: None,
                        assumed: question.assumed().cloned(),
                        has_validation: masked_q.validate.is_some(),
                    });
                }
                QuestionKind::Int(int_q) => {
                    flat.push(FlatQuestion {
                        id,
                        prompt: question.prompt().to_string(),
                        kind: FlatQuestionKind::Int {
                            min: int_q.min,
                            max: int_q.max,
                        },
                        default_value: int_q.default.map(|d| d.to_string()),
                        assumed: question.assumed().cloned(),
                        has_validation: int_q.validate.is_some(),
                    });
                }
                QuestionKind::Float(float_q) => {
                    flat.push(FlatQuestion {
                        id,
                        prompt: question.prompt().to_string(),
                        kind: FlatQuestionKind::Float {
                            min: float_q.min,
                            max: float_q.max,
                        },
                        default_value: float_q.default.map(|d| d.to_string()),
                        assumed: question.assumed().cloned(),
                        has_validation: float_q.validate.is_some(),
                    });
                }
                QuestionKind::Confirm(confirm_q) => {
                    flat.push(FlatQuestion {
                        id,
                        prompt: question.prompt().to_string(),
                        kind: FlatQuestionKind::Confirm {
                            default: confirm_q.default,
                        },
                        default_value: Some(
                            if confirm_q.default { "yes" } else { "no" }.to_string(),
                        ),
                        assumed: question.assumed().cloned(),
                        has_validation: false,
                    });
                }
                QuestionKind::MultiSelect(multi_q) => {
                    flat.push(FlatQuestion {
                        id,
                        prompt: question.prompt().to_string(),
                        kind: FlatQuestionKind::MultiSelect {
                            options: multi_q.options.clone(),
                            defaults: multi_q.defaults.clone(),
                        },
                        default_value: None,
                        assumed: question.assumed().cloned(),
                        has_validation: false,
                    });
                }
                QuestionKind::Sequence(nested) => {
                    if question.kind().is_enum_alternatives() {
                        let options: Vec<String> =
                            nested.iter().map(|q| q.name().to_string()).collect();
                        let parent_id = id.strip_suffix(".alternatives").unwrap_or(&id);

                        flat.push(FlatQuestion {
                            id: format!("{}.{}", parent_id, crate::SELECTED_ALTERNATIVE_KEY),
                            prompt: question.prompt().to_string(),
                            kind: FlatQuestionKind::Select {
                                options,
                                default_idx: 0,
                                alternatives: Some(nested.clone()),
                                id_prefix: Some(parent_id.to_string()),
                            },
                            default_value: None,
                            assumed: question.assumed().cloned(),
                            has_validation: false,
                        });
                    } else {
                        flat.extend(Self::flatten_questions(nested, &id));
                    }
                }
                QuestionKind::Alternative(default_idx, alternatives) => {
                    let options: Vec<String> =
                        alternatives.iter().map(|a| a.name().to_string()).collect();
                    flat.push(FlatQuestion {
                        id: crate::SELECTED_ALTERNATIVE_KEY.to_string(),
                        prompt: question.prompt().to_string(),
                        kind: FlatQuestionKind::Select {
                            options,
                            default_idx: *default_idx,
                            alternatives: Some(alternatives.clone()),
                            id_prefix: None,
                        },
                        default_value: None,
                        assumed: question.assumed().cloned(),
                        has_validation: false,
                    });
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

    fn validate_and_submit(&mut self, validator: Option<ValidatorFn<'_>>) -> bool {
        let Some(question) = self.current_question().cloned() else {
            return false;
        };

        // Use default if input is empty and default exists
        let value = if self.input.is_empty() {
            question.default_value.clone().unwrap_or_default()
        } else {
            self.input.clone()
        };

        match &question.kind {
            FlatQuestionKind::Input | FlatQuestionKind::Multiline | FlatQuestionKind::Masked => {
                // Run custom validation if field has it and validator is provided
                if question.has_validation
                    && let Some(validate) = validator
                    && let Err(err) = validate(&question.id, &value, &self.answers)
                {
                    self.error_message = Some(err);
                    return false;
                }
                self.answers
                    .insert(question.id.clone(), AnswerValue::String(value));
            }
            FlatQuestionKind::Int { min, max } => match value.parse::<i64>() {
                Ok(n) => {
                    if let Some(min_val) = min
                        && n < *min_val
                    {
                        self.error_message = Some(format!("Value must be at least {}", min_val));
                        return false;
                    }
                    if let Some(max_val) = max
                        && n > *max_val
                    {
                        self.error_message = Some(format!("Value must be at most {}", max_val));
                        return false;
                    }
                    // Run custom validation if field has it and validator is provided
                    if question.has_validation
                        && let Some(validate) = validator
                        && let Err(err) = validate(&question.id, &value, &self.answers)
                    {
                        self.error_message = Some(err);
                        return false;
                    }
                    self.answers
                        .insert(question.id.clone(), AnswerValue::Int(n));
                }
                Err(_) => {
                    self.error_message = Some("Please enter a valid integer".to_string());
                    return false;
                }
            },
            FlatQuestionKind::Float { min, max } => match value.parse::<f64>() {
                Ok(n) => {
                    if let Some(min_val) = min
                        && n < *min_val
                    {
                        self.error_message = Some(format!("Value must be at least {}", min_val));
                        return false;
                    }
                    if let Some(max_val) = max
                        && n > *max_val
                    {
                        self.error_message = Some(format!("Value must be at most {}", max_val));
                        return false;
                    }
                    // Run custom validation if field has it and validator is provided
                    if question.has_validation
                        && let Some(validate) = validator
                        && let Err(err) = validate(&question.id, &value, &self.answers)
                    {
                        self.error_message = Some(err);
                        return false;
                    }
                    self.answers
                        .insert(question.id.clone(), AnswerValue::Float(n));
                }
                Err(_) => {
                    self.error_message = Some("Please enter a valid number".to_string());
                    return false;
                }
            },
            FlatQuestionKind::Confirm { .. } => {
                let answer = self.selected_option == 0; // 0 = Yes, 1 = No
                self.answers
                    .insert(question.id.clone(), AnswerValue::Bool(answer));
            }
            FlatQuestionKind::Select {
                alternatives,
                id_prefix,
                ..
            } => {
                // Store the selected variant index
                self.answers.insert(
                    question.id.clone(),
                    AnswerValue::Int(self.selected_option as i64),
                );

                // For enum variants: expand the selected variant's fields
                if let Some(alts) = alternatives
                    && let Some(selected_variant) = alts.get(self.selected_option)
                    && let QuestionKind::Alternative(_, variant_fields) = selected_variant.kind()
                {
                    // Flatten the variant's fields and insert them after the current question
                    let prefix = id_prefix.clone().unwrap_or_default();
                    let variant_questions = Self::flatten_questions(variant_fields, &prefix);
                    if !variant_questions.is_empty() {
                        // Insert after current position
                        let insert_pos = self.current_index + 1;
                        for (i, q) in variant_questions.into_iter().enumerate() {
                            self.questions.insert(insert_pos + i, q);
                        }
                    }
                }
            }
            FlatQuestionKind::MultiSelect { .. } => {
                // Collect indices of all selected options
                let selected_indices: Vec<i64> = self
                    .multi_selected
                    .iter()
                    .enumerate()
                    .filter_map(|(i, &selected)| if selected { Some(i as i64) } else { None })
                    .collect();
                self.answers
                    .insert(question.id.clone(), AnswerValue::IntList(selected_indices));
            }
        }

        true
    }

    fn next_question(&mut self, validator: Option<ValidatorFn<'_>>) {
        if self.validate_and_submit(validator) {
            self.current_index += 1;
            self.input.clear();
            self.cursor_pos = 0;
            self.selected_option = 0;
            self.multi_selected.clear();
            self.error_message = None;

            // Skip assumed questions
            while self.current_index < self.questions.len() {
                if let Some(assumed) = &self.questions[self.current_index].assumed {
                    self.answers.insert(
                        self.questions[self.current_index].id.clone(),
                        assumed.into(),
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
                            FlatQuestionKind::MultiSelect { options, defaults } => {
                                // Copy values to avoid borrow issue
                                let opts_len = options.len();
                                let defs = defaults.clone();
                                // Initialize multi_selected with defaults
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

            // Restore previous answer as input
            if let Some(q) = self.current_question()
                && let Some(prev_answer) = self.answers.get(&q.id)
            {
                match prev_answer {
                    AnswerValue::String(s) => {
                        self.input = s.clone();
                        self.cursor_pos = self.input.len();
                    }
                    AnswerValue::Int(n) => {
                        self.input = n.to_string();
                        self.cursor_pos = self.input.len();
                    }
                    AnswerValue::Float(n) => {
                        self.input = n.to_string();
                        self.cursor_pos = self.input.len();
                    }
                    AnswerValue::Bool(b) => {
                        self.selected_option = if *b { 0 } else { 1 };
                    }
                    AnswerValue::IntList(indices) => {
                        // Restore multi-select state
                        if let FlatQuestionKind::MultiSelect { options, .. } = &q.kind {
                            self.multi_selected = vec![false; options.len()];
                            for &idx in indices {
                                if (idx as usize) < self.multi_selected.len() {
                                    self.multi_selected[idx as usize] = true;
                                }
                            }
                        }
                    }
                    AnswerValue::Nested(_) => {
                        // Nested answers are handled through flattening,
                        // individual fields should be restored separately
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

    // Progress
    let (current, total) = state.progress();
    let progress_text = format!("Question {} of {}", current, total);
    let progress_bar_width = chunks[1]
        .width
        .saturating_sub(progress_text.len() as u16 + 4);
    let filled = (current as f32 / total as f32 * progress_bar_width as f32) as usize;
    let empty = progress_bar_width as usize - filled;
    let progress_bar = format!(
        "{} [{}{}]",
        progress_text,
        "█".repeat(filled),
        "░".repeat(empty)
    );
    let progress = Paragraph::new(progress_bar)
        .style(Style::default().fg(state.theme.secondary))
        .alignment(Alignment::Center);
    frame.render_widget(progress, chunks[1]);

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
                let masked_input = "•".repeat(state.input.len());
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
            "↑/↓: Select  Enter: Confirm  Esc: Cancel  ←: Back"
        }
        Some(FlatQuestionKind::MultiSelect { .. }) => {
            "↑/↓: Navigate  Space: Toggle  Enter: Confirm  Esc: Cancel  ←: Back"
        }
        _ => "Enter: Submit  Esc: Cancel  ←: Back",
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
        .title(" ✓ Complete ")
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

impl InterviewBackend for RatatuiBackend {
    fn execute(&self, interview: &Interview) -> Result<Answers, BackendError> {
        self.execute_internal(interview, None)
    }

    fn execute_with_validator(
        &self,
        interview: &Interview,
        validator: ValidatorFn<'_>,
    ) -> Result<Answers, BackendError> {
        self.execute_internal(interview, Some(validator))
    }
}

impl RatatuiBackend {
    fn execute_internal(
        &self,
        interview: &Interview,
        validator: Option<ValidatorFn<'_>>,
    ) -> Result<Answers, BackendError> {
        let mut terminal = self.setup_terminal()?;
        let mut state = WizardState::new(interview, self.theme.clone(), self.title.clone());

        // Skip initially assumed questions
        while state.current_index < state.questions.len() {
            if let Some(assumed) = &state.questions[state.current_index].assumed {
                state.answers.insert(
                    state.questions[state.current_index].id.clone(),
                    assumed.into(),
                );
                state.current_index += 1;
            } else {
                // Initialize first question's defaults
                if let Some(q) = state.current_question() {
                    match &q.kind {
                        FlatQuestionKind::Confirm { default } => {
                            state.selected_option = if *default { 0 } else { 1 };
                        }
                        FlatQuestionKind::Select { default_idx, .. } => {
                            state.selected_option = *default_idx;
                        }
                        _ => {
                            if let Some(def) = &q.default_value {
                                state.input = def.clone();
                                state.cursor_pos = state.input.len();
                            }
                        }
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
                            state.next_question(validator);
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
            return Err(BackendError::ExecutionError(
                "Wizard cancelled by user".to_string(),
            ));
        }

        Ok(state.answers)
    }
}
