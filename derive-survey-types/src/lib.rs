//! Core types for the derive-survey crate.
//!
//! This crate provides the foundational types for defining surveys:
//! - `SurveyDefinition` - The top-level survey structure
//! - `Question` and `QuestionKind` - Individual questions and their types
//! - `Responses` and `ResponsePath` - Collected data and path-based keys
//! - `Survey` and `SurveyBackend` traits - For implementing surveys and backends

mod response_path;
pub use response_path::ResponsePath;

mod response_value;
pub use response_value::ResponseValue;

mod responses;
pub use responses::{ResponseError, Responses};

mod default_value;
pub use default_value::DefaultValue;

mod question;
pub use question::{
    AllOfQuestion, AnyOfQuestion, ConfirmQuestion, FloatQuestion, InputQuestion, IntQuestion,
    MaskedQuestion, MultilineQuestion, OneOfQuestion, Question, QuestionKind, SELECTED_VARIANT_KEY,
    SELECTED_VARIANTS_KEY, Variant,
};

mod survey_definition;
pub use survey_definition::SurveyDefinition;

mod error;
pub use error::SurveyError;

mod traits;
pub use traits::{Survey, SurveyBackend};
