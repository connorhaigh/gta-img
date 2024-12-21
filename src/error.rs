use core::fmt;
use std::{error::Error, io};

/// Represents a read-related error.
#[derive(Debug)]
pub enum ReadError {
	/// Indicates that a generic I/O error occurred.
	IoError(io::Error),

	/// Indicates that the header was not in the expected format for the version.
	InvalidHeader,
}

/// Represents a write-related error.
#[derive(Debug)]
pub enum WriteError {
	/// Indicates that a generic I/O error occurred.
	IoError(io::Error),

	/// Indicates that there is insufficient size in the header to add further entries.
	InsufficientHeaderSize,

	/// Indicates that the provided name of an entry is longer than 23 characters.
	InvalidNameLength
}

impl Error for ReadError {}
impl Error for WriteError {}

impl fmt::Display for ReadError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::IoError(err) => write!(f, "input/output error [{}]", err),
			Self::InvalidHeader => write!(f, "invalid header"),
		}
	}
}

impl fmt::Display for WriteError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match self {
			Self::IoError(err) => write!(f, "input/output error [{}]", err),
			Self::InsufficientHeaderSize => write!(f, "insufficient header size"),
			Self::InvalidNameLength => write!(f, "invalid name length")
		}
	}
}

impl From<io::Error> for ReadError {
	fn from(value: io::Error) -> Self {
		Self::IoError(value)
	}
}

impl From<io::Error> for WriteError {
	fn from(value: io::Error) -> Self {
		Self::IoError(value)
	}
}
