use serde::{Serialize, Deserialize};
use sized_number::Context;
use simple_error::SimpleResult;

use crate::datatype::{helpers, StaticType, PartiallyResolvedType};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct H2Array {
    field_type: Box<StaticType>,
    length: u64,
    byte_alignment: Option<u64>,
}

impl From<H2Array> for StaticType {
    fn from(o: H2Array) -> StaticType {
        StaticType::from(StaticType::H2Array(o))
    }
}

impl H2Array {
    pub fn new(length: u64, field_type: StaticType) -> Self {
        Self {
            field_type: Box::new(field_type),
            length: length,
            byte_alignment: None,
        }
    }

    pub fn new_aligned(length: u64, alignment: u64, field_type: StaticType) -> Self {
        Self {
            field_type: Box::new(field_type),
            length: length,
            byte_alignment: Some(alignment),
        }
    }

    pub fn partially_resolve(&self, start: u64) -> Vec<PartiallyResolvedType> {
        let mut result = vec![];
        let mut offset: u64 = start;

        for i in 0..self.length {
            // Where the current entry ends
            let end_offset = offset + self.field_type.size();

            // Where the next entry starts
            let next_offset = match self.byte_alignment {
                Some(a) => helpers::round_up(end_offset, a),
                None    => end_offset,
            };

            result.push(PartiallyResolvedType {
                offset: offset..(offset + self.size()),
                field_name: Some(i.to_string()),
                field_type: (*self.field_type).clone(),
            });

            offset = next_offset;
        }

        result
    }

    // Includes alignment
    pub fn size(&self) -> u64 {
        match self.byte_alignment {
            Some(a) => self.length * helpers::round_up(self.field_type.size(), a),
            None => self.length * self.field_type.size(),
        }
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

    use crate::datatype::basic_type::h2number::H2Number;

    #[test]
    fn test_array() -> SimpleResult<()> {
        let data = b"AAAABBBBCCCCDDDD".to_vec();
        let context = Context::new(&data);

        // An array of 4 32-bit unsigned integers
        let t: StaticType = H2Array::new(4,
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
        let t: StaticType = H2Array::new(4,
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

    #[test]
    fn test_alignment() -> SimpleResult<()> {
        let data = b"AAAABBBBCCCCDDDD".to_vec();
        let context = Context::new(&data);

        // An array of 4 32-bit unsigned integers
        let t: StaticType = H2Array::new_aligned(4, 4,
            H2Number::new(SizedDefinition::U8, SizedDisplay::Hex(Default::default())).into()
        ).into();

        // Even though it's 4x U8 values, with padding it should be 16
        assert_eq!(16, t.size());

        let resolved = t.fully_resolve(0, None);
        assert_eq!(4, resolved.len());

        assert_eq!(0..1,   resolved[0].offset);
        assert_eq!("0x41", resolved[0].to_string(&context)?);

        assert_eq!(4..5,   resolved[1].offset);
        assert_eq!("0x42", resolved[1].to_string(&context)?);

        assert_eq!(8..9,   resolved[2].offset);
        assert_eq!("0x43", resolved[2].to_string(&context)?);

        assert_eq!(12..13, resolved[3].offset);
        assert_eq!("0x44", resolved[3].to_string(&context)?);

        Ok(())
    }

    #[test]
    fn test_nested_alignment() -> SimpleResult<()> {
        let data = b"AABBCCDDEEFFGGHH".to_vec();
        let context = Context::new(&data);

        // An array of 4 32-bit unsigned integers
        let t: StaticType = H2Array::new_aligned(4, 4,
            H2Array::new_aligned(2, 2,
                H2Number::new(SizedDefinition::U8, SizedDisplay::Hex(Default::default())).into()
            ).into()
        ).into();

        // Even though it's 4x U8 values, with padding it should be 16
        assert_eq!(16, t.size());

        let resolved = t.fully_resolve(0, None);
        assert_eq!(8, resolved.len());

        assert_eq!(0..1,   resolved[0].offset);
        assert_eq!("0x41", resolved[0].to_string(&context)?);

        assert_eq!(2..3,   resolved[1].offset);
        assert_eq!("0x42", resolved[1].to_string(&context)?);

        assert_eq!(4..5,   resolved[2].offset);
        assert_eq!("0x43", resolved[2].to_string(&context)?);

        assert_eq!(6..7,   resolved[3].offset);
        assert_eq!("0x44", resolved[3].to_string(&context)?);

        Ok(())
    }

    #[test]
    fn test_array_not_starting_at_zero() -> SimpleResult<()> {
        //           ----------- ignored ------------
        let data = b"\x00\x00\x00\x00\x00\x00\x00\x00AAAABBBBCCCCDDDD".to_vec();
        let context = Context::new(&data);

        // An array of 4 32-bit unsigned integers
        let t: StaticType = H2Array::new(4,
            H2Number::new(SizedDefinition::U32(Endian::Big), SizedDisplay::Hex(Default::default())).into()
        ).into();

        assert_eq!(16, t.size());

        let resolved = t.fully_resolve(8, None);
        assert_eq!(4, resolved.len());

        assert_eq!(8..12, resolved[0].offset);
        assert_eq!("0x41414141", resolved[0].to_string(&context)?);

        assert_eq!(12..16, resolved[1].offset);
        assert_eq!("0x42424242", resolved[1].to_string(&context)?);

        assert_eq!(16..20, resolved[2].offset);
        assert_eq!("0x43434343", resolved[2].to_string(&context)?);

        assert_eq!(20..24, resolved[3].offset);
        assert_eq!("0x44444444", resolved[3].to_string(&context)?);

        Ok(())
    }
}
