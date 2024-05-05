use std::{
	cell::RefCell,
	io::{self, Read, Seek},
	rc::{Rc, Weak},
};

use byteorder::{LittleEndian, ReadBytesExt};

use crate::error::ReadError;

/// Represents the structure for a version 2-style header.
pub const VERSION_2_HEADER: [u8; 4] = [0x56, 0x45, 0x52, 0x32]; // VER2
pub const VERSION_2_HEADER_SIZE: usize = VERSION_2_HEADER.len();

/// Represents the number of bytes used for sector alignment.
pub const SECTOR_SIZE: u64 = 2048;

/// Represents the maximum length of the name of an entry.
pub const NAME_SIZE: usize = 24;

/// Represents an individual entry.
pub struct Entry<R>
where
	R: Read + Seek,
{
	/// The name of the entry, up to a maximum of 24 characters.
	pub name: String,

	/// The absolute offset of the entry.
	pub off: u64,

	/// The absolute length of the entry.
	pub len: u64,

	inner: Weak<RefCell<R>>,
}

/// Represents a V1-styled reader, where the directory file for indices and the image file for data are separate files.
pub struct V1Read<D, I>
where
	D: Read,
	I: Read + Seek,
{
	dir: Rc<RefCell<D>>,
	img: Rc<RefCell<I>>,
}

/// Represents a V2-styled reader, where the indices and the data are the same file.
pub struct V2Read<I>
where
	I: Read + Seek,
{
	idx: usize,
	len: usize,

	img: Rc<RefCell<I>>,
}

impl<R> Read for Entry<R>
where
	R: Read + Seek,
{
	fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
		let rc = self.inner.upgrade().unwrap();
		let mut read = rc.borrow_mut();

		// Limit the read to the length of the offset or the length of the buffer, whichever is smallest.

		let max = self.len.min(buf.len() as u64) as usize;

		read.seek(io::SeekFrom::Start(self.off))?;
		read.read(&mut buf[..max])
	}
}

impl<D, I> V1Read<D, I>
where
	D: Read,
	I: Read + Seek,
{
	/// Creates a new V1-styled reader for the specified `dir` source and specified `img` source.
	pub fn new(dir: D, img: I) -> Result<V1Read<D, I>, ReadError> {
		Ok(Self {
			dir: Rc::new(RefCell::new(dir)),
			img: Rc::new(RefCell::new(img)),
		})
	}
}

impl<I> V2Read<I>
where
	I: Read + Seek,
{
	/// Creates a new V2-styled reader for the specified `img` source.
	pub fn new(mut img: I) -> Result<V2Read<I>, ReadError> {
		// Check if the header is a valid version 2 header.

		let mut buffer = [0; VERSION_2_HEADER_SIZE];

		img.read_exact(&mut buffer)?;

		if buffer != VERSION_2_HEADER {
			return Err(ReadError::InvalidHeader);
		}

		// Read the number of entries.

		let count = img.read_u32::<LittleEndian>()? as usize;

		Ok(Self {
			idx: 0,
			len: count,
			img: Rc::new(RefCell::new(img)),
		})
	}
}

impl<D, I> Iterator for V1Read<D, I>
where
	D: Read,
	I: Read + Seek,
{
	type Item = Result<Entry<I>, ReadError>;

	fn next(&mut self) -> Option<Self::Item> {
		// Attempt to read the offset for the next entry. As we do not know how many entries are in the directory, gracefully handle an end-of-file error.
		// Any other errors should be returned as normal.

		let mut dir = self.dir.borrow_mut();

		let off = match dir.read_u32::<LittleEndian>() {
			Ok(off) => off as u64,
			Err(err) if matches!(err.kind(), io::ErrorKind::UnexpectedEof) => return None,
			Err(err) => return Some(Err(err.into())),
		};

		// Read the length.

		let len = match dir.read_u32::<LittleEndian>() {
			Ok(len) => len as u64,
			Err(err) => return Some(Err(err.into())),
		};

		// Read the name as a null-terminated string.

		let mut buf = [0; NAME_SIZE];

		if let Err(err) = dir.read_exact(&mut buf) {
			return Some(Err(err.into()));
		};

		let name = to_name(buf);

		Some(Ok(Entry {
			name,
			off: off * SECTOR_SIZE,
			len: len * SECTOR_SIZE,
			inner: Rc::downgrade(&self.img),
		}))
	}
}

