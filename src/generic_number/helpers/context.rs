use std::io::{Cursor, Read};

use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use simple_error::{SimpleError, SimpleResult, bail};

use crate::generic_number::helpers::Endian;

/// The maximum size of a UTF8 character
pub const MAX_UTF8_BYTES: usize = 4;

/// The maximum number of 2-byte words in a UTF16 character
pub const MAX_UTF16_WORDS: usize = 2;

/// A structure to hold a data structure and a position while reading the data.
///
/// This is essentially a [`Cursor`], but with some convenience functions to
/// clone and set the position more quickly.
#[derive(Debug, Clone, Copy)]
pub struct Context<'a> {
    v: &'a Vec<u8>,
    position: u64,
    //c: Cursor<&'a Vec<u8>>,
}

impl<'a> Context<'a> {
    /// Create a new [`Context`] at position 0.
    ///
    /// Cannot fail, even if the Vec is empty.
    pub fn new(v: &'a Vec<u8>) -> Self {
        Self {
            v: v,
            position: 0,
            //c: Cursor::new(v)
        }
    }

    /// Create a new [`Context`] at a given position.
    ///
    /// Cannot fail, even if the Vec is empty or if the position is crazy. Those
    /// are checked when using the cursor, not while creating it.
    pub fn new_at(v: &'a Vec<u8>, position: u64) -> Self {
        //let mut c = Cursor::new(v);
        //c.set_position(position);

        Self {
            v: v,
            position: position,
            //c: c
        }
    }

    /// Return a clone of the Cursor.
    ///
    /// This is for internal use only. We clone a lot while reading values, but
    /// this operation is reasonably inexpensive since we don't actually clone
    /// the data - just a reference.
    fn cursor(self) -> Cursor<&'a Vec<u8>> {
        let mut cursor = Cursor::new(self.v);
        cursor.set_position(self.position);

