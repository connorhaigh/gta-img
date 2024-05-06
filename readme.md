# GTA-IMG

GTA-IMG is a Rust-based library for reading from `IMG` archives used throughout the 3D universe-era of Grand Theft Auto games.

## Usage

```rust
let mut img = File::open("gta3.img").expect("failed to open img");
let mut dir = File::open("gta3.dir").expect("failed to open dir");

gta_img::read(V1Reader::new(&mut dir, &mut img))
	.expect("failed to read archive")
	.iter()
	.for_each(|entry| {
		println!("{}, offset: {} length: {}", entry.name, entry.off, entry.len);
	})
```

## Support

Presently, the library supports reading archives in both V1 and V2 format, which extends to supporting the following games:

- Grand Theft Auto: III
- Grand Theft Auto: Vice City
- Grand Theft Auto: San Andreas
- Bully: Scholarship Edition (PC only)

## Supplementary

Included within the repository is also a Rust-based command-line application which can be used to perform a few basic operations on `IMG` and `DIR` files, namely the inspection and extraction of them.
