use macaddr::MacAddr8;
use serde::{Serialize, Deserialize};
use simple_error::SimpleResult;

use generic_number::Context;

use crate::{Alignment, Data, H2Type, H2TypeTrait};

/// Defines a MAC address in EUI-64 format.
///
/// An EUI-64 MAC address is always 8 bytes long.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacAddress8 {
    #[serde(skip_serializing_if = "Option::is_none")]
    alignment: Option<Alignment>,
}

impl From<MacAddress8> for H2Type {
    fn from(t: MacAddress8) -> H2Type {
        H2Type::MacAddress8(t)
    }
}

impl MacAddress8 {
    pub fn new_aligned(alignment: Option<Alignment>) -> Self {
        Self {
            alignment: alignment,
        }
    }

    pub fn new() -> Self {
        Self::new_aligned(None)
    }
}

impl H2TypeTrait for MacAddress8 {
    fn base_size(&self, _context: Context, _data: &Data) -> SimpleResult<usize> {
        Ok(8)
    }

    fn to_display(&self, context: Context, _data: &Data) -> SimpleResult<String> {
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
        let data = b"\x01\x23\x45\x67\x89\xab\xcd\xef".to_vec();
        let context = Context::new(&data);

        assert_eq!("01:23:45:67:89:AB:CD:EF", MacAddress8::new().to_display(context, &Data::default())?);

        Ok(())
    }
}
