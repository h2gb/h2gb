use serde::{Serialize, Deserialize};

use macaddr::MacAddr6;
use simple_error::SimpleResult;

use crate::datatype::{Alignment, H2Type, H2Types, H2TypeTrait, Offset};

/// Defines a MAC address in EUI-48 format.
///
/// An EUI-48 MAC address is always 6 bytes long.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacAddress {
}

impl MacAddress {
    pub fn new_aligned(alignment: Alignment) -> H2Type {
        H2Type::new(alignment, H2Types::MacAddress(Self {
        }))
    }

    pub fn new() -> H2Type {
        Self::new_aligned(Alignment::None)
    }
}

impl H2TypeTrait for MacAddress {
    fn is_static(&self) -> bool {
        true
    }

    fn actual_size(&self, _offset: Offset) -> SimpleResult<u64> {
        Ok(6)
    }

    fn to_display(&self, offset: Offset) -> SimpleResult<String> {
        match offset {
            Offset::Static(_) => Ok("MAC Address".to_string()),
            Offset::Dynamic(context) => {
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;
    use crate::sized_number::Context;

    #[test]
    fn test_mac() -> SimpleResult<()> {
        let data = b"\x01\x23\x45\x67\x89\xab".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        assert_eq!("01:23:45:67:89:AB", MacAddress::new().to_display(offset)?);

        Ok(())
    }
}
