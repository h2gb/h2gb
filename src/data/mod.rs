//! A library for loading data that analyzers and users can consume.
//!
//! Currently, there are two datatypes that we use: Enum and Bitmask.
//!
//! # Enums
//!
//! An enum is a collection of named values, with a type. For example,
//! Terraria's "game mode" has 4 possible values:

mod enums;
pub use enums::{from_enum, enum_exists};

mod bitmasks;
pub use bitmasks::{from_bitmask, bitmask_exists};
