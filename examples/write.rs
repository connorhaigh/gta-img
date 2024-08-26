//! Reads all files in the current directory, and writes 'gta3.img' and 'gta3.dir'.

use std::{
	env,
	fs::{self, File},
};

use gta_img::write::{V1Writer, Writer};

fn main() {
	let root = env::current_dir().expect("failed to get current directory");

	let mut dir = File::create("gta3.dir").expect("failed to create dir");
	let mut img = File::create("gta3.img").expect("failed to create img");

	let mut writer = V1Writer::new(&mut dir, &mut img);

	for file in fs::read_dir(root).expect("failed to read directory") {
		let file = file.expect("failed to open file");
		let mut src = File::open(file.path()).expect("failed to read file");

		writer
			.write(file.file_name().to_str().expect("failed to convert file name to string"), &mut src)
			.expect("failed to write entry");
	}
}
