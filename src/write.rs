use std::io::{self, Read, Seek, Write};

use byteorder::{LittleEndian, WriteBytesExt};

use crate::{error::WriteError, NAME_SIZE, NULL_TERMINATOR, SECTOR_SIZE, VERSION_2_HEADER};

/// Represents the offset for where the entries are located in the header of a V2-styled archive.
const VERSION_2_HEADER_ENTRY_OFFSET: usize = 8;

/// Represents the size of an individual entry in the header of a V2-styled archive.
const VERSION_2_HEADER_ENTRY_SIZE: usize = 32;

/// Represents a writer of V1-styled archives, to both an `img` file and a `dir` file.
#[derive(Debug)]
pub struct V1Writer<'a, 'b, D, I>
where
	D: Write,
	I: Write + Seek,
{
	dir: &'b mut D,
	img: &'a mut I,

	sector: u64,
}

/// Represents a writer of V2-styled archives, to a single `img` file.
#[derive(Debug)]
pub struct V2Writer<'a, I>
where
	I: Write + Seek,
{
	img: &'a mut I,

	sector: u64,

	entries: usize,
	written: usize,
}

/// Represents a generic archive writer that can persist archives.
pub trait Writer {
	/// Attempts to write a single entry called `name` from `src` to the head.
	fn write<T>(&mut self, name: &str, src: &mut T) -> Result<(), WriteError>
	where
		T: Read;
}

impl<'a, 'b, D, I> V1Writer<'a, 'b, D, I>
where
	D: Write,
	I: Write + Seek,
{
	/// Creates a new V1-styled writer with the specified `dir` destination and specified `img` destination.
	pub fn new(dir: &'b mut D, img: &'a mut I) -> Self {
		Self {
			dir,
			img,
			sector: 0,
		}
	}
}

impl<'a, I> V2Writer<'a, I>
where
	I: Write + Seek,
{
	/// Creates a new V2-styled writer with the specified `img` destination.
	/// Immediately writes the V2-styled header with the prefix and (expected) number of entries.
	pub fn new(img: &'a mut I, entries: usize) -> Result<Self, io::Error> {
		// Write the fixed header and (expected) number of entries.

		img.seek(io::SeekFrom::Start(0u64))?;

		img.write_all(&VERSION_2_HEADER)?;
		img.write_u32::<LittleEndian>(entries as u32)?;

		// Calculate the initial sector accommodating the size of the header.

		let sector = (VERSION_2_HEADER_ENTRY_OFFSET as u64 + (VERSION_2_HEADER_ENTRY_SIZE as u64 * entries as u64)).div_ceil(SECTOR_SIZE);

		Ok(Self {
			img,
			sector,
			entries,
			written: 0,
		})
	}
}

impl<D, I> Writer for V1Writer<'_, '_, D, I>
where
	D: Write,
	I: Write + Seek,
{
	fn write<T>(&mut self, name: &str, src: &mut T) -> Result<(), WriteError>
	where
		T: Read,
	{
		// Seek to the offset for the data.

		let offset = self.sector;

		self.img.seek(io::SeekFrom::Start(offset * SECTOR_SIZE))?;

		// Copy the source to the current sector in the archive.

		let bytes = io::copy(src, self.img)?;

		// Pad the remainder as necessary.

		let length = bytes.div_ceil(SECTOR_SIZE);
		let remainder = remainder_padded_bytes(length, bytes);

		self.img.write_all(&remainder)?;

		// Write the properties of the entry.

		self.dir.write_u32::<LittleEndian>(offset as u32)?;
		self.dir.write_u32::<LittleEndian>(length as u32)?;

		// Write the name as a null-terminated string.

		self.dir.write_all(&to_null_terminated(name))?;

		self.sector += length;

		Ok(())
	}
}

impl<I> Writer for V2Writer<'_, I>
where
	I: Write + Seek,
{
	fn write<T>(&mut self, name: &str, src: &mut T) -> Result<(), WriteError>
	where
		T: Read,
	{
		// Check if we have capacity for another entry.

		if self.written >= self.entries {
			return Err(WriteError::InsufficientHeaderSize);
		}

		// Seek to the offset for the data.

		let offset = self.sector;

		self.img.seek(io::SeekFrom::Start(offset * SECTOR_SIZE))?;

		// Copy the source to the current sector in the archive.

		let bytes = io::copy(src, self.img)?;

		// Pad the remainder as necessary.

		let length = bytes.div_ceil(SECTOR_SIZE);
		let remainder = remainder_padded_bytes(length, bytes);

		self.img.write_all(&remainder)?;

		// Seek to the offset for the header.

		self.img
			.seek(io::SeekFrom::Start(VERSION_2_HEADER_ENTRY_OFFSET as u64 + (VERSION_2_HEADER_ENTRY_SIZE as u64 * self.written as u64)))?;

		// Write the properties of the entry.

		self.img.write_u32::<LittleEndian>(offset as u32)?;
		self.img.write_u16::<LittleEndian>(length as u16)?;
		self.img.write_u16::<LittleEndian>(0u16)?; // Unused (always 0)

		// Write the name as a null-terminated string.

		self.img.write_all(&to_null_terminated(name))?;

		self.sector += length;
		self.written += 1;

		Ok(())
	}
}

fn remainder_padded_bytes(sectors: u64, bytes: u64) -> Vec<u8> {
	vec![0; ((sectors * SECTOR_SIZE).saturating_sub(bytes)) as usize]
}

fn to_null_terminated(string: &str) -> Vec<u8> {
	#[rustfmt::skip]
	let bytes = string.chars()
		.flat_map(u8::try_from)
		.chain(std::iter::repeat(NULL_TERMINATOR)).take(NAME_SIZE)
		.chain(std::iter::once(NULL_TERMINATOR))
		.collect();

	bytes
}

