use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, bail};

use generic_number::{Context, Integer, IntegerReader, IntegerRenderer};

use crate::{Alignment, Data, H2Type, H2Types, H2TypeTrait};

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

    /// The enum type, as loaded into the [`Data`] structure.
    enum_type: String,
}

impl H2Enum {
    pub fn new_aligned(alignment: Alignment, reader: IntegerReader, fallback_renderer: IntegerRenderer, enum_type: &str, data: &Data) -> SimpleResult<H2Type> {
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

    pub fn new(reader: IntegerReader, fallback_renderer: IntegerRenderer, enum_type: &str, data: &Data) -> SimpleResult<H2Type> {
        Self::new_aligned(Alignment::None, reader, fallback_renderer, enum_type, data)
    }

    fn render(&self, value: Integer, data: &Data) -> SimpleResult<String> {
        match data.lookup_enum(&self.enum_type, &value) {
            Ok(v) => {
                match v.len() {
                    0 => {
                        Ok(format!("{}::Unknown_{}", self.enum_type, self.fallback_renderer.render(value)))
                    },
                    1 => {
                        Ok(format!("{}::{}", self.enum_type, v[0]))
                    },
                    _ => {
                        Ok(format!("{}::(ambiguous)", self.enum_type))
                    }
                }
            },
            Err(e) => bail!("Could not render Enum: {}", e),
        }
    }
}

impl H2TypeTrait for H2Enum {
    fn base_size(&self, _context: Context) -> SimpleResult<usize> {
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
    use generic_number::{Context, Endian, IntegerReader, HexFormatter};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_enum_reader() -> SimpleResult<()> {
        let mut data = Data::new();
        data.load_enums(&[env!("CARGO_MANIFEST_DIR"), "testdata/enums/"].iter().collect::<PathBuf>(), None)?;

        let test_buffer = b"\x01\x64\xff\xff\x01\x00\x00\x00".to_vec();

        let t = H2Enum::new(IntegerReader::U8, HexFormatter::pretty_integer(), "test1", &data)?;
        assert_eq!("test1::TEST1",        t.resolve(Context::new_at(&test_buffer, 0), None, &data)?.display);
        assert_eq!("test1::TEST2",        t.resolve(Context::new_at(&test_buffer, 1), None, &data)?.display);
        assert_eq!("test1::Unknown_0xff", t.resolve(Context::new_at(&test_buffer, 2), None, &data)?.display);

        let t = H2Enum::new(IntegerReader::U32(Endian::Little), HexFormatter::pretty_integer(), "test1", &data)?;
        assert_eq!("test1::TEST1",        t.resolve(Context::new_at(&test_buffer, 4), None, &data)?.display);

        Ok(())
    }
}
