use core::fmt;

/// Schema CLI error.
#[derive(Debug)]
pub enum SchemaError {
    /// Unknown or invalid CLI argument.
    InvalidArgument(String),
    /// File IO failed.
    Io(std::io::Error),
    /// JSON serialization failed.
    Json(serde_json::Error),
    /// Existing catalog does not match generated output.
    CatalogMismatch(String),
}

impl fmt::Display for SchemaError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidArgument(argument) => write!(formatter, "invalid argument: {argument}"),
            Self::Io(error) => write!(formatter, "io error: {error}"),
            Self::Json(error) => write!(formatter, "json error: {error}"),
            Self::CatalogMismatch(path) => {
                write!(formatter, "schema catalog is stale: {path}")
            }
        }
    }
}

impl std::error::Error for SchemaError {}

impl From<std::io::Error> for SchemaError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for SchemaError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}
