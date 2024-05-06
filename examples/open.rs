//! Reads `gta3.img` from the current directory, and opens each entry for reading.

use std::{
	fs::File,
	io::{self},
};

use gta_img::read::V2Reader;

fn main() {
	let mut img = File::open("gta3.img").expect("failed to open img");
	let mut archive = gta_img::read(V2Reader::new(&mut img)).expect("failed to read archive");

	for index in 0..archive.len() {
		let mut source = archive.open(index).expect("failed to open entry");
		let mut dest = io::empty();

		io::copy(&mut source, &mut dest).expect("failed to copy entry");
	}
}
