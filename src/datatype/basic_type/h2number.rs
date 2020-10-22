use serde::{Serialize, Deserialize};
use simple_error::SimpleResult;

use sized_number::{Context, SizedDefinition, SizedDisplay};

use crate::datatype::H2Type;
use crate::datatype::basic_type::{H2BasicTrait, H2BasicType, H2BasicTypes};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct H2Number {
    definition: SizedDefinition,
    display: SizedDisplay,
}

impl From<H2Number> for H2BasicType {
    fn from(o: H2Number) -> H2BasicType {
        H2BasicType::new(H2BasicTypes::Number(o))
    }
}

impl H2Number {
    pub fn new(definition: SizedDefinition, display: SizedDisplay) -> Self {
        Self {
            definition: definition,
            display: display,
        }
    }
}

impl H2BasicTrait for H2Number {
    fn to_string(&self, context: &Context) -> SimpleResult<String> {
        self.definition.to_string(context, self.display)
    }

    fn size(&self) -> u64 {
        self.definition.size()
    }

    fn related(&self, _context: &Context) -> SimpleResult<Vec<(u64, H2Type)>> {
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;
    use sized_number::{Context, Endian};
    use sized_number::{SizedDefinition, SizedDisplay};

    #[test]
    fn test_u8_hex() -> SimpleResult<()> {
        let data = b"\x00\x7f\x80\xff".to_vec();

        let t = H2Number::new(
            SizedDefinition::U8,
            SizedDisplay::Hex(Default::default()),
        );

        let c = Context::new(&data);

        assert_eq!(1, t.size());
        assert_eq!(0, t.related(&c)?.len());
        assert_eq!("0x00", t.to_string(&c.at(0))?);
        assert_eq!("0x7f", t.to_string(&c.at(1))?);
        assert_eq!("0x80", t.to_string(&c.at(2))?);
        assert_eq!("0xff", t.to_string(&c.at(3))?);

        Ok(())
    }

    #[test]
    fn test_i16_decimal() -> SimpleResult<()> {
        let data = b"\x00\x00\x7f\xff\x80\x00\xff\xff".to_vec();

        let t = H2Number::new(
            SizedDefinition::I16(Endian::Big),
            SizedDisplay::Decimal,
        );

        let c = Context::new(&data);

        assert_eq!(2, t.size());
        assert_eq!(0, t.related(&c.at(0))?.len());
        assert_eq!("0", t.to_string(&c.at(0))?);
        assert_eq!("32767", t.to_string(&c.at(2))?);
        assert_eq!("-32768", t.to_string(&c.at(4))?);
        assert_eq!("-1", t.to_string(&c.at(6))?);

        Ok(())
    }
}
