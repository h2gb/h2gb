use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, bail};

use generic_number::{Context, Integer, IntegerReader, IntegerRenderer};

use crate::{Alignment, Data, H2Type, H2InnerType, H2TypeTrait};

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

    /// The unknown renderer, for when the value isn't found
    unknown_renderer: Option<IntegerRenderer>,

    /// The bitmask type, as loaded into the [`Data`] structure
    bitmask_type: String,

    /// Show negative bitmask elements?
    show_negative: bool,
}

impl H2Bitmask {
    pub fn new_aligned(alignment: Alignment, reader: IntegerReader, unknown_renderer: Option<IntegerRenderer>, bitmask_type: &str, show_negative: bool, data: &Data) -> SimpleResult<H2Type> {
        // Make sure the bitmask type exists
        if !data.bitmasks.contains_key(bitmask_type) {
            bail!("No such Bitmask: {}", bitmask_type);
        }

        Ok(H2Type::new_inline(alignment, H2InnerType::H2Bitmask(Self {
            reader: reader,
            unknown_renderer: unknown_renderer,
            bitmask_type: bitmask_type.to_string(),
            show_negative: show_negative,
        })))

    }

    pub fn new(reader: IntegerReader, unknown_renderer: Option<IntegerRenderer>, bitmask_type: &str, show_negative: bool, data: &Data) -> SimpleResult<H2Type> {
        Self::new_aligned(Alignment::None, reader, unknown_renderer, bitmask_type, show_negative, data)
    }

    fn render(&self, value: Integer, data: &Data) -> SimpleResult<String> {
        let unknown_renderer = self.unknown_renderer.map(|r| ("Unknown_", r));

        match data.lookup_bitmask(&self.bitmask_type, &value, unknown_renderer, self.show_negative) {
            Ok(v) => {
                if v.len() == 0 {
                    Ok("(n/a)".to_string())
                } else {
                    Ok(v.join(" | "))
                }
            },
            Err(e) => bail!("Could not render Enum: {}", e),
        }
    }
}

impl H2TypeTrait for H2Bitmask {
    fn base_size(&self, _context: Context, _data: &Data) -> SimpleResult<usize> {
        Ok(self.reader.size())
    }

    fn to_display(&self, context: Context, data: &Data) -> SimpleResult<String> {
        self.render(self.reader.read(context)?, data)
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

    use std::path::PathBuf;

    use simple_error::SimpleResult;
    use generic_number::{Context, IntegerReader, Endian, HexFormatter};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_bitmask_reader() -> SimpleResult<()> {
        let mut data = Data::new();
        data.load_bitmasks(&[env!("CARGO_MANIFEST_DIR"), "testdata/terraria/visibility.csv"].iter().collect::<PathBuf>(), Some("Terraria"))?;

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
            (8,      false,         "HIDE_SLOT_HEAD | Unknown_0x00008000"),
        ];

        for (o, show_negative, expected) in tests {
            let t = H2Bitmask::new(
                IntegerReader::U16(Endian::Big),
                Some(HexFormatter::pretty_integer()),
                "Terraria::visibility",
                show_negative,
                &data
            )?;

            assert_eq!(
                expected,
                t.to_display(context.at(o), &data)?,
            );
        }

        Ok(())
    }
}
