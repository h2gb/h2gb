use std::fmt::Octal;
use std::mem;

use serde::{Serialize, Deserialize};

/// Configure display options for [`SizedDisplay::Octal`]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OctalOptions {
    /// Prefix octal strings with `0o`
    pub prefix: bool,

    /// Zero-pad octal strings to the full width - `0001` vs `1`)
    pub padded: bool,
}

impl Default for OctalOptions {
    fn default() -> Self {
        Self {
            prefix: true,
            padded: false,
        }
    }
}

impl OctalOptions {
    /// An internal function to help with displaying octal
    pub fn to_s(self, v: Box<dyn Octal>) -> String {
        let v = v.as_ref();

        if self.padded {
            match (self.prefix, mem::size_of_val(v)) {
                (false, 1)  => format!("{:03o}", v),
                (false, 2)  => format!("{:06o}", v),
                (false, 4)  => format!("{:011o}", v),
                (false, 8)  => format!("{:022o}", v),
                (false, 16) => format!("{:043o}", v),
                (false, _) => format!("{:o}", v),

                (true,  1)  => format!("0o{:03o}", v),
                (true,  2)  => format!("0o{:06o}", v),
                (true,  4)  => format!("0o{:011o}", v),
                (true,  8)  => format!("0o{:022o}", v),
                (true,  16) => format!("0o{:043o}", v),

                (true,   _) => format!("0o{:o}", v),
            }
        } else {
            match self.prefix {
                false => format!("{:o}", v),
                true  => format!("0o{:o}", v),
            }
        }
    }
}
