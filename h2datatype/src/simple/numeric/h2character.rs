use serde::{Serialize, Deserialize};

use simple_error::SimpleResult;
use generic_number::{Character, CharacterReader, GenericFormatter, CharacterFormatter};

use crate::{Alignment, H2Type, H2Types, H2TypeTrait, Offset};

/// Defines a numerical value.
///
/// This represents any standard numerical value - [`u8`], [`i32`], stuff like
/// that. The way it's defined, read, and displayed heavily leverages the
/// [`generic_number`] module.
///
/// The size a given numeric type is always known in advance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct H2Character {
    /// The sign, signedness, and endianness of the value.
    reader: CharacterReader,

    /// How the value is to be displayed - [`GenericFormatter::Hex`],
    /// [`GenericFormatter::Octal`], and so on.
    display: GenericFormatter,
}

impl H2Character {
    pub fn new_aligned(alignment: Alignment, reader: CharacterReader, display: GenericFormatter) -> H2Type {
        H2Type::new(alignment, H2Types::H2Character(Self {
            reader: reader,
            display: display,
        }))
    }

    pub fn new(reader: CharacterReader, display: GenericFormatter) -> H2Type {
        Self::new_aligned(Alignment::None, reader, display)
    }

    /// Convenience function to pre-set a definition.
    ///
    /// Reads a character as ASCII, formats the characters in the style of `'a'`.
    pub fn new_ascii() -> H2Type {
        Self::new(
            CharacterReader::ASCII,
            CharacterFormatter::pretty(),
        )
    }

    /// Convenience function to pre-set a definition.
    ///
    /// Reads a character as ASCII, formats as just a letter, like `a`.
    pub fn new_ascii_string() -> H2Type {
        Self::new(
            CharacterReader::ASCII,
            CharacterFormatter::pretty_str(),
        )
    }

    /// Convenience function to pre-set a definition.
    ///
    /// Reads a character as UTF8, formats the characters in the style of `'a'`.
    pub fn new_utf8() -> H2Type {
        Self::new(
            CharacterReader::UTF8,
            CharacterFormatter::pretty(),
        )
    }

    /// Convenience function to pre-set a definition.
    ///
    /// Reads a character as UTF8, formats the characters in the style of `a`.
    pub fn new_utf8_string() -> H2Type {
        Self::new(
            CharacterReader::UTF8,
            CharacterFormatter::pretty_str(),
        )
    }
}

impl H2TypeTrait for H2Character {
    fn is_static(&self) -> bool {
        match self.reader.size() {
            Some(_) => true,
            None    => false
        }
    }

    fn actual_size(&self, offset: Offset) -> SimpleResult<u64> {
        match self.reader.size() {
            Some(v) => Ok(v as u64),
            None    => Ok(self.reader.read(offset.get_dynamic()?)?.size() as u64),
        }
    }

    fn to_display(&self, offset: Offset) -> SimpleResult<String> {
        match offset {
            Offset::Static(_) => Ok("Character".to_string()),
            Offset::Dynamic(context) => {
                Ok(format!("{:?}", self.reader.read(context)?))
            }
        }
    }

    fn can_be_character(&self) -> bool {
        true
    }