        cursor
    }

    /// Clone the [`Context`] and change the position at the same time.
    ///
    /// I found myself doing a clone-then-set-position operation a bunch, so
    /// this simplifies it.
    pub fn at(self, new_position: u64) -> Self {
        // Since this has the Copy trait, we can copy it super easy
        let mut c = self;
        c.position = new_position;

        c
    }

    /// Get the current position.
    pub fn position(self) -> u64 {
        self.position
    }

    pub fn read_u8(self) -> SimpleResult<u8> {
        match self.cursor().read_u8() {
            Ok(i) => Ok(i),
            Err(e) => Err(SimpleError::from(e)),
        }
    }

    pub fn read_u16(self, endian: Endian) -> SimpleResult<u16> {
        match endian {
            Endian::Big => match self.cursor().read_u16::<BigEndian>() {
                Ok(i) => Ok(i),
                Err(e) => Err(SimpleError::from(e)),
            },
            Endian::Little => match self.cursor().read_u16::<LittleEndian>() {
                Ok(i) => Ok(i),
                Err(e) => Err(SimpleError::from(e)),
            },
        }
    }

    pub fn read_u32(self, endian: Endian) -> SimpleResult<u32> {
        match endian {
            Endian::Big => match self.cursor().read_u32::<BigEndian>() {
                Ok(i) => Ok(i),
                Err(e) => Err(SimpleError::from(e)),
            },
            Endian::Little => match self.cursor().read_u32::<LittleEndian>() {
                Ok(i) => Ok(i),
                Err(e) => Err(SimpleError::from(e)),
            },
        }
    }

    pub fn read_u64(self, endian: Endian) -> SimpleResult<u64> {
        match endian {
            Endian::Big => match self.cursor().read_u64::<BigEndian>() {
                Ok(i) => Ok(i),
                Err(e) => Err(SimpleError::from(e)),
            },
            Endian::Little => match self.cursor().read_u64::<LittleEndian>() {
                Ok(i) => Ok(i),
                Err(e) => Err(SimpleError::from(e)),
            },
        }
    }

    pub fn read_u128(self, endian: Endian) -> SimpleResult<u128> {
        match endian {
            Endian::Big => match self.cursor().read_u128::<BigEndian>() {
                Ok(i) => Ok(i),
                Err(e) => Err(SimpleError::from(e)),
            },
            Endian::Little => match self.cursor().read_u128::<LittleEndian>() {
                Ok(i) => Ok(i),
                Err(e) => Err(SimpleError::from(e)),
            },
        }
    }

    pub fn read_i8(self) -> SimpleResult<i8> {
        match self.cursor().read_i8() {
            Ok(i) => Ok(i),
            Err(e) => Err(SimpleError::from(e)),
        }
    }

    pub fn read_i16(self, endian: Endian) -> SimpleResult<i16> {
        match endian {
            Endian::Big => match self.cursor().read_i16::<BigEndian>() {
                Ok(i) => Ok(i),
                Err(e) => Err(SimpleError::from(e)),
            },
            Endian::Little => match self.cursor().read_i16::<LittleEndian>() {
                Ok(i) => Ok(i),
                Err(e) => Err(SimpleError::from(e)),
            },
        }
    }

    pub fn read_i32(self, endian: Endian) -> SimpleResult<i32> {
        match endian {
            Endian::Big => match self.cursor().read_i32::<BigEndian>() {
                Ok(i) => Ok(i),
                Err(e) => Err(SimpleError::from(e)),
            },
            Endian::Little => match self.cursor().read_i32::<LittleEndian>() {
                Ok(i) => Ok(i),
                Err(e) => Err(SimpleError::from(e)),
            },
        }
    }

    pub fn read_i64(self, endian: Endian) -> SimpleResult<i64> {
        match endian {
            Endian::Big => match self.cursor().read_i64::<BigEndian>() {
                Ok(i) => Ok(i),
                Err(e) => Err(SimpleError::from(e)),
            },
            Endian::Little => match self.cursor().read_i64::<LittleEndian>() {
                Ok(i) => Ok(i),
                Err(e) => Err(SimpleError::from(e)),
            },
        }
    }

    pub fn read_i128(self, endian: Endian) -> SimpleResult<i128> {
        match endian {
            Endian::Big => match self.cursor().read_i128::<BigEndian>() {
                Ok(i) => Ok(i),
                Err(e) => Err(SimpleError::from(e)),
            },
            Endian::Little => match self.cursor().read_i128::<LittleEndian>() {
                Ok(i) => Ok(i),
                Err(e) => Err(SimpleError::from(e)),
            },
        }
    }

    pub fn read_f32(self, endian: Endian) -> SimpleResult<f32> {
        match endian {
            Endian::Big => match self.cursor().read_f32::<BigEndian>() {
                Ok(i) => Ok(i),
                Err(e) => Err(SimpleError::from(e)),
            },
            Endian::Little => match self.cursor().read_f32::<LittleEndian>() {
                Ok(i) => Ok(i),
                Err(e) => Err(SimpleError::from(e)),
            },
        }
    }

    pub fn read_f64(self, endian: Endian) -> SimpleResult<f64> {
        match endian {
            Endian::Big => match self.cursor().read_f64::<BigEndian>() {
                Ok(i) => Ok(i),
                Err(e) => Err(SimpleError::from(e)),
            },
            Endian::Little => match self.cursor().read_f64::<LittleEndian>() {
                Ok(i) => Ok(i),
                Err(e) => Err(SimpleError::from(e)),
            },
        }
    }

    pub fn read_bytes(self, size: usize) -> SimpleResult<Vec<u8>> {
        let mut v: Vec<u8> = Vec::with_capacity(size);

        match self.cursor().take(size as u64).read_to_end(&mut v) {
            Ok(read_size) => {
                if read_size < size {
                    bail!("Read past end of buffer");
                }

                Ok(v)
            }
            Err(e) => Err(SimpleError::from(e)),
        }
    }

    /// Read an 8-bit ASCII character.
    pub fn read_ascii(self) -> SimpleResult<char> {
        self.cursor().read_u8()
            .map(|b| b as char)
            .map_err(|e| SimpleError::from(e))
    }

    /// Read a UTF-8 character, and return the size (in bytes) and character.
    ///
    /// A UTF-8 character is between 1 and [`MAX_UTF8_BYTES`] bytes long. The
    /// size is automatically determined. If the character is invalid or the
    /// read would go off the end of the buffer, an error is returned.
    pub fn read_utf8(self) -> SimpleResult<(usize, char)> {
        let mut c = self.cursor();

        let mut v: Vec<u8> = Vec::new();
        for i in 1..=MAX_UTF8_BYTES {
            v.push(match c.read_u8() {
                Ok(i) => i,
                Err(e) => bail!("Couldn't read UTF-8: {}", e),
            });

            if let Ok(s) = std::str::from_utf8(&v) {
                if let Some(c) = s.chars().next() {
                    return Ok((i, c));
                }
            }
        }

        bail!("Couldn't find a valid UTF-8 character");
    }

    /// Read a UTF-16 character, and return the size (in bytes) and character.
    ///
    /// A UTF-16 character is between 1 and [`MAX_UTF16_WORDS`]. The size (in
    /// bytes) is automatically determined (and returned).
    pub fn read_utf16(self, endian: Endian) -> SimpleResult<(usize, char)> {
        let mut c = self.cursor();
        let mut v: Vec<u16> = Vec::new();
        for i in 1..=MAX_UTF16_WORDS {
            // Read 16 bits in the proper endian
            v.push(match endian {
                Endian::Big => match c.read_u16::<BigEndian>() {
                    Ok(i) => i,
                    Err(e) => bail!("Couldn't read UTF-16: {}", e),
                },
                Endian::Little => match c.read_u16::<LittleEndian>() {
                    Ok(i) => i,
                    Err(e) => bail!("Couldn't read UTF-16: {}", e),
                },
            });

            // Attempt to convert to a string
            if let Ok(s) = String::from_utf16(&v) {
                if let Some(c) = s.chars().next() {
                    return Ok((i*2, c));
                }
            }
        }

        bail!("Couldn't find a valid UTF-16 character");
    }

    /// Read a UTF-32 character, and return the size (in bytes) and character.
    ///
    /// A UTF-32 character is always 4 bytes (32-bits) long, so we don't return
    /// the size.
    pub fn read_utf32(self, endian: Endian) -> SimpleResult<char> {
        match char::from_u32(self.read_u32(endian)?) {
            Some(c) => Ok(c),
            None    => bail!("Couldn't find a valid UTF-32 character"),
        }
    }

    /// Get a [`u8`] slice starting at the current `position`
    pub fn as_slice(self) -> &'a [u8] {
        &self.v[(self.position as usize)..]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;
    use simple_error::SimpleResult;

    #[test]
    fn test_read_bytes() -> SimpleResult<()> {
        // Most functionality on context is implicitly exercised by the tests
        // in lib.rs, but read_bytes is not so test it
        let data = b"ABCD".to_vec();

        // Valid
        assert_eq!(b"ABCD".to_vec(), Context::new(&data).read_bytes(4)?);
        assert_eq!(b"ABC".to_vec(), Context::new(&data).read_bytes(3)?);
        assert_eq!(b"BCD".to_vec(), Context::new_at(&data, 1).read_bytes(3)?);
        assert_eq!(b"".to_vec(), Context::new(&data).read_bytes(0)?);

        // Technically, we can read 0 bytes from way off the buffer.. I think
        // that's okay?
        assert_eq!(b"".to_vec(), Context::new_at(&data, 1000).read_bytes(0)?);


        // Not valid
        assert!(Context::new(&data).read_bytes(5).is_err());
        assert!(Context::new_at(&data, 5).read_bytes(1).is_err());

        Ok(())
    }

    #[test]
    fn test_get_slice() -> SimpleResult<()> {
        let data = b"ABCDEF".to_vec();

        let c = Context::new_at(&data, 2);
        let slice = c.as_slice();
        assert_eq!(b"CDEF".to_vec(), slice);

        Ok(())
    }

    #[test]
    fn test_utf8() -> SimpleResult<()> {
        //             --  --  ------  ----------  ----------  --------------  --------------
        let data = b"\x41\x42\xc3\xb7\xE2\x9D\x84\xE2\x98\xA2\xF0\x9D\x84\x9E\xF0\x9F\x98\x88".to_vec();
        let c = Context::new(&data);

        assert_eq!((1, 'A'),  c.at(0).read_utf8()?);
        assert_eq!((1, 'B'),  c.at(1).read_utf8()?);
        assert_eq!((2, '÷'),  c.at(2).read_utf8()?);
        assert_eq!((3, '❄'),  c.at(4).read_utf8()?);
        assert_eq!((3, '☢'),  c.at(7).read_utf8()?);
        assert_eq!((4, '𝄞'),  c.at(10).read_utf8()?);
        assert_eq!((4, '😈'), c.at(14).read_utf8()?);

        assert!(c.at(3).read_utf8().is_err());
        assert!(c.at(5).read_utf8().is_err());
        assert!(c.at(6).read_utf8().is_err());
        assert!(c.at(8).read_utf8().is_err());
        assert!(c.at(9).read_utf8().is_err());
        assert!(c.at(3).read_utf8().is_err());
        assert!(c.at(11).read_utf8().is_err());
        assert!(c.at(12).read_utf8().is_err());
        assert!(c.at(13).read_utf8().is_err());
        assert!(c.at(15).read_utf8().is_err());
        assert!(c.at(16).read_utf8().is_err());
        assert!(c.at(17).read_utf8().is_err());
        assert!(c.at(18).read_utf8().is_err());
        assert!(c.at(19).read_utf8().is_err());
        assert!(c.at(20).read_utf8().is_err());

        Ok(())
    }

    #[test]
    fn test_utf16_big_endian() -> SimpleResult<()> {
        //           ------------ single -----------  ----------- double ------------
        let data = b"\x00\x41\x00\x42\x27\x44\x26\x22\xD8\x34\xDD\x1E\xD8\x3D\xDE\x08".to_vec();
        let c = Context::new(&data);

        // Single
        assert_eq!((2, 'A'),  c.at(0).read_utf16(Endian::Big)?);
        assert_eq!((2, 'B'),  c.at(2).read_utf16(Endian::Big)?);
        assert_eq!((2, '❄'),  c.at(4).read_utf16(Endian::Big)?);
        assert_eq!((2, '☢'),  c.at(6).read_utf16(Endian::Big)?);

        // Double
        assert_eq!((4, '𝄞'),  c.at(8).read_utf16(Endian::Big)?);
        assert_eq!((4, '😈'), c.at(12).read_utf16(Endian::Big)?);

        Ok(())
    }

    #[test]
    fn test_utf16_little_endian() -> SimpleResult<()> {
        //           ------------ single -----------  ----------- double ------------
        let data = b"\x41\x00\x42\x00\x44\x27\x22\x26\x34\xd8\x1e\xdd\x3d\xd8\x08\xde".to_vec();
        let c = Context::new(&data);

        // Single
        assert_eq!((2, 'A'),  c.at(0).read_utf16(Endian::Little)?);
        assert_eq!((2, 'B'),  c.at(2).read_utf16(Endian::Little)?);
        assert_eq!((2, '❄'),  c.at(4).read_utf16(Endian::Little)?);
        assert_eq!((2, '☢'),  c.at(6).read_utf16(Endian::Little)?);

        // Double
        assert_eq!((4, '𝄞'),  c.at(8).read_utf16(Endian::Little)?);
        assert_eq!((4, '😈'), c.at(12).read_utf16(Endian::Little)?);

        Ok(())
    }

    #[test]
    fn test_utf32_big_endian() -> SimpleResult<()> {
        let data = b"\x00\x00\x00\x41\x00\x00\x00\x42\x00\x00\x27\x44\x00\x00\x26\x22\x00\x01\xD1\x1E\x00\x01\xF6\x08".to_vec();
        let c = Context::new(&data);

        assert_eq!('A',  c.at(0).read_utf32(Endian::Big)?);
        assert_eq!('B',  c.at(4).read_utf32(Endian::Big)?);
        assert_eq!('❄',  c.at(8).read_utf32(Endian::Big)?);
        assert_eq!('☢',  c.at(12).read_utf32(Endian::Big)?);
        assert_eq!('𝄞',  c.at(16).read_utf32(Endian::Big)?);
        assert_eq!('😈', c.at(20).read_utf32(Endian::Big)?);

        Ok(())
    }

    #[test]
    fn test_utf32_little_endian() -> SimpleResult<()> {
        let data = b"\x41\x00\x00\x00\x42\x00\x00\x00\x44\x27\x00\x00\x22\x26\x00\x00\x1E\xd1\x01\x00\x08\xf6\x01\x00".to_vec();
        let c = Context::new(&data);

        assert_eq!('A',  c.at(0).read_utf32(Endian::Little)?);
        assert_eq!('B',  c.at(4).read_utf32(Endian::Little)?);
        assert_eq!('❄',  c.at(8).read_utf32(Endian::Little)?);
        assert_eq!('☢',  c.at(12).read_utf32(Endian::Little)?);
        assert_eq!('𝄞',  c.at(16).read_utf32(Endian::Little)?);
        assert_eq!('😈', c.at(20).read_utf32(Endian::Little)?);

        Ok(())
    }
}

