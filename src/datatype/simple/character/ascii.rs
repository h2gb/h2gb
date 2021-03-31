#[cfg(feature = "serialize")]
use serde::{Serialize, Deserialize};

use simple_error::{bail, SimpleResult};
use sized_number::Context;

use crate::datatype::{Alignment, H2Type, H2Types, H2TypeTrait, Offset};
use crate::datatype::simple::character::common;

/// Configuration options for ASCII characters.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub enum StrictASCII {
    /// Throw an error and fail if the string isn't strict ASCII.
    Strict,

    /// Replace ASCII bytes with the unicode "unknown" character.
    Permissive,
}

/// Defines an ASCII value.
///
/// ASCII characters are 8-bit characters between `0x00` and `0x7F`. If a
/// byte is out of range, [`StrictASCII`] determines whether it'll be accepted
/// or not.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(Serialize, Deserialize))]
pub struct ASCII {
    strict: StrictASCII,
}

impl ASCII {
    pub fn new_aligned(alignment: Alignment, strict: StrictASCII) -> H2Type {
        H2Type::new(alignment, H2Types::ASCII(Self {
            strict: strict
        }))
    }

    pub fn new(strict: StrictASCII) -> H2Type {
        Self::new_aligned(Alignment::None, strict)
    }

    fn read_ascii_strict(context: Context) -> SimpleResult<char> {
        let number = context.read_u8()?;

        match number < 0x7F {
            true  => Ok(number as char),
            false => bail!("Invalid ASCII character: {:#x}", number),
        }
    }

    fn read_ascii_permissive(context: Context) -> SimpleResult<char> {
        let number = context.read_u8()?;

        match number < 0x7F {
            true  => Ok(number as char),
            false => Ok('�'),
        }
    }
}

impl H2TypeTrait for ASCII {
    fn is_static(&self) -> bool {
        true
    }

    fn actual_size(&self, _offset: Offset) -> SimpleResult<u64> {
        Ok(1)
    }

    fn to_display(&self, offset: Offset) -> SimpleResult<String> {
        Ok(common::display_character(self.to_char(offset)?))
    }

    fn can_be_char(&self) -> bool {
        true
    }

    fn to_char(&self, offset: Offset) -> SimpleResult<char> {
        let context = offset.get_dynamic()?;

        match self.strict {
            StrictASCII::Strict     => Ok(Self::read_ascii_strict(context)?),
            StrictASCII::Permissive => Ok(Self::read_ascii_permissive(context)?),
        }
    }

    fn can_be_u64(&self) -> bool {
        true
    }

