use std::io::{self, Read, Seek, Write};

use byteorder::{LittleEndian, WriteBytesExt};

use crate::{error::WriteError, NAME_SIZE, SECTOR_SIZE};

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

		let name = to_name(name);

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

fn to_name(str: &str) -> Vec<u8> {
	str.chars()
		.map(u8::try_from)
		.flatten()
		.chain(std::iter::repeat(b'\0'))
		.take(NAME_SIZE)
		.chain(std::iter::once(b'\0'))
		.collect()
}

#[cfg(test)]
mod tests {
	use std::io::Cursor;

	use super::{to_name, V1Writer, Writer};

	#[test]
	pub fn test_to_name() {
		let string = "SomebodyOnceToldMeWorldGonnaRollMe";
		let slice = to_name(&string);

		assert_eq!(slice, vec![83, 111, 109, 101, 98, 111, 100, 121, 79, 110, 99, 101, 84, 111, 108, 100, 77, 101, 87, 111, 114, 108, 100, 0]);
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
			86, 73, 82, 71, 79, 46, 68, 70, 70, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, // Name
			0, 8, 0, 0, // Offset
			1, 0, 0, 0, // Length,
			76, 65, 78, 68, 83, 84, 65, 76, 46, 68, 70, 70, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 // Name
		];

		assert_eq!(dir.get_ref(), &dir_bytes);
	}
}
