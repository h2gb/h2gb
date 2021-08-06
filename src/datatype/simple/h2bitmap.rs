use serde::{Serialize, Deserialize};

use simple_error::{SimpleResult, bail};

use crate::data::{BITMAPS, from_bitmap};
use crate::generic_number::GenericReader;
use crate::datatype::{Alignment, H2Type, H2Types, H2TypeTrait, Offset};

/// Defines a numerical value.
///
/// This represents any standard numerical value - [`u8`], [`i32`], stuff like
/// that. The way it's defined, read, and displayed heavily leverages the
/// [`crate::generic_number`] module.
///
/// The size a given numeric type is always known in advance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct H2Bitmap {
    /// The sign, signedness, and endianness of the value.
    definition: GenericReader,


    bitmap_type: String,
    show_negative: bool,
}

impl H2Bitmap {
    pub fn new_aligned(alignment: Alignment, definition: GenericReader, bitmap_type: &str, show_negative: bool) -> SimpleResult<H2Type> {
        if !definition.can_be_u64() {
            bail!("Bitmap types must be compatible with u64 values");
        }

        // Make sure the bitmap type exists
        if !BITMAPS.contains_key(bitmap_type) {
            bail!("No such Bitmap: {}", bitmap_type);
        }

        Ok(H2Type::new(alignment, H2Types::H2Bitmap(Self {
            definition: definition,
            bitmap_type: bitmap_type.to_string(),
            show_negative: show_negative,
        })))

    }

    pub fn new(definition: GenericReader, bitmap_type: &str, show_negative: bool) -> SimpleResult<H2Type> {
        Self::new_aligned(Alignment::None, definition, bitmap_type, show_negative)
    }

    fn render(&self, as_u64: u64) -> SimpleResult<String> {
        let mut out: Vec<String> = Vec::new();

        // TODO: Add in the remainder
        let (output, _remainder) = from_bitmap(&self.bitmap_type, as_u64)?;

        output.into_iter().for_each(|(_value, name, present)| {
            match (present, self.show_negative) {
                // The flag is present
                (true, _) => out.push(name),

                // The flag is not present, but we want to see it
                (false, true) => out.push(format!("~{}", name)),

                // The flag is not present, and we don't want to see it
                (false, false) => (),
            }
        });

        if out.len() == 0 {
            Ok("(n/a)".to_string())
        } else {
            Ok(out.join(" | "))
        }
    }
}

impl H2TypeTrait for H2Bitmap {
    fn is_static(&self) -> bool {
        true
    }

    fn actual_size(&self, offset: Offset) -> SimpleResult<u64> {
        // TODO: I'm not sure if using the static size here is really something I should care about, as opposed to just reading + checking
        match self.definition.size() {
            Some(v) => Ok(v as u64),
            None    => Ok(self.definition.read(offset.get_dynamic()?)?.size() as u64),
        }
    }

    fn to_display(&self, offset: Offset) -> SimpleResult<String> {
        match offset {
            Offset::Static(_) => Ok("Bitmap".to_string()),
            Offset::Dynamic(context) => {
                self.render(self.definition.read(context)?.as_u64()?)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;
    use crate::generic_number::{Context, GenericReader};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_bitmap_reader() -> SimpleResult<()> {
        let test_buffer = b"\x00\x01\x02\x03".to_vec();
        let offset = Offset::Dynamic(Context::new(&test_buffer));

        let tests = vec![
          // offset  show_negative  expected
            (0,      false,         "(n/a)"),
            (1,      false,         "HIDE_SLOT_HEAD"),
            (2,      false,         "HIDE_SLOT_BODY"),
            (3,      false,         "HIDE_SLOT_HEAD | HIDE_SLOT_BODY"),

            (1,      true,          "HIDE_SLOT_HEAD | ~HIDE_SLOT_BODY | ~HIDE_SLOT_LEGS | ~HIDE_SLOT_ACCESSORY1 | ~HIDE_SLOT_ACCESSORY2 | ~HIDE_SLOT_ACCESSORY3 | ~HIDE_SLOT_ACCESSORY4 | ~HIDE_SLOT_ACCESSORY5 | ~HIDE_SLOT_ACCESSORY6 | ~HIDE_SLOT_ACCESSORY8"),
        ];

        for (o, show_negative, expected) in tests {
            let t = H2Bitmap::new(
                GenericReader::U8,
                "TerrariaVisibility",
                show_negative,
            )?;

            assert_eq!(
                expected,
                t.to_display(offset.at(o))?,
            );
        }

        Ok(())
    }
}
