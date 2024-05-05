use std::io::{self, Read, Seek};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::error::ReadError;

/// Represents the structure for a V2-style header.
pub const VERSION_2_HEADER: [u8; 4] = [0x56, 0x45, 0x52, 0x32]; // VER2

/// Represents the length of the structure for a V2-style header; always `4`.
pub const VERSION_2_HEADER_SIZE: usize = 4;

/// Represents the number of bytes used for sector alignment.
const SECTOR_SIZE: u64 = 2048;

/// Represents the maximum length of the name of an entry.
const NAME_SIZE: usize = 24;

/// Represents an archive.
pub struct Archive<'a, R> {
	inner: &'a mut R,

	entries: Vec<Entry>,
}

/// Represents an entry opened for reading.
pub struct EntryRead<'a, R> {
	inner: &'a mut R,

	off: u64,
	len: u64,
}

/// Represents an entry.
pub struct Entry {
	/// The name of the entry, up to 24 characters.
	pub name: String,

	/// The offset, in sectors, of the entry.
	pub off: u64,

	/// The length, in sectors, of the entry.
	pub len: u64,
}

/// Represents a reader of V1-styled archives.
pub struct V1Reader<'a, D, I> {
	dir: &'a mut D,
	img: &'a mut I,
}

/// Represents a reader of V2-styled archives.
pub struct V2Reader<'a, I> {
	img: &'a mut I,
}

impl<'a, D, I> V1Reader<'a, D, I> {
	/// Creates a new V1-styled reader with the specified `dir` source and specified `img` source.
	pub fn new(dir: &'a mut D, img: &'a mut I) -> Self {
		Self {
			dir,
			img,
		}
	}
}

impl<'a, I> V2Reader<'a, I> {
	/// Creates a new V2-styled reader with the specified `img` source.
	pub fn new(img: &'a mut I) -> Self {
		Self {
			img,
		}
	}
}

impl<'a, D, I> TryInto<Archive<'a, I>> for V1Reader<'a, D, I>
where
	D: Read,
	I: Read + Seek,
{
	type Error = ReadError;

	fn try_into(self) -> Result<Archive<'a, I>, Self::Error> {
		let mut entries: Vec<Entry> = Vec::new();

		loop {
			let off = match self.dir.read_u32::<LittleEndian>() {
				Ok(off) => off as u64,
				Err(err) if err.kind() == io::ErrorKind::UnexpectedEof => break,
				Err(err) => return Err(err.into()),
			};

			let len = self.dir.read_u32::<LittleEndian>()? as u64;
			let name = {
				let mut buf = [0; NAME_SIZE];

				self.dir.read_exact(&mut buf)?;

				to_name(buf)
			};

			entries.push(Entry {
				name,
				off,
				len,
			})
		}

		Ok(Archive {
			inner: self.img,
			entries,
		})
	}
}

impl<'a, I> TryInto<Archive<'a, I>> for V2Reader<'a, I>
where
	I: Read + Seek,
{
	type Error = ReadError;

	fn try_into(self) -> Result<Archive<'a, I>, Self::Error> {
		let header = {
			let mut buffer = [0; VERSION_2_HEADER_SIZE];

			self.img.read_exact(&mut buffer)?;

			buffer
		};

		if header != VERSION_2_HEADER {
			return Err(ReadError::InvalidHeader);
		}

		let count = self.img.read_u32::<LittleEndian>()? as usize;
		let mut entries: Vec<Entry> = Vec::with_capacity(count);

		for _ in 0..count {
			let off = self.img.read_u32::<LittleEndian>()? as u64;
			let len = self.img.read_u16::<LittleEndian>()? as u64;

			let _ = self.img.read_u16::<LittleEndian>()?;

			let name = {
				let mut buf = [0; NAME_SIZE];

				self.img.read_exact(&mut buf)?;

				to_name(buf)
			};

			entries.push(Entry {
				name,
				off,
				len,
			})
		}

		Ok(Archive {
			inner: self.img,
			entries,
		})
	}
}

impl<'a, I> Archive<'a, I> {
	/// Returns the number of entries in the archive.
	pub fn len(&self) -> usize {
		self.entries.len()
	}

	/// Returns if the archive is void of any entries.
	pub fn is_empty(&self) -> bool {
		self.entries.is_empty()
	}

