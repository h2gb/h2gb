use serde::{Serialize, Deserialize};
use simple_error::{bail, SimpleResult};

use generic_number::Context;

use crate::{Alignment, Data, H2Type, H2TypeTrait};

/// Defines a struct.
///
/// A struct is a series of values with a name and a type that are sequential
/// in memory (with possible alignment).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct H2Struct {
    fields: Vec<(String, H2Type)>,

    #[serde(skip_serializing_if = "Option::is_none")]
    alignment: Option<Alignment>,
}

impl From<H2Struct> for H2Type {
    fn from(t: H2Struct) -> H2Type {
        H2Type::H2Struct(t)
    }
}

impl H2Struct {
    pub fn new_aligned(alignment: Option<Alignment>, fields: Vec<(String, H2Type)>) -> SimpleResult<Self> {
        if fields.len() == 0 {
            bail!("Structs must contain at least one field");
        }

        Ok(Self {
            fields: fields.into_iter().map(|(s, t)| (s, t)).collect(),
            alignment: alignment,
        })
    }

    pub fn new(fields: Vec<(String, H2Type)>) -> SimpleResult<Self> {
        Self::new_aligned(None, fields)
    }
}

impl H2TypeTrait for H2Struct {
    fn children(&self, _context: Context) -> SimpleResult<Vec<(Option<String>, H2Type)>> {
        Ok(self.fields.iter().map(|(name, field_type)| {
            (Some(name.clone()), field_type.clone())
        }).collect())
    }

    fn to_display(&self, context: Context, data: &Data) -> SimpleResult<String> {
        // Because the collect() expects a result, this will end and bubble
        // up errors automatically!
        let strings: Vec<String> = self.children_with_range(context, data)?.iter().map(|(range, name, child)| {
            Ok(format!("{}: {}", name.clone().unwrap_or("<name unknown>".to_string()), child.as_trait(data)?.to_display(context.at(range.start), data)?))
        }).collect::<SimpleResult<Vec<String>>>()?;

        Ok(format!("{{ {} }}", strings.join(", ")))
    }

