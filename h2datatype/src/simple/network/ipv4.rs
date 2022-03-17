use serde::{Serialize, Deserialize};

use simple_error::SimpleResult;
use generic_number::{Context, Endian};
use std::net::Ipv4Addr;

use crate::{Alignment, Data, H2Type, H2TypeTrait};

/// Defines an IPv4 address.
///
/// An IPv4 address is always represented as a 4-byte value. It's always
/// displayed in dotted-decimal notation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IPv4 {
    endian: Endian,
    alignment: Option<Alignment>,
}

impl From<IPv4> for H2Type {
    fn from(t: IPv4) -> H2Type {
        H2Type::IPv4(t)
    }
}

impl IPv4 {
    pub fn new_aligned(alignment: Option<Alignment>, endian: Endian) -> Self {
        Self {
            endian: endian,
            alignment: alignment,
        }
    }

    pub fn new(endian: Endian) -> Self {
        Self::new_aligned(None, endian)
    }
}

impl H2TypeTrait for IPv4 {
    fn base_size(&self, _context: Context) -> SimpleResult<usize> {
        Ok(4)
    }

    fn to_display(&self, context: Context, _data: &Data) -> SimpleResult<String> {
        let number = context.read_u32(self.endian)?;

        Ok(Ipv4Addr::from(number).to_string())
    }

    fn alignment(&self) -> Option<Alignment> {
        self.alignment
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use simple_error::SimpleResult;
    use generic_number::{Context, Endian};

    #[test]
    fn test_ipv4() -> SimpleResult<()> {
        let data = b"\x7f\x00\x00\x01".to_vec();
        let context = Context::new(&data);

        assert_eq!("127.0.0.1", IPv4::new(Endian::Big).to_display(context, &Data::default())?);

        Ok(())
    }

    #[test]
    fn test_ipv4_little() -> SimpleResult<()> {
        let data = b"\x01\x02\x02\x04".to_vec();
        let context = Context::new(&data);

        assert_eq!("4.2.2.1", IPv4::new(Endian::Little).to_display(context, &Data::default())?);

        Ok(())
    }

    #[test]
    fn test_ipv4_error() -> SimpleResult<()> {
        let data = b"\x7f\x00\x00".to_vec();
        let context = Context::new(&data);

        assert!(IPv4::new(Endian::Big).to_display(context, &Data::default()).is_err());

        Ok(())
    }
}
