use serde::{Serialize, Deserialize};
use simple_error::SimpleResult;
use sized_number::Context;

use crate::datatype::helpers;
use crate::datatype::{H2Type, PartiallyResolvedType};

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

    pub fn partially_resolve(&self, start: u64) -> Vec<PartiallyResolvedType> {
        let mut result = vec![];
        let mut offset: u64 = start;

        for (name, field_type) in self.fields.iter() {
            let end_offset = match self.byte_alignment {
                Some(a) => helpers::round_up(offset + field_type.size(), a),
                None    => offset + field_type.size(),
            };

            result.push(PartiallyResolvedType {
                offset: offset..end_offset,
                field_name: Some(name.clone()),
                field_type: field_type.clone(),
            });

            offset = end_offset;
        }

        result
    }

    pub fn size(&self) -> u64 {
        self.fields.iter().fold(0, |sum, (_, t)| {
            sum + t.size()
        })
    }

    pub fn to_string(&self, context: &Context) -> SimpleResult<String> {
        let mut strings: Vec<String> = vec![];

        for r in self.partially_resolve(context.position()) {
            strings.push(format!("{}: {}",
                r.field_name.unwrap_or("unknown".to_string()),
                r.field_type.to_string(&context.at(r.offset.start))?
            ));
        }

        Ok(format!("{{{}}}", strings.join(", ")))
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
        //           ----- hex ------ --hex-- -o- ----decimal----
        let data = b"\x00\x01\x02\x03\x00\x01\x0f\x0f\x0e\x0d\x0c".to_vec();
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
                    SizedDisplay::Octal(Default::default()),
                ).into()
            ),
            (
                "field_u32_little".to_string(),
                H2Number::new(
                    SizedDefinition::U32(Endian::Little),
                    SizedDisplay::Decimal,
                ).into()
            ),
        ]).into();

        assert_eq!(11, t.size());

        let resolved = t.fully_resolve(0, None);
        assert_eq!(4, resolved.len());
        assert_eq!(0..4, resolved[0].offset);
        assert_eq!("0x00010203", resolved[0].to_string(&context)?);

        assert_eq!(4..6, resolved[1].offset);
        assert_eq!("0x0001", resolved[1].to_string(&context)?);

        assert_eq!(6..7, resolved[2].offset);
        assert_eq!("0o17", resolved[2].to_string(&context)?);

        assert_eq!(7..11, resolved[3].offset);
        assert_eq!("202182159", resolved[3].to_string(&context)?);

        Ok(())
    }
}
