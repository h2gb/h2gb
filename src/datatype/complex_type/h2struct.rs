use serde::{Serialize, Deserialize};
use simple_error::{bail, SimpleResult};

use crate::datatype::{H2Type, H2Types, ResolvedType, H2TypeTrait, ResolveOffset};
use crate::datatype::alignment::Alignment;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct H2Struct {
    fields: Vec<(String, H2Type)>,
}

impl H2Struct {
    // TODO: We need to prevent zero-length arrays
    pub fn new_aligned(alignment: Alignment, fields: Vec<(String, H2Type)>) -> H2Type {
        H2Type::new(alignment, H2Types::H2Struct(Self {
            fields: fields
        }))
    }

    pub fn new(fields: Vec<(String, H2Type)>) -> H2Type {
        Self::new_aligned(Alignment::None, fields)
    }
}

impl H2TypeTrait for H2Struct {
    // Is the size known ahead of time?
    fn is_static(&self) -> bool {
        // Loop over each field
        self.fields.iter().find(|(_, t)|
            // Stop at the first non-static field
            t.is_static() == false
        ).is_some()
    }

    // Get the static size, if possible
    fn size(&self, offset: ResolveOffset) -> SimpleResult<u64> {
        // Loop through each field

        if let Some(t) = self.resolve_partial(offset)?.last() {
            Ok(t.aligned_range.end - offset.position())
        } else {
            bail!("No elements");
        }
    }

    fn resolve_partial(&self, offset: ResolveOffset) -> SimpleResult<Vec<ResolvedType>> {
        let mut start = offset.position();

        self.fields.iter().map(|(name, field_type)| {
            let this_offset = offset.at(start);

            let resolved = ResolvedType {
                actual_range: field_type.actual_range(this_offset)?,
                aligned_range: field_type.aligned_range(this_offset)?,
                field_name: Some(name.clone()),
                field_type: field_type.clone(),
            };

            start = resolved.aligned_range.end;

            Ok(resolved)
        }).collect::<SimpleResult<Vec<ResolvedType>>>()
    }

    // Get the user-facing name of the type
    fn to_string(&self, offset: ResolveOffset) -> SimpleResult<String> {
        let elements = self.resolve_partial(offset)?.iter().map(|t| {
            Ok(format!("{}: {}", t.field_name.clone().unwrap_or("(unnamed)".to_string()), t.to_string(offset)?))
        }).collect::<SimpleResult<Vec<String>>>()?;

        Ok(elements.join(", "))
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
        let s_offset = ResolveOffset::Static(0);
        let d_offset = ResolveOffset::Dynamic(Context::new(&data));

        let t = H2Struct::new(vec![
            (
                "field_u32".to_string(),
                H2Number::new(
                    SizedDefinition::U32(Endian::Big),
                    SizedDisplay::Hex(Default::default()),
                )
            ),
            (
                "field_u16".to_string(),
                H2Number::new(
                    SizedDefinition::U16(Endian::Big),
                    SizedDisplay::Hex(Default::default()),
                )
            ),
            (
                "field_u8".to_string(),
                H2Number::new(
                    SizedDefinition::U8,
                    SizedDisplay::Octal(Default::default()),
                )
            ),
            (
                "field_u32_little".to_string(),
                H2Number::new(
                    SizedDefinition::U32(Endian::Little),
                    SizedDisplay::Decimal,
                )
            ),
        ]);

        assert_eq!(11, t.actual_size(s_offset)?);
        assert_eq!(11, t.actual_size(d_offset)?);

        let resolved = t.resolve_full(d_offset)?;

        assert_eq!(4, resolved.len());
        assert_eq!(0..4, resolved[0].actual_range);
        assert_eq!("0x00010203", resolved[0].to_string(d_offset)?);

        assert_eq!(4..6, resolved[1].actual_range);
        assert_eq!("0x0001", resolved[1].to_string(d_offset)?);

        assert_eq!(6..7, resolved[2].actual_range);
        assert_eq!("0o17", resolved[2].to_string(d_offset)?);

        assert_eq!(7..11, resolved[3].actual_range);
        assert_eq!("202182159", resolved[3].to_string(d_offset)?);

        Ok(())
    }

