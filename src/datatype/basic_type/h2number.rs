use serde::{Serialize, Deserialize};
use simple_error::SimpleResult;

use sized_number::{SizedDefinition, SizedDisplay};

use crate::datatype::{H2Type, H2Types, H2TypeTrait, ResolveOffset, AlignValue};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct H2Number {
    definition: SizedDefinition,
    display: SizedDisplay,
}

impl H2Number {
    pub fn new_aligned(alignment: AlignValue, definition: SizedDefinition, display: SizedDisplay) -> H2Type {
        H2Type::new(alignment, H2Types::H2Number(Self {
            definition: definition,
            display: display,
        }))
    }

    pub fn new(definition: SizedDefinition, display: SizedDisplay) -> H2Type {
        Self::new_aligned(AlignValue::None, definition, display)
    }
}

impl H2TypeTrait for H2Number {
    fn is_static(&self) -> bool {
        true
    }

    fn size(&self, _offset: &ResolveOffset) -> SimpleResult<u64> {
        Ok(self.definition.size())
    }

    fn to_string(&self, offset: &ResolveOffset) -> SimpleResult<String> {
        match offset {
            ResolveOffset::Static(_) => Ok("Number".to_string()),
            ResolveOffset::Dynamic(context) => {
                self.definition.to_string(context, self.display)
            }
        }
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
        let s_offset = ResolveOffset::Static(0);
        let d_offset = ResolveOffset::Dynamic(Context::new(&data));

        let t = H2Number::new(
            SizedDefinition::U8,
            SizedDisplay::Hex(Default::default()),
        );

        assert_eq!(1, t.actual_size(&s_offset).unwrap());
        assert_eq!(1, t.actual_size(&d_offset).unwrap());

        assert_eq!(0, t.related(&s_offset)?.len());
        assert_eq!(0, t.related(&d_offset)?.len());

        assert_eq!("0x00", t.to_string(&d_offset.at(0))?);
        assert_eq!("0x7f", t.to_string(&d_offset.at(1))?);
        assert_eq!("0x80", t.to_string(&d_offset.at(2))?);
        assert_eq!("0xff", t.to_string(&d_offset.at(3))?);

        Ok(())
    }

    #[test]
    fn test_i16_decimal() -> SimpleResult<()> {
        let data = b"\x00\x00\x7f\xff\x80\x00\xff\xff".to_vec();
        let s_offset = ResolveOffset::Static(0);
        let d_offset = ResolveOffset::Dynamic(Context::new(&data));

        let t = H2Number::new(
            SizedDefinition::I16(Endian::Big),
            SizedDisplay::Decimal,
        );

        assert_eq!(2, t.actual_size(&s_offset).unwrap());
        assert_eq!(2, t.actual_size(&d_offset).unwrap());

        assert_eq!(0, t.related(&s_offset)?.len());
        assert_eq!(0, t.related(&d_offset)?.len());

        assert_eq!("0",      t.to_string(&d_offset.at(0))?);
        assert_eq!("32767",  t.to_string(&d_offset.at(2))?);
        assert_eq!("-32768", t.to_string(&d_offset.at(4))?);
        assert_eq!("-1",     t.to_string(&d_offset.at(6))?);

        Ok(())
    }

    #[test]
    fn test_alignment() -> SimpleResult<()> {
        let data = b"\x00\x00\x7f\xff\x80\x00\xff\xff".to_vec();
        let s_offset = ResolveOffset::Static(0);
        let d_offset = ResolveOffset::Dynamic(Context::new(&data));

        let t = H2Number::new_aligned(
            AlignValue::After(8),
            SizedDefinition::I16(Endian::Big),
            SizedDisplay::Decimal,
        );

        assert_eq!(2, t.actual_size(&s_offset)?);
        assert_eq!(8, t.aligned_size(&s_offset)?);

        assert_eq!(2, t.actual_size(&d_offset)?);
        assert_eq!(8, t.aligned_size(&d_offset)?);

        assert_eq!("0",      t.to_string(&d_offset.at(0))?);
        assert_eq!("32767",  t.to_string(&d_offset.at(2))?);
        assert_eq!("-32768", t.to_string(&d_offset.at(4))?);
        assert_eq!("-1",     t.to_string(&d_offset.at(6))?);

        Ok(())
    }
}
