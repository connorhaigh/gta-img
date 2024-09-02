use std::io::{self, Read, Seek, Write};

use byteorder::{LittleEndian, WriteBytesExt};

use crate::{error::WriteError, NAME_SIZE, NULL_TERMINATOR, SECTOR_SIZE};

/// Represents a write of V1-styled archives, to both an `img` file and a `dir` file.
#[derive(Debug)]
pub struct V1Writer<'a, 'b, D, I> {
	dir: &'b mut D,
	img: &'a mut I,

	pos: u64,
}

/// Represents a reader of V2-styled archives, to a single `img` file.
#[derive(Debug)]
pub struct V2Writer<'a, I> {
	img: &'a mut I,
}

/// Represents a generic archive writer that can persist archives.
pub trait Writer {
	/// Attempts to write a single entry called `name` from `src` to the head.
	fn write<T>(&mut self, name: &str, src: &mut T) -> Result<(), WriteError>
	where
		T: Read;
}

impl<'a, 'b, D, I> V1Writer<'a, 'b, D, I> {
	/// Creates a new V1-styled writer with the specified `dir` destination and specified `img` destination.
	pub fn new(dir: &'b mut D, img: &'a mut I) -> Self {
		Self {
			dir,
			img,
			pos: 0,
		}
	}
}

impl<'a, I> V2Writer<'a, I> {
	/// Creates a new V2-styled writer with the specified `img` destination.
	pub fn new(img: &'a mut I) -> Self {
		Self {
			img,
		}
	}
}

impl<'a, 'b, D, I> Writer for V1Writer<'a, 'b, D, I>
where
	D: Write,
	I: Write + Seek,
{
	fn write<T>(&mut self, name: &str, src: &mut T) -> Result<(), WriteError>
	where
		T: Read,
	{
		// Copy the source to the current sector in the archive.

		let pos = self.pos * SECTOR_SIZE;

		self.img.seek(io::SeekFrom::Start(pos))?;

		let bytes = io::copy(src, self.img)?;

		self.pos += 1;

		// Write the properties of the entry.

		let off = pos as u32;
		let len = bytes.div_ceil(SECTOR_SIZE) as u32;

		self.dir.write_u32::<LittleEndian>(off)?;
		self.dir.write_u32::<LittleEndian>(len)?;

		// Write the name as a null-terminated string.

		let name = to_null_terminated(name);

		self.dir.write_all(&name)?;

		Ok(())
	}
}

impl<'a, I> Writer for V2Writer<'a, I>
where
	I: Write + Seek,
{
	fn write<T>(&mut self, name: &str, src: &mut T) -> Result<(), WriteError>
	where
		T: Read,
	{
		todo!("implementation undecided")
	}
}

fn to_null_terminated(str: &str) -> Vec<u8> {
	str.chars()
		.flat_map(u8::try_from)
		.chain(std::iter::repeat(NULL_TERMINATOR))
		.take(NAME_SIZE)
		.chain(std::iter::once(NULL_TERMINATOR))
		.collect()
}

#[cfg(test)]
mod tests {
	use std::io::Cursor;

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

		#[rustfmt::skip]
		let dir_bytes = vec![
			0, 0, 0, 0, // Offset
			1, 0, 0, 0, // Length
			b'V', b'I', b'R', b'G', b'O', b'.', b'D', b'F', b'F', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // VIRGO.DFF
			0, 8, 0, 0, // Offset
			1, 0, 0, 0, // Length,
			b'L', b'A', b'N', b'D', b'S', b'T', b'A', b'L', b'.', b'D', b'F', b'F', 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 // LANDSTAL.DFF
		];

		assert_eq!(dir.get_ref(), &dir_bytes);
	}
}
