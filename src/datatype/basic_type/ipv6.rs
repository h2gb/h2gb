use serde::{Serialize, Deserialize};
use simple_error::SimpleResult;
use std::net::{Ipv6Addr};

use sized_number::Endian;

use crate::datatype::{H2Type, H2Types, H2TypeTrait, ResolveOffset, AlignValue};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IPv6 {
    endian: Endian,
}

impl IPv6 {
    pub fn new_aligned(alignment: AlignValue, endian: Endian) -> H2Type {
        H2Type::new(alignment, H2Types::IPv6(Self {
            endian: endian
        }))
    }

    pub fn new(endian: Endian) -> H2Type {
        Self::new_aligned(AlignValue::None, endian)
    }
}

impl H2TypeTrait for IPv6 {
    fn is_static(&self) -> bool {
        true
    }

    fn size(&self, _offset: &ResolveOffset) -> SimpleResult<u64> {
        Ok(16)
    }

    fn to_string(&self, offset: &ResolveOffset) -> SimpleResult<String> {
        match offset {
            ResolveOffset::Static(_) => Ok("IPv6 Address".to_string()),
            ResolveOffset::Dynamic(context) => {
                let number = context.read_u128(self.endian)?;

                Ok(Ipv6Addr::from(number).to_string())
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
    fn test_ipv6() -> SimpleResult<()> {
        let data = b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".to_vec();
        let d_offset = ResolveOffset::Dynamic(Context::new(&data));

        assert_eq!("::", IPv6::new(Endian::Big).to_string(&d_offset)?);

        let data = b"\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f".to_vec();
        let d_offset = ResolveOffset::Dynamic(Context::new(&data));

        assert_eq!("1:203:405:607:809:a0b:c0d:e0f", IPv6::new(Endian::Big).to_string(&d_offset)?);

        Ok(())
    }

    #[test]
    fn test_ipv6_little() -> SimpleResult<()> {
        let data = b"\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".to_vec();
        let d_offset = ResolveOffset::Dynamic(Context::new(&data));

        assert_eq!("::1", IPv6::new(Endian::Little).to_string(&d_offset)?);

        Ok(())
    }

    #[test]
    fn test_ipv6_error() -> SimpleResult<()> {
        let data = b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".to_vec();
        let d_offset = ResolveOffset::Dynamic(Context::new(&data));

        assert!(IPv6::new(Endian::Big).to_string(&d_offset).is_err());

        Ok(())
    }
}
