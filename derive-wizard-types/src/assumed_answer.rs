/// Represents an assumed value that skips the question entirely.
#[derive(Debug, Clone)]
pub enum AssumedAnswer {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}

impl From<String> for AssumedAnswer {
    fn from(v: String) -> Self {
        Self::String(v)
    }
}

impl From<i64> for AssumedAnswer {
    fn from(v: i64) -> Self {
        Self::Int(v)
    }
}

impl From<f64> for AssumedAnswer {
    fn from(v: f64) -> Self {
        Self::Float(v)
    }
}

impl From<bool> for AssumedAnswer {
    fn from(v: bool) -> Self {
        Self::Bool(v)
    }
}
