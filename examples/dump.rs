//! Reads `gta3.img` and `gta3.dir` from the current directory, and outputs its contents.

use std::fs::File;

use gta_img::read::V1Reader;

fn main() {
	let mut img = File::open("gta3.img").expect("failed to open img");
	let mut dir = File::open("gta3.dir").expect("failed to open dir");

	gta_img::read(V1Reader::new(&mut dir, &mut img))
		.expect("failed to read archive")
		.iter()
		.for_each(|entry| {
			println!("{} - offset: {}, length: {}", entry.name, entry.off, entry.len);
		})
}
