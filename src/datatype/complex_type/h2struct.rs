use serde::{Serialize, Deserialize};
use simple_error::SimpleResult;
use sized_number::Context;

use crate::datatype::helpers;
use crate::datatype::{StaticType, PartiallyResolvedType};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct H2Struct {
    // An array of strings and types (which might be other types)
    fields: Vec<(String, StaticType)>,
    byte_alignment: Option<u64>,
}

impl From<H2Struct> for StaticType {
    fn from(o: H2Struct) -> StaticType {
        StaticType::from(StaticType::H2Struct(o))
    }
}

impl H2Struct {
    pub fn new(fields: Vec<(String, StaticType)>) -> Self {
        Self {
            fields: fields,
            byte_alignment: None,
        }
    }

    pub fn new_aligned(byte_alignment: u64, fields: Vec<(String, StaticType)>) -> Self {
        Self {
            fields: fields,
            byte_alignment: Some(byte_alignment),
        }
    }

    pub fn partially_resolve(&self, start: u64) -> Vec<PartiallyResolvedType> {
        let mut result = vec![];
        let mut offset: u64 = start;

        for (name, field_type) in self.fields.iter() {
            // The entry ends after the field_type
            let end_offset = offset + field_type.size();

            // The next offset is the end_offset, aligned
            let next_offset = match self.byte_alignment {
                Some(a) => helpers::round_up(end_offset, a),
                None    => end_offset,
            };

            result.push(PartiallyResolvedType {
                offset: offset..end_offset,
                field_name: Some(name.clone()),
                field_type: field_type.clone(),
            });

            offset = next_offset;
        }

        result
    }

