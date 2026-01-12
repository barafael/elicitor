use crate::{DefaultValue, ResponsePath, ResponseValue};

/// A single question in a survey.
#[derive(Debug, Clone, PartialEq)]
pub struct Question {
    /// The path to this question's response in the Responses map.
    path: ResponsePath,

    /// The prompt text shown to the user.
    ask: String,

    /// The kind of question (determines input type and nested structure).
    kind: QuestionKind,

    /// Default value for this question (none, suggested, or assumed).
    default: DefaultValue,
}

impl Question {
    /// Create a new question.
    pub fn new(path: impl Into<ResponsePath>, ask: impl Into<String>, kind: QuestionKind) -> Self {
        Self {
            path: path.into(),
            ask: ask.into(),
            kind,
            default: DefaultValue::None,
        }
    }

    /// Get the response path for this question.
    pub fn path(&self) -> &ResponsePath {
        &self.path
    }

    /// Get the prompt text.
    pub fn ask(&self) -> &str {
        &self.ask
    }

    /// Get the question kind.
    pub fn kind(&self) -> &QuestionKind {
        &self.kind
    }

    /// Get a mutable reference to the question kind.
    pub fn kind_mut(&mut self) -> &mut QuestionKind {
        &mut self.kind
    }

    /// Get the default value.
    pub fn default(&self) -> &DefaultValue {
        &self.default
    }

    /// Set a suggested default value (user can modify).
    pub fn set_suggestion(&mut self, value: impl Into<ResponseValue>) {
        self.default = DefaultValue::Suggested(value.into());
    }

    /// Set an assumed value (question is skipped entirely).
    pub fn set_assumption(&mut self, value: impl Into<ResponseValue>) {
        self.default = DefaultValue::Assumed(value.into());
    }

    /// Clear any default value.
    pub fn clear_default(&mut self) {
        self.default = DefaultValue::None;
    }

    /// Check if this question should be skipped (has an assumed value).
    pub fn is_assumed(&self) -> bool {
        self.default.is_assumed()
    }
}

/// The kind of question, determining input type and structure.
#[derive(Debug, Clone, PartialEq)]
pub enum QuestionKind {
    /// No data to collect (unit enum variants, unit structs).
    Unit,

    /// Single-line text input.
    Input(InputQuestion),

    /// Multi-line text input (opens editor or textarea).
    Multiline(MultilineQuestion),

    /// Masked input for passwords.
    Masked(MaskedQuestion),

    /// Integer input with optional min/max bounds.
    Int(IntQuestion),

    /// Floating-point input with optional min/max bounds.
    Float(FloatQuestion),

    /// Yes/no confirmation.
    Confirm(ConfirmQuestion),

    /// List of values (Vec<T> where T is a primitive type).
    List(ListQuestion),

    /// Select any number of options from a list (Vec<Enum>).
    AnyOf(AnyOfQuestion),

    /// A group of questions — answer all (nested structs, struct variants).
    AllOf(AllOfQuestion),

    /// Choose one variant — pick one, then answer its questions (enums).
    OneOf(OneOfQuestion),
}

impl QuestionKind {
    /// Check if this is a Unit kind (no data to collect).
    pub fn is_unit(&self) -> bool {
        self == &Self::Unit
    }

    pub fn is_basic(&self) -> bool {
        matches!(
            self,
            Self::Input(_)
                | Self::Multiline(_)
                | Self::Masked(_)
                | Self::Int(_)
                | Self::Float(_)
                | Self::Confirm(_)
                | Self::List(_)
        )
    }

    /// Check if this is a structural kind (AllOf, OneOf, AnyOf).
    pub fn is_structural(&self) -> bool {
        matches!(self, Self::AllOf(_) | Self::OneOf(_) | Self::AnyOf(_))
    }
}

/// A variant in a OneOf question (enum variant).
#[derive(Debug, Clone, PartialEq)]
pub struct Variant {
    /// Variant name for display (e.g., "Male", "Female", "Other").
    pub name: String,

    /// What to collect for this variant.
    /// - Unit for unit variants (no data)
    /// - Input for newtype variants with String
    /// - AllOf for struct variants
    /// - OneOf for nested enums
    pub kind: QuestionKind,
}

impl Variant {
    /// Create a new variant with the given name and kind.
    pub fn new(name: impl Into<String>, kind: QuestionKind) -> Self {
        Self {
            name: name.into(),
            kind,
        }
    }