	/// Returns the entry at the specified index, if it exists.
	pub fn entry_at(&self, index: usize) -> Option<&Entry> {
		self.entries.get(index)
	}

	/// Opens and returns the entry at the specified index for reading, if it exists.
	pub fn read_at(&mut self, index: usize) -> Option<EntryRead<I>> {
		let entry = self.entries.get(index)?;

		Some(EntryRead {
			inner: self.inner,
			off: entry.off * SECTOR_SIZE,
			len: entry.len * SECTOR_SIZE,
		})
	}
}

impl<'a, R> Read for EntryRead<'a, R>
where
	R: Read + Seek,
{
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		self.inner.seek(io::SeekFrom::Start(self.off))?;
		self.inner.take(self.len).read(buf)
	}
}

fn to_name(buf: [u8; NAME_SIZE]) -> String {
	let position = buf.iter().position(|&b| b == b'\0').unwrap_or(buf.len());
	let value = buf.into_iter().map(char::from).take(position).collect();

	value
}

#[cfg(test)]
mod tests {
	use std::io::{Cursor, Read};

	use crate::read::{V1Reader, V2Reader};

	use super::Archive;

	#[test]
	fn test_read_v1() {
		let mut dir = Cursor::new(include_bytes!("../test/v1.dir"));
		let mut img = Cursor::new(include_bytes!("../test/v1.img"));

		let archive: Archive<_> = V1Reader::new(&mut dir, &mut img).try_into().expect("failed to read archive");

		assert_eq!(archive.len(), 3);

		let virgo = archive.entry_at(0).expect("expected first entry");
		let landstal = archive.entry_at(1).expect("expected second entry");
		let test = archive.entry_at(2).expect("expected third entry");

		assert_eq!(virgo.name, "VIRGO.DFF");
		assert_eq!(virgo.off, 0);
		assert_eq!(virgo.len, 1);

		assert_eq!(landstal.name, "LANDSTAL.DFF");
		assert_eq!(landstal.off, 1);
		assert_eq!(landstal.len, 2);

		assert_eq!(test.name, "abcdefghijklmnopqrstuvwx");
		assert_eq!(test.off, 3);
		assert_eq!(test.len, 8);
	}

	#[test]
	fn test_read_v1_entry() {
		let mut dir = Cursor::new(include_bytes!("../test/v1.dir"));
		let mut img = Cursor::new(include_bytes!("../test/v1.img"));

		let mut archive: Archive<_> = V1Reader::new(&mut dir, &mut img).try_into().expect("failed to read archive");
		let mut virgo = archive.read_at(0).expect("expected first entry");

		let mut buf = [0; 8];
		let len = virgo.read(&mut buf).unwrap();

		assert_eq!(buf, [b'V', b'i', b'r', b'g', b'o', b'-', b'v', b'1']); // 'Virgo-v1'
		assert_eq!(len, 8);
	}

	#[test]
	fn test_read_v2() {
		let mut img = Cursor::new(include_bytes!("../test/v2.img"));

		let archive: Archive<_> = V2Reader::new(&mut img).try_into().expect("failed to read archive");

		assert_eq!(archive.len(), 3);

		let virgo = archive.entry_at(0).expect("expected first entry");
		let landstal = archive.entry_at(1).expect("expected second entry");
		let longer = archive.entry_at(2).expect("expected third entry");

		assert_eq!(virgo.name, "VIRGO.DFF");
		assert_eq!(virgo.off, 1);
		assert_eq!(virgo.len, 1);

		assert_eq!(landstal.name, "LANDSTAL.DFF");
		assert_eq!(landstal.off, 2);
		assert_eq!(landstal.len, 1);

		assert_eq!(longer.name, "abcdefghijklmnopqrstuvwx");
		assert_eq!(longer.off, 3);
		assert_eq!(longer.len, 8);
	}

	#[test]
	fn test_read_v2_entry() {
		let mut img = Cursor::new(include_bytes!("../test/v2.img"));

		let mut archive: Archive<_> = V2Reader::new(&mut img).try_into().expect("failed to read archive");
		let mut virgo = archive.read_at(0).expect("expected first entry");

		let mut buf = [0; 8];
		let len = virgo.read(&mut buf).unwrap();

		assert_eq!(buf, [b'V', b'i', b'r', b'g', b'o', b'-', b'v', b'2']); // 'Virgo-v2'
		assert_eq!(len, 8);
	}
}
