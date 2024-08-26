# gta-img

`gta-img` is a Rust-based library for reading from `IMG` archives (and supplementary `DIR` files) used throughout the 3D universe-era of Grand Theft Auto games.

## Overview

For all of the early 3D-based Grand Theft Auto games, the majority of their assets (such as models and textures) are stored in `img` files (with an accompanying `dir` file for older games), which is effectively a single archive which contains the metadata for each entry, as well as the entire contents of the entry itself. The structure of these files is relatively simple: effectively containing the name of an entry, as well as its offset and length. However, these entries are stored in sector-aligned sections of 2048 bytes, much like a typical [disk sector](https://en.wikipedia.org/wiki/Disk_sector). Upon start-up, the game will read the contents of these archives into memory as necessary.

Further documentation on the format, as well as additional reading, may be ascertained from the very helpful [IMG archive](https://gtamods.com/wiki/IMG_archive) article on [gtamods.com](https://gtamods.com).

## Usage

As there is a distinction between the versions of an archive in terms of the behaviour and the format, there are unique types to identify when reading from and/or writing to a V1-style or V2-style archive.

Reading all of the entries in an existing archive:

```rust
let mut img = File::open("gta3.img").expect("failed to open img");
let mut dir = File::open("gta3.dir").expect("failed to open dir");

V1Reader::new(&mut dir, &mut img)
	.expect("failed to read archive")
	.iter()
	.for_each(|entry| {
		println!("{} - offset: {}, length: {}", entry.name, entry.off, entry.len);
	})
```

Writing a single entry to a new archive:

```rust
let mut src = File::open("virgo.dff").expect("failed to open src");

let mut dir = File::create("gta3.dir").expect("failed to create dir");
let mut img = File::create("gta3.img").expect("failed to create img");

V1Writer::new(&mut dir, &mut img)
	.write("virgo.dff", &mut src)
	.expect("failed to write archive");
```

Opening each of the entries in the archive for reading:

```rust
let mut img = File::open("gta3.img").expect("failed to open img");
let mut archive = gta_img::read(V2Reader::new(&mut img)).expect("failed to read archive");

for index in 0..archive.len() {
	let mut source = archive.open(index).expect("failed to open entry");
	let mut dest = io::empty();

	io::copy(&mut source, &mut dest).expect("failed to copy entry");
}
```

## Support

Presently, the library supports reading archives in both V1 and V2 format, which extends to supporting the following games:

- Grand Theft Auto: III
- Grand Theft Auto: Vice City
- Grand Theft Auto: San Andreas
- Bully: Scholarship Edition (PC only)

## Supplementary

Included within the repository is also a example Rust-based command-line application which can be used to perform a few basic operations on `IMG` and `DIR` files, namely the inspection and extraction of them.

```
gta-img inspect v1 gta3.img gta3.dir
gta-img extract --target out v1 gta3.img gta3.dir
```
