use serde::{Serialize, Deserialize};
use sized_number::Context;
use simple_error::SimpleResult;

use crate::datatype::{helpers, H2Type, PartiallyResolvedType};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct H2Array {
    field_type: Box<H2Type>,
    length: u64,
    byte_alignment: Option<u64>,
}

impl From<H2Array> for H2Type {
    fn from(o: H2Array) -> H2Type {
        H2Type::from(H2Type::H2Array(o))
    }
}

impl H2Array {
    pub fn new(length: u64, field_type: H2Type) -> Self {
        Self {
            field_type: Box::new(field_type),
            length: length,
            byte_alignment: None,
        }
    }

    pub fn partially_resolve(&self, start: u64) -> Vec<PartiallyResolvedType> {
        let mut result = vec![];
        let mut offset: u64 = start;

        for i in 0..self.length {
            let end_offset = match self.byte_alignment {
                Some(a) => helpers::round_up(offset + self.field_type.size(), a),
                None    => offset + self.field_type.size(),
            };

            result.push(PartiallyResolvedType {
                offset: offset..end_offset,
                field_name: Some(i.to_string()),
                field_type: (*self.field_type).clone(),
            });

            offset = end_offset;
        }

        result
    }

    pub fn size(&self) -> u64 {
        self.length * self.field_type.size()
    }

    pub fn to_string(&self, context: &Context) -> SimpleResult<String> {
        let mut strings: Vec<String> = vec![];

        for r in self.partially_resolve(context.position()) {
            strings.push(r.field_type.to_string(&context.at(r.offset.start))?);
        }

        Ok(format!("[{}]", strings.join(", ")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;
    use sized_number::{Context, SizedDefinition, SizedDisplay, Endian};

    use crate::datatype::basic::h2number::H2Number;

    #[test]
    fn test_array() -> SimpleResult<()> {
        let data = b"AAAABBBBCCCCDDDD".to_vec();
        let context = Context::new(&data);

        // An array of 4 32-bit unsigned integers
        let t: H2Type = H2Array::new(4,
            H2Number::new(SizedDefinition::U32(Endian::Big), SizedDisplay::Hex(Default::default())).into()
        ).into();

        assert_eq!(16, t.size());

        let resolved = t.fully_resolve(0, None);
        assert_eq!(4, resolved.len());

        assert_eq!(0..4, resolved[0].offset);
        assert_eq!("0x41414141", resolved[0].to_string(&context)?);

        assert_eq!(4..8, resolved[1].offset);
        assert_eq!("0x42424242", resolved[1].to_string(&context)?);

        assert_eq!(8..12, resolved[2].offset);
        assert_eq!("0x43434343", resolved[2].to_string(&context)?);

        assert_eq!(12..16, resolved[3].offset);
        assert_eq!("0x44444444", resolved[3].to_string(&context)?);

        Ok(())
    }

    #[test]
    fn test_nested_array() -> SimpleResult<()> {
        let data = b"\x00\x00\x00\x00\x7f\x7f\x7f\x7f\x80\x80\xff\xff".to_vec();
        let context = Context::new(&data);

        // An array of 4 4-element I8 arrays that will print as decimal
        let t: H2Type = H2Array::new(4,
            H2Array::new(3,
                H2Number::new(SizedDefinition::I8, SizedDisplay::Decimal).into()
            ).into(),
        ).into();

        assert_eq!(12, t.size());

        // This will resolve to the 12 I8 values
        let resolved = t.fully_resolve(0, None);
        assert_eq!(12, resolved.len());

        assert_eq!("0",    resolved[0].to_string(&context)?);
        assert_eq!("0",    resolved[1].to_string(&context)?);
        assert_eq!("0",    resolved[2].to_string(&context)?);
        assert_eq!("0",    resolved[3].to_string(&context)?);

        assert_eq!("127",  resolved[4].to_string(&context)?);
        assert_eq!("127",  resolved[5].to_string(&context)?);
        assert_eq!("127",  resolved[6].to_string(&context)?);
        assert_eq!("127",  resolved[7].to_string(&context)?);

        assert_eq!("-128", resolved[8].to_string(&context)?);
        assert_eq!("-128", resolved[9].to_string(&context)?);
        assert_eq!("-1",  resolved[10].to_string(&context)?);
        assert_eq!("-1",  resolved[11].to_string(&context)?);

        Ok(())
    }
}
