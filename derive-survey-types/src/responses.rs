use std::collections::HashMap;

use crate::{ResponsePath, ResponseValue};

/// Error type for response access operations.
#[derive(Debug, thiserror::Error)]
pub enum ResponseError {
    #[error("Missing response for path: {0}")]
    MissingPath(ResponsePath),

    #[error("Type mismatch at path '{path}': expected {expected}, got {actual}")]
    TypeMismatch {
        path: ResponsePath,
        expected: &'static str,
        actual: &'static str,
    },
}

/// Collected responses from a survey.
///
/// Uses `ResponsePath` as keys to support hierarchical field access.
/// Response paths are flat (not nested) - a nested field like `address.street`
/// is stored with the key `ResponsePath::from("address.street")`.
#[derive(Debug, Clone, Default)]
pub struct Responses {
    values: HashMap<ResponsePath, ResponseValue>,
}

impl Responses {
    /// Create a new empty responses collection.
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    /// Insert a response value at the given path.
    pub fn insert(&mut self, path: impl Into<ResponsePath>, value: impl Into<ResponseValue>) {
        self.values.insert(path.into(), value.into());
    }

    /// Get a response value at the given path.
    pub fn get(&self, path: &ResponsePath) -> Option<&ResponseValue> {
        self.values.get(path)
    }

    /// Check if a response exists at the given path.
    pub fn contains(&self, path: &ResponsePath) -> bool {
        self.values.contains_key(path)
    }

    /// Remove a response at the given path.
    pub fn remove(&mut self, path: &ResponsePath) -> Option<ResponseValue> {
        self.values.remove(path)
    }

    /// Get an iterator over all path-value pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&ResponsePath, &ResponseValue)> {
        self.values.iter()
    }

    /// Get the number of responses.
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Check if there are no responses.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Merge another responses collection into this one.
    pub fn extend(&mut self, other: Responses) {
        self.values.extend(other.values);
    }

    /// Filter responses to only those with the given path prefix, removing the prefix from keys.
    ///
    /// This is used when reconstructing nested types - extract responses for a nested
    /// struct and strip the prefix so the nested `from_responses` sees root-level paths.
    ///
    /// # Example
    /// ```
    /// use derive_survey_types::{Responses, ResponsePath, ResponseValue};
    ///
    /// let mut responses = Responses::new();
    /// responses.insert("address.street", "123 Main St");
    /// responses.insert("address.city", "Springfield");
    /// responses.insert("name", "Alice");
    ///
    /// let address_responses = responses.filter_prefix(&ResponsePath::new("address"));
    /// assert!(address_responses.get(&ResponsePath::new("street")).is_some());
    /// assert!(address_responses.get(&ResponsePath::new("city")).is_some());
    /// assert!(address_responses.get(&ResponsePath::new("name")).is_none());
    /// ```
    pub fn filter_prefix(&self, prefix: &ResponsePath) -> Self {
        let mut filtered = Responses::new();
        for (path, value) in &self.values {
            if let Some(stripped) = path.strip_path_prefix(prefix) {
                filtered.values.insert(stripped, value.clone());
            }
        }
        filtered
    }

    // === Convenience accessors ===

    /// Get a string value at the given path.
    pub fn get_string(&self, path: &ResponsePath) -> Result<&str, ResponseError> {
        match self.get(path) {
            Some(ResponseValue::String(s)) => Ok(s),
            Some(other) => Err(ResponseError::TypeMismatch {
                path: path.clone(),
                expected: "String",
                actual: other.type_name(),
            }),
            None => Err(ResponseError::MissingPath(path.clone())),
        }
    }

    /// Get an integer value at the given path.
    pub fn get_int(&self, path: &ResponsePath) -> Result<i64, ResponseError> {
        match self.get(path) {
            Some(ResponseValue::Int(i)) => Ok(*i),
            Some(other) => Err(ResponseError::TypeMismatch {
                path: path.clone(),
                expected: "Int",
                actual: other.type_name(),
            }),
            None => Err(ResponseError::MissingPath(path.clone())),
        }
    }

    /// Get a float value at the given path.
    pub fn get_float(&self, path: &ResponsePath) -> Result<f64, ResponseError> {
        match self.get(path) {
            Some(ResponseValue::Float(f)) => Ok(*f),
            Some(other) => Err(ResponseError::TypeMismatch {
                path: path.clone(),
                expected: "Float",
                actual: other.type_name(),
            }),
            None => Err(ResponseError::MissingPath(path.clone())),
        }
    }

    /// Get a boolean value at the given path.
    pub fn get_bool(&self, path: &ResponsePath) -> Result<bool, ResponseError> {
        match self.get(path) {
            Some(ResponseValue::Bool(b)) => Ok(*b),
            Some(other) => Err(ResponseError::TypeMismatch {
                path: path.clone(),
                expected: "Bool",
                actual: other.type_name(),
            }),
            None => Err(ResponseError::MissingPath(path.clone())),
        }
    }

    /// Get a chosen variant index at the given path.
    pub fn get_chosen_variant(&self, path: &ResponsePath) -> Result<usize, ResponseError> {
        match self.get(path) {
            Some(ResponseValue::ChosenVariant(idx)) => Ok(*idx),
            Some(other) => Err(ResponseError::TypeMismatch {
                path: path.clone(),
                expected: "ChosenVariant",
                actual: other.type_name(),
            }),
            None => Err(ResponseError::MissingPath(path.clone())),
        }
    }

    /// Get chosen variant indices at the given path.
    pub fn get_chosen_variants(&self, path: &ResponsePath) -> Result<&[usize], ResponseError> {
        match self.get(path) {
            Some(ResponseValue::ChosenVariants(indices)) => Ok(indices),
            Some(other) => Err(ResponseError::TypeMismatch {
                path: path.clone(),
                expected: "ChosenVariants",
                actual: other.type_name(),
            }),
            None => Err(ResponseError::MissingPath(path.clone())),
        }
    }

    /// Get a string list at the given path.
    pub fn get_string_list(&self, path: &ResponsePath) -> Result<&[String], ResponseError> {
        match self.get(path) {
            Some(ResponseValue::StringList(list)) => Ok(list),
            Some(other) => Err(ResponseError::TypeMismatch {
                path: path.clone(),
                expected: "StringList",
                actual: other.type_name(),
            }),
            None => Err(ResponseError::MissingPath(path.clone())),
        }
    }

    /// Get an integer list at the given path.
    pub fn get_int_list(&self, path: &ResponsePath) -> Result<&[i64], ResponseError> {
        match self.get(path) {
            Some(ResponseValue::IntList(list)) => Ok(list),
            Some(other) => Err(ResponseError::TypeMismatch {
                path: path.clone(),
                expected: "IntList",
                actual: other.type_name(),
            }),
            None => Err(ResponseError::MissingPath(path.clone())),
        }
    }

    /// Get a float list at the given path.
    pub fn get_float_list(&self, path: &ResponsePath) -> Result<&[f64], ResponseError> {
        match self.get(path) {
            Some(ResponseValue::FloatList(list)) => Ok(list),
            Some(other) => Err(ResponseError::TypeMismatch {
                path: path.clone(),
                expected: "FloatList",
                actual: other.type_name(),
            }),
            None => Err(ResponseError::MissingPath(path.clone())),
        }
    }

    /// Check if a response at the given path has a non-empty value.
    ///
    /// This is used for `Option<T>` fields: returns `false` if the response
    /// is missing OR if it's an empty string (user skipped the optional field).
    pub fn has_value(&self, path: &ResponsePath) -> bool {
        match self.get(path) {
            Some(ResponseValue::String(s)) => !s.is_empty(),
            Some(_) => true,
            None => false,
        }
    }
}

