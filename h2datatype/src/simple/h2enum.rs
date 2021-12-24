use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, bail};

use generic_number::{Context, IntegerReader, Integer};
use h2data::{enum_exists, from_enum};

use crate::{Alignment, H2Type, H2Types, H2TypeTrait};

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
    reader: IntegerReader,

    enum_type: String,
}

impl H2Enum {
    pub fn new_aligned(alignment: Alignment, reader: IntegerReader, enum_type: &str) -> SimpleResult<H2Type> {
        if !reader.can_be_usize() {
            bail!("Enum types must be compatible with u64 values");
        }

        // Make sure the enum type exists
        if !enum_exists(enum_type) {
            bail!("No such Enum: {}", enum_type);
        }

        Ok(H2Type::new(alignment, H2Types::H2Enum(Self {
            reader: reader,
            enum_type: enum_type.to_string(),
        })))

    }

    pub fn new(reader: IntegerReader, enum_type: &str) -> SimpleResult<H2Type> {
        Self::new_aligned(Alignment::None, reader, enum_type)
    }

    fn render(&self, value: usize) -> SimpleResult<String> {
        let output = match from_enum(&self.enum_type, value)? {
            Some(o) => o.to_string(),
            None => format!("Unknown_0x{:x}", value),
        };

        Ok(format!("{}::{}", self.enum_type, output))
    }
}

impl H2TypeTrait for H2Enum {
    fn is_static(&self) -> bool {
        true
    }

    fn actual_size(&self, _context: Context) -> SimpleResult<u64> {
        Ok(self.reader.size() as u64)
    }

    fn to_display(&self, context: Context) -> SimpleResult<String> {
        let as_u64 = self.reader.read(context)?.as_usize()?;
        self.render(as_u64)
    }

    fn can_be_string(&self) -> bool {
        true
    }

    fn to_string(&self, context: Context) -> SimpleResult<String> {
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
    use generic_number::{Context, IntegerReader};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_enum_reader() -> SimpleResult<()> {
        let test_buffer = b"\x00\x01\x02\x03\x20".to_vec();
        let context = Context::new(&test_buffer);

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
                IntegerReader::U8,
                "TerrariaGameMode",
            )?;

            assert_eq!(
                expected,
                t.to_display(context.at(o))?,
            );
        }

        Ok(())
    }
}
