use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, bail};
use std::char;

use sized_number::{Context, Endian};

use crate::datatype::H2StaticType;
use crate::datatype::basic::H2BasicType;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Unicode {
    endian: Endian,
}

impl From<Unicode> for H2StaticType {
    fn from(o: Unicode) -> H2StaticType {
        H2StaticType::from(H2BasicType::Unicode(o))
    }
}

impl From<Unicode> for H2BasicType {
    fn from(o: Unicode) -> H2BasicType {
        H2BasicType::Unicode(o)
    }
}

impl Unicode {
    pub fn new(endian: Endian) -> Self {
        Self {
            endian: endian,
        }
    }

    pub fn to_string(&self, context: &Context) -> SimpleResult<String> {
        let number = context.read_u16(self.endian)?;

        match char::decode_utf16(vec![number]).next() {
            Some(r) => match r {
                Ok(c)  => Ok(c.to_string()),
                Err(e) => bail!("Not valid unicode: 0x{:04x}: {}", number, e),
            }
            None => bail!("Not valid unicode: 0x{:04x}", number),
        }
    }

    pub fn size(&self) -> u64 {
        2
    }

    pub fn related(&self, _context: &Context) -> SimpleResult<Vec<(u64, H2StaticType)>> {
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;
    use sized_number::Context;

    #[test]
    fn test_unicode_big_endian() -> SimpleResult<()> {
        let data = b"\x00\x41\x03\xce\xd8\x34".to_vec();

        assert_eq!("A", Unicode::new(Endian::Big).to_string(&Context::new(&data).at(0))?);
        assert_eq!("ώ", Unicode::new(Endian::Big).to_string(&Context::new(&data).at(2))?);
        assert!(Unicode::new(Endian::Big).to_string(&Context::new(&data).at(4)).is_err());


        Ok(())
    }

    #[test]
    fn test_unicode_little_endian() -> SimpleResult<()> {
        let data = b"\x41\x00\xce\x03\x34\xd8".to_vec();

        assert_eq!("A", Unicode::new(Endian::Little).to_string(&Context::new(&data).at(0))?);
        assert_eq!("ώ", Unicode::new(Endian::Little).to_string(&Context::new(&data).at(2))?);
        assert!(Unicode::new(Endian::Little).to_string(&Context::new(&data).at(4)).is_err());

        Ok(())
    }
}