    fn to_character(&self, offset: Offset) -> SimpleResult<Character> {
        self.reader.read(offset.get_dynamic()?)
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use simple_error::SimpleResult;
//     use generic_number::{Context, Endian, CharacterReader, HexFormatter, DefaultFormatter};

//     #[test]
//     fn test_u8_hex() -> SimpleResult<()> {
//         let data = b"\x00\x7f\x80\xff".to_vec();
//         let s_offset = Offset::Static(0);
//         let d_offset = Offset::Dynamic(Context::new(&data));

//         let t = H2Character::new(
//             CharacterReader::U8,
//             HexFormatter::pretty(),
//         );

//         assert_eq!(1, t.actual_size(s_offset).unwrap());
//         assert_eq!(1, t.actual_size(d_offset).unwrap());

//         assert_eq!(0, t.related(s_offset)?.len());
//         assert_eq!(0, t.related(d_offset)?.len());

//         assert_eq!("0x00", t.to_display(d_offset.at(0))?);
//         assert_eq!("0x7f", t.to_display(d_offset.at(1))?);
//         assert_eq!("0x80", t.to_display(d_offset.at(2))?);
//         assert_eq!("0xff", t.to_display(d_offset.at(3))?);

//         Ok(())
//     }

//     #[test]
//     fn test_i16_decimal() -> SimpleResult<()> {
//         let data = b"\x00\x00\x7f\xff\x80\x00\xff\xff".to_vec();
//         let s_offset = Offset::Static(0);
//         let d_offset = Offset::Dynamic(Context::new(&data));

//         let t = H2Character::new(
//             CharacterReader::I16(Endian::Big),
//             DefaultFormatter::new(),
//         );

//         assert_eq!(2, t.actual_size(s_offset).unwrap());
//         assert_eq!(2, t.actual_size(d_offset).unwrap());

//         assert_eq!(0, t.related(s_offset)?.len());
//         assert_eq!(0, t.related(d_offset)?.len());

//         assert_eq!("0",      t.to_display(d_offset.at(0))?);
//         assert_eq!("32767",  t.to_display(d_offset.at(2))?);
//         assert_eq!("-32768", t.to_display(d_offset.at(4))?);
//         assert_eq!("-1",     t.to_display(d_offset.at(6))?);

//         Ok(())
//     }

//     #[test]
//     fn test_number_alignment() -> SimpleResult<()> {
//         let data = b"\x00\x00\x7f\xff\x80\x00\xff\xff".to_vec();
//         let offset = Offset::Dynamic(Context::new(&data));

//         let t = H2Character::new_aligned(
//             Alignment::Loose(8),
//             CharacterReader::I16(Endian::Big),
//             DefaultFormatter::new(),
//         );

//         // Starting at 0
//         let this_offset = offset.at(0);
//         assert_eq!(2, t.actual_size(this_offset)?);
//         assert_eq!(0..2, t.actual_range(this_offset)?);

//         assert_eq!(8, t.aligned_size(this_offset)?);
//         assert_eq!(0..8, t.aligned_range(this_offset)?);

//         // Starting at 2
//         let this_offset = offset.at(2);
//         assert_eq!(2, t.actual_size(this_offset)?);
//         assert_eq!(2..4, t.actual_range(this_offset)?);

//         assert_eq!(8, t.aligned_size(this_offset)?);
//         assert_eq!(2..10, t.aligned_range(this_offset)?);

//         // Starting at 7
//         let this_offset = offset.at(7);
//         assert_eq!(2, t.actual_size(this_offset)?);
//         assert_eq!(7..9, t.actual_range(this_offset)?);

//         assert_eq!(8, t.aligned_size(this_offset)?);
//         assert_eq!(7..15, t.aligned_range(this_offset)?);

//         // Make sure the strings are correct
//         assert_eq!("0",      t.to_display(offset.at(0))?);
//         assert_eq!("32767",  t.to_display(offset.at(2))?);
//         assert_eq!("-32768", t.to_display(offset.at(4))?);
//         assert_eq!("-1",     t.to_display(offset.at(6))?);

//         Ok(())
//     }

//     #[test]
//     fn test_to_i64() -> SimpleResult<()> {
//         let data = b"\x00\x00\x7f\xff\x80\x00\xff\xff".to_vec();
//         let offset = Offset::Dynamic(Context::new(&data));

//         let t = H2Character::new(
//             CharacterReader::I16(Endian::Big),
//             DefaultFormatter::new(),
//         );

//         assert_eq!(0,      t.to_number(offset.at(0))?.as_i64()?);
//         assert_eq!(32767,  t.to_number(offset.at(2))?.as_i64()?);
//         assert_eq!(-32768, t.to_number(offset.at(4))?.as_i64()?);
//         assert_eq!(-1,     t.to_number(offset.at(6))?.as_i64()?);

//         Ok(())
//     }

//     #[test]
//     fn test_to_u64() -> SimpleResult<()> {
//         let data = b"\x00\x00\x7f\xff\x80\x00\xff\xff".to_vec();
//         let offset = Offset::Dynamic(Context::new(&data));

//         let t = H2Character::new(
//             CharacterReader::U16(Endian::Big),
//             DefaultFormatter::new(),
//         );

//         assert_eq!(0,     t.to_number(offset.at(0))?.as_u64()?);
//         assert_eq!(32767, t.to_number(offset.at(2))?.as_u64()?);
//         assert_eq!(32768, t.to_number(offset.at(4))?.as_u64()?);
//         assert_eq!(65535, t.to_number(offset.at(6))?.as_u64()?);

//         Ok(())
//     }
// }
