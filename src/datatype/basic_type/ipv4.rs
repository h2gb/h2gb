use serde::{Serialize, Deserialize};
use simple_error::SimpleResult;
use std::net::Ipv4Addr;

use sized_number::Endian;

use crate::datatype::{H2Type, H2Types, H2TypeTrait, ResolveOffset, AlignValue};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IPv4 {
    endian: Endian,
}

impl IPv4 {
    pub fn new_aligned(alignment: AlignValue, endian: Endian) -> H2Type {
        H2Type::new(alignment, H2Types::IPv4(Self {
            endian: endian
        }))
    }

    pub fn new(endian: Endian) -> H2Type {
        Self::new_aligned(AlignValue::None, endian)
    }
}

impl H2TypeTrait for IPv4 {
    fn is_static(&self) -> bool {
        true
    }

    fn size(&self, _offset: &ResolveOffset) -> SimpleResult<u64> {
        Ok(4)
    }

    fn to_string(&self, offset: &ResolveOffset) -> SimpleResult<String> {
        match offset {
            ResolveOffset::Static(_) => Ok("IPv4 Address".to_string()),
            ResolveOffset::Dynamic(context) => {
                let number = context.read_u32(self.endian)?;

                Ok(Ipv4Addr::from(number).to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;
    use sized_number::{Context, Endian};

    #[test]
    fn test_ipv4() -> SimpleResult<()> {
        let data = b"\x7f\x00\x00\x01".to_vec();
        let d_offset = ResolveOffset::Dynamic(Context::new(&data));

        assert_eq!("127.0.0.1", IPv4::new(Endian::Big).to_string(&d_offset)?);

        Ok(())
    }

    #[test]
    fn test_ipv4_little() -> SimpleResult<()> {
        let data = b"\x01\x02\x02\x04".to_vec();
        let d_offset = ResolveOffset::Dynamic(Context::new(&data));

        assert_eq!("4.2.2.1", IPv4::new(Endian::Little).to_string(&d_offset)?);

        Ok(())
    }

    #[test]
    fn test_ipv4_error() -> SimpleResult<()> {
        let data = b"\x7f\x00\x00".to_vec();
        let d_offset = ResolveOffset::Dynamic(Context::new(&data));

        assert!(IPv4::new(Endian::Big).to_string(&d_offset).is_err());

        Ok(())
    }
}
