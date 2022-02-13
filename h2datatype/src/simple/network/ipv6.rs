use serde::{Serialize, Deserialize};

use simple_error::SimpleResult;
use generic_number::{Context, Endian};
use std::net::Ipv6Addr;

use crate::{Alignment, Data, H2Type, H2Types, H2TypeTrait};

/// Defines an IPv6 address.
///
/// An IPv6 address is always represented as a 4-byte value. It's always
/// displayed in standard colon notation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IPv6 {
    endian: Endian,
}

impl IPv6 {
    pub fn new_aligned(alignment: Alignment, endian: Endian) -> H2Type {
        H2Type::new_inline(alignment, H2Types::IPv6(Self {
            endian: endian
        }))
    }

    pub fn new(endian: Endian) -> H2Type {
        Self::new_aligned(Alignment::None, endian)
    }
}

impl H2TypeTrait for IPv6 {
    fn base_size(&self, _context: Context, _data: &Data) -> SimpleResult<usize> {
        Ok(16)
    }

    fn to_display(&self, context: Context, _data: &Data) -> SimpleResult<String> {
        let number = context.read_u128(self.endian)?;

        Ok(Ipv6Addr::from(number).to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use simple_error::SimpleResult;
    use generic_number::{Context, Endian};

    #[test]
    fn test_ipv6() -> SimpleResult<()> {
        let data = b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".to_vec();
        let context = Context::new(&data);

        assert_eq!("::", IPv6::new(Endian::Big).to_display(context, &Data::default())?);

        let data = b"\x00\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f".to_vec();
        let context = Context::new(&data);

        assert_eq!("1:203:405:607:809:a0b:c0d:e0f", IPv6::new(Endian::Big).to_display(context, &Data::default())?);

        Ok(())
    }

    #[test]
    fn test_ipv6_little() -> SimpleResult<()> {
        let data = b"\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".to_vec();
        let context = Context::new(&data);

        assert_eq!("::1", IPv6::new(Endian::Little).to_display(context, &Data::default())?);

        Ok(())
    }

    #[test]
    fn test_ipv6_error() -> SimpleResult<()> {
        let data = b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".to_vec();
        let context = Context::new(&data);

        assert!(IPv6::new(Endian::Big).to_display(context, &Data::default()).is_err());

        Ok(())
    }
}
