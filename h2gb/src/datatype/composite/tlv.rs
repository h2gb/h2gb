use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use simple_error::{bail, SimpleResult};

use crate::datatype::{Alignment, H2Type, H2Types, H2TypeTrait, Offset};
use crate::datatype::simple::H2Number;
use crate::datatype::composite::H2Array;
use generic_number::{GenericReader, HexFormatter};

/// Defines a struct.
///
/// A struct is a series of values with a name and a type that are sequential
/// in memory (with possible alignment).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct H2TLV {
    type_field: Box<H2Type>,
    length_field: Box<H2Type>,
    value_fields: HashMap<u64, H2Type>,
}

impl H2TLV {
    pub fn new_aligned(alignment: Alignment, type_field: H2Type, length_field: H2Type, value_fields: HashMap<u64, H2Type>) -> SimpleResult<H2Type> {
        if !type_field.can_be_u64() {
            bail!("Type field isn't u64-compatible!");
        }

        if !length_field.can_be_u64() {
            bail!("Length field isn't u64-compatible!");
        }

        Ok(H2Type::new(alignment, H2Types::H2TLV(Self {
            type_field: Box::new(type_field),
            length_field: Box::new(length_field),
            value_fields: value_fields,
        })))
    }

    pub fn new(type_field: H2Type, length_field: H2Type, value_fields: HashMap<u64, H2Type>) -> SimpleResult<H2Type> {
        Self::new_aligned(Alignment::None, type_field, length_field, value_fields)
    }
}

impl H2TypeTrait for H2TLV {
    fn is_static(&self) -> bool {
        false
    }

    fn children(&self, offset: Offset) -> SimpleResult<Vec<(Option<String>, H2Type)>> {
        // The first field is the type
        let actual_type = self.type_field.to_number(offset)?;

        // The length is immediately after the type
        let actual_length = self.length_field.to_number(offset.at(offset.position() + self.type_field.actual_size(offset)?))?;

        // Either use the type field, or an array of U8's as the value
        let value_field = match self.value_fields.get(&actual_type.as_u64()?) {
            Some(v) => v.clone(),
            None => H2Array::new(actual_length.as_u64()?, H2Number::new(GenericReader::U8, HexFormatter::pretty()))?,
        };

        Ok(vec![
            (Some("Type".to_string()),   self.type_field.as_ref().clone()),
            (Some("Length".to_string()), self.length_field.as_ref().clone()),
            (Some("Value".to_string()),  value_field),
        ])
    }

    fn to_display(&self, offset: Offset) -> SimpleResult<String> {
        // Because the collect() expects a result, this will end and bubble
        // up errors automatically!
        let strings: Vec<String> = self.children_with_range(offset)?.iter().map(|(range, name, child)| {
            Ok(format!("{}: {}", name.clone().unwrap_or("<name unknown>".to_string()), child.to_display(offset.at(range.start))?))
        }).collect::<SimpleResult<Vec<String>>>()?;

        Ok(format!("{{ {} }}", strings.join(", ")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;
    use generic_number::{Context, GenericReader, HexFormatter, Endian};
    use crate::datatype::simple::H2Number;

    #[test]
    fn test_tlv_numbers() -> SimpleResult<()> {
        let data = b"\x00\x01A\x01\x02AA\x02\x04AAAA".to_vec();
        let offset = Offset::Dynamic(Context::new(&data));

        let mut types: HashMap<u64, H2Type> = HashMap::new();
        types.insert(0, H2Number::new(GenericReader::U8, HexFormatter::pretty()));
        types.insert(1, H2Number::new(GenericReader::U16(Endian::Big), HexFormatter::pretty()));
        types.insert(2, H2Number::new(GenericReader::U32(Endian::Big), HexFormatter::pretty()));

        let t = H2TLV::new(
            H2Number::new(GenericReader::U8, HexFormatter::pretty()), // Type
            H2Number::new(GenericReader::U8, HexFormatter::pretty()), // Length
            types,
        )?;

        let resolved = t.resolve(offset, None)?;
        assert_eq!(resolved.actual_range, 0..3);
        assert_eq!(resolved.display, "{ Type: 0x00, Length: 0x01, Value: 0x41 }");
        assert_eq!(resolved.children.len(), 3);

        let offset = offset.at(offset.position() + t.aligned_size(offset)?);
        let resolved = t.resolve(offset, None)?;
        assert_eq!(resolved.actual_range, 3..7);
        assert_eq!(resolved.display, "{ Type: 0x01, Length: 0x02, Value: 0x4141 }");
        assert_eq!(resolved.children.len(), 3);

        Ok(())
    }
}