    /// Create a unit variant (no data to collect).
    pub fn unit(name: impl Into<String>) -> Self {
        Self::new(name, QuestionKind::Unit)
    }
}

/// Configuration for an AnyOf question (multi-select with potential follow-up questions).
#[derive(Debug, Clone, PartialEq)]
pub struct AnyOfQuestion {
    /// The available variants to choose from.
    pub variants: Vec<Variant>,

    /// Default selected indices (if any).
    pub defaults: Vec<usize>,
}

impl AnyOfQuestion {
    /// Create a new AnyOf question with the given variants.
    pub fn new(variants: Vec<Variant>) -> Self {
        Self {
            variants,
            defaults: Vec::new(),
        }
    }

    /// Create with default selections.
    pub fn with_defaults(variants: Vec<Variant>, defaults: Vec<usize>) -> Self {
        Self { variants, defaults }
    }
}

/// Configuration for an AllOf question (a group of questions that are all answered).
///
/// Used for nested structs and struct enum variants.
#[derive(Debug, Clone, PartialEq)]
pub struct AllOfQuestion {
    /// The questions in this group.
    pub questions: Vec<Question>,
}

impl AllOfQuestion {
    /// Create a new AllOf question with the given questions.
    pub fn new(questions: Vec<Question>) -> Self {
        Self { questions }
    }

    /// Create an empty AllOf question.
    pub fn empty() -> Self {
        Self {
            questions: Vec::new(),
        }
    }

    /// Get the questions.
    pub fn questions(&self) -> &[Question] {
        &self.questions
    }

    /// Get a mutable reference to the questions.
    pub fn questions_mut(&mut self) -> &mut Vec<Question> {
        &mut self.questions
    }
}

/// Configuration for a OneOf question (choose exactly one variant).
///
/// Used for enums where the user selects one variant, then answers
/// any follow-up questions for that variant.
#[derive(Debug, Clone, PartialEq)]
pub struct OneOfQuestion {
    /// The available variants to choose from.
    pub variants: Vec<Variant>,

    /// Default selected variant index (if any).
    pub default: Option<usize>,
}

impl OneOfQuestion {
    /// Create a new OneOf question with the given variants.
    pub fn new(variants: Vec<Variant>) -> Self {
        Self {
            variants,
            default: None,
        }
    }

    /// Create with a default selection.
    pub fn with_default(variants: Vec<Variant>, default: usize) -> Self {
        Self {
            variants,
            default: Some(default),
        }
    }

    /// Get the variants.
    pub fn variants(&self) -> &[Variant] {
        &self.variants
    }

    /// Get a mutable reference to the variants.
    pub fn variants_mut(&mut self) -> &mut Vec<Variant> {
        &mut self.variants
    }
}

/// Configuration for a text input question.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct InputQuestion {
    /// Optional default value.
    pub default: Option<String>,

    /// Validation function name (resolved at compile time).
    pub validate: Option<String>,
}

impl InputQuestion {
    /// Create a new input question.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with a default value.
    pub fn with_default(default: impl Into<String>) -> Self {
        Self {
            default: Some(default.into()),
            validate: None,
        }
    }

    /// Create with a validator.
    pub fn with_validator(validate: impl Into<String>) -> Self {
        Self {
            default: None,
            validate: Some(validate.into()),
        }
    }
}

/// Configuration for a multi-line text editor question.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MultilineQuestion {
    /// Optional default value.
    pub default: Option<String>,

    /// Validation function name.
    pub validate: Option<String>,
}

impl MultilineQuestion {
    /// Create a new multiline question.
    pub fn new() -> Self {
        Self::default()
    }
}

/// Configuration for a password/masked input question.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MaskedQuestion {
    /// The masking character (default: '*').
    pub mask: Option<char>,

    /// Validation function name.
    pub validate: Option<String>,
}

impl MaskedQuestion {
    /// Create a new masked question.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with a custom mask character.
    pub fn with_mask(mask: char) -> Self {
        Self {
            mask: Some(mask),
            validate: None,
        }
    }
}

/// Configuration for an integer input question.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct IntQuestion {
    /// Optional default value.
    pub default: Option<i64>,

    /// Optional minimum value.
    pub min: Option<i64>,

    /// Optional maximum value.
    pub max: Option<i64>,

    /// Validation function name.
    pub validate: Option<String>,
}

