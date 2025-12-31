use crate::default::SuggestedAnswer;

/// A sequence of sections, which contain questions.
#[derive(Debug, Clone)]
pub struct Interview {
    pub sections: Vec<Question>,
}

#[derive(Debug, Clone)]
pub struct Question {
    /// The unique identifier for the question.
    id: Option<String>,

    /// The field name.
    name: String,

    /// The prompt message to display.
    prompt: String,

    kind: QuestionKind,
}

impl Question {
    /// Create a new question with the given id, name, prompt, and kind.
    pub fn new(id: Option<String>, name: String, prompt: String, kind: QuestionKind) -> Self {
        Self {
            id,
            name,
            prompt,
            kind,
        }
    }

    pub fn id(&self) -> Option<&str> {
        self.id.as_deref()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn prompt(&self) -> &str {
        &self.prompt
    }

    pub fn kind(&self) -> &QuestionKind {
        &self.kind
    }

    /// Set the suggested value for this question based on its kind.
    pub fn set_suggestion(&mut self, value: impl Into<SuggestedAnswer>) {
        match (&mut self.kind, value.into()) {
            (QuestionKind::Input(q), SuggestedAnswer::String(v)) => {
                q.default = Some(v);
            }
            (QuestionKind::Multiline(q), SuggestedAnswer::String(v)) => {
                q.default = Some(v);
            }
            (QuestionKind::Int(q), SuggestedAnswer::Int(v)) => {
                q.default = Some(v);
            }
            (QuestionKind::Float(q), SuggestedAnswer::Float(v)) => {
                q.default = Some(v);
            }
            (QuestionKind::Confirm(q), SuggestedAnswer::Bool(v)) => {
                q.default = v;
            }
            _ => {}
        }
    }
}

/// Possible question kinds which a wizard may ask.
#[derive(Debug, Clone)]
pub enum QuestionKind {
    /// A text input question for string values.
    Input(InputQuestion),

    /// A multi-line text input.
    Multiline(MultilineQuestion),

    /// A password/masked input question.
    Masked(MaskedQuestion),

    /// A number input question (integers).
    Int(IntQuestion),

    /// A number input question (floating point).
    Float(FloatQuestion),

    /// A yes/no confirmation question.
    Confirm(ConfirmQuestion),

    Sequence(Vec<Question>),

    Alternative(usize, Vec<Question>),
}

/// Configuration for a text input question.
#[derive(Debug, Clone)]
pub struct InputQuestion {
    /// Optional default value.
    pub default: Option<String>,

    /// Validation function to call on each keystroke.
    pub validate_on_key: Option<String>,

    /// Validation function to call on submission.
    pub validate_on_submit: Option<String>,
}

/// Configuration for a multi-line text editor question.
#[derive(Debug, Clone)]
pub struct MultilineQuestion {
    /// Optional default value.
    pub default: Option<String>,

    /// Validation function to call on each keystroke.
    pub validate_on_key: Option<String>,

    /// Validation function to call on submission.
    pub validate_on_submit: Option<String>,
}

/// Configuration for a password/masked input question.
#[derive(Debug, Clone)]
pub struct MaskedQuestion {
    /// The masking character (default: '*').
    pub mask: Option<char>,

    /// Validation function to call on each keystroke.
    pub validate_on_key: Option<String>,

    /// Validation function to call on submission.
    pub validate_on_submit: Option<String>,
}

/// Configuration for an integer input question.
#[derive(Debug, Clone)]
pub struct IntQuestion {
    /// Optional default value
    pub default: Option<i64>,

    /// Optional minimum value
    pub min: Option<i64>,

    /// Optional maximum value
    pub max: Option<i64>,

    /// Validation function to call on each keystroke.
    pub validate_on_key: Option<String>,

    /// Validation function to call on submission.
    pub validate_on_submit: Option<String>,
}

/// Configuration for a floating-point input question.
#[derive(Debug, Clone)]
pub struct FloatQuestion {
    /// Optional default value.
    pub default: Option<f64>,

    /// Optional minimum value
    pub min: Option<f64>,

    /// Optional maximum value
    pub max: Option<f64>,

    /// Validation function to call on each keystroke.
    pub validate_on_key: Option<String>,

    /// Validation function to call on submission.
    pub validate_on_submit: Option<String>,
}

/// Configuration for a yes/no confirmation question.
#[derive(Debug, Clone)]
pub struct ConfirmQuestion {
    /// Default value (true for yes, false for no)
    pub default: bool,
}
