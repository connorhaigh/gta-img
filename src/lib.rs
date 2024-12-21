//! Library for reading from/writing to `IMG` archives (and supplementary `DIR` files) used throughout the 3D universe-era of Grand Theft Auto games.

/// Contains types for errors.
pub mod error;

/// Contains types and the accompanying logic for reading from archives of different versions.
pub mod read;

/// Contains types and the accompanying logic for writing to archives of different versions.
pub mod write;

/// Represents the number of bytes of a sector.
pub const SECTOR_SIZE: u64 = 2048;

/// Represents the maximum length of the name of an entry, excluding the null-terminator.
pub const NAME_SIZE: usize = 23;

/// Represents the null terminator for the names of entries.
pub const NULL_TERMINATOR: u8 = b'\0';

/// Represents the structure for a V2-style header.
pub const VERSION_2_HEADER: [u8; 4] = [0x56, 0x45, 0x52, 0x32]; // VER2
