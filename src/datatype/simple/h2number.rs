use serde::{Serialize, Deserialize};

use simple_error::SimpleResult;
use crate::sized_number::{SizedDefinition, SizedDisplay};

use crate::datatype::{Alignment, H2Type, H2Types, H2TypeTrait, Offset};

/// Defines a numerical value.
///
/// This represents any standard numerical value - [`u8`], [`i32`], stuff like
/// that. The way it's defined, read, and displayed heavily leverages the
/// [`sized_number`] package.
///
/// The size a given numeric type is always known in advance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct H2Number {
    /// The sign, signedness, and endianness of the value.
    definition: SizedDefinition,

    /// How the value is to be displayed - [`SizedDisplay::Hex`],
    /// [`SizedDisplay::Octal`], and so on.
    display: SizedDisplay,
}

impl H2Number {
    pub fn new_aligned(alignment: Alignment, definition: SizedDefinition, display: SizedDisplay) -> H2Type {
        H2Type::new(alignment, H2Types::H2Number(Self {
            definition: definition,
            display: display,
        }))
    }

    pub fn new(definition: SizedDefinition, display: SizedDisplay) -> H2Type {
        Self::new_aligned(Alignment::None, definition, display)
    }
}

impl H2TypeTrait for H2Number {
    fn is_static(&self) -> bool {
        true
    }

    fn actual_size(&self, _offset: Offset) -> SimpleResult<u64> {
        Ok(self.definition.size() as u64)
    }

    fn to_display(&self, offset: Offset) -> SimpleResult<String> {
        match offset {
            Offset::Static(_) => Ok("Number".to_string()),
            Offset::Dynamic(context) => {
                self.display.to_string(self.definition.read(context)?)
            }
        }
    }

    fn can_be_u64(&self) -> bool {
        self.definition.can_be_u64()
    }

    fn to_u64(&self, offset: Offset) -> SimpleResult<u64> {
        self.definition.read(offset.get_dynamic()?)?.as_u64()
    }

    fn can_be_i64(&self) -> bool {
        self.definition.can_be_i64()
    }

    fn to_i64(&self, offset: Offset) -> SimpleResult<i64> {
        self.definition.read(offset.get_dynamic()?)?.as_i64()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;
    use crate::sized_number::{Context, Endian, SizedDefinition, HexOptions, DecimalOptions};

    #[test]
    fn test_u8_hex() -> SimpleResult<()> {
        let data = b"\x00\x7f\x80\xff".to_vec();
        let s_offset = Offset::Static(0);
        let d_offset = Offset::Dynamic(Context::new(&data));

        let t = H2Number::new(
            SizedDefinition::U8,
            HexOptions::pretty(),
        );

        assert_eq!(1, t.actual_size(s_offset).unwrap());
        assert_eq!(1, t.actual_size(d_offset).unwrap());

        assert_eq!(0, t.related(s_offset)?.len());
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

        let t = H2Number::new(
            SizedDefinition::I16(Endian::Big),
            DecimalOptions::new(),
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

        let t = H2Number::new_aligned(
            Alignment::Loose(8),
            SizedDefinition::I16(Endian::Big),
            DecimalOptions::new(),
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

        let t = H2Number::new(
            SizedDefinition::I16(Endian::Big),
            DecimalOptions::new(),
        );

        assert_eq!(0,      t.to_i64(offset.at(0))?);
        assert_eq!(32767,  t.to_i64(offset.at(2))?);
        assert_eq!(-32768, t.to_i64(offset.at(4))?);
        assert_eq!(-1,     t.to_i64(offset.at(6))?);

        Ok(())
    }

    #[test]
    fn test_to_u64() -> SimpleResult<()> {
        let data = b"\x00\x00\x7f\xff\x80\x00\xff\xff".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        let t = H2Number::new(
            SizedDefinition::U16(Endian::Big),
            DecimalOptions::new(),
        );

        assert_eq!(0,     t.to_u64(offset.at(0))?);
        assert_eq!(32767, t.to_u64(offset.at(2))?);
        assert_eq!(32768, t.to_u64(offset.at(4))?);
        assert_eq!(65535, t.to_u64(offset.at(6))?);

        Ok(())
    }
}
