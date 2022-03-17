use macaddr::MacAddr6;
use serde::{Serialize, Deserialize};
use simple_error::SimpleResult;

use generic_number::Context;

use crate::{Alignment, Data, H2Type, H2Types, H2TypeTrait};

/// Defines a MAC address in EUI-48 format.
///
/// An EUI-48 MAC address is always 6 bytes long.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacAddress {
    alignment: Option<Alignment>,
}

impl MacAddress {
    pub fn new_aligned(alignment: Option<Alignment>) -> H2Type {
        H2Type::new(H2Types::MacAddress(Self {
            alignment: alignment,
        }))
    }

    pub fn new() -> H2Type {
        Self::new_aligned(None)
    }
}

impl H2TypeTrait for MacAddress {
    fn base_size(&self, _context: Context) -> SimpleResult<usize> {
        Ok(6)
    }

    fn to_display(&self, context: Context, _data: &Data) -> SimpleResult<String> {
        let b = context.read_bytes(6)?;
        let b: [u8; 6] = [
            b[0],
            b[1],
            b[2],
            b[3],
            b[4],
            b[5],
        ];

        Ok(MacAddr6::from(b).to_string())
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
    use generic_number::Context;

    #[test]
    fn test_mac() -> SimpleResult<()> {
        let data = b"\x01\x23\x45\x67\x89\xab".to_vec();
        let context = Context::new(&data);

        assert_eq!("01:23:45:67:89:AB", MacAddress::new().to_display(context, &Data::default())?);

        Ok(())
    }
}