    #[test]
    fn test_nested_struct() -> SimpleResult<()> {
        //           ----- hex ------  ----struct----
        //                            -A- -B- ---C---
        let data = b"\x00\x01\x02\x03\x41\x42\x43\x43\x01\x00\x00\x00".to_vec();
        let s_offset = ResolveOffset::Static(0);
        let d_offset = ResolveOffset::Dynamic(Context::new(&data));

        let t = H2Struct::new(vec![
            (
                "field_u32".to_string(),
                H2Number::new(
                    SizedDefinition::U32(Endian::Big),
                    SizedDisplay::Hex(Default::default()),
                )
            ),
            (
                "struct".to_string(),
                H2Struct::new(vec![
                    ("A".to_string(), H2Number::new(SizedDefinition::U8, SizedDisplay::Hex(Default::default())).into()),
                    ("B".to_string(), H2Number::new(SizedDefinition::U8, SizedDisplay::Hex(Default::default())).into()),
                    ("C".to_string(), H2Number::new(SizedDefinition::U16(Endian::Big), SizedDisplay::Hex(Default::default())).into()),
                ])
            ),
            (
                "field_u32_little".to_string(),
                H2Number::new(
                    SizedDefinition::U32(Endian::Little),
                    SizedDisplay::Decimal,
                )
            ),
        ]);

        assert_eq!(12, t.actual_size(s_offset)?);
        assert_eq!(12, t.actual_size(d_offset)?);

        // let resolved = t.resolve_full(0, None);
        // assert_eq!(5, resolved.len());

        // assert_eq!(0..4,         resolved[0].offset);
        // assert_eq!("0x00010203", resolved[0].to_string(context)?);
        // assert_eq!(Some(vec!["field_u32".to_string()]), resolved[0].breadcrumbs);

        // assert_eq!(4..5,     resolved[1].offset);
        // assert_eq!("0x41",   resolved[1].to_string(context)?);
        // assert_eq!(Some(vec!["struct".to_string(), "A".to_string()]), resolved[1].breadcrumbs);

        // assert_eq!(5..6,     resolved[2].offset);
        // assert_eq!("0x42",   resolved[2].to_string(context)?);
        // assert_eq!(Some(vec!["struct".to_string(), "B".to_string()]), resolved[2].breadcrumbs);

        // assert_eq!(6..8,     resolved[3].offset);
        // assert_eq!("0x4343", resolved[3].to_string(context)?);
        // assert_eq!(Some(vec!["struct".to_string(), "C".to_string()]), resolved[3].breadcrumbs);

        // assert_eq!(8..12,    resolved[4].offset);
        // assert_eq!("1",      resolved[4].to_string(context)?);
        // assert_eq!(Some(vec!["field_u32_little".to_string()]), resolved[4].breadcrumbs);

        Ok(())
    }

    //#[test]
    //fn test_alignment() -> SimpleResult<()> {
    //    // P = padding / alignment bytes
    //    //           ----- hex ------ --hex--   -o-    ----decimal----
    //    let data = b"\x00\x01\x02\x03\x00\x01PP\x0fPPP\x0f\x0e\x0d\x0c".to_vec();
    //    let context = Context::new(&data);

    //    let t: StaticType = H2Struct::new_aligned(4, vec![
    //        (
    //            "field_u32".to_string(),
    //            H2Number::new(
    //                SizedDefinition::U32(Endian::Big),
    //                SizedDisplay::Hex(Default::default()),
    //            ).into()
    //        ),
    //        (
    //            "field_u16".to_string(),
    //            H2Number::new(
    //                SizedDefinition::U16(Endian::Big),
    //                SizedDisplay::Hex(Default::default()),
    //            ).into()
    //        ),
    //        (
    //            "field_u8".to_string(),
    //            H2Number::new(
    //                SizedDefinition::U8,
    //                SizedDisplay::Octal(Default::default()),
    //            ).into()
    //        ),
    //        (
    //            "field_u32_little".to_string(),
    //            H2Number::new(
    //                SizedDefinition::U32(Endian::Little),
    //                SizedDisplay::Decimal,
    //            ).into()
    //        ),
    //    ]).into();

    //    assert_eq!(16, t.size());

    //    let resolved = t.resolve_full(0, None);

    //    assert_eq!(4, resolved.len());
    //    assert_eq!(0..4, resolved[0].offset);
    //    assert_eq!("0x00010203", resolved[0].to_string(context)?);

    //    assert_eq!(4..6, resolved[1].offset);
    //    assert_eq!("0x0001", resolved[1].to_string(context)?);

    //    assert_eq!(8..9, resolved[2].offset);
    //    assert_eq!("0o17", resolved[2].to_string(context)?);

    //    assert_eq!(12..16, resolved[3].offset);
    //    assert_eq!("202182159", resolved[3].to_string(context)?);