    fn to_u64(&self, offset: Offset) -> SimpleResult<u64> {
        Ok(self.to_char(offset)? as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;
    use sized_number::Context;

    #[test]
    fn test_ascii_type_unaligned() -> SimpleResult<()> {
        let c = ASCII::new(StrictASCII::Permissive);

        assert_eq!(true, c.is_static());

        assert_eq!(1, c.actual_size(Offset::Static(0))?);
        assert_eq!(0..1, c.actual_range(Offset::Static(0))?);

        assert_eq!(1, c.aligned_size(Offset::Static(0))?);
        assert_eq!(0..1, c.aligned_range(Offset::Static(0))?);

        assert_eq!(0, c.children(Offset::Static(0))?.len());
        assert_eq!(0, c.related(Offset::Static(0))?.len());

        Ok(())
    }

    #[test]
    fn test_ascii_resolve() -> SimpleResult<()> {
        let data = b"\x41".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        let r = ASCII::new(StrictASCII::Permissive).resolve(offset, None)?;
        assert_eq!(1, r.actual_size());
        assert_eq!(0..1, r.actual_range);

        assert_eq!(1, r.aligned_size());
        assert_eq!(0..1, r.aligned_range);

        assert_eq!(0, r.children.len());
        assert_eq!(0, r.related.len());
        assert_eq!("'A'", r.display);

        Ok(())
    }

    #[test]
    fn test_ascii_type_aligned() -> SimpleResult<()> {
        let c = ASCII::new_aligned(Alignment::Loose(4), StrictASCII::Permissive);

        assert_eq!(true, c.is_static());

        assert_eq!(1, c.actual_size(Offset::Static(0))?);
        assert_eq!(0..1, c.actual_range(Offset::Static(0))?);

        assert_eq!(4, c.aligned_size(Offset::Static(0))?);
        assert_eq!(0..4, c.aligned_range(Offset::Static(0))?);

        assert_eq!(0, c.children(Offset::Static(0))?.len());
        assert_eq!(0, c.related(Offset::Static(0))?.len());

        Ok(())
    }

    #[test]
    fn test_ascii_resolve_aligned() -> SimpleResult<()> {
        let data = b"\x41".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        let r = ASCII::new_aligned(Alignment::Loose(4), StrictASCII::Permissive).resolve(offset, None)?;
        assert_eq!(1, r.actual_size());
        assert_eq!(0..1, r.actual_range);

        assert_eq!(4, r.aligned_size());
        assert_eq!(0..4, r.aligned_range);

        assert_eq!(0, r.children.len());
        assert_eq!(0, r.related.len());
        assert_eq!("'A'", r.display);

        Ok(())
    }

    #[test]
    fn test_ascii_to_display_permissive() -> SimpleResult<()> {
        let data = b"\x00\x06\x20\x41\x42\x7e\x7f\x80\xff".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));
        let t = ASCII::new(StrictASCII::Permissive);

        assert_eq!("'\\0'",   t.to_display(offset.at(0))?);
        assert_eq!("'\\x06'", t.to_display(offset.at(1))?);
        assert_eq!("' '", t.to_display(offset.at(2))?);
        assert_eq!("'A'", t.to_display(offset.at(3))?);
        assert_eq!("'B'", t.to_display(offset.at(4))?);
        assert_eq!("'~'", t.to_display(offset.at(5))?);
        assert_eq!("'�'", t.to_display(offset.at(6))?);
        assert_eq!("'�'", t.to_display(offset.at(7))?);
        assert_eq!("'�'", t.to_display(offset.at(8))?);

        Ok(())
    }

    #[test]
    fn test_ascii_to_display_strict() -> SimpleResult<()> {
        let data = b"\x00\x06\x20\x41\x42\x7e\x7f\x80\xff".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));
        let t = ASCII::new(StrictASCII::Strict);

        assert!(t.to_display(offset.at(6)).is_err());
        assert!(t.to_display(offset.at(7)).is_err());
        assert!(t.to_display(offset.at(8)).is_err());

        assert_eq!("'\\0'",   t.to_display(offset.at(0))?);
        assert_eq!("'\\x06'", t.to_display(offset.at(1))?);
        assert_eq!("' '",     t.to_display(offset.at(2))?);
        assert_eq!("'A'",     t.to_display(offset.at(3))?);
        assert_eq!("'B'",     t.to_display(offset.at(4))?);
        assert_eq!("'~'",     t.to_display(offset.at(5))?);

        Ok(())
    }

    #[test]
    fn test_null() -> SimpleResult<()> {
        let data = b"\x41\x00\x41".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        assert_eq!('A',  ASCII::new(StrictASCII::Permissive).to_char(offset.at(0))?);
        assert_eq!('\0', ASCII::new(StrictASCII::Permissive).to_char(offset.at(1))?);
        assert_eq!('A',  ASCII::new(StrictASCII::Permissive).to_char(offset.at(2))?);

        assert_eq!('A',  ASCII::new(StrictASCII::Strict).to_char(offset.at(0))?);
        assert_eq!('\0', ASCII::new(StrictASCII::Strict).to_char(offset.at(1))?);
        assert_eq!('A',  ASCII::new(StrictASCII::Strict).to_char(offset.at(2))?);

        Ok(())
    }
}
