use serde::{Serialize, Deserialize};

use simple_error::{SimpleResult, bail};

use h2data::{bitmask_exists, from_bitmask_str};
use generic_number::{GenericReader, GenericNumber};

use crate::{Alignment, H2Type, H2Types, H2TypeTrait, Offset};

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
    definition: GenericReader,


    bitmask_type: String,
    show_negative: bool,
}

impl H2Bitmask {
    pub fn new_aligned(alignment: Alignment, definition: GenericReader, bitmask_type: &str, show_negative: bool) -> SimpleResult<H2Type> {
        if !definition.can_be_u64() {
            bail!("Bitmask types must be compatible with u64 values");
        }

        // Make sure the bitmask type exists
        if !bitmask_exists(bitmask_type) {
            bail!("No such Bitmask: {}", bitmask_type);
        }

        Ok(H2Type::new(alignment, H2Types::H2Bitmask(Self {
            definition: definition,
            bitmask_type: bitmask_type.to_string(),
            show_negative: show_negative,
        })))

    }

    pub fn new(definition: GenericReader, bitmask_type: &str, show_negative: bool) -> SimpleResult<H2Type> {
        Self::new_aligned(Alignment::None, definition, bitmask_type, show_negative)
    }

    fn render(&self, as_u64: u64) -> SimpleResult<String> {
        let out = from_bitmask_str(&self.bitmask_type, as_u64, self.show_negative)?;
        Ok(out.join(" | "))
    }
}

impl H2TypeTrait for H2Bitmask {
    fn is_static(&self) -> bool {
        true
    }

    fn actual_size(&self, offset: Offset) -> SimpleResult<u64> {
        match self.definition.size() {
            Some(v) => Ok(v as u64),
            None    => Ok(self.definition.read(offset.get_dynamic()?)?.size() as u64),
        }
    }

    fn to_display(&self, offset: Offset) -> SimpleResult<String> {
        match offset {
            Offset::Static(_) => Ok("Bitmask".to_string()),
            Offset::Dynamic(context) => {
                self.render(self.definition.read(context)?.as_u64()?)
            }
        }
    }

    fn can_be_number(&self) -> bool {
        true
    }

    fn to_number(&self, offset: Offset) -> SimpleResult<GenericNumber> {
        self.definition.read(offset.get_dynamic()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;
    use generic_number::{Context, GenericReader, Endian};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_bitmask_reader() -> SimpleResult<()> {
        let test_buffer = b"\x00\x00\x00\x01\x00\x02\x00\x03\x80\x01".to_vec();
        let offset = Offset::Dynamic(Context::new(&test_buffer));

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
                GenericReader::U16(Endian::Big),
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
