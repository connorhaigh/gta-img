use std::{
	cmp,
	hash::{self, Hash},
	io::{self, Read, Seek},
};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::{error::ReadError, NAME_SIZE, SECTOR_SIZE};

/// Represents the structure for a V2-style header.
pub const VERSION_2_HEADER: [u8; 4] = [0x56, 0x45, 0x52, 0x32]; // VER2

/// Represents the length of the structure for a V2-style header; always `4`.
pub const VERSION_2_HEADER_SIZE: usize = 4;

/// Represents an archive.
#[derive(Debug)]
pub struct Archive<'a, R> {
	inner: &'a mut R,

	entries: Vec<Entry>,
}

/// Represents an entry.
#[derive(Debug, Clone, Hash, Eq, PartialEq, PartialOrd)]
pub struct Entry {
	/// The name of the entry, up to 23 characters.
	pub name: String,

	/// The offset, in sectors, of the entry.
	pub off: u64,

	/// The length, in sectors, of the entry.
	pub len: u64,
}

/// Represents an entry opened for reading.
#[derive(Debug)]
pub struct OpenEntry<'a, R> {
	inner: &'a mut R,

	off: u64,
	len: u64,
	pos: u64,
}

/// Represents a reader of V1-styled archives, from both an `img` file and a `dir` file.
#[derive(Debug)]
pub struct V1Reader<'a, 'b, D, I> {
	dir: &'b mut D,
	img: &'a mut I,
}

/// Represents a reader of V2-styled archives, from a single `img` file.
#[derive(Debug)]
pub struct V2Reader<'a, I> {
	img: &'a mut I,
}

/// Represents a generic archive reader that can produce archives.
pub trait Reader<'a, R> {
	/// Attempts to fully read an entire archive.
	fn read(self) -> Result<Archive<'a, R>, ReadError>;
}