impl<I> Iterator for V2Read<I>
where
	I: Read + Seek,
{
	type Item = Result<Entry<I>, ReadError>;

	fn next(&mut self) -> Option<Self::Item> {
		// Check if we are at the known end of the archive.
		// Always return None if so.

		if self.idx >= self.len {
			return None;
		}

		self.idx += 1;

		// Read the offset and the length.

		let mut img = self.img.borrow_mut();

		let off = match img.read_u32::<LittleEndian>() {
			Ok(off) => off as u64,
			Err(err) => return Some(Err(err.into())),
		};

		let len = match img.read_u16::<LittleEndian>() {
			Ok(len) => len as u64,
			Err(err) => return Some(Err(err.into())),
		};

		// Read the 'size in archive' which is unused.

		let _ = match img.read_u16::<LittleEndian>() {
			Ok(sin) => sin as u64,
			Err(err) => return Some(Err(err.into())),
		};

		// Read the name as a null-terminated string.

		let mut buf = [0; NAME_SIZE];

		if let Err(err) = img.read_exact(&mut buf) {
			return Some(Err(err.into()));
		};

		let name = to_name(buf);

		Some(Ok(Entry {
			name,
			off: off * SECTOR_SIZE,
			len: len * SECTOR_SIZE,
			inner: Rc::downgrade(&self.img),
		}))
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

	use super::{V1Read, V2Read};

	#[test]
	fn test_read_v1() {
		let dir = Cursor::new(include_bytes!("../test/v1.dir"));
		let img = Cursor::new(include_bytes!("../test/v1.img"));

		let mut read = V1Read::new(dir, img).expect("failed to create V1Read");

		let virgo = read.next().expect("expected first entry").unwrap();
		let landstal = read.next().expect("expected second entry").unwrap();
		let longer = read.next().expect("expected third entry").unwrap();

		assert_eq!(virgo.name, "VIRGO.DFF");
		assert_eq!(virgo.off, 0);
		assert_eq!(virgo.len, 2048);

		assert_eq!(landstal.name, "LANDSTAL.DFF");
		assert_eq!(landstal.off, 2048);
		assert_eq!(landstal.len, 4096);

		assert_eq!(longer.name, "abcdefghijklmnopqrstuvwx");
		assert_eq!(longer.off, 6144);
		assert_eq!(longer.len, 16384);

		assert!(read.next().is_none());
	}

	#[test]
	fn test_read_v1_entry() {
		let dir = Cursor::new(include_bytes!("../test/v1.dir"));
		let img = Cursor::new(include_bytes!("../test/v1.img"));

		let mut read = V1Read::new(dir, img).expect("failed to create V1Read");
		let mut virgo = read.next().expect("expected first entry").unwrap();

		let mut buf = [0; 8];
		let len = virgo.read(&mut buf).unwrap();

		assert_eq!(buf, [b'V', b'i', b'r', b'g', b'o', b'-', b'v', b'1']); // 'Virgo-v1'
		assert_eq!(len, 8);
	}

	#[test]
	fn test_read_v2() {
		let img = Cursor::new(include_bytes!("../test/v2.img"));

		let mut read = V2Read::new(img).expect("failed to create V2Read");

		assert_eq!(read.len, 3);

		let virgo = read.next().expect("expected first entry").unwrap();
		let landstal = read.next().expect("expected second entry").unwrap();
		let longer = read.next().expect("expected third entry").unwrap();

		assert_eq!(virgo.name, "VIRGO.DFF");
		assert_eq!(virgo.off, 2048);
		assert_eq!(virgo.len, 2048);

		assert_eq!(landstal.name, "LANDSTAL.DFF");
		assert_eq!(landstal.off, 4096);
		assert_eq!(landstal.len, 2048);

		assert_eq!(longer.name, "abcdefghijklmnopqrstuvwx");
		assert_eq!(longer.off, 6144);
		assert_eq!(longer.len, 16384);

		assert!(read.next().is_none());
	}

	#[test]
	fn test_read_v2_entry() {
		let img = Cursor::new(include_bytes!("../test/v2.img"));

		let mut read = V2Read::new(img).expect("failed to create V2Read");
		let mut virgo = read.next().expect("expected first entry").unwrap();

		let mut buf = [0; 8];
		let len = virgo.read(&mut buf).unwrap();

		assert_eq!(buf, [b'V', b'i', b'r', b'g', b'o', b'-', b'v', b'2']); // 'Virgo-v2'
		assert_eq!(len, 8);
	}
}
