use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, bail};

use h2data::{bitmask_exists, from_bitmask_str};
use generic_number::{Context, IntegerReader, Integer};

use crate::{Alignment, DataNg, H2Type, H2Types, H2TypeTrait};

/// Defines a numerical value.
///
/// This represents any standard numerical value - [`u8`], [`i32`], stuff like
/// that. The way it's defined, read, and displayed heavily leverages the
/// [`generic_number`] module.
///
/// The size a given numeric type is always known in advance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct H2Bitmask {
    /// The sign, signedness, and endianness of the value.
    reader: IntegerReader,

    bitmask_type: String,
    show_negative: bool,
}

impl H2Bitmask {
    pub fn new_aligned(alignment: Alignment, reader: IntegerReader, bitmask_type: &str, show_negative: bool) -> SimpleResult<H2Type> {
        if !reader.can_be_usize() {
            bail!("Bitmask types must be compatible with usize values");
        }

        // Make sure the bitmask type exists
        if !bitmask_exists(bitmask_type) {
            bail!("No such Bitmask: {}", bitmask_type);
        }

        Ok(H2Type::new(alignment, H2Types::H2Bitmask(Self {
            reader: reader,
            bitmask_type: bitmask_type.to_string(),
            show_negative: show_negative,
        })))

    }

    pub fn new(reader: IntegerReader, bitmask_type: &str, show_negative: bool) -> SimpleResult<H2Type> {
        Self::new_aligned(Alignment::None, reader, bitmask_type, show_negative)
    }

    fn render(&self, number: usize) -> SimpleResult<String> {
        let out = from_bitmask_str(&self.bitmask_type, number, self.show_negative)?;
        Ok(out.join(" | "))
    }
}

impl H2TypeTrait for H2Bitmask {
    fn base_size(&self, _context: Context) -> SimpleResult<usize> {
        Ok(self.reader.size())
    }

    fn to_display(&self, context: Context, _data: &DataNg) -> SimpleResult<String> {
        self.render(self.reader.read(context)?.as_usize()?)
    }

    fn can_be_integer(&self) -> bool {
        true
    }

    fn to_integer(&self, context: Context) -> SimpleResult<Integer> {
        self.reader.read(context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;
    use generic_number::{Context, IntegerReader, Endian};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_bitmask_reader() -> SimpleResult<()> {
        let test_buffer = b"\x00\x00\x00\x01\x00\x02\x00\x03\x80\x01".to_vec();
        let context = Context::new(&test_buffer);

        let tests = vec![
          // offset  show_negative  expected
            (0,      false,         "(n/a)"),
            (2,      false,         "HIDE_SLOT_HEAD"),
            (4,      false,         "HIDE_SLOT_BODY"),
            (6,      false,         "HIDE_SLOT_HEAD | HIDE_SLOT_BODY"),

            // With negatives
            (2,      true,          "HIDE_SLOT_HEAD | ~HIDE_SLOT_BODY | ~HIDE_SLOT_LEGS | ~HIDE_SLOT_ACCESSORY1 | ~HIDE_SLOT_ACCESSORY2 | ~HIDE_SLOT_ACCESSORY3 | ~HIDE_SLOT_ACCESSORY4 | ~HIDE_SLOT_ACCESSORY5 | ~HIDE_SLOT_ACCESSORY6 | ~HIDE_SLOT_ACCESSORY8"),

            // With an unknown value
            (8,      false,         "HIDE_SLOT_HEAD | Unknown_0x8000"),
        ];

        for (o, show_negative, expected) in tests {
            let t = H2Bitmask::new(
                IntegerReader::U16(Endian::Big),
                "TerrariaVisibility",
                show_negative,
            )?;

            assert_eq!(
                expected,
                t.to_display(context.at(o), &DataNg::default())?,
            );
        }

        Ok(())
    }
}
