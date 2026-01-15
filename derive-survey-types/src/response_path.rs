use std::fmt;

/// A path to a response value, e.g., `"address.street"`.
///
/// Used as keys in `Responses` to identify specific fields, including nested ones.
/// Paths are hierarchical, using dot notation for nested fields.
///
/// This is an internal type. Users interact with surveys through the
/// generated builder methods like `suggest_name()` or `assume_address_street()`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ResponsePath {
    /// Dot-separated path string, e.g., "address.street"
    path: String,
}

impl ResponsePath {
    /// Create a new path from a dot-separated string.
    pub fn new(path: impl Into<String>) -> Self {
        Self { path: path.into() }
    }

    /// Create an empty path (used for top-level enums).
    pub fn empty() -> Self {
        Self {
            path: String::new(),
        }
    }

    /// Append a child segment to this path, returning a new path.
    pub fn child(&self, name: &str) -> Self {
        if name.is_empty() {
            self.clone()
        } else if self.path.is_empty() {
            Self::new(name)
        } else {
            Self::new(format!("{}.{}", self.path, name))
        }
    }

    /// Get the path as a string slice.
    pub fn as_str(&self) -> &str {
        &self.path
    }

    /// Check if the path is empty.
    pub fn is_empty(&self) -> bool {
        self.path.is_empty()
    }

    /// Get the segments of this path as an iterator.
    pub fn segments(&self) -> impl Iterator<Item = &str> {
        self.path.split('.').filter(|s| !s.is_empty())
    }

    /// Get the number of segments in this path.
    pub fn len(&self) -> usize {
        if self.path.is_empty() {
            0
        } else {
            self.path.split('.').count()
        }
    }

    /// Returns a new path with the given prefix segment removed, if it matches.
    pub fn strip_prefix(&self, prefix: &str) -> Option<Self> {
        if self.path == prefix {
            Some(Self::empty())
        } else if self.path.starts_with(prefix) && self.path[prefix.len()..].starts_with('.') {
            Some(Self::new(&self.path[prefix.len() + 1..]))
        } else {
            None
        }
    }

    /// Strip a ResponsePath prefix from this path.
    pub fn strip_path_prefix(&self, prefix: &ResponsePath) -> Option<Self> {
        self.strip_prefix(prefix.as_str())
    }

    /// Get the first segment, if any.
    pub fn first(&self) -> Option<&str> {
        self.segments().next()
    }

    /// Get the last segment, if any.
    pub fn last(&self) -> Option<&str> {
        self.path.rsplit('.').next().filter(|s| !s.is_empty())
    }

    /// Get the parent path by removing the last segment.
    /// Returns an empty path if this path has only one segment.
    pub fn parent(&self) -> Self {
        if let Some(last_dot) = self.path.rfind('.') {
            Self::new(&self.path[..last_dot])
        } else {
            Self::empty()
        }
    }
}

impl fmt::Display for ResponsePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path)
    }
}

impl From<&str> for ResponsePath {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for ResponsePath {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&String> for ResponsePath {
    fn from(s: &String) -> Self {
        Self::new(s.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new() {
        let path = ResponsePath::new("name");
        assert_eq!(path.as_str(), "name");
    }

    #[test]
    fn child() {
        let path = ResponsePath::new("address").child("street");
        assert_eq!(path.as_str(), "address.street");
    }

    #[test]
    fn child_from_empty() {
        let path = ResponsePath::empty().child("name");
        assert_eq!(path.as_str(), "name");
    }

    #[test]
    fn strip_prefix() {
        let path = ResponsePath::new("address.street");
        let stripped = path.strip_prefix("address").unwrap();
        assert_eq!(stripped.as_str(), "street");

        assert!(path.strip_prefix("other").is_none());
    }

    #[test]
    fn strip_prefix_exact_match() {
        let path = ResponsePath::new("name");
        let stripped = path.strip_prefix("name").unwrap();
        assert!(stripped.is_empty());
    }

    #[test]
    fn segments() {
        let path = ResponsePath::new("address.location.city");
        let segments: Vec<_> = path.segments().collect();
        assert_eq!(segments, vec!["address", "location", "city"]);
    }

    #[test]
    fn display() {
        let path = ResponsePath::new("address.street");
        assert_eq!(format!("{}", path), "address.street");
    }

    #[test]
    fn from_str() {
        let path: ResponsePath = "address.street".into();
        assert_eq!(path.as_str(), "address.street");
    }
}
