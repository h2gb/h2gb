use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, bail};
use sized_number::Context;

use crate::datatype::{H2Type, ResolvedType};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct H2Struct {
    // An array of strings and types (which might be other types)
    fields: Vec<(String, H2Type)>,
    byte_alignment: Option<u64>,
}

impl From<H2Struct> for H2Type {
    fn from(o: H2Struct) -> H2Type {
        H2Type::from(H2Type::H2Struct(o))
    }
}

impl H2Struct {
    pub fn new_aligned(fields: Vec<(String, H2Type)>, byte_alignment: u64) -> Self {
        Self {
            fields: fields,
            byte_alignment: Some(byte_alignment),
        }
    }

    pub fn new(fields: Vec<(String, H2Type)>) -> Self {
        Self {
            fields: fields,
            byte_alignment: None,
        }
    }

    pub fn resolve(&self, starting_offset: u64, field_names: Option<Vec<String>>) -> (Vec<ResolvedType>, u64) {
        let mut result: Vec<ResolvedType> = Vec::new();
        let field_names = field_names.unwrap_or(Vec::new());
        let mut offset = starting_offset;

        for (name, field_type) in self.fields.iter() {
            let mut this_field_name = field_names.clone();
            this_field_name.push(name.clone());

            let (mut basic, new_offset) = field_type.resolve_from_offset(Some(offset), Some(this_field_name));
            result.append(&mut basic);

            offset = new_offset;
        }

        (result, offset)
    }

    pub fn size(&self) -> u64 {
        self.fields.iter().fold(0, |sum, (_, t)| {
            sum + t.size()
        })
    }

    pub fn to_string(&self, context: &Context) -> SimpleResult<String> {
        let mut strings: Vec<String> = vec![];
        let mut offset = 0;

        for (name, field_type) in self.fields.iter() {
            strings.push(format!("{}: {}", name, field_type.to_string(&context.at(offset))?));

            offset += field_type.size();
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
    fn test_struct() -> SimpleResult<()> {
        let data = b"\x00\x01\x02\x03\x00\x01\x00\x0f\x0e\x0d\x0c".to_vec();
        let context = Context::new(&data);

        let t: H2Type = H2Struct::new(vec![
            (
                "field_u32".to_string(),
                H2Number::new(
                    SizedDefinition::U32(Endian::Big),
                    SizedDisplay::Hex(Default::default()),
                ).into()
            ),
            (
                "field_u16".to_string(),
                H2Number::new(
                    SizedDefinition::U16(Endian::Big),
                    SizedDisplay::Hex(Default::default()),
                ).into()
            ),
            (
                "field_u8".to_string(),
                H2Number::new(
                    SizedDefinition::U8,
                    SizedDisplay::Hex(Default::default()),
                ).into()
            ),
            (
                "field_u32_little".to_string(),
                H2Number::new(
                    SizedDefinition::U32(Endian::Little),
                    SizedDisplay::Hex(Default::default()),
                ).into()
            ),
        ]).into();

        assert_eq!(11, t.size());

        let resolved = t.resolve();
        assert_eq!(4, resolved.len());
        assert_eq!(0, resolved[0].offset);
        assert_eq!(4, resolved[1].offset);
        assert_eq!(6, resolved[2].offset);
        assert_eq!(7, resolved[3].offset);

        println!("Type: {:?}", t);
        println!("\nto_string:\n{}", t.to_string(&context)?);

        Ok(())
    }
}
