//! Reads `gta3.img` from the current directory, and opens each entry for reading.

use std::{
	fs::File,
	io::{self},
};

use gta_img::read::{Reader, V2Reader};

fn main() {
	let mut img = File::open("gta3.img").expect("failed to open img");
	let mut archive = V2Reader::new(&mut img).read().expect("failed to read archive");

	for index in 0..archive.len() {
		let mut src = archive.open(index).expect("failed to open entry");
		let mut dst = io::empty();

		io::copy(&mut src, &mut dst).expect("failed to copy entry");
	}
}
