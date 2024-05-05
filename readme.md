# GTA-IMG

GTA-IMG is a Rust-based library for reading from `IMG` archives used throughout the 3D universe-era of Grand Theft Auto games.

## Usage

```rust
let img = File::open("gta3.img").expect("failed to open img");
let dir = File::open("gta3.dir").expect("failed to open dir");

gta_img::read(gta_img::Version::V1 { dir, img, })
	.expect("failed to read archive")
	.entries()
	.for_each(|entry_info| {
		println!("entry info: {}; {} offset, {} length", entry_info.name, entry_info.off, entry_info.len);
	});
```

## Support

Presently, the library supports manipulating archives in both V1 and V2 format, which extends to supporting the following games:

- Grand Theft Auto: III
- Grand Theft Auto: Vice City
- Grand Theft Auto: San Andreas
- Bully: Scholarship Edition (PC only)
