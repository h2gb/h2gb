use serde::{Serialize, Deserialize};

use simple_error::SimpleResult;
use generic_number::{Context, Float, FloatReader, FloatRenderer};

use crate::{Alignment, DataNg, H2Type, H2Types, H2TypeTrait};

/// Defines a numerical value.
///
/// This represents any standard numerical value - [`u8`], [`i32`], stuff like
/// that. The way it's defined, read, and displayed heavily leverages the
/// [`generic_number`] module.
///
/// The size a given numeric type is always known in advance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct H2Float {
    /// The sign, signedness, and endianness of the value.
    reader: FloatReader,

    /// How the value is to be displayed.
    ///
    /// This is created by the various --Formatter modules in GenericNumber.
    /// For example, [`DefaultFormatter::new()`] or [`HexFormatter::pretty()`].
    renderer: FloatRenderer,
}

impl H2Float {
    pub fn new_aligned(alignment: Alignment, reader: FloatReader, renderer: FloatRenderer) -> H2Type {
        H2Type::new(alignment, H2Types::H2Float(Self {
            reader: reader,
            renderer: renderer,
        }))
    }

    pub fn new(reader: FloatReader, renderer: FloatRenderer) -> H2Type {
        Self::new_aligned(Alignment::None, reader, renderer)
    }
}

impl H2TypeTrait for H2Float {
    fn base_size(&self, _context: Context) -> SimpleResult<usize> {
        Ok(self.reader.size())
    }

    fn to_display(&self, context: Context, _data: &DataNg) -> SimpleResult<String> {
        Ok(self.renderer.render(self.to_float(context)?))
    }

    fn can_be_float(&self) -> bool {
        true
    }

    fn to_float(&self, context: Context) -> SimpleResult<Float> {
        self.reader.read(context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use simple_error::SimpleResult;
    use pretty_assertions::assert_eq;

    use generic_number::{Endian, FloatReader, DefaultFormatter};

    #[test]
    fn test_f32() -> SimpleResult<()> {
        // Should be ~3.14
        let data = b"\x40\x48\xf5\xc3".to_vec();

        let t = H2Float::new(FloatReader::F32(Endian::Big), DefaultFormatter::new_float());

        assert_eq!("3.14", t.to_display(Context::new_at(&data, 0), &DataNg::default())?);
        assert_eq!(4,      t.base_size(Context::new_at(&data, 0))?);

        Ok(())
    }

    #[test]
    fn test_f64() -> SimpleResult<()> {
        // Should be ~3.14
        let data = b"\x40\x09\x1e\xb8\x51\xeb\x85\x1f".to_vec();

        assert_eq!("3.14", H2Float::new(FloatReader::F64(Endian::Big), DefaultFormatter::new_float()).to_display(Context::new_at(&data, 0), &DataNg::default())?);
        assert_eq!(8,      H2Float::new(FloatReader::F64(Endian::Big), DefaultFormatter::new_float()).base_size(Context::new_at(&data, 0))?);

        Ok(())
    }
}