    pub fn size(&self) -> u64 {
        self.fields.iter().fold(0, |sum, (_, t)| {
            match self.byte_alignment {
                Some(a) => sum + helpers::round_up(t.size(), a),
                None    => sum + t.size(),
            }
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

    use crate::datatype::basic_type::h2number::H2Number;

    #[test]
    fn test_struct() -> SimpleResult<()> {
        //           ----- hex ------ --hex-- -o- ----decimal----
        let data = b"\x00\x01\x02\x03\x00\x01\x0f\x0f\x0e\x0d\x0c".to_vec();
        let context = Context::new(&data);

        let t: StaticType = H2Struct::new(vec![
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
        //println!("{:#?}", resolved);
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

    #[test]
    fn test_nested_struct() -> SimpleResult<()> {
        //           ----- hex ------  ----struct----
        //                            -A- -B- ---C---
        let data = b"\x00\x01\x02\x03\x41\x42\x43\x43\x01\x00\x00\x00".to_vec();
        let context = Context::new(&data);

        let t: StaticType = H2Struct::new(vec![
            (
                "field_u32".to_string(),
                H2Number::new(
                    SizedDefinition::U32(Endian::Big),
                    SizedDisplay::Hex(Default::default()),
                ).into()
            ),
            (
                "struct".to_string(),
                H2Struct::new(vec![
                    ("A".to_string(), H2Number::new(SizedDefinition::U8, SizedDisplay::Hex(Default::default())).into()),
                    ("B".to_string(), H2Number::new(SizedDefinition::U8, SizedDisplay::Hex(Default::default())).into()),
                    ("C".to_string(), H2Number::new(SizedDefinition::U16(Endian::Big), SizedDisplay::Hex(Default::default())).into()),
                ]).into(),
            ),
            (
                "field_u32_little".to_string(),
                H2Number::new(
                    SizedDefinition::U32(Endian::Little),
                    SizedDisplay::Decimal,
                ).into()
            ),
        ]).into();

        assert_eq!(12, t.size());

        let resolved = t.fully_resolve(0, None);
        assert_eq!(5, resolved.len());

        assert_eq!(0..4,         resolved[0].offset);
        assert_eq!("0x00010203", resolved[0].to_string(&context)?);
        assert_eq!(Some(vec!["field_u32".to_string()]), resolved[0].breadcrumbs);

        assert_eq!(4..5,     resolved[1].offset);
        assert_eq!("0x41",   resolved[1].to_string(&context)?);
        assert_eq!(Some(vec!["struct".to_string(), "A".to_string()]), resolved[1].breadcrumbs);

        assert_eq!(5..6,     resolved[2].offset);
        assert_eq!("0x42",   resolved[2].to_string(&context)?);
        assert_eq!(Some(vec!["struct".to_string(), "B".to_string()]), resolved[2].breadcrumbs);

        assert_eq!(6..8,     resolved[3].offset);
        assert_eq!("0x4343", resolved[3].to_string(&context)?);
        assert_eq!(Some(vec!["struct".to_string(), "C".to_string()]), resolved[3].breadcrumbs);

        assert_eq!(8..12,    resolved[4].offset);
        assert_eq!("1",      resolved[4].to_string(&context)?);
        assert_eq!(Some(vec!["field_u32_little".to_string()]), resolved[4].breadcrumbs);

        Ok(())
    }

    #[test]
    fn test_alignment() -> SimpleResult<()> {
        // P = padding / alignment bytes
        //           ----- hex ------ --hex--   -o-    ----decimal----
        let data = b"\x00\x01\x02\x03\x00\x01PP\x0fPPP\x0f\x0e\x0d\x0c".to_vec();
        let context = Context::new(&data);

        let t: StaticType = H2Struct::new_aligned(4, vec![
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

        assert_eq!(16, t.size());

        let resolved = t.fully_resolve(0, None);

        assert_eq!(4, resolved.len());
        assert_eq!(0..4, resolved[0].offset);
        assert_eq!("0x00010203", resolved[0].to_string(&context)?);

        assert_eq!(4..6, resolved[1].offset);
        assert_eq!("0x0001", resolved[1].to_string(&context)?);

        assert_eq!(8..9, resolved[2].offset);
        assert_eq!("0o17", resolved[2].to_string(&context)?);

        assert_eq!(12..16, resolved[3].offset);
        assert_eq!("202182159", resolved[3].to_string(&context)?);

        Ok(())
    }

    #[test]
    fn test_nested_alignment() -> SimpleResult<()> {
        // P = padding / align bytes
        //           ----- hex ------      ----struct----
        //                                -A-  -B-  ---C---
        let data = b"\x00\x01\x02\x03PPPP\x41P\x42P\x43\x43PP\x01\x00\x00\x00PPPP".to_vec();
        let context = Context::new(&data);

        let t: StaticType = H2Struct::new_aligned(8, vec![
            (
                "field_u32".to_string(),
                H2Number::new(
                    SizedDefinition::U32(Endian::Big),
                    SizedDisplay::Hex(Default::default()),
                ).into()
            ),
            (
                "struct".to_string(),
                H2Struct::new_aligned(2, vec![
                    ("A".to_string(), H2Number::new(SizedDefinition::U8, SizedDisplay::Hex(Default::default())).into()),
                    ("B".to_string(), H2Number::new(SizedDefinition::U8, SizedDisplay::Hex(Default::default())).into()),
                    ("C".to_string(), H2Number::new(SizedDefinition::U16(Endian::Big), SizedDisplay::Hex(Default::default())).into()),
                ]).into(),
            ),
            (
                "field_u32_little".to_string(),
                H2Number::new(
                    SizedDefinition::U32(Endian::Little),
                    SizedDisplay::Decimal,
                ).into()
            ),
        ]).into();

        assert_eq!(24, t.size());

        let resolved = t.fully_resolve(0, None);
        assert_eq!(5, resolved.len());

        assert_eq!(0..4,         resolved[0].offset);
        assert_eq!("0x00010203", resolved[0].to_string(&context)?);

        assert_eq!(8..9,     resolved[1].offset);
        assert_eq!("0x41",   resolved[1].to_string(&context)?);

        assert_eq!(10..11,     resolved[2].offset);
        assert_eq!("0x42",   resolved[2].to_string(&context)?);

        assert_eq!(12..14,     resolved[3].offset);
        assert_eq!("0x4343", resolved[3].to_string(&context)?);

        assert_eq!(16..20,    resolved[4].offset);
        assert_eq!("1",      resolved[4].to_string(&context)?);

        Ok(())
    }

    #[test]
    fn test_struct_not_starting_at_zero() -> SimpleResult<()> {
        //                  ----- hex ------ --hex-- -o- ----decimal----
        let data = b"AAAAAAA\x00\x01\x02\x03\x00\x01\x0f\x0f\x0e\x0d\x0c".to_vec();
        let context = Context::new(&data);

        let t: StaticType = H2Struct::new(vec![
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

        let resolved = t.fully_resolve(7, None);
        //println!("{:#?}", resolved);
        assert_eq!(4, resolved.len());
        assert_eq!(7..11, resolved[0].offset);
        assert_eq!("0x00010203", resolved[0].to_string(&context)?);

        assert_eq!(11..13, resolved[1].offset);
        assert_eq!("0x0001", resolved[1].to_string(&context)?);

        assert_eq!(13..14, resolved[2].offset);
        assert_eq!("0o17", resolved[2].to_string(&context)?);

        assert_eq!(14..18, resolved[3].offset);
        assert_eq!("202182159", resolved[3].to_string(&context)?);

        Ok(())
    }
}
