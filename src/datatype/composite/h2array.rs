use serde::{Serialize, Deserialize};
use sized_number::Context;
use simple_error::{SimpleResult, bail};

use crate::datatype::{H2Type, ResolvedType};

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

    pub fn resolve(&self, starting_offset: u64, field_names: Option<Vec<String>>) -> (Vec<ResolvedType>, u64) {
        let mut result: Vec<ResolvedType> = Vec::new();
        let field_names = field_names.unwrap_or(Vec::new());
        let mut offset = starting_offset;

        for i in 0..self.length {
            let mut this_field_name = field_names.clone();
            this_field_name.push(i.to_string());

            let (mut basic, new_offset) = self.field_type.resolve_from_offset(Some(offset), Some(this_field_name));
            result.append(&mut basic);

            offset = new_offset;
        }

        (result, offset)
    }

    pub fn size(&self) -> u64 {
        self.length * self.field_type.size()
    }

    pub fn to_string(&self, context: &Context) -> SimpleResult<String> {
        let mut strings: Vec<String> = vec![];

        for i in 0..self.length {
            strings.push(self.field_type.to_string(&context.at(i * self.field_type.size()))?);
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

        let resolved = t.resolve();
        assert_eq!(4, resolved.len());

        assert_eq!(0, resolved[0].offset);
        assert_eq!("0x41414141", resolved[0].basic_type.to_string(&context.at(resolved[0].offset))?);

        assert_eq!(4, resolved[1].offset);
        assert_eq!("0x42424242", resolved[0].basic_type.to_string(&context.at(resolved[1].offset))?);

        assert_eq!(8, resolved[2].offset);
        assert_eq!("0x43434343", resolved[0].basic_type.to_string(&context.at(resolved[2].offset))?);

        assert_eq!(12, resolved[3].offset);
        assert_eq!("0x44444444", resolved[0].basic_type.to_string(&context.at(resolved[3].offset))?);

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
        let resolved = t.resolve();
        assert_eq!(12, resolved.len());

        assert_eq!("0",    resolved[0].basic_type.to_string(&context.at(resolved[0].offset))?);
        assert_eq!("0",    resolved[1].basic_type.to_string(&context.at(resolved[1].offset))?);
        assert_eq!("0",    resolved[2].basic_type.to_string(&context.at(resolved[2].offset))?);
        assert_eq!("0",    resolved[3].basic_type.to_string(&context.at(resolved[3].offset))?);

        assert_eq!("127",  resolved[4].basic_type.to_string(&context.at(resolved[4].offset))?);
        assert_eq!("127",  resolved[5].basic_type.to_string(&context.at(resolved[5].offset))?);
        assert_eq!("127",  resolved[6].basic_type.to_string(&context.at(resolved[6].offset))?);
        assert_eq!("127",  resolved[7].basic_type.to_string(&context.at(resolved[7].offset))?);

        assert_eq!("-128", resolved[8].basic_type.to_string(&context.at(resolved[8].offset))?);
        assert_eq!("-128", resolved[9].basic_type.to_string(&context.at(resolved[9].offset))?);
        assert_eq!("-1",   resolved[10].basic_type.to_string(&context.at(resolved[10].offset))?);
        assert_eq!("-1",   resolved[11].basic_type.to_string(&context.at(resolved[11].offset))?);

        Ok(())
    }
}
