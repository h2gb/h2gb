use serde::{Serialize, Deserialize};
use simple_error::{bail, SimpleResult};

use crate::datatype::{H2Type, H2Types, ResolvedType, H2TypeTrait, ResolveOffset, AlignValue};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct H2Array {
    field_type: Box<H2Type>,
    length: u64,
}

impl H2Array {
    // TODO: We need to prevent zero-length arrays
    pub fn new_aligned(alignment: AlignValue, length: u64, field_type: H2Type) -> H2Type {
        H2Type::new(alignment, H2Types::H2Array(Self {
            field_type: Box::new(field_type),
            length: length,
        }))
    }

    pub fn new(length: u64, field_type: H2Type) -> H2Type {
        Self::new_aligned(AlignValue::None, length, field_type)
    }
}

impl H2TypeTrait for H2Array {
    fn is_static(&self) -> bool {
        self.field_type.is_static()
    }

    fn size(&self, offset: &ResolveOffset) -> SimpleResult<u64> {
        match self.is_static() {
            // TODO: Alignment might make this weird
            true => Ok(self.length * self.field_type.aligned_size(offset)?),
            false => bail!("We can't calculate size of Dynamic arrays yet"),
        }
    }

    fn resolve_partial(&self, offset: &ResolveOffset) -> SimpleResult<Vec<ResolvedType>> {
        let mut result = vec![];
        let mut start: u64 = offset.position();

        for i in 0..self.length {
            let this_offset = offset.at(start);

            result.push(ResolvedType {
                // Note: the end depends on the normal size, not the static one
                offset: self.field_type.actual_range(&this_offset)?,
                field_name: Some(i.to_string()),
                field_type: (*self.field_type).clone(),
            });

            start = start + self.field_type.aligned_size(&this_offset)?
        };

        Ok(result)
    }

