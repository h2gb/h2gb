use serde::{Serialize, Deserialize};
use simple_error::{bail, SimpleResult};

use generic_number::Context;

use crate::{H2Type, H2Types, H2TypeTrait, Alignment};

/// Defines a string with a configured length.
///
/// The length (in characters) is chosen when creating the type. The length in
/// bytes may be longer if the character type is non-ASCII, however. See
/// [`generic_number::GenericReader`] for a list of possible character
/// types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct H2Blob {
    length: u64,
}

impl H2Blob {
    pub fn new_aligned(alignment: Alignment, length_in_bytes: u64) -> SimpleResult<H2Type> {
        if length_in_bytes == 0 {
            bail!("Length must be at least 1 character long");
        }

        Ok(H2Type::new(alignment, H2Types::H2Blob(Self {
            length: length_in_bytes,
        })))
    }

    pub fn new(length_in_bytes: u64) -> SimpleResult<H2Type> {
        Self::new_aligned(Alignment::None, length_in_bytes)
    }
}

impl H2TypeTrait for H2Blob {
    fn actual_size(&self, _context: Context) -> SimpleResult<u64> {
        Ok(self.length)
    }

    fn to_display(&self, _context: Context) -> SimpleResult<String> {
        Ok(format!("Binary blob ({} bytes)", self.length))
    }
}

// #[cfg(test)]
// TODO: Tests
// mod tests {
//     use super::*;
//     use simple_error::SimpleResult;
//     use generic_number::{Context, Endian};
//     use crate::simple::H2Number;
//     use crate::simple::network::IPv4;

//     #[test]
//     fn test_test() -> SimpleResult<()> {
//         Ok(())
//     }
// }
