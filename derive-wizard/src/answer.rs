use std::collections::HashMap;

#[derive(Debug, thiserror::Error)]
pub enum AnswerError {
    #[error("Missing key: {0}")]
    MissingKey(String),

    #[error("Type mismatch for key '{key}': expected {expected}")]
    TypeMismatch { key: String, expected: &'static str },
}

/// Represents the answers collected from an interview
#[derive(Debug, Clone, Default)]
pub struct Answers {
    values: HashMap<String, AnswerValue>,
}

/// A single answer value
#[derive(Debug, Clone)]
pub enum AnswerValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Nested(Box<Answers>),
    /// List of selected indices (for multi-select questions)
    IntList(Vec<i64>),
}

impl Answers {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: String, value: AnswerValue) {
        self.values.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<&AnswerValue> {
        self.values.get(key)
    }

    pub fn merge(&mut self, other: Answers) {
        self.values.extend(other.values);
    }

    pub fn as_string(&self, key: &str) -> Result<String, AnswerError> {
        match self.get(key) {
            Some(AnswerValue::String(s)) => Ok(s.clone()),
            Some(_) => Err(AnswerError::TypeMismatch {
                key: key.to_string(),
                expected: "String",
            }),
            None => Err(AnswerError::MissingKey(key.to_string())),
        }
    }

    pub fn as_int(&self, key: &str) -> Result<i64, AnswerError> {
        match self.get(key) {
            Some(AnswerValue::Int(i)) => Ok(*i),
            Some(_) => Err(AnswerError::TypeMismatch {
                key: key.to_string(),
                expected: "Int",
            }),
            None => Err(AnswerError::MissingKey(key.to_string())),
        }
    }

    pub fn as_float(&self, key: &str) -> Result<f64, AnswerError> {
        match self.get(key) {
            Some(AnswerValue::Float(f)) => Ok(*f),
            Some(_) => Err(AnswerError::TypeMismatch {
                key: key.to_string(),
                expected: "Float",
            }),
            None => Err(AnswerError::MissingKey(key.to_string())),
        }
    }

    pub fn as_bool(&self, key: &str) -> Result<bool, AnswerError> {
        match self.get(key) {
            Some(AnswerValue::Bool(b)) => Ok(*b),
            Some(_) => Err(AnswerError::TypeMismatch {
                key: key.to_string(),
                expected: "Bool",
            }),
            None => Err(AnswerError::MissingKey(key.to_string())),
        }
    }

    pub fn as_nested(&self, key: &str) -> Result<&Answers, AnswerError> {
        match self.get(key) {
            Some(AnswerValue::Nested(nested)) => Ok(nested),
            Some(_) => Err(AnswerError::TypeMismatch {
                key: key.to_string(),
                expected: "Nested",
            }),
            None => Err(AnswerError::MissingKey(key.to_string())),
        }
    }

    /// Get a list of integers (for multi-select answers)
    pub fn as_int_list(&self, key: &str) -> Result<Vec<i64>, AnswerError> {
        match self.get(key) {
            Some(AnswerValue::IntList(list)) => Ok(list.clone()),
            Some(_) => Err(AnswerError::TypeMismatch {
                key: key.to_string(),
                expected: "IntList",
            }),
            None => Err(AnswerError::MissingKey(key.to_string())),
        }
    }

    /// Iterate over all key-value pairs
    pub fn iter(&self) -> impl Iterator<Item = (&String, &AnswerValue)> {
        self.values.iter()
    }
}