    fn alignment(&self) -> Option<Alignment> {
        self.alignment
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;
    use generic_number::{Context, IntegerReader, Endian, HexFormatter, OctalFormatter, DefaultFormatter};
    use pretty_assertions::assert_eq;

    use crate::simple::numeric::{H2Integer, H2Character};
    use crate::simple::network::IPv4;
    use crate::composite::H2Array;

    #[test]
    fn test_struct() -> SimpleResult<()> {
        //           ----- hex ------ --hex-- -o-    ----decimal----
        let data = b"\x00\x01\x02\x03\x00\x01p\x0fppp\x0f\x0e\x0d\x0c".to_vec();

        let t = H2Struct::new(vec![
            (
                "field_u32".to_string(),
                H2Integer::new(
                    IntegerReader::U32(Endian::Big),
                    HexFormatter::new_pretty(),
                ).into()
            ),
            (
                "field_u16".to_string(),
                H2Integer::new_aligned(
                    Some(Alignment::Loose(3)),
                    IntegerReader::U16(Endian::Big),
                    HexFormatter::new_pretty(),
                ).into()
            ),
            (
                "field_u8".to_string(),
                H2Integer::new_aligned(
                    Some(Alignment::Loose(4)),
                    IntegerReader::U8,
                    OctalFormatter::new(true, false),
                ).into()
            ),
            (
                "field_u32_little".to_string(),
                H2Integer::new(
                    IntegerReader::U32(Endian::Little),
                    DefaultFormatter::new(),
                ).into()
            ),
        ])?;

        // Use real data
        let context = Context::new(&data);
        assert_eq!(15, t.base_size(context, &Data::default())?);
        assert_eq!(15, t.aligned_size(context, &Data::default())?);
        assert_eq!(0..15, t.base_range(context, &Data::default())?);
        assert_eq!(0..15, t.aligned_range(context, &Data::default())?);
        assert_eq!("{ field_u32: 0x00010203, field_u16: 0x0001, field_u8: 0o17, field_u32_little: 202182159 }", t.to_display(context, &Data::default())?);
        assert_eq!(0, t.related(context)?.len());
        assert_eq!(4, t.children(context)?.len());

        // Resolve and validate the resolved version
        let r = t.resolve(context, None, &Data::default())?;
        assert_eq!(15, r.base_size());
        assert_eq!(15, r.aligned_size());
        assert_eq!(0..15, r.base_range);
        assert_eq!(0..15, r.aligned_range);
        assert_eq!("{ field_u32: 0x00010203, field_u16: 0x0001, field_u8: 0o17, field_u32_little: 202182159 }", r.display);
        assert_eq!(0, r.related.len());
        assert_eq!(4, r.children.len());

        Ok(())
    }

    #[test]
    fn test_nested_struct() -> SimpleResult<()> {
        //              -- hex --  ----------------struct----------------  ----- ipv4 ----
        //                         -A- -B- ---C--- ----- char_array -----
        let data = b"...\x00\x01pp\x41\x42\x43\x43\x61\x62\x63\x64\x65ppp\x7f\x00\x00\x01".to_vec();

        let t = H2Struct::new(vec![
            (
                "hex".to_string(),
                H2Integer::new_aligned(
                    Some(Alignment::Loose(4)),
                    IntegerReader::U16(Endian::Big),
                    HexFormatter::new_pretty(),
                ).into()
            ),
            (
                "struct".to_string(),
                H2Struct::new(vec![
                    (
                        "A".to_string(),
                        H2Integer::new(
                            IntegerReader::U8,
                            HexFormatter::new_pretty(),
                        ).into()
                    ),
                    (
                        "B".to_string(),
                        H2Integer::new(
                            IntegerReader::U8,
                            HexFormatter::new_pretty(),
                        ).into()
                    ),
                    (
                        "C".to_string(),
                        H2Integer::new(
                            IntegerReader::U16(Endian::Big),
                            HexFormatter::new_pretty(),
                        ).into()
                    ),
                    (
                        "char_array".to_string(),
                        H2Array::new_aligned(
                            Some(Alignment::Loose(8)),
                            5,
                            H2Character::new_ascii(),
                        )?.into(),
                    )
                ])?.into(),
            ),
            (
                "ipv4".to_string(),
                IPv4::new(Endian::Big).into(),
            ),
        ])?;

        // Start at 3 to test offsets and alignment
        let context = Context::new_at(&data, 3);
        assert_eq!(20, t.base_size(context, &Data::default())?);
        assert_eq!(20, t.aligned_size(context, &Data::default())?);
        assert_eq!(3..23, t.base_range(context, &Data::default())?);
        assert_eq!(3..23, t.aligned_range(context, &Data::default())?);
        assert_eq!("{ hex: 0x0001, struct: { A: 0x41, B: 0x42, C: 0x4343, char_array: [ 'a', 'b', 'c', 'd', 'e' ] }, ipv4: 127.0.0.1 }", t.to_display(context, &Data::default())?);
        assert_eq!(0, t.related(context)?.len());
        assert_eq!(3, t.children(context)?.len());

        // Make sure it resolves sanely
        let r = t.resolve(context, None, &Data::default())?;
        assert_eq!(20, r.base_size());
        assert_eq!(20, r.aligned_size());
        assert_eq!(3..23, r.base_range);
        assert_eq!(3..23, r.aligned_range);
        assert_eq!("{ hex: 0x0001, struct: { A: 0x41, B: 0x42, C: 0x4343, char_array: [ 'a', 'b', 'c', 'd', 'e' ] }, ipv4: 127.0.0.1 }", r.display);
        assert_eq!(0, r.related.len());
        assert_eq!(3, r.children.len());

        // Check the first child
        assert_eq!(2, r.children[0].base_size());
        assert_eq!(4, r.children[0].aligned_size());
        assert_eq!("0x0001", r.children[0].display);
        assert_eq!(0, r.children[0].children.len());
        assert_eq!("hex", r.children[0].field_name.as_ref().unwrap());

        // Check the second child
        assert_eq!(12, r.children[1].base_size());
        assert_eq!(12, r.children[1].aligned_size());
        assert_eq!("{ A: 0x41, B: 0x42, C: 0x4343, char_array: [ 'a', 'b', 'c', 'd', 'e' ] }", r.children[1].display);
        assert_eq!(4, r.children[1].children.len());
        assert_eq!("struct", r.children[1].field_name.as_ref().unwrap());

        // Check the character array
        assert_eq!(5, r.children[1].children[3].base_size());
        assert_eq!(8, r.children[1].children[3].aligned_size());
        assert_eq!(5, r.children[1].children[3].children.len());
        assert_eq!("char_array", r.children[1].children[3].field_name.as_ref().unwrap());

        Ok(())
    }
}
