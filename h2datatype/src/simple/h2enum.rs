use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, bail};

use generic_number::{Context, Integer, IntegerReader, IntegerRenderer};

use crate::{Alignment, Data, H2Type, H2TypeTrait};

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
    #[serde(default)]
    fallback_renderer: IntegerRenderer,

    /// The enum type, as loaded into the [`Data`] structure.
    enum_type: String,

    // TODO: This needs more options
    namespace: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    alignment: Option<Alignment>,
}

impl From<H2Enum> for H2Type {
    fn from(t: H2Enum) -> H2Type {
        H2Type::H2Enum(t)
    }
}

impl H2Enum {
    pub fn new_aligned(alignment: Option<Alignment>, reader: impl Into<IntegerReader>, fallback_renderer: impl Into<IntegerRenderer>, namespace: Option<&str>, enum_type: impl AsRef<str>, data: &Data) -> SimpleResult<Self> {
        // Make sure the enum type exists
        if !data.enums.contains(namespace, &enum_type)? {
            bail!("No such Enum: {:?}", enum_type.as_ref());
        }

        Ok(Self {
            reader: reader.into(),
            fallback_renderer: fallback_renderer.into(),
            enum_type: enum_type.as_ref().to_string(),
            namespace: namespace.map(|n| n.to_string()),
            alignment: alignment,
        })

    }

    pub fn new(reader: impl Into<IntegerReader>, fallback_renderer: impl Into<IntegerRenderer>, namespace: Option<&str>, enum_type: impl AsRef<str>, data: &Data) -> SimpleResult<Self> {
        Self::new_aligned(None, reader, fallback_renderer, namespace, enum_type, data)
    }

    fn render(&self, value: impl Into<Integer> + Copy, data: &Data) -> SimpleResult<String> {
        match data.enums.lookup(self.namespace.as_ref().map(|n| n.as_ref()), &self.enum_type, &value.into()) {
            Ok(v) => {
                match v.len() {
                    0 => {
                        Ok(format!("{}::Unknown_{}", self.enum_type, self.fallback_renderer.render_integer(value)))
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

    fn alignment(&self) -> Option<Alignment> {
        self.alignment
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;
    use pretty_assertions::assert_eq;

    use simple_error::SimpleResult;
    use generic_number::{Context, Endian, IntegerReader, HexFormatter};

    use crate::data::{DataTrait, LoadOptions, LoadNamespace, LoadName};

    #[test]
    fn test_enum_reader() -> SimpleResult<()> {
        let mut data = Data::new();
        data.enums.load(&[env!("CARGO_MANIFEST_DIR"), "testdata/enums/"].iter().collect::<PathBuf>(), &LoadOptions::new(LoadNamespace::None, LoadName::Auto))?;

        let test_buffer = b"\x01\x64\xff\xff\x01\x00\x00\x00".to_vec();

        let t = H2Enum::new(IntegerReader::U8, HexFormatter::new_pretty(), None, "test1", &data)?;
        assert_eq!("test1::TEST1",        t.resolve(Context::new_at(&test_buffer, 0), None, &data)?.display);
        assert_eq!("test1::TEST2",        t.resolve(Context::new_at(&test_buffer, 1), None, &data)?.display);
        assert_eq!("test1::Unknown_0xff", t.resolve(Context::new_at(&test_buffer, 2), None, &data)?.display);

        let t = H2Enum::new(IntegerReader::U32(Endian::Little), HexFormatter::new_pretty(), None, "test1", &data)?;
        assert_eq!("test1::TEST1",        t.resolve(Context::new_at(&test_buffer, 4), None, &data)?.display);

        Ok(())
    }
}
