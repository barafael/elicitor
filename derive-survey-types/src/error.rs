/// Error type for survey operations.
#[derive(Debug, thiserror::Error)]
pub enum SurveyError {
    /// User cancelled the survey (Ctrl+C, closed window, etc.)
    #[error("Survey cancelled by user")]
    Cancelled,

    /// Backend-specific failure (I/O, UI framework crash, etc.)
    #[error("Backend error: {0}")]
    Backend(#[from] anyhow::Error),
}

impl SurveyError {
    /// Create a backend error from any error type.
    pub fn backend(err: impl Into<anyhow::Error>) -> Self {
        Self::Backend(err.into())
    }

    /// Check if this error represents user cancellation.
    pub fn is_cancelled(&self) -> bool {
        matches!(self, Self::Cancelled)
    }
}
