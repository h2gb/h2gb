use serde::{Serialize, Deserialize};

use simple_error::SimpleResult;
use generic_number::{Integer, IntegerReader, IntegerRenderer};

use crate::{Alignment, H2Type, H2Types, H2TypeTrait, Offset};

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
}

impl H2Integer {
    pub fn new_aligned(alignment: Alignment, reader: IntegerReader, renderer: IntegerRenderer) -> H2Type {
        H2Type::new(alignment, H2Types::H2Integer(Self {
            reader: reader,
            renderer: renderer,
        }))
    }

    pub fn new(reader: IntegerReader, renderer: IntegerRenderer) -> H2Type {
        Self::new_aligned(Alignment::None, reader, renderer)
    }
}

impl H2TypeTrait for H2Integer {
    fn is_static(&self) -> bool {
        true
    }

    fn actual_size(&self, _offset: Offset) -> SimpleResult<u64> {
        Ok(self.reader.size() as u64)
    }

    fn to_display(&self, offset: Offset) -> SimpleResult<String> {
        match offset {
            Offset::Static(_) => Ok("Integer".to_string()),
            Offset::Dynamic(context) => {
                Ok(self.renderer.render(self.reader.read(context)?))
            }
        }
    }

    fn can_be_integer(&self) -> bool {
        true
    }

    fn to_integer(&self, offset: Offset) -> SimpleResult<Integer> {
        self.reader.read(offset.get_dynamic()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;
    use generic_number::{Context, Endian, IntegerReader, HexFormatter, DefaultFormatter};

    #[test]
    fn test_u8_hex() -> SimpleResult<()> {
        let data = b"\x00\x7f\x80\xff".to_vec();
        let d_offset = Offset::Dynamic(Context::new(&data));

        let t = H2Integer::new(
            IntegerReader::U8,
            HexFormatter::pretty_integer(),
        );

        assert_eq!(1, t.actual_size(d_offset).unwrap());
        assert_eq!(0, t.related(d_offset)?.len());

        assert_eq!("0x00", t.to_display(d_offset.at(0))?);
        assert_eq!("0x7f", t.to_display(d_offset.at(1))?);
        assert_eq!("0x80", t.to_display(d_offset.at(2))?);
        assert_eq!("0xff", t.to_display(d_offset.at(3))?);

        Ok(())
    }

    #[test]
    fn test_i16_decimal() -> SimpleResult<()> {
        let data = b"\x00\x00\x7f\xff\x80\x00\xff\xff".to_vec();
        let s_offset = Offset::Static(0);
        let d_offset = Offset::Dynamic(Context::new(&data));

        let t = H2Integer::new(
            IntegerReader::I16(Endian::Big),
            DefaultFormatter::new_integer(),
        );

        assert_eq!(2, t.actual_size(s_offset).unwrap());
        assert_eq!(2, t.actual_size(d_offset).unwrap());

        assert_eq!(0, t.related(s_offset)?.len());
        assert_eq!(0, t.related(d_offset)?.len());

        assert_eq!("0",      t.to_display(d_offset.at(0))?);
        assert_eq!("32767",  t.to_display(d_offset.at(2))?);
        assert_eq!("-32768", t.to_display(d_offset.at(4))?);
        assert_eq!("-1",     t.to_display(d_offset.at(6))?);

        Ok(())
    }

    #[test]
    fn test_number_alignment() -> SimpleResult<()> {
        let data = b"\x00\x00\x7f\xff\x80\x00\xff\xff".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        let t = H2Integer::new_aligned(
            Alignment::Loose(8),
            IntegerReader::I16(Endian::Big),
            DefaultFormatter::new_integer(),
        );

        // Starting at 0
        let this_offset = offset.at(0);
        assert_eq!(2, t.actual_size(this_offset)?);
        assert_eq!(0..2, t.actual_range(this_offset)?);

        assert_eq!(8, t.aligned_size(this_offset)?);
        assert_eq!(0..8, t.aligned_range(this_offset)?);

        // Starting at 2
        let this_offset = offset.at(2);
        assert_eq!(2, t.actual_size(this_offset)?);
        assert_eq!(2..4, t.actual_range(this_offset)?);

        assert_eq!(8, t.aligned_size(this_offset)?);
        assert_eq!(2..10, t.aligned_range(this_offset)?);

        // Starting at 7
        let this_offset = offset.at(7);
        assert_eq!(2, t.actual_size(this_offset)?);
        assert_eq!(7..9, t.actual_range(this_offset)?);

        assert_eq!(8, t.aligned_size(this_offset)?);
        assert_eq!(7..15, t.aligned_range(this_offset)?);

        // Make sure the strings are correct
        assert_eq!("0",      t.to_display(offset.at(0))?);
        assert_eq!("32767",  t.to_display(offset.at(2))?);
        assert_eq!("-32768", t.to_display(offset.at(4))?);
        assert_eq!("-1",     t.to_display(offset.at(6))?);

        Ok(())
    }

    #[test]
    fn test_to_i64() -> SimpleResult<()> {
        let data = b"\x00\x00\x7f\xff\x80\x00\xff\xff".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        let t = H2Integer::new(
            IntegerReader::I16(Endian::Big),
            DefaultFormatter::new_integer(),
        );

        assert_eq!(0,      t.to_integer(offset.at(0))?.as_isize()?);
        assert_eq!(32767,  t.to_integer(offset.at(2))?.as_isize()?);
        assert_eq!(-32768, t.to_integer(offset.at(4))?.as_isize()?);
        assert_eq!(-1,     t.to_integer(offset.at(6))?.as_isize()?);

        Ok(())
    }

    #[test]
    fn test_to_u64() -> SimpleResult<()> {
        let data = b"\x00\x00\x7f\xff\x80\x00\xff\xff".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        let t = H2Integer::new(
            IntegerReader::U16(Endian::Big),
            DefaultFormatter::new_integer(),
        );

        assert_eq!(0,      t.to_integer(offset.at(0))?.as_usize()?);
        assert_eq!(32767,  t.to_integer(offset.at(2))?.as_usize()?);
        assert_eq!(32768,  t.to_integer(offset.at(4))?.as_usize()?);
        assert_eq!(65535,  t.to_integer(offset.at(6))?.as_usize()?);

        Ok(())
    }
}
