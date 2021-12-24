use serde::{Serialize, Deserialize};

use simple_error::SimpleResult;
use uuid::Uuid;

use generic_number::{Context, Endian};
use crate::{Alignment, H2Type, H2Types, H2TypeTrait};

/// Defines a UUID.
///
/// An UUID address is always represented as a 16-byte value. It's always
/// displayed in standard UUID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct H2UUID {
    endian: Endian,
}

impl H2UUID {
    pub fn new_aligned(alignment: Alignment, endian: Endian) -> H2Type {
        H2Type::new(alignment, H2Types::H2UUID(Self {
            endian: endian
        }))
    }

    pub fn new(endian: Endian) -> H2Type {
        Self::new_aligned(Alignment::None, endian)
    }
}

impl H2TypeTrait for H2UUID {
    fn actual_size(&self, _context: Context) -> SimpleResult<u64> {
        Ok(16)
    }

    fn to_display(&self, context: Context) -> SimpleResult<String> {
        let number = context.read_u128(self.endian)?;
        Ok(Uuid::from_u128(number).to_string())
    }
}

// #[cfg(test)]
// mod tests {
// TODO: Tests
//     use super::*;
//     use simple_error::SimpleResult;
//     use generic_number::{Context, Endian};

//     #[test]
//     fn test_uuid() -> SimpleResult<()> {
//         Ok(())
//     }
// }
