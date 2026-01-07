use crate::Question;

/// The top-level structure containing all questions and metadata for a survey.
///
/// A survey is a structured collection of questions. It's presentation-agnostic —
/// it can be rendered as a sequential interview, a fill-in form, or used to
/// generate documents.
#[derive(Debug, Clone)]
pub struct SurveyDefinition {
    /// Optional message shown before the survey starts.
    pub prelude: Option<String>,

    /// All questions in the survey (may contain nested AllOf/OneOf/AnyOf).
    pub questions: Vec<Question>,

    /// Optional message shown after the survey completes.
    pub epilogue: Option<String>,
}

impl SurveyDefinition {
    /// Create a new survey definition with the given questions.
    pub fn new(questions: Vec<Question>) -> Self {
        Self {
            prelude: None,
            questions,
            epilogue: None,
        }
    }

    /// Create an empty survey definition.
    pub fn empty() -> Self {
        Self {
            prelude: None,
            questions: Vec::new(),
            epilogue: None,
        }
    }

    /// Set the prelude message.
    pub fn with_prelude(mut self, prelude: impl Into<String>) -> Self {
        self.prelude = Some(prelude.into());
        self
    }

    /// Set the epilogue message.
    pub fn with_epilogue(mut self, epilogue: impl Into<String>) -> Self {
        self.epilogue = Some(epilogue.into());
        self
    }

    /// Get the questions.
    pub fn questions(&self) -> &[Question] {
        &self.questions
    }

    /// Get a mutable reference to the questions.
    pub fn questions_mut(&mut self) -> &mut Vec<Question> {
        &mut self.questions
    }

    /// Check if the survey has any questions.
    pub fn is_empty(&self) -> bool {
        self.questions.is_empty()
    }

    /// Get the number of top-level questions.
    pub fn len(&self) -> usize {
        self.questions.len()
    }
}

impl Default for SurveyDefinition {
    fn default() -> Self {
        Self::empty()
    }
}
