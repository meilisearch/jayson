use core::fmt::{self, Display};

use crate::de::VisitorError;

/// Error type when deserialization fails.
///
/// jayson errors contain no information about what went wrong. **If you need
/// more than no information, use Serde.**
#[derive(Copy, Clone, Debug)]
pub struct Error;

impl VisitorError for Error {
    fn unexpected(_: &str) -> Self {
        Self
    }

    fn format_error(_line: usize, _pos: usize, _msg: &str) -> Self {
        Self
    }

    fn missing_field(_field: &str) -> Self {
        Self
    }
}

/// Result type returned by deserialization functions.
pub type Result<T> = core::result::Result<T, Error>;

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("jayson error")
    }
}

impl std::error::Error for Error {}
