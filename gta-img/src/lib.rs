use std::io::{Read, Seek};

use error::ReadError;
use read::{V1Read, V2Read};

pub mod error;
pub mod read;

/// Attempts to read a V1-style archive from the specified `dir` source and specified `img` source.
///
/// If successful, returns a `V1Read<D, I>` which may be enumerated to retrieve each `Entry<I>` within the archive.
/// If unsuccessful, returns a `ReadError`.
pub fn read_v1<D, I>(dir: D, img: I) -> Result<V1Read<D, I>, ReadError>
where
	D: Read,
	I: Read + Seek,
{
	V1Read::new(dir, img)
}

/// Attempts to read a V1-style archive from the specified `img` source.
///
/// If successful, returns a `V2Read<I>` which may be enumerated to retrieve each `Entry<I>` within the archive.
/// If unsuccessful, returns a `ReadError`.
pub fn read_v2<I>(img: I) -> Result<V2Read<I>, ReadError>
where
	I: Read + Seek,
{
	V2Read::new(img)
}
