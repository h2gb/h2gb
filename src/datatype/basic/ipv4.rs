use serde::{Serialize, Deserialize};
use simple_error::SimpleResult;
use std::net::{Ipv4Addr};

use sized_number::{Endian, Context};

use crate::datatype::H2StaticType;
use crate::datatype::basic::H2BasicType;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IPv4 {
    endian: Endian,
}

impl From<IPv4> for H2StaticType {
    fn from(o: IPv4) -> H2StaticType {
        H2StaticType::from(H2BasicType::IPv4(o))
    }
}

impl From<IPv4> for H2BasicType {
    fn from(o: IPv4) -> H2BasicType {
        H2BasicType::IPv4(o)
    }
}

impl IPv4 {
    pub fn new(endian: Endian) -> Self {
        Self {
            endian: endian,
        }
    }

    pub fn to_string(&self, context: &Context) -> SimpleResult<String> {
        let number = context.read_u32(self.endian)?;

        Ok(Ipv4Addr::from(number).to_string())
    }

    pub fn size(&self) -> u64 {
        4
    }

    pub fn related(&self, _context: &Context) -> SimpleResult<Vec<(u64, H2StaticType)>> {
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
        let data = b"\x7f\x00\x00\x01".to_vec();

        assert_eq!("127.0.0.1", IPv4::new(Endian::Big).to_string(&Context::new(&data))?);

        Ok(())
    }

    #[test]
    fn test_ipv4_little() -> SimpleResult<()> {
        let data = b"\x01\x02\x02\x04".to_vec();

        assert_eq!("4.2.2.1", IPv4::new(Endian::Little).to_string(&Context::new(&data))?);

        Ok(())
    }

    #[test]
    fn test_ipv4_error() -> SimpleResult<()> {
        let data = b"\x7f\x00\x00".to_vec();

        assert!(IPv4::new(Endian::Big).to_string(&Context::new(&data)).is_err());

        Ok(())
    }
}
