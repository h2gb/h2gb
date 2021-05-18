use std::fmt::Binary;
use std::mem;

use serde::{Serialize, Deserialize};

/// Configure display options for [`SizedDisplay::Binary`]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BinaryOptions {
    /// Prefix binary strings with `0b`
    pub prefix: bool,

    /// Zero-pad binary strings to the full width - `00000001` vs `1`
    pub padded: bool,
}

impl Default for BinaryOptions {
    fn default() -> Self {
        Self {
            padded: true,
            prefix: true,
        }
    }
}

impl BinaryOptions {
    /// An internal function to help with displaying binary
    pub fn to_s(self, v: Box<dyn Binary>) -> String {
        let v = v.as_ref();

        match (self.padded, self.prefix) {
            (false, false) => format!("{:b}", v),
            (false, true ) => format!("0b{:b}", v),
            (true, false) => {
                match mem::size_of_val(v) * 8 {
                    8   => format!("{:08b}",   v),
                    16  => format!("{:016b}",  v),
                    32  => format!("{:032b}",  v),
                    64  => format!("{:064b}",  v),
                    128 => format!("{:0128b}", v),
                    _   => format!("{:b}",     v),
                }
            },
            (true, true) => {
                match mem::size_of_val(v) * 8 {
                    8   => format!("0b{:08b}",   v),
                    16  => format!("0b{:016b}",  v),
                    32  => format!("0b{:032b}",  v),
                    64  => format!("0b{:064b}",  v),
                    128 => format!("0b{:0128b}", v),
                    _   => format!("0b{:b}",     v),
                }
            }
        }
    }
}

