use serde::{Serialize, Deserialize};

use simple_error::SimpleResult;

use crate::datatype::{Alignment, H2Type, H2Types, H2TypeTrait, Offset};

/// Defines a numerical value.
///
/// This represents any standard numerical value - [`u8`], [`i32`], stuff like
/// that. The way it's defined, read, and displayed heavily leverages the
/// [`crate::generic_number`] module.
///
/// The size a given numeric type is always known in advance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rgb {
}

impl Rgb {
    pub fn new_aligned(alignment: Alignment) -> H2Type {
        H2Type::new(alignment, H2Types::Rgb(Self {}))
    }

    pub fn new() -> H2Type {
        Self::new_aligned(Alignment::None)
    }
}

impl H2TypeTrait for Rgb {
    fn is_static(&self) -> bool {
        true
    }

    fn actual_size(&self, _offset: Offset) -> SimpleResult<u64> {
        Ok(3)
    }

    fn to_display(&self, offset: Offset) -> SimpleResult<String> {
        match offset {
            Offset::Static(_) => Ok("Rgb".to_string()),
            Offset::Dynamic(context) => {
                // Read the 24-bit value
                let byte1 = context.read_u8()? as u32;
                let byte2 = context.read_u8()? as u32;
                let byte3 = context.read_u8()? as u32;
                let value = byte1 << 16 | byte2 << 8 | byte3;
                Ok(format!("#{:06x}", value))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    // use super::*;
    // use simple_error::SimpleResult;
    // use crate::generic_number::{Context, Endian, GenericReader, HexFormatter, DefaultFormatter};

}
