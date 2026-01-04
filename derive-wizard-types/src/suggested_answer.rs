/// Represents a suggested value that can be set on a question.
pub enum SuggestedAnswer {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}

impl From<String> for SuggestedAnswer {
    fn from(v: String) -> Self {
        Self::String(v)
    }
}

impl From<i64> for SuggestedAnswer {
    fn from(v: i64) -> Self {
        Self::Int(v)
    }
}

impl From<f64> for SuggestedAnswer {
    fn from(v: f64) -> Self {
        Self::Float(v)
    }
}

impl From<bool> for SuggestedAnswer {
    fn from(v: bool) -> Self {
        Self::Bool(v)
    }
}