impl IntQuestion {
    /// Create a new integer question.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with bounds.
    pub fn with_bounds(min: Option<i64>, max: Option<i64>) -> Self {
        Self {
            default: None,
            min,
            max,
            validate: None,
        }
    }

    /// Create with bounds and a validator.
    pub fn with_bounds_and_validator(
        min: Option<i64>,
        max: Option<i64>,
        validate: Option<String>,
    ) -> Self {
        Self {
            default: None,
            min,
            max,
            validate,
        }
    }
}

/// Configuration for a floating-point input question.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FloatQuestion {
    /// Optional default value.
    pub default: Option<f64>,

    /// Optional minimum value.
    pub min: Option<f64>,

    /// Optional maximum value.
    pub max: Option<f64>,

    /// Validation function name.
    pub validate: Option<String>,
}

impl FloatQuestion {
    /// Create a new float question.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with bounds.
    pub fn with_bounds(min: Option<f64>, max: Option<f64>) -> Self {
        Self {
            default: None,
            min,
            max,
            validate: None,
        }
    }

    /// Create with bounds and a validator.
    pub fn with_bounds_and_validator(
        min: Option<f64>,
        max: Option<f64>,
        validate: Option<String>,
    ) -> Self {
        Self {
            default: None,
            min,
            max,
            validate,
        }
    }
}

/// Configuration for a yes/no confirmation question.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ConfirmQuestion {
    /// Default value (true for yes, false for no).
    pub default: bool,
}

impl ConfirmQuestion {
    /// Create a new confirm question with default false.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with a default value.
    pub fn with_default(default: bool) -> Self {
        Self { default }
    }
}

/// The type of elements in a list question.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum ListElementKind {
    /// String elements.
    #[default]
    String,
    /// Integer elements with optional bounds.
    Int { min: Option<i64>, max: Option<i64> },
    /// Float elements with optional bounds.
    Float { min: Option<f64>, max: Option<f64> },
}

/// Configuration for a list input question (Vec<T>).
///
/// Allows collecting multiple values of the same type.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ListQuestion {
    /// The type of elements in the list.
    pub element_kind: ListElementKind,

    /// Optional minimum number of elements.
    pub min_items: Option<usize>,

    /// Optional maximum number of elements.
    pub max_items: Option<usize>,

    /// Validation function name.
    pub validate: Option<String>,
}

impl ListQuestion {
    /// Create a new list question for strings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a list question for strings.
    pub fn strings() -> Self {
        Self {
            element_kind: ListElementKind::String,
            ..Default::default()
        }
    }

    /// Create a list question for integers.
    pub fn ints() -> Self {
        Self {
            element_kind: ListElementKind::Int {
                min: None,
                max: None,
            },
            ..Default::default()
        }
    }

    /// Create a list question for integers with bounds.
    pub fn ints_with_bounds(min: Option<i64>, max: Option<i64>) -> Self {
        Self {
            element_kind: ListElementKind::Int { min, max },
            ..Default::default()
        }
    }

    /// Create a list question for floats.
    pub fn floats() -> Self {
        Self {
            element_kind: ListElementKind::Float {
                min: None,
                max: None,
            },
            ..Default::default()
        }
    }

    /// Create a list question for floats with bounds.
    pub fn floats_with_bounds(min: Option<f64>, max: Option<f64>) -> Self {
        Self {
            element_kind: ListElementKind::Float { min, max },
            ..Default::default()
        }
    }

    /// Set item count constraints.
    pub fn with_item_bounds(mut self, min: Option<usize>, max: Option<usize>) -> Self {
        self.min_items = min;
        self.max_items = max;
        self
    }

    /// Set a validator function.
    pub fn with_validator(mut self, validate: impl Into<String>) -> Self {
        self.validate = Some(validate.into());
        self
    }
}

/// The key suffix used to store the selected enum variant index in responses.
/// For a field "method", the selection is stored at "method.selected_variant".
pub const SELECTED_VARIANT_KEY: &str = "selected_variant";

/// The key suffix used to store selected variant indices for AnyOf questions.
/// For a field "features", the selections are stored at "features.selected_variants".
pub const SELECTED_VARIANTS_KEY: &str = "selected_variants";
