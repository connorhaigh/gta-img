use core::fmt;
use std::{error::Error, fmt::Display, io};

/// Represents a read-related error.
#[derive(Debug)]
pub enum ReadError {
	/// Indicates that a generic I/O error occurred.
	IoError(io::Error),

	/// Indicates that the header was not in the expected format for the version.
	InvalidHeader,
}

impl Error for ReadError {}

impl Display for ReadError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			Self::IoError(err) => write!(f, "input/output error [{}]", err),
			Self::InvalidHeader => write!(f, "invalid header"),
		}
	}
}

impl From<io::Error> for ReadError {
	fn from(value: io::Error) -> Self {
		Self::IoError(value)
	}
}
