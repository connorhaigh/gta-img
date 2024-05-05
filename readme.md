# GTA-IMG

GTA-IMG is a Rust-based library for reading from `IMG` archives used throughout the 3D universe-era of Grand Theft Auto games.

## Usage

```rust
let img = File::open("gta3.img").expect("failed to open img");
let dir = File::open("gta3.dir").expect("failed to open dir");

gta_img::read_v1(dir, img)
	.expect("failed to read archive")
	.for_each(|result| match result {
		Ok(entry) => println!("successfully read entry: {}; {} offset, {} length", entry.name, entry.off, entry.len),
		Err(err) => println!("failed to read entry: {}", err)
	});
let
```

## Support

Presently, the library supports manipulating archives in both V1 and V2 format, which extends to supporting the following games:

- Grand Theft Auto: III
- Grand Theft Auto: Vice City
- Grand Theft Auto: San Andreas
- Bully: Scholarship Edition (PC only)