impl IntoIterator for Responses {
    type Item = (ResponsePath, ResponseValue);
    type IntoIter = std::collections::hash_map::IntoIter<ResponsePath, ResponseValue>;

    fn into_iter(self) -> Self::IntoIter {
        self.values.into_iter()
    }
}

impl<'a> IntoIterator for &'a Responses {
    type Item = (&'a ResponsePath, &'a ResponseValue);
    type IntoIter = std::collections::hash_map::Iter<'a, ResponsePath, ResponseValue>;

    fn into_iter(self) -> Self::IntoIter {
        self.values.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_get() {
        let mut responses = Responses::new();
        responses.insert("name", "Alice");
        responses.insert("age", ResponseValue::Int(30));

        assert_eq!(
            responses.get_string(&ResponsePath::new("name")).unwrap(),
            "Alice"
        );
        assert_eq!(responses.get_int(&ResponsePath::new("age")).unwrap(), 30);
    }

    #[test]
    fn filter_prefix() {
        let mut responses = Responses::new();
        responses.insert("address.street", "123 Main St");
        responses.insert("address.city", "Springfield");
        responses.insert("name", "Alice");

        let filtered = responses.filter_prefix(&ResponsePath::new("address"));
        assert_eq!(filtered.len(), 2);
        assert_eq!(
            filtered.get_string(&ResponsePath::new("street")).unwrap(),
            "123 Main St"
        );
        assert_eq!(
            filtered.get_string(&ResponsePath::new("city")).unwrap(),
            "Springfield"
        );
    }

    #[test]
    fn type_mismatch_error() {
        let mut responses = Responses::new();
        responses.insert("age", ResponseValue::Int(30));

        let result = responses.get_string(&ResponsePath::new("age"));
        assert!(matches!(result, Err(ResponseError::TypeMismatch { .. })));
    }
}