impl<'a, 'b, D, I> V1Reader<'a, 'b, D, I> {
	/// Creates a new V1-styled reader with the specified `dir` source and specified `img` source.
	pub fn new(dir: &'b mut D, img: &'a mut I) -> Self {
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

impl<'a, 'b, D, I> Reader<'a, I> for V1Reader<'a, 'b, D, I>
where
	D: Read,
	I: Read + Seek,
{
	fn read(self) -> Result<Archive<'a, I>, ReadError> {
		let mut entries: Vec<Entry> = Vec::new();

		loop {
			// Attempt to read the offset for the next entry, however graciously handle an EOF.
			// Return any other kind of errors as normal.

			let off = match self.dir.read_u32::<LittleEndian>() {
				Ok(off) => off as u64,
				Err(err) => match err.kind() {
					io::ErrorKind::UnexpectedEof => break,
					_ => return Err(err.into()),
				},
			};

			// Read the properties of the entry.

			let len = self.dir.read_u32::<LittleEndian>()? as u64;

			// Read the name as a null-terminated string.

			let name = {
				let mut buf = [0; NAME_SIZE];

				self.dir.read_exact(&mut buf)?;

				to_name(&buf)
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

impl<'a, I> Reader<'a, I> for V2Reader<'a, I>
where
	I: Read + Seek,
{
	fn read(self) -> Result<Archive<'a, I>, ReadError> {
		// Read the header of the archive.

		let header = {
			let mut buffer = [0; VERSION_2_HEADER_SIZE];

			self.img.read_exact(&mut buffer)?;

			buffer
		};

		// Check if the header is of the expected format.

		if header != VERSION_2_HEADER {
			return Err(ReadError::InvalidHeader);
		}

		// Read the (expected) number of entries in the archive.

		let count = self.img.read_u32::<LittleEndian>()? as usize;
		let mut entries: Vec<Entry> = Vec::with_capacity(count);

		for _ in 0..count {
			// Read the properties of the entry.

			let off = self.img.read_u32::<LittleEndian>()? as u64;
			let len = self.img.read_u16::<LittleEndian>()? as u64;
			let _ = self.img.read_u16::<LittleEndian>()?; // Unused (always 0)

			// Read the name as a null-terminated string.

			let name = {
				let mut buf = [0; NAME_SIZE];

				self.img.read_exact(&mut buf)?;

				to_name(&buf)
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
	pub fn get(&self, index: usize) -> Option<&Entry> {
		self.entries.get(index)
	}

	/// Returns an iterator over each of the entries in the archive.
	pub fn iter(&self) -> impl Iterator<Item = &Entry> {
		self.entries.iter()
	}
}

impl<'a, I> Archive<'a, I>
where
	I: Read + Seek,
{
	/// Opens and returns the entry at the specified index for reading, if it exists.
	pub fn open(&mut self, index: usize) -> Option<OpenEntry<I>> {
		let entry = self.entries.get(index)?;

		Some(OpenEntry {
			inner: self.inner,
			off: entry.off * SECTOR_SIZE,
			len: entry.len * SECTOR_SIZE,
			pos: 0,
		})
	}
}

impl<'a, R> Read for OpenEntry<'a, R>
where
	R: Read + Seek,
{
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		// Check if we are not at EoF (for the entry).

		if self.pos >= self.len {
			return Ok(0);
		}

		// Seek to the start of the entry including any currently read bytes.

		self.inner.seek(io::SeekFrom::Start(self.off + self.pos))?;

		// Calculate the maximum possible number of bytes to read for the entry, to forbid reading beyond it.
		// Includes the number of bytes already read, honouring the length of the entry and the length of the buffer.

		let max = (self.len - self.pos.min(self.len)).min(buf.len() as u64) as usize;
		let read = self.inner.read(&mut buf[0..max])?;

		self.pos += read as u64;

		Ok(read)
	}
}

impl<'a, I> Hash for Archive<'a, I> {
	fn hash<H: hash::Hasher>(&self, state: &mut H) {
		self.entries.hash(state);
	}
}

impl<'a, I> PartialEq for Archive<'a, I> {
	fn eq(&self, other: &Self) -> bool {
		self.entries == other.entries
	}
}

impl<'a, I> PartialOrd for Archive<'a, I> {
	fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
		self.entries.partial_cmp(&other.entries)
	}
}

fn to_name(buf: &[u8]) -> String {
	buf.iter()
		.map(|&b| char::from(b))
		.take(buf.iter().position(|&b| b == b'\0').unwrap_or(buf.len()).min(NAME_SIZE))
		.collect()
}

#[cfg(test)]
mod tests {
	use std::io::{Cursor, Read};

	use crate::read::{Reader, V1Reader, V2Reader};

	use super::{to_name, Archive};

	#[test]
	fn test_to_name() {
		let slice = vec![83, 111, 109, 101, 98, 111, 100, 121, 79, 110, 99, 101, 84, 111, 108, 100, 77, 101, 87, 111, 114, 108, 100, 71, 111, 110, 110, 97, 82, 111, 108, 108, 77, 101]; // SomebodyOnceToldMeWorldGonnaRollMe
		let string = to_name(&slice);

		assert_eq!(string, "SomebodyOnceToldMeWorld");
	}

	#[test]
	fn test_read_v1() {
		let mut dir = Cursor::new(include_bytes!("../test/v1.dir"));
		let mut img = Cursor::new(include_bytes!("../test/v1.img"));

		let archive: Archive<_> = V1Reader::new(&mut dir, &mut img).read().expect("failed to read archive");

		assert_eq!(archive.len(), 3);

		let virgo = archive.get(0).expect("expected first entry");
		let landstal = archive.get(1).expect("expected second entry");
		let test = archive.get(2).expect("expected third entry");

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

		let mut archive: Archive<_> = V1Reader::new(&mut dir, &mut img).read().expect("failed to read archive");
		let mut virgo = archive.open(0).expect("expected first entry");

		let mut buf = [0; 8];
		let len = virgo.read(&mut buf).expect("failed to read entry");

		assert_eq!(buf, [b'V', b'i', b'r', b'g', b'o', b'-', b'v', b'1']); // 'Virgo-v1'
		assert_eq!(len, 8);
	}

	#[test]
	fn test_read_v2() {
		let mut img = Cursor::new(include_bytes!("../test/v2.img"));

		let archive: Archive<_> = V2Reader::new(&mut img).read().expect("failed to read archive");

		assert_eq!(archive.len(), 3);

		let virgo = archive.get(0).expect("expected first entry");
		let landstal = archive.get(1).expect("expected second entry");
		let longer = archive.get(2).expect("expected third entry");

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

		let mut archive: Archive<_> = V2Reader::new(&mut img).read().expect("failed to read archive");
		let mut virgo = archive.open(0).expect("expected first entry");

		let mut buf = [0; 8];
		let len = virgo.read(&mut buf).expect("failed to read entry");

		assert_eq!(buf, [b'V', b'i', b'r', b'g', b'o', b'-', b'v', b'2']); // 'Virgo-v2'
		assert_eq!(len, 8);
	}

	#[test]
	fn test_read_entry_partial() {
		let mut dir = Cursor::new(include_bytes!("../test/v1.dir"));
		let mut img = Cursor::new(include_bytes!("../test/v1.img"));

		let mut archive: Archive<_> = V1Reader::new(&mut dir, &mut img).read().expect("failed to read archive");
		let mut entry = archive.open(0).expect("expected first entry");

		let mut buf = [0; 1024];
		let num = entry.read(&mut buf).expect("failed to read entry first time");

		assert_eq!(buf[0..8], [b'V', b'i', b'r', b'g', b'o', b'-', b'v', b'1']); // 'Virgo-v1'
		assert_eq!(num, 1024);

		let num = entry.read(&mut buf).expect("failed to read entry second time");

		assert_eq!(num, 1024);

		let num = entry.read(&mut buf);

		assert!(matches!(num, Ok(0)));
	}
}
