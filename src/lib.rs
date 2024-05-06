//! Library for reading from `IMG` archives (and supplementary `DIR` files) used throughout the 3D universe-era of Grand Theft Auto games.

use std::io::{Read, Seek};

use error::ReadError;
use read::Archive;

/// Contains types for errors.
pub mod error;

/// Contains types and the accompanying logic for reading from archives of different versions.
pub mod read;

/// Attempts to read the archive using the specified version-specific reader, by way of calling `TryInto`.
///
/// If the read is successful, a `Archive<V>` is returned which may be inspected for the contents of the archive.
/// If the read is unsuccessful, a `ReadError` is returned.
pub fn read<'a, R, I>(reader: R) -> Result<Archive<'a, I>, ReadError>
where
	R: TryInto<Archive<'a, I>, Error = ReadError>,
	I: Read + Seek,
{
	reader.try_into()
}
