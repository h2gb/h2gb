use serde::{Serialize, Deserialize};

use simple_error::{SimpleResult, bail};

use crate::data::{enum_exists, from_enum};
use generic_number::{GenericReader, GenericNumber};
use crate::datatype::{Alignment, H2Type, H2Types, H2TypeTrait, Offset};

/// Defines a numerical value.
///
/// This represents any standard numerical value - [`u8`], [`i32`], stuff like
/// that. The way it's defined, read, and displayed heavily leverages the
/// [`generic_number`] module.
///
/// The size a given numeric type is always known in advance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct H2Enum {
    /// The sign, signedness, and endianness of the value.
    definition: GenericReader,

    enum_type: String,
}

impl H2Enum {
    pub fn new_aligned(alignment: Alignment, definition: GenericReader, enum_type: &str) -> SimpleResult<H2Type> {
        if !definition.can_be_u64() {
            bail!("Enum types must be compatible with u64 values");
        }

        // Make sure the enum type exists
        if !enum_exists(enum_type) {
            bail!("No such Enum: {}", enum_type);
        }

        Ok(H2Type::new(alignment, H2Types::H2Enum(Self {
            definition: definition,
            enum_type: enum_type.to_string(),
        })))

    }

    pub fn new(definition: GenericReader, enum_type: &str) -> SimpleResult<H2Type> {
        Self::new_aligned(Alignment::None, definition, enum_type)
    }

    fn render(&self, as_u64: u64) -> SimpleResult<String> {
        let output = match from_enum(&self.enum_type, as_u64)? {
            Some(o) => o.to_string(),
            None => format!("Unknown_0x{:x}", as_u64),
        };

        Ok(format!("{}::{}", self.enum_type, output))
    }
}

impl H2TypeTrait for H2Enum {
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
            Offset::Static(_) => Ok("Enum".to_string()),
            Offset::Dynamic(context) => {
                let as_u64 = self.definition.read(context)?.as_u64()?;
                self.render(as_u64)
            }
        }
    }

    fn can_be_string(&self) -> bool {
        true
    }

    fn to_string(&self, offset: Offset) -> SimpleResult<String> {
        self.render(self.definition.read(offset.get_dynamic()?)?.as_u64()?)
    }

    fn can_be_number(&self) -> bool {
        true
    }

    fn can_be_u64(&self) -> bool {
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
    use generic_number::{Context, GenericReader};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_enum_reader() -> SimpleResult<()> {
        let test_buffer = b"\x00\x01\x02\x03\x20".to_vec();
        let offset = Offset::Dynamic(Context::new(&test_buffer));

        let tests = vec![
          // offset  expected
            (0,      "TerrariaGameMode::Classic"),
            (1,      "TerrariaGameMode::MediumCore"),
            (2,      "TerrariaGameMode::HardCore"),
            (3,      "TerrariaGameMode::JourneyMode"),
            (4,      "TerrariaGameMode::Unknown_0x20"),
        ];

        for (o, expected) in tests {
            let t = H2Enum::new(
                GenericReader::U8,
                "TerrariaGameMode",
            )?;

            assert_eq!(
                expected,
                t.to_display(offset.at(o))?,
            );
        }

        Ok(())
    }
}
