use serde::{Serialize, Deserialize};
use simple_error::SimpleResult;
use std::net::{Ipv6Addr};

use sized_number::{Endian, Context};

use crate::datatype::H2Type;
use crate::datatype::basic::H2BasicType;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IPv6 {
    endian: Endian,
}

impl From<IPv6> for H2Type {
    fn from(o: IPv6) -> H2Type {
        H2Type::from(H2BasicType::IPv6(o))
    }
}

impl From<IPv6> for H2BasicType {
    fn from(o: IPv6) -> H2BasicType {
        H2BasicType::IPv6(o)
    }
}

impl IPv6 {
    pub fn new(endian: Endian) -> Self {
        Self {
            endian: endian,
        }
    }

    pub fn to_string(&self, context: &Context) -> SimpleResult<String> {
        let number = context.read_u128(self.endian)?;

        Ok(Ipv6Addr::from(number).to_string())
    }

    pub fn size(&self) -> u64 {
        16
    }

    pub fn related(&self, _context: &Context) -> SimpleResult<Vec<(u64, H2Type)>> {
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;
    use sized_number::{Context, Endian};

    #[test]
    fn test_ipv4() -> SimpleResult<()> {
        let data = b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".to_vec();
        assert_eq!("::", IPv6::new(Endian::Big).to_string(&Context::new(&data))?);

        let data = b"\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f".to_vec();
        assert_eq!("1:203:405:607:809:a0b:c0d:e0f", IPv6::new(Endian::Big).to_string(&Context::new(&data))?);

        Ok(())
    }

    #[test]
    fn test_ipv4_little() -> SimpleResult<()> {
        let data = b"\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".to_vec();

        assert_eq!("::1", IPv6::new(Endian::Little).to_string(&Context::new(&data))?);

        Ok(())
    }

    #[test]
    fn test_ipv4_error() -> SimpleResult<()> {
        let data = b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".to_vec();
        assert!(IPv6::new(Endian::Big).to_string(&Context::new(&data)).is_err());

        Ok(())
    }

}
