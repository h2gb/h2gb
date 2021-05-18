use std::fmt::LowerHex;
use std::mem;

use serde::{Serialize, Deserialize};

/// Configure display options for [`SizedDisplay::Hex`]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct HexOptions {
    /// Print hex characters uppercase - `1A2B` vs `1a2b`.
    pub uppercase: bool,

    /// Prefix hex strings with `0x`
    pub prefix: bool,

    /// Zero-pad hex strings to the full width - `0001` vs `1`)
    pub padded: bool,
}

impl Default for HexOptions {
    fn default() -> Self {
        Self {
            uppercase: false,
            prefix: true,
            padded: true,
        }
    }
}

impl HexOptions {
    /// An internal function to help with displaying hex.
    ///
    /// Unfortunately, I don't know of a way to require both [`UpperHex`] and
    /// [`LowerHex`] traits, so I do some manual formatting :-/
    pub fn to_s(self, v: Box<dyn LowerHex>) -> String {
        let v = v.as_ref();

        let mut h = match self.padded {
            // No padding is easy
            false => format!("{:x}",   v),

            // Padding requires a bit more tinkering to do dynamically
            true => {
                match (self.padded, mem::size_of_val(v) * 2) {
                    (true, 2)   => format!(  "{:02x}",  v),
                    (true, 4)   => format!(  "{:04x}",  v),
                    (true, 8)   => format!(  "{:08x}",  v),
                    (true, 16)  => format!(  "{:016x}", v),
                    (true, 32)  => format!(  "{:032x}", v),

                    // When not padded, or in doubt about length, just print normally
                    (_, _)      => format!(  "{:x}",     v),
                }
            }
        };

        // There's no way to make the parameter both LowerHex and UpperHex
        if self.uppercase {
            h = h.to_uppercase();
        }

        if self.prefix {
            h = format!("0x{}", h);
        }

        h
    }
}
