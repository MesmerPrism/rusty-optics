use core::fmt;

/// Optics validation or fixture failure.
#[derive(Clone, Debug, PartialEq)]
pub enum OpticsError {
    /// Schema identifier did not match the expected contract.
    UnexpectedSchema {
        /// Expected schema identifier.
        expected: &'static str,
        /// Actual schema identifier.
        actual: String,
    },
    /// Required identifier was empty.
    EmptyId(&'static str),
    /// Color channels were non-finite.
    NonFiniteColor(&'static str),
    /// Two-dimensional point was non-finite.
    NonFiniteVec2(&'static str),
    /// Three-dimensional vector was non-finite.
    NonFiniteVec3(&'static str),
    /// A range or scalar value was invalid.
    InvalidValue(&'static str),
    /// A count was invalid.
    InvalidCount(&'static str),
    /// A payload shape was invalid.
    InvalidPayload(&'static str),
}

impl fmt::Display for OpticsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedSchema { expected, actual } => {
                write!(formatter, "expected schema {expected}, got {actual}")
            }
            Self::EmptyId(field) => write!(formatter, "{field} must not be empty"),
            Self::NonFiniteColor(field) => write!(formatter, "{field} color must be finite"),
            Self::NonFiniteVec2(field) => write!(formatter, "{field} vec2 must be finite"),
            Self::NonFiniteVec3(field) => write!(formatter, "{field} vec3 must be finite"),
            Self::InvalidValue(field) => write!(formatter, "{field} value is invalid"),
            Self::InvalidCount(field) => write!(formatter, "{field} count is invalid"),
            Self::InvalidPayload(reason) => formatter.write_str(reason),
        }
    }
}

impl std::error::Error for OpticsError {}
