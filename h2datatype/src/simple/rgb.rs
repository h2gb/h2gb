use serde::{Serialize, Deserialize};

use simple_error::SimpleResult;

use generic_number::{Context, Endian, IntegerReader, HexFormatter, IntegerRendererTrait};
use crate::{Alignment, Data, H2Type, H2Types, H2TypeTrait};

/// Defines a numerical value.
///
/// This represents any standard numerical value - [`u8`], [`i32`], stuff like
/// that. The way it's defined, read, and displayed heavily leverages the
/// [`generic_number`] module.
///
/// The size a given numeric type is always known in advance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rgb {
    alignment: Option<Alignment>,
}

impl Rgb {
    pub fn new_aligned(alignment: Option<Alignment>) -> H2Type {
        H2Type::new(H2Types::Rgb(Self {
            alignment: alignment,
        }))
    }

    pub fn new() -> H2Type {
        Self::new_aligned(None)
    }
}

impl H2TypeTrait for Rgb {
    fn base_size(&self, _context: Context) -> SimpleResult<usize> {
        Ok(3)
    }

    fn to_display(&self, context: Context, _data: &Data) -> SimpleResult<String> {
        let reader = IntegerReader::U24(Endian::Big);
        let number = reader.read(context)?;
        let renderer = HexFormatter::new(false, false, true);

        Ok(format!("#{}", renderer.render_integer(number)))
    }

    fn alignment(&self) -> Option<Alignment> {
        self.alignment
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;
    use pretty_assertions::assert_eq;
    use generic_number::Context;

    #[test]
    fn test_rgb() -> SimpleResult<()> {
        let data = b"\x41\x42\x43".to_vec();
        let context = Context::new(&data);

        assert_eq!("#414243", Rgb::new().to_display(context, &Data::default())?);

        Ok(())
    }
}