    //    Ok(())
    //}

    //#[test]
    //fn test_nested_alignment() -> SimpleResult<()> {
    //    // P = padding / align bytes
    //    //           ----- hex ------      ----struct----
    //    //                                -A-  -B-  ---C---
    //    let data = b"\x00\x01\x02\x03PPPP\x41P\x42P\x43\x43PP\x01\x00\x00\x00PPPP".to_vec();
    //    let context = Context::new(&data);

    //    let t: StaticType = H2Struct::new_aligned(8, vec![
    //        (
    //            "field_u32".to_string(),
    //            H2Number::new(
    //                SizedDefinition::U32(Endian::Big),
    //                SizedDisplay::Hex(Default::default()),
    //            ).into()
    //        ),
    //        (
    //            "struct".to_string(),
    //            H2Struct::new_aligned(2, vec![
    //                ("A".to_string(), H2Number::new(SizedDefinition::U8, SizedDisplay::Hex(Default::default())).into()),
    //                ("B".to_string(), H2Number::new(SizedDefinition::U8, SizedDisplay::Hex(Default::default())).into()),
    //                ("C".to_string(), H2Number::new(SizedDefinition::U16(Endian::Big), SizedDisplay::Hex(Default::default())).into()),
    //            ]).into(),
    //        ),
    //        (
    //            "field_u32_little".to_string(),
    //            H2Number::new(
    //                SizedDefinition::U32(Endian::Little),
    //                SizedDisplay::Decimal,
    //            ).into()
    //        ),
    //    ]).into();

    //    assert_eq!(24, t.size());

    //    let resolved = t.resolve_full(0, None);
    //    assert_eq!(5, resolved.len());

    //    assert_eq!(0..4,         resolved[0].offset);
    //    assert_eq!("0x00010203", resolved[0].to_string(context)?);

    //    assert_eq!(8..9,     resolved[1].offset);
    //    assert_eq!("0x41",   resolved[1].to_string(context)?);

    //    assert_eq!(10..11,     resolved[2].offset);
    //    assert_eq!("0x42",   resolved[2].to_string(context)?);

    //    assert_eq!(12..14,     resolved[3].offset);
    //    assert_eq!("0x4343", resolved[3].to_string(context)?);

    //    assert_eq!(16..20,    resolved[4].offset);
    //    assert_eq!("1",      resolved[4].to_string(context)?);

    //    Ok(())
    //}

    //#[test]
    //fn test_struct_not_starting_at_zero() -> SimpleResult<()> {
    //    //                  ----- hex ------ --hex-- -o- ----decimal----
    //    let data = b"AAAAAAA\x00\x01\x02\x03\x00\x01\x0f\x0f\x0e\x0d\x0c".to_vec();
    //    let context = Context::new(&data);

    //    let t: StaticType = H2Struct::new(vec![
    //        (
    //            "field_u32".to_string(),
    //            H2Number::new(
    //                SizedDefinition::U32(Endian::Big),
    //                SizedDisplay::Hex(Default::default()),
    //            ).into()
    //        ),
    //        (
    //            "field_u16".to_string(),
    //            H2Number::new(
    //                SizedDefinition::U16(Endian::Big),
    //                SizedDisplay::Hex(Default::default()),
    //            ).into()
    //        ),
    //        (
    //            "field_u8".to_string(),
    //            H2Number::new(
    //                SizedDefinition::U8,
    //                SizedDisplay::Octal(Default::default()),
    //            ).into()
    //        ),
    //        (
    //            "field_u32_little".to_string(),
    //            H2Number::new(
    //                SizedDefinition::U32(Endian::Little),
    //                SizedDisplay::Decimal,
    //            ).into()
    //        ),
    //    ]).into();

    //    assert_eq!(11, t.size());

    //    let resolved = t.resolve_full(7, None);
    //    //println!("{:#?}", resolved);
    //    assert_eq!(4, resolved.len());
    //    assert_eq!(7..11, resolved[0].offset);
    //    assert_eq!("0x00010203", resolved[0].to_string(context)?);

    //    assert_eq!(11..13, resolved[1].offset);
    //    assert_eq!("0x0001", resolved[1].to_string(context)?);

    //    assert_eq!(13..14, resolved[2].offset);
    //    assert_eq!("0o17", resolved[2].to_string(context)?);

    //    assert_eq!(14..18, resolved[3].offset);
    //    assert_eq!("202182159", resolved[3].to_string(context)?);

    //    Ok(())
    //}
}
