//! Library for reading `IMG` archives used throughout the 3D universa-era of Grand Theft Auto games.

use std::io::{Read, Seek};

use error::ReadError;
use read::{V1Reader, V2Reader};

/// Contains types for errors.
pub mod error;

/// Contains types and the accompanying logic for reading from archives of different versions.
pub mod read;

/// Represents the version of a particular archive.
///
/// The `D` type represents the source of the `dir` archive.
/// The `I` type represents the source of the `img` archive.
pub enum Version<'a, D, I> {
	/// Represents a V1-styled archive, where the directory for indices and the image for data are in separate files.
	V1 {
		dir: &'a mut D,
		img: &'a mut I,
	},

	/// Represents a V2-styled archive, where the indices and data are in the same file.
	V2 {
		img: &'a mut I,
	},
}

/// Attempts to read the archive represented by the specified version.
///
/// If the read is successful, a `Archive<D, I>` is returned which may be inspected for the contents of the archive.
/// If the read is unsuccessful, a `ReadError` is returned.
pub fn read<D, I>(version: Version<D, I>) -> Result<read::Archive<I>, ReadError>
where
	D: Read,
	I: Read + Seek,
{
	match version {
		Version::V1 {
			dir,
			img,
		} => V1Reader::new(dir, img).try_into(),
		Version::V2 {
			img,
		} => V2Reader::new(img).try_into(),
	}
}
