//! Reads `gta3.img` and `gta3.dir` from the current directory, and inspects the metadata for each entry.

use std::fs::File;

use gta_img::read::{Reader, V1Reader};

fn main() {
	let mut img = File::open("gta3.img").expect("failed to open img");
	let mut dir = File::open("gta3.dir").expect("failed to open dir");

	V1Reader::new(&mut dir, &mut img)
		.read()
		.expect("failed to read archive")
		.iter()
		.for_each(|entry| {
			println!("{} - offset: {}, length: {}", entry.name, entry.offset, entry.length);
		})
}
