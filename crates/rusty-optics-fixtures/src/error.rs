use core::fmt;

/// Fixture CLI error.
#[derive(Debug)]
pub enum FixtureError {
    /// Unknown or invalid CLI argument.
    InvalidArgument(String),
    /// File IO failed.
    Io(std::io::Error),
    /// JSON serialization failed.
    Json(serde_json::Error),
    /// Matter payload generation failed.
    Matter(String),
    /// Optics validation failed.
    Optics(String),
    /// Existing fixture does not match generated output.
    FixtureMismatch(String),
}

impl fmt::Display for FixtureError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidArgument(argument) => write!(formatter, "invalid argument: {argument}"),
            Self::Io(error) => write!(formatter, "io error: {error}"),
            Self::Json(error) => write!(formatter, "json error: {error}"),
            Self::Matter(error) => write!(formatter, "matter fixture error: {error}"),
            Self::Optics(error) => write!(formatter, "optics fixture error: {error}"),
            Self::FixtureMismatch(path) => write!(formatter, "fixture is stale: {path}"),
        }
    }
}

impl std::error::Error for FixtureError {}

impl From<std::io::Error> for FixtureError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for FixtureError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}
