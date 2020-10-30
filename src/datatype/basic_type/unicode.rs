use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, bail};
use std::char;

use sized_number::Endian;

use crate::datatype::{H2Type, H2Types, H2TypeTrait, ResolveOffset, AlignValue};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Unicode {
    endian: Endian,
}

impl Unicode {
    pub fn new_aligned(alignment: AlignValue, endian: Endian) -> H2Type {
        H2Type::new(alignment, H2Types::Unicode(Self {
            endian: endian
        }))
    }

    pub fn new(endian: Endian) -> H2Type {
        Self::new_aligned(AlignValue::None, endian)
    }
}

impl H2TypeTrait for Unicode {
    fn is_static(&self) -> bool {
        true
    }

    fn size(&self, _offset: &ResolveOffset) -> SimpleResult<u64> {
        // TODO: Maybe I should do this "right"?
        Ok(2)
    }

    fn to_string(&self, offset: &ResolveOffset) -> SimpleResult<String> {
        match offset {
            ResolveOffset::Static(_) => Ok("2-byte Unicode Character".to_string()),
            ResolveOffset::Dynamic(context) => {
                let number = context.read_u16(self.endian)?;

                match char::decode_utf16(vec![number]).next() {
                    Some(r) => match r {
                        Ok(c)  => Ok(c.to_string()),
                        Err(e) => bail!("Not valid unicode: 0x{:04x}: {}", number, e),
                    }
                    None => bail!("Not valid unicode: 0x{:04x}", number),
                }
            }
        }
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
        let d_offset = ResolveOffset::Dynamic(Context::new(&data));

        assert_eq!("A", Unicode::new(Endian::Big).to_string(&d_offset.at(0))?);
        assert_eq!("ώ", Unicode::new(Endian::Big).to_string(&d_offset.at(2))?);
        assert!(Unicode::new(Endian::Big).to_string(&d_offset.at(4)).is_err());


        Ok(())
    }

    #[test]
    fn test_unicode_little_endian() -> SimpleResult<()> {
        let data = b"\x41\x00\xce\x03\x34\xd8".to_vec();
        let d_offset = ResolveOffset::Dynamic(Context::new(&data));

        assert_eq!("A", Unicode::new(Endian::Little).to_string(&d_offset.at(0))?);
        assert_eq!("ώ", Unicode::new(Endian::Little).to_string(&d_offset.at(2))?);
        assert!(Unicode::new(Endian::Little).to_string(&d_offset.at(4)).is_err());

        Ok(())
    }
}
