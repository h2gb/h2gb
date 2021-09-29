use serde::{Serialize, Deserialize};

/// Define the endianness for reading multi-byte integers
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Endian {
    /// Most significant byte is first (eg, `0x1234` -> `12 34`)
    Big,

    /// Most significant byte is last (eg, `0x1234` -> `34 12`)
    Little,
}
