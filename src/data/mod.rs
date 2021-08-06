//! A library for loading data that analyzers and users can consume.
//!
//! Currently, there are two datatypes that we use: Enum and Bitmask.
//!
//! * Enums

mod enums;
pub use enums::{from_enum, enum_exists};

mod bitmaps;
pub use bitmaps::{from_bitmap, bitmap_exists};

// use enums::terraria_game_mode::*;
// use enums::terraria_buff::*;
// use enums::terraria_item::*;
// use enums::terraria_version::*;
// use enums::terraria_affix::*;
