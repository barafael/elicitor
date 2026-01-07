/// A single response value collected from a survey.
///
/// This is the value stored in `Responses` for each answered question.
#[derive(Debug, Clone, PartialEq)]
pub enum ResponseValue {
    /// A string value (from Input, Multiline, or Masked questions).
    String(String),

    /// An integer value (from Int questions).
    Int(i64),

    /// A floating-point value (from Float questions).
    Float(f64),

    /// A boolean value (from Confirm questions).
    Bool(bool),

    /// The index of the chosen variant in a OneOf question (enum selection).
    ChosenVariant(usize),

    /// The indices of chosen variants in an AnyOf question (multi-select).
    ChosenVariants(Vec<usize>),
}

impl ResponseValue {
    /// Try to get this value as a string reference.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    /// Try to get this value as an integer.
    pub fn as_int(&self) -> Option<i64> {
        match self {
            Self::Int(i) => Some(*i),
            _ => None,
        }
    }

    /// Try to get this value as a float.
    pub fn as_float(&self) -> Option<f64> {
        match self {
            Self::Float(f) => Some(*f),
            _ => None,
        }
    }

    /// Try to get this value as a bool.
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Try to get this value as a chosen variant index.
    pub fn as_chosen_variant(&self) -> Option<usize> {
        match self {
            Self::ChosenVariant(idx) => Some(*idx),
            _ => None,
        }
    }

    /// Try to get this value as chosen variant indices.
    pub fn as_chosen_variants(&self) -> Option<&[usize]> {
        match self {
            Self::ChosenVariants(indices) => Some(indices),
            _ => None,
        }
    }

    /// Get the type name of this value for error messages.
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::String(_) => "String",
            Self::Int(_) => "Int",
            Self::Float(_) => "Float",
            Self::Bool(_) => "Bool",
            Self::ChosenVariant(_) => "ChosenVariant",
            Self::ChosenVariants(_) => "ChosenVariants",
        }
    }
}

impl From<String> for ResponseValue {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<&str> for ResponseValue {
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}

impl From<i64> for ResponseValue {
    fn from(i: i64) -> Self {
        Self::Int(i)
    }
}

impl From<i32> for ResponseValue {
    fn from(i: i32) -> Self {
        Self::Int(i64::from(i))
    }
}

impl From<f64> for ResponseValue {
    fn from(f: f64) -> Self {
        Self::Float(f)
    }
}

impl From<bool> for ResponseValue {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

impl From<Vec<usize>> for ResponseValue {
    fn from(indices: Vec<usize>) -> Self {
        Self::ChosenVariants(indices)
    }
}
