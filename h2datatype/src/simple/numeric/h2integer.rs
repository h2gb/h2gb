use serde::{Serialize, Deserialize};

use simple_error::SimpleResult;
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
pub struct H2Integer {
    /// The sign, signedness, and endianness of the value.
    reader: IntegerReader,

    /// How the value is to be displayed.
    ///
    /// This is created by the various --Formatter modules in GenericNumber.
    /// For example, [`DefaultFormatter::new()`] or [`HexFormatter::pretty()`].
    renderer: IntegerRenderer,
    alignment: Option<Alignment>,
}

impl H2Integer {
    pub fn new_aligned(alignment: Option<Alignment>, reader: impl Into<IntegerReader>, renderer: impl Into<IntegerRenderer>) -> H2Type {
        H2Type::new(H2Types::H2Integer(Self {
            reader: reader.into(),
            renderer: renderer.into(),
            alignment: alignment,
        }))
    }

    pub fn new(reader: impl Into<IntegerReader>, renderer: impl Into<IntegerRenderer>) -> H2Type {
        Self::new_aligned(None, reader, renderer)
    }
}

impl H2TypeTrait for H2Integer {
    fn base_size(&self, _context: Context) -> SimpleResult<usize> {
        Ok(self.reader.size())
    }

    fn to_display(&self, context: Context, _data: &Data) -> SimpleResult<String> {
        Ok(self.renderer.render_integer(self.to_integer(context)?))
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
    use pretty_assertions::assert_eq;
    use simple_error::SimpleResult;
    use generic_number::{Context, Endian, IntegerReader, HexFormatter, DefaultFormatter};

    #[test]
    fn test_u8_hex() -> SimpleResult<()> {
        let data = b"\x00\x7f\x80\xff".to_vec();
        let context = Context::new(&data);

        let t = H2Integer::new(
            IntegerReader::U8,
            HexFormatter::new_pretty(),
        );

        assert_eq!(1, t.base_size(context).unwrap());
        assert_eq!(0, t.related(context)?.len());

        assert_eq!("0x00", t.to_display(context.at(0), &Data::default())?);
        assert_eq!("0x7f", t.to_display(context.at(1), &Data::default())?);
        assert_eq!("0x80", t.to_display(context.at(2), &Data::default())?);
        assert_eq!("0xff", t.to_display(context.at(3), &Data::default())?);

        Ok(())
    }

    #[test]
    fn test_i16_decimal() -> SimpleResult<()> {
        let data = b"\x00\x00\x7f\xff\x80\x00\xff\xff".to_vec();
        let context = Context::new(&data);

        let t = H2Integer::new(
            IntegerReader::I16(Endian::Big),
            DefaultFormatter::new(),
        );

        assert_eq!(2, t.base_size(context).unwrap());
        assert_eq!(0, t.related(context)?.len());

        assert_eq!("0",      t.to_display(context.at(0), &Data::default())?);
        assert_eq!("32767",  t.to_display(context.at(2), &Data::default())?);
        assert_eq!("-32768", t.to_display(context.at(4), &Data::default())?);
        assert_eq!("-1",     t.to_display(context.at(6), &Data::default())?);

        Ok(())
    }

    #[test]
    fn test_number_alignment() -> SimpleResult<()> {
        let data = b"\x00\x00\x7f\xff\x80\x00\xff\xff".to_vec();
        let context = Context::new(&data);

        let t = H2Integer::new_aligned(
            Some(Alignment::Loose(8)),
            IntegerReader::I16(Endian::Big),
            DefaultFormatter::new(),
        );

        // Starting at 0
        let this_context = context.at(0);
        assert_eq!(2, t.base_size(this_context)?);
        assert_eq!(0..2, t.actual_range(this_context)?);

        assert_eq!(8, t.aligned_size(this_context)?);
        assert_eq!(0..8, t.aligned_range(this_context)?);

        // Starting at 2
        let this_context = context.at(2);
        assert_eq!(2, t.base_size(this_context)?);
        assert_eq!(2..4, t.actual_range(this_context)?);

        assert_eq!(8, t.aligned_size(this_context)?);
        assert_eq!(2..10, t.aligned_range(this_context)?);

        // Starting at 7
        let this_context = context.at(7);
        assert_eq!(2, t.base_size(this_context)?);
        assert_eq!(7..9, t.actual_range(this_context)?);

        assert_eq!(8, t.aligned_size(this_context)?);
        assert_eq!(7..15, t.aligned_range(this_context)?);

        // Make sure the strings are correct
        assert_eq!("0",      t.to_display(context.at(0), &Data::default())?);
        assert_eq!("32767",  t.to_display(context.at(2), &Data::default())?);
        assert_eq!("-32768", t.to_display(context.at(4), &Data::default())?);
        assert_eq!("-1",     t.to_display(context.at(6), &Data::default())?);

        Ok(())
    }

    #[test]
    fn test_to_isize() -> SimpleResult<()> {
        let data = b"\x00\x00\x7f\xff\x80\x00\xff\xff".to_vec();
        let context = Context::new(&data);

        let t = H2Integer::new(
            IntegerReader::I16(Endian::Big),
            DefaultFormatter::new(),
        );

        assert_eq!(0isize,      TryInto::<isize>::try_into(t.to_integer(context.at(0))?)?);
        assert_eq!(32767isize,  TryInto::<isize>::try_into(t.to_integer(context.at(2))?)?);
        assert_eq!(-32768isize, TryInto::<isize>::try_into(t.to_integer(context.at(4))?)?);
        assert_eq!(-1isize,     TryInto::<isize>::try_into(t.to_integer(context.at(6))?)?);

        Ok(())
    }

    #[test]
    fn test_to_usize() -> SimpleResult<()> {
        let data = b"\x00\x00\x7f\xff\x80\x00\xff\xff".to_vec();
        let context = Context::new(&data);

        let t = H2Integer::new(
            IntegerReader::U16(Endian::Big),
            DefaultFormatter::new(),
        );

        assert_eq!(0usize,     TryInto::<usize>::try_into(t.to_integer(context.at(0))?)?);
        assert_eq!(32767usize, TryInto::<usize>::try_into(t.to_integer(context.at(2))?)?);
        assert_eq!(32768usize, TryInto::<usize>::try_into(t.to_integer(context.at(4))?)?);
        assert_eq!(65535usize, TryInto::<usize>::try_into(t.to_integer(context.at(6))?)?);

        Ok(())
    }
}
