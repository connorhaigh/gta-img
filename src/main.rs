//! Command-line application demonstrating usage of the `gta-img` library.

use std::{fs::File, io, path::PathBuf};

use clap::{command, Parser, Subcommand};
use gta_img::read::{V1Reader, V2Reader};

/// Performs basic read-only operations on IMG/DIR archives
#[derive(Debug, Parser)]
struct Cli {
	/// Indicates the operation to perform
	#[command(subcommand)]
	operation: Operation,
}

/// Represents the operation to perform
#[derive(Debug, Subcommand)]
enum Operation {
	/// Inspect the contents of an archive
	Inspect {
		/// Specifies the archive to inspect
		#[command(subcommand)]
		version: Version,
	},

	/// Extract the contents of an archive to an output directory
	Extract {
		/// Specifies the archive to extract
		#[command(subcommand)]
		version: Version,

		/// Specifies the output directory
		#[arg(short, long)]
		target: PathBuf,
	},
}

/// Represents the version of an archive
#[derive(Debug, Subcommand)]
enum Version {
	/// Dictates a V1-styled archive (img file and dir file)
	V1 {
		/// Specifies the img file
		img: PathBuf,

		/// Specifies the dir file
		dir: PathBuf,
	},
	/// Dictates a V2-styled archive (img file only)
	V2 {
		/// Specifies the img file
		img: PathBuf,
	},
}

fn main() {
	let cli = Cli::parse();

	let mut img_file: File;
	let mut dir_file: File;

	// Ascertain the version based on the operation.

	let version = match &cli.operation {
		Operation::Inspect {
			version,
		} => version,
		Operation::Extract {
			version,
			target: _,
		} => version,
	};

	// Read the archive depending on the provided version.

	let mut archive = match version {
		Version::V1 {
			img,
			dir,
		} => {
			img_file = File::open(img).expect("failed to open img file");
			dir_file = File::open(dir).expect("failed to open dir file");

			println!("Reading V1-styled archive...");

			gta_img::read(V1Reader::new(&mut dir_file, &mut img_file)).expect("failed to read V1-styled archive")
		}
		Version::V2 {
			img,
		} => {
			img_file = File::open(img).expect("failed to open img file");

			println!("Reading V2-styled archive...");

			gta_img::read(V2Reader::new(&mut img_file)).expect("failed to read V2-styled archive")
		}
	};

	// Perform the operation.

	match cli.operation {
		Operation::Inspect {
			version: _,
		} => {
			println!("Inspecting contents of archive...");

			for entry in archive.iter() {
				println!("[{:<24}] offset: {}, length: {}", entry.name, entry.off, entry.len);
			}

			println!("Inspected {} entries.", archive.len());
		}
		Operation::Extract {
			version: _,
			target,
		} => {
			println!("Extracting contents of archive to path...");

			for index in 0..archive.len() {
				let entry = archive.get(index).expect("failed to get entry");
				let path = target.join(&entry.name);

				println!("Extracting entry [{}] to file <{}>...", entry.name, &path.display());

				let mut open = archive.open(index).expect("failed to open entry");
				let mut file = File::create(&path).expect("failed to create entry file");

				io::copy(&mut open, &mut file).expect("failed to extract entry to file");
			}

			println!("Extracted {} entries.", archive.len());
		}
	}
}
