use macaddr::MacAddr8;
use serde::{Serialize, Deserialize};
use simple_error::SimpleResult;

use generic_number::Context;

use crate::{Alignment, H2Type, H2Types, H2TypeTrait};

/// Defines a MAC address in EUI-64 format.
///
/// An EUI-64 MAC address is always 8 bytes long.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacAddress8 {
}

impl MacAddress8 {
    pub fn new_aligned(alignment: Alignment) -> H2Type {
        H2Type::new(alignment, H2Types::MacAddress8(Self {
        }))
    }

    pub fn new() -> H2Type {
        Self::new_aligned(Alignment::None)
    }
}

impl H2TypeTrait for MacAddress8 {
    fn is_static(&self) -> bool {
        true
    }

    fn actual_size(&self, _context: Context) -> SimpleResult<u64> {
        Ok(8)
    }

    fn to_display(&self, context: Context) -> SimpleResult<String> {
        let b = context.read_bytes(8)?;
        let b: [u8; 8] = [
            b[0],
            b[1],
            b[2],
            b[3],
            b[4],
            b[5],
            b[6],
            b[7],
        ];

        Ok(MacAddr8::from(b).to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;
    use generic_number::Context;

    #[test]
    fn test_mac() -> SimpleResult<()> {
        let data = b"\x01\x23\x45\x67\x89\xab\xcd\xef".to_vec();
        let context = Context::new(&data);

        assert_eq!("01:23:45:67:89:AB:CD:EF", MacAddress8::new().to_display(context)?);

        Ok(())
    }
}