#[cfg(test)]
mod tests {
	use std::io::Cursor;

	use crate::{error::WriteError, write::V2Writer};

	use super::{to_null_terminated, V1Writer, Writer};

	#[test]
	pub fn test_to_name_truncate() {
		let string = "SomebodyOnceToldMeWorldGonnaRollMe";
		let slice = to_null_terminated(&string);

		assert_eq!(slice, vec![b'S', b'o', b'm', b'e', b'b', b'o', b'd', b'y', b'O', b'n', b'c', b'e', b'T', b'o', b'l', b'd', b'M', b'e', b'W', b'o', b'r', b'l', b'd', 0]); // SomebodyOnceToldMeWorld
		assert_eq!(slice.len(), 24);
	}

	#[test]
	pub fn test_to_name() {
		let string = "VIRGO.DFF";
		let slice = to_null_terminated(&string);

		assert_eq!(slice, vec![b'V', b'I', b'R', b'G', b'O', b'.', b'D', b'F', b'F', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]); // VIRGO.DFF
		assert_eq!(slice.len(), 24);
	}

	#[test]
	pub fn test_write_v1() {
		let mut dir: Cursor<Vec<u8>> = Cursor::new(Vec::new());
		let mut img: Cursor<Vec<u8>> = Cursor::new(Vec::new());

		let mut writer = V1Writer::new(&mut dir, &mut img);

		let mut virgo: Cursor<_> = Cursor::new(include_bytes!("../test/virgo.dff"));
		let mut landstal: Cursor<_> = Cursor::new(include_bytes!("../test/landstal.dff"));

		writer.write("VIRGO.DFF", &mut virgo).expect("failed to write first entry");
		writer.write("LANDSTAL.DFF", &mut landstal).expect("failed to write second entry");

		let dir_bytes = dir.get_ref();

		assert_eq!(dir_bytes[00..04], [0, 0, 0, 0]); // Offset
		assert_eq!(dir_bytes[04..08], [1, 0, 0, 0]); // Length
		assert_eq!(dir_bytes[08..32], [b'V', b'I', b'R', b'G', b'O', b'.', b'D', b'F', b'F', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]); // VIRGO.DFF

		assert_eq!(dir_bytes[32..36], [1, 0, 0, 0]); // Offset
		assert_eq!(dir_bytes[36..40], [1, 0, 0, 0]); // Length
		assert_eq!(dir_bytes[40..64], [b'L', b'A', b'N', b'D', b'S', b'T', b'A', b'L', b'.', b'D', b'F', b'F', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]); // LANDSTAL.DFF

		let img_bytes = img.get_ref();

		assert_eq!(img_bytes[0000..0009], [b'V', b'I', b'R', b'G', b'O', b'!', b'D', b'F', b'F']); // VIRGO!DFF
		assert_eq!(img_bytes[2048..2060], [b'L', b'A', b'N', b'D', b'S', b'T', b'A', b'L', b'!', b'D', b'F', b'F']); // LANDSTAL!DFF

		assert_eq!(img_bytes.len(), 4096);
	}

	#[test]
	pub fn test_write_v2() {
		let mut img: Cursor<_> = Cursor::new(Vec::new());

		let mut writer = V2Writer::new(&mut img, 2).expect("failed to create writer");

		let mut virgo: Cursor<_> = Cursor::new(include_bytes!("../test/virgo.dff"));
		let mut landstal: Cursor<_> = Cursor::new(include_bytes!("../test/landstal.dff"));

		writer.write("VIRGO.DFF", &mut virgo).expect("failed to write first entry");
		writer.write("LANDSTAL.DFF", &mut landstal).expect("failed to write second entry");

		let bytes = img.get_ref();

		assert_eq!(bytes[0..4], [0x56, 0x45, 0x52, 0x32]); // VER2
		assert_eq!(bytes[4..8], [2, 0, 0, 0]); // Entries

		assert_eq!(bytes[08..12], [1, 0, 0, 0]); // Offset
		assert_eq!(bytes[12..16], [1, 0, 0, 0]); // Length
		assert_eq!(bytes[16..40], [b'V', b'I', b'R', b'G', b'O', b'.', b'D', b'F', b'F', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]); // VIRGO.DFF

		assert_eq!(bytes[40..44], [2, 0, 0, 0]); // Offset
		assert_eq!(bytes[44..48], [1, 0, 0, 0]); // Length
		assert_eq!(bytes[48..72], [b'L', b'A', b'N', b'D', b'S', b'T', b'A', b'L', b'.', b'D', b'F', b'F', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]); // LANDSTAL.DFF

		assert_eq!(bytes[2048..2057], [b'V', b'I', b'R', b'G', b'O', b'!', b'D', b'F', b'F']); // VIRGO!DFF
		assert_eq!(bytes[4096..4108], [b'L', b'A', b'N', b'D', b'S', b'T', b'A', b'L', b'!', b'D', b'F', b'F']); // VIRGO!DFF

		assert_eq!(bytes.len(), 6144);
	}

	#[test]
	pub fn test_write_v2_space() {
		let mut img: Cursor<_> = Cursor::new(Vec::new());

		let mut writer = V2Writer::new(&mut img, 1).expect("failed to create writer");

		let mut virgo: Cursor<_> = Cursor::new(include_bytes!("../test/virgo.dff"));
		let mut landstal: Cursor<_> = Cursor::new(include_bytes!("../test/landstal.dff"));

		let first_write = writer.write("VIRGO.DFF", &mut virgo);
		let second_write = writer.write("LANDSTAL.DFF", &mut landstal);

		assert!(matches!(first_write, Ok(())));
		assert!(matches!(second_write, Err(WriteError::InsufficientHeaderSize)));
	}
}
