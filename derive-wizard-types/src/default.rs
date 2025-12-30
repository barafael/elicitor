/// Represents a default value that can be set on a question.
pub enum QuestionDefault {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}

impl From<String> for QuestionDefault {
    fn from(v: String) -> Self {
        QuestionDefault::String(v)
    }
}

impl From<i64> for QuestionDefault {
    fn from(v: i64) -> Self {
        QuestionDefault::Int(v)
    }
}

impl From<f64> for QuestionDefault {
    fn from(v: f64) -> Self {
        QuestionDefault::Float(v)
    }
}

impl From<bool> for QuestionDefault {
    fn from(v: bool) -> Self {
        QuestionDefault::Bool(v)
    }
}
