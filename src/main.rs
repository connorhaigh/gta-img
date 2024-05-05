use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
struct Args {
	/// Specifies the image archive
	#[arg(short, long)]
	img: PathBuf,

	/// Specifies the directory archive
	#[arg(short, long)]
	dir: Option<PathBuf>,
}

fn main() {
	let _ = Args::parse();
}
