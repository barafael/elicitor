/// A field path for accessing nested wizard fields.
///
/// This can be created from a simple string (for flat fields) or from
/// a path array (for nested fields). Paths use dot (`.`) as the separator.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FieldPath {
    segments: Vec<String>,
}

impl FieldPath {
    /// Create a new field path from segments.
    pub const fn new(segments: Vec<String>) -> Self {
        Self { segments }
    }

    /// Create a field path from a dot-separated string.
    pub fn from_path(path: &str) -> Self {
        Self {
            segments: path.split('.').map(|s| s.to_string()).collect(),
        }
    }

    /// Get the segments of this path.
    pub fn segments(&self) -> &[String] {
        &self.segments
    }

    /// Convert this path to a dot-separated string.
    pub fn to_path(&self) -> String {
        self.segments.join(".")
    }

    /// Get the depth of this path (number of segments).
    pub const fn depth(&self) -> usize {
        self.segments.len()
    }
}

impl From<&str> for FieldPath {
    fn from(s: &str) -> Self {
        // Check if it contains a dot - if so, treat as path
        if s.contains('.') {
            Self::from_path(s)
        } else {
            // Single segment
            Self::new(vec![s.to_string()])
        }
    }
}

impl From<String> for FieldPath {
    fn from(s: String) -> Self {
        Self::from(s.as_str())
    }
}

impl From<Vec<String>> for FieldPath {
    fn from(segments: Vec<String>) -> Self {
        Self::new(segments)
    }
}

impl From<&[&str]> for FieldPath {
    fn from(segments: &[&str]) -> Self {
        Self::new(segments.iter().map(|s| s.to_string()).collect())
    }
}

/// Macro for creating type-safe field paths.
///
/// # Examples
///
/// ```rust
/// use derive_wizard::field;
/// // Single field (top-level)
/// field!(name); // expands to FieldPath from "name"
///
/// // Nested field
/// field!(Person::contact::email); // expands to FieldPath from "contact.email"
///
/// // Multiple levels
/// field!(Company::address::location::city); // expands to FieldPath from "address.location.city"
/// ```
#[macro_export]
macro_rules! field {
    // Single identifier - top level field
    ($field:ident) => {
        $crate::field_path::FieldPath::from(stringify!($field))
    };

    // Type::field - one level nesting
    ($ty:ident :: $field:ident) => {
        $crate::field_path::FieldPath::from(stringify!($field))
    };

    // Type::nested::field - two level nesting (dot-separated for namespace prefixes)
    ($ty:ident :: $nested:ident :: $field:ident) => {
        $crate::field_path::FieldPath::from(concat!(stringify!($nested), ".", stringify!($field)))
    };

    // Type::nested1::nested2::field - three level nesting (dot-separated)
    ($ty:ident :: $nested1:ident :: $nested2:ident :: $field:ident) => {
        $crate::field_path::FieldPath::from(concat!(
            stringify!($nested1),
            ".",
            stringify!($nested2),
            ".",
            stringify!($field)
        ))
    };

    // Type::nested1::nested2::nested3::field - four level nesting (dot-separated)
    ($ty:ident :: $nested1:ident :: $nested2:ident :: $nested3:ident :: $field:ident) => {
        $crate::field_path::FieldPath::from(concat!(
            stringify!($nested1),
            ".",
            stringify!($nested2),
            ".",
            stringify!($nested3),
            ".",
            stringify!($field)
        ))
    };
}
