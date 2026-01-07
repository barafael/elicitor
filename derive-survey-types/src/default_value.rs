use crate::ResponseValue;

/// Default value for a question.
///
/// Controls whether a question has a pre-filled value and whether it's shown to the user.
#[derive(Debug, Clone, Default, PartialEq)]
pub enum DefaultValue {
    /// No default value - user must provide input.
    #[default]
    None,

    /// A suggested value that the user can accept or modify.
    /// The question is shown with this value pre-filled.
    Suggested(ResponseValue),

    /// An assumed value that skips the question entirely.
    /// The question is not shown; this value is used directly.
    Assumed(ResponseValue),
}

impl DefaultValue {
    /// Check if this is the None variant.
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    /// Check if this is an assumed value (question should be skipped).
    pub fn is_assumed(&self) -> bool {
        matches!(self, Self::Assumed(_))
    }

    /// Check if this is a suggested value.
    pub fn is_suggested(&self) -> bool {
        matches!(self, Self::Suggested(_))
    }

    /// Get the inner value if this is Suggested or Assumed.
    pub fn value(&self) -> Option<&ResponseValue> {
        match self {
            Self::None => None,
            Self::Suggested(v) | Self::Assumed(v) => Some(v),
        }
    }
}