    fn to_string(&self, offset: &ResolveOffset) -> SimpleResult<String> {
        // Because the collect() expects a result, this will end and bubble
        // up errors automatically!
        let strings: Vec<String> = self.resolve_partial(offset)?.iter().map(|c| {
            c.field_type.to_string(&offset.at(c.offset.start))
        }).collect::<SimpleResult<Vec<String>>>()?;

        Ok(format!("[{}]", strings.join(", ")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;
    use sized_number::{Context, SizedDefinition, SizedDisplay, Endian};

    use crate::datatype::basic_type::h2number::H2Number;

    #[test]
    fn test_array() -> SimpleResult<()> {
        let data = b"AAAABBBBCCCCDDDD".to_vec();
        let s_offset = ResolveOffset::Static(0);
        let d_offset = ResolveOffset::Dynamic(Context::new(&data));

        // An array of 4 32-bit unsigned integers
        let t = H2Array::new(4,
            H2Number::new(SizedDefinition::U32(Endian::Big), SizedDisplay::Hex(Default::default()))
        );

        assert_eq!(true, t.is_static());
        assert_eq!(16, t.actual_size(&s_offset)?);

        assert_eq!(4, t.resolve_partial(&s_offset)?.len());
        assert_eq!(4, t.resolve_partial(&d_offset)?.len());

        let resolved = t.resolve_full(&d_offset)?;
        assert_eq!(4, resolved.len());

        assert_eq!(0..4, resolved[0].offset);
        assert_eq!("0x41414141", resolved[0].to_string(&d_offset)?);

        assert_eq!(4..8, resolved[1].offset);
        assert_eq!("0x42424242", resolved[1].to_string(&d_offset)?);

        assert_eq!(8..12, resolved[2].offset);
        assert_eq!("0x43434343", resolved[2].to_string(&d_offset)?);

        assert_eq!(12..16, resolved[3].offset);
        assert_eq!("0x44444444", resolved[3].to_string(&d_offset)?);

        Ok(())
    }

    #[test]
    fn test_nested_array() -> SimpleResult<()> {
        let data = b"\x00\x00\x00\x00\x7f\x7f\x7f\x7f\x80\x80\xff\xff".to_vec();
        let s_offset = ResolveOffset::Static(0);
        let d_offset = ResolveOffset::Dynamic(Context::new(&data));

        // An array of 4 4-element I8 arrays that will print as decimal
        let t = H2Array::new(4,
            H2Array::new(3,
                H2Number::new(SizedDefinition::I8, SizedDisplay::Decimal)
            ),
        );

        assert_eq!(12, t.actual_size(&s_offset)?);
        assert_eq!(12, t.actual_size(&d_offset)?);

        // Should have 4 direct children
        assert_eq!(4, t.resolve_partial(&s_offset)?.len());
        assert_eq!(4, t.resolve_partial(&d_offset)?.len());

        // And a total length of 12
        let resolved = t.resolve_full(&d_offset)?;
        assert_eq!(12, resolved.len());

        assert_eq!("0",    resolved[0].to_string(&d_offset)?);
        assert_eq!("0",    resolved[1].to_string(&d_offset)?);
        assert_eq!("0",    resolved[2].to_string(&d_offset)?);
        assert_eq!("0",    resolved[3].to_string(&d_offset)?);

        assert_eq!("127",  resolved[4].to_string(&d_offset)?);
        assert_eq!("127",  resolved[5].to_string(&d_offset)?);
        assert_eq!("127",  resolved[6].to_string(&d_offset)?);
        assert_eq!("127",  resolved[7].to_string(&d_offset)?);

        assert_eq!("-128", resolved[8].to_string(&d_offset)?);
        assert_eq!("-128", resolved[9].to_string(&d_offset)?);
        assert_eq!("-1",  resolved[10].to_string(&d_offset)?);
        assert_eq!("-1",  resolved[11].to_string(&d_offset)?);

        Ok(())
    }

    #[test]
    fn test_alignment() -> SimpleResult<()> {
        let data = b"AAAABBBBCCCCDDDD".to_vec();
        let s_offset = ResolveOffset::Static(0);
        let d_offset = ResolveOffset::Dynamic(Context::new(&data));

        // An array of 4 32-bit unsigned integers
        let t = H2Array::new(4,
            H2Number::new_aligned(AlignValue::After(4), SizedDefinition::U8, SizedDisplay::Hex(Default::default()))
        );

        // Even though it's 4x U8 values, with padding it should be 16
        // (We don't want the array itself to be aligned - hence, `Align::No`)
        assert_eq!(16, t.actual_size(&s_offset)?);
        assert_eq!(16, t.actual_size(&d_offset)?);

        let children = t.resolve_partial(&d_offset)?;
        assert_eq!(4, children.len());
        assert_eq!(0..1, children[0].offset);
        assert_eq!("0x41", children[0].to_string(&d_offset)?);

        let resolved = t.resolve_full(&d_offset)?;
        assert_eq!(4, resolved.len());

        assert_eq!(0..1,   resolved[0].offset);
        assert_eq!("0x41", resolved[0].to_string(&d_offset)?);

        assert_eq!(4..5,   resolved[1].offset);
        assert_eq!("0x42", resolved[1].to_string(&d_offset)?);

        assert_eq!(8..9,   resolved[2].offset);
        assert_eq!("0x43", resolved[2].to_string(&d_offset)?);

        assert_eq!(12..13, resolved[3].offset);
        assert_eq!("0x44", resolved[3].to_string(&d_offset)?);

        Ok(())
    }

    #[test]
    fn test_nested_alignment() -> SimpleResult<()> {
        let data = b"AABBCCDDEEFFGGHH".to_vec();
        let s_offset = ResolveOffset::Static(0);
        let d_offset = ResolveOffset::Dynamic(Context::new(&data));

        // An array of 4 elements
        let t = H2Array::new(4,
            // Array of 2 elements, each of which is aligned to a 4-byte boundary
            H2Array::new_aligned(AlignValue::After(4), 2,
                // Each element is a 1-byte hex number aligned to a 2-byte bounary
                H2Number::new_aligned(AlignValue::After(2), SizedDefinition::U8, SizedDisplay::Hex(Default::default()))
            )
        );

        // Even though it's 4x U8 values, with padding it should be 16
        assert_eq!(16, t.actual_size(&s_offset)?);
        assert_eq!(16, t.actual_size(&d_offset)?);

        // Make sure there are 4 direct children
        assert_eq!(4, t.resolve_partial(&d_offset)?.len());

        // Make sure there are 8 total values
        let resolved = t.resolve_full(&d_offset)?;
        assert_eq!(8, resolved.len());

        assert_eq!(0..1,   resolved[0].offset);
        assert_eq!("0x41", resolved[0].to_string(&d_offset)?);

        assert_eq!(2..3,   resolved[1].offset);
        assert_eq!("0x42", resolved[1].to_string(&d_offset)?);

        assert_eq!(4..5,   resolved[2].offset);
        assert_eq!("0x43", resolved[2].to_string(&d_offset)?);

        assert_eq!(6..7,   resolved[3].offset);
        assert_eq!("0x44", resolved[3].to_string(&d_offset)?);

        Ok(())
    }

    // // #[test]
    // // fn test_array_not_starting_at_zero() -> SimpleResult<()> {
    // //     //           ----------- ignored ------------
    // //     let data = b"\x00\x00\x00\x00\x00\x00\x00\x00AAAABBBBCCCCDDDD".to_vec();
    // //     let context = Context::new(&data);

    // //     // An array of 4 32-bit unsigned integers
    // //     let t: StaticType = H2Array::new(4,
    // //         H2Number::new(SizedDefinition::U32(Endian::Big), SizedDisplay::Hex(Default::default())).into()
    // //     ).into();

    // //     assert_eq!(16, t.size());

    // //     let resolved = t.resolve_full(8, None);
    // //     assert_eq!(4, resolved.len());

    // //     assert_eq!(8..12, resolved[0].offset);
    // //     assert_eq!("0x41414141", resolved[0].to_string(&context)?);

    // //     assert_eq!(12..16, resolved[1].offset);
    // //     assert_eq!("0x42424242", resolved[1].to_string(&context)?);

    // //     assert_eq!(16..20, resolved[2].offset);
    // //     assert_eq!("0x43434343", resolved[2].to_string(&context)?);

    // //     assert_eq!(20..24, resolved[3].offset);
    // //     assert_eq!("0x44444444", resolved[3].to_string(&context)?);

    // //     Ok(())
    // // }

    // #[test]
    // fn test_dynamic_array() -> SimpleResult<()> {
    //     Ok(())
    // }
}
