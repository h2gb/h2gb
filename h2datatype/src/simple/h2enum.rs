use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, bail};

use generic_number::{Context, Integer, IntegerReader, IntegerRenderer};

use crate::{Alignment, DataNg, H2Type, H2Types, H2TypeTrait};

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

    /// The fallback renderer, for when the enum_type doesn't work
    fallback_renderer: IntegerRenderer,

    /// The enum type, as loaded into the [`DataNg`] structure.
    enum_type: String,
}

impl H2Enum {
    pub fn new_aligned(alignment: Alignment, reader: IntegerReader, fallback_renderer: IntegerRenderer, enum_type: &str, data: &DataNg) -> SimpleResult<H2Type> {
        // Make sure the enum type exists
        if !data.enums.contains_key(enum_type) {
            bail!("No such Enum: {}", enum_type);
        }

        Ok(H2Type::new(alignment, H2Types::H2Enum(Self {
            reader: reader,
            fallback_renderer: fallback_renderer,
            enum_type: enum_type.to_string(),
        })))

    }

    pub fn new(reader: IntegerReader, fallback_renderer: IntegerRenderer, enum_type: &str, data: &DataNg) -> SimpleResult<H2Type> {
        Self::new_aligned(Alignment::None, reader, fallback_renderer, enum_type, data)
    }

    fn render(&self, value: Integer, data: &DataNg) -> SimpleResult<String> {
        match data.enums.get(&self.enum_type) {
            Some(enums) => {
                match enums.get_by_value(&value) {
                    Some(v) => {
                        if v.len() == 0 {
                            Ok(format!("{}::Unknown_{}", self.enum_type, self.fallback_renderer.render(value)))
                        } else {
                            Ok(v.join(" / "))
                        }
                    },
                    None => Ok(format!("{}::Unknown_{}", self.enum_type, self.fallback_renderer.render(value))),
                }
            },
            None => bail!("Unknown enum type: {}", self.enum_type),
        }
    }
}

impl H2TypeTrait for H2Enum {
    fn base_size(&self, _context: Context) -> SimpleResult<usize> {
        Ok(self.reader.size())
    }

    fn to_display(&self, context: Context, data: &DataNg) -> SimpleResult<String> {
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
    use generic_number::{Context, IntegerReader, HexFormatter};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_enum_reader() -> SimpleResult<()> {
        let mut d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        d.push("testdata/terraria/enums/");
        let mut data = DataNg::new();
        data.load_enums(&d, Some("Terraria"))?;

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
                HexFormatter::pretty_integer(),
                "TerrariaGameMode",
                &data,
            )?;

            assert_eq!(
                expected,
                t.to_display(context.at(o), &DataNg::default())?,
            );
        }

        Ok(())
    }
}
