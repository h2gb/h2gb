use serde::{Serialize, Deserialize};
use simple_error::SimpleResult;
use std::cmp;

use sized_number::Context;

use crate::datatype::helpers;
use crate::datatype::{StaticType, PartiallyResolvedType};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct H2Enum {
    // An array of strings and types (which might be other types)
    options: Vec<(String, StaticType)>,
    byte_alignment: Option<u64>,
}

impl From<H2Enum> for StaticType {
    fn from(o: H2Enum) -> StaticType {
        StaticType::from(StaticType::H2Enum(o))
    }
}

impl H2Enum {
    pub fn new(options: Vec<(String, StaticType)>) -> Self {
        Self {
            options: options,
            byte_alignment: None,
        }
    }

    pub fn new_aligned(byte_alignment: u64, options: Vec<(String, StaticType)>) -> Self {
        Self {
            options: options,
            byte_alignment: Some(byte_alignment),
        }
    }

    pub fn partially_resolve(&self, start: u64) -> Vec<PartiallyResolvedType> {
        let mut result = vec![];

        for (name, field_type) in self.options.iter() {
            result.push(PartiallyResolvedType {
                offset: start..(start + field_type.size()),
                field_name: Some(name.clone()),
                field_type: field_type.clone(),
            });
        }

        result
    }

    pub fn size(&self) -> u64 {
        let max_size = self.options.iter().fold(0, |sum, (_, t)| {
            // Only change if it's bigger
            cmp::max(t.size(), sum)
        });

        // Align, if needed
        match self.byte_alignment {
            Some(a) => helpers::round_up(max_size, a),
            None    => max_size,
        }
    }

    pub fn to_string(&self, context: &Context) -> SimpleResult<String> {
        let mut strings: Vec<String> = vec![];

        for r in self.partially_resolve(context.position()) {
            strings.push(format!("{}: {}",
                r.field_name.unwrap_or("unknown".to_string()),
                r.field_type.to_string(&context.at(r.offset.start))?
            ));
        }

        Ok(format!("{{ {} }}", strings.join(" | ")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;
    use sized_number::{Context, SizedDefinition, SizedDisplay, Endian};

    use crate::datatype::basic_type::h2number::H2Number;

    #[test]
    fn test_enum() -> SimpleResult<()> {
        let data = b"ABCD".to_vec();

        let e: StaticType = H2Enum::new(vec![
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
                "field_u8_octal".to_string(),
                H2Number::new(
                    SizedDefinition::U8,
                    SizedDisplay::Octal(Default::default()),
                ).into()
            ),
            (
                "field_u8_decimal".to_string(),
                H2Number::new(
                    SizedDefinition::U8,
                    SizedDisplay::Decimal,
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

        // Size will be the longest field, which is 32-bit
        assert_eq!(4, e.size());

        let r = e.fully_resolve(0, None);
        assert_eq!(0..4, r[0].offset);
        assert_eq!("0x41424344", r[0].to_string(&Context::new(&data))?);

        assert_eq!(0..2, r[1].offset);
        assert_eq!("0x4142", r[1].to_string(&Context::new(&data))?);

        assert_eq!(0..1, r[2].offset);
        assert_eq!("0x41", r[2].to_string(&Context::new(&data))?);

        assert_eq!(0..1, r[3].offset);
        assert_eq!("0o101", r[3].to_string(&Context::new(&data))?);

        assert_eq!(0..1, r[4].offset);
        assert_eq!("65", r[4].to_string(&Context::new(&data))?);

        assert_eq!(0..4, r[5].offset);
        assert_eq!("0x44434241", r[5].to_string(&Context::new(&data))?);

        Ok(())
    }
}
