use serde::{Serialize, Deserialize};

use simple_error::{bail, SimpleResult};

use crate::datatype::{Alignment, H2Type, H2Types, H2TypeTrait, Offset};

/// Defines a struct.
///
/// A struct is a series of values with a name and a type that are sequential
/// in memory (with possible alignment).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct H2Struct {
    fields: Vec<(String, H2Type)>,
}

impl H2Struct {
    pub fn new_aligned(alignment: Alignment, fields: Vec<(String, H2Type)>) -> SimpleResult<H2Type> {
        if fields.len() == 0 {
            bail!("Structs must contain at least one field");
        }

        Ok(H2Type::new(alignment, H2Types::H2Struct(Self {
            fields: fields
        })))
    }

    pub fn new(fields: Vec<(String, H2Type)>) -> SimpleResult<H2Type> {
        Self::new_aligned(Alignment::None, fields)
    }
}

impl H2TypeTrait for H2Struct {
    // Is the size known ahead of time?
    fn is_static(&self) -> bool {
        // Loop over each field - return an object as soon as is_static() is
        // false
        self.fields.iter().find(|(_, t)| {
            t.is_static() == false
        }).is_none()
    }

    fn children(&self, _offset: Offset) -> SimpleResult<Vec<(Option<String>, H2Type)>> {
        Ok(self.fields.iter().map(|(name, field_type)| {
            (Some(name.clone()), field_type.clone())
        }).collect())
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
    use generic_number::{Context, GenericReader, Endian, HexFormatter, OctalFormatter, DefaultFormatter};
    use crate::datatype::simple::H2Number;
    use crate::datatype::simple::network::IPv4;
    use crate::datatype::composite::H2Array;

    #[test]
    fn test_struct() -> SimpleResult<()> {
        //           ----- hex ------ --hex-- -o-    ----decimal----
        let data = b"\x00\x01\x02\x03\x00\x01p\x0fppp\x0f\x0e\x0d\x0c".to_vec();

        let t = H2Struct::new(vec![
            (
                "field_u32".to_string(),
                H2Number::new(
                    GenericReader::U32(Endian::Big),
                    HexFormatter::pretty(),
                )
            ),
            (
                "field_u16".to_string(),
                H2Number::new_aligned(
                    Alignment::Loose(3),
                    GenericReader::U16(Endian::Big),
                    HexFormatter::pretty(),
                )
            ),
            (
                "field_u8".to_string(),
                H2Number::new_aligned(
                    Alignment::Loose(4),
                    GenericReader::U8,
                    OctalFormatter::new(true, false),
                )
            ),
            (
                "field_u32_little".to_string(),
                H2Number::new(
                    GenericReader::U32(Endian::Little),
                    DefaultFormatter::new(),
                )
            ),
        ])?;

        // Use real data
        let offset = Offset::Dynamic(Context::new(&data));
        assert_eq!(true, t.is_static());
        assert_eq!(15, t.actual_size(offset)?);
        assert_eq!(15, t.aligned_size(offset)?);
        assert_eq!(0..15, t.actual_range(offset)?);
        assert_eq!(0..15, t.aligned_range(offset)?);
        assert_eq!("{ field_u32: 0x00010203, field_u16: 0x0001, field_u8: 0o17, field_u32_little: 202182159 }", t.to_display(offset)?);
        assert_eq!(0, t.related(offset)?.len());
        assert_eq!(4, t.children(offset)?.len());

        // Resolve and validate the resolved version
        let r = t.resolve(offset, None)?;
        assert_eq!(15, r.actual_size());
        assert_eq!(15, r.aligned_size());
        assert_eq!(0..15, r.actual_range);
        assert_eq!(0..15, r.aligned_range);
        assert_eq!("{ field_u32: 0x00010203, field_u16: 0x0001, field_u8: 0o17, field_u32_little: 202182159 }", r.display);
        assert_eq!(0, r.related.len());
        assert_eq!(4, r.children.len());

        // Use abstract data
        let offset = Offset::Static(0);
        assert_eq!(true, t.is_static());
        assert_eq!(15, t.actual_size(offset)?);
        assert_eq!(15, t.aligned_size(offset)?);
        assert_eq!(0..15, t.actual_range(offset)?);
        assert_eq!(0..15, t.aligned_range(offset)?);
        assert_eq!("{ field_u32: Number, field_u16: Number, field_u8: Number, field_u32_little: Number }", t.to_display(offset)?);
        assert_eq!(0, t.related(offset)?.len());
        assert_eq!(4, t.children(offset)?.len());

        // Resolve and validate the resolved version
        let r = t.resolve(offset, None)?;
        assert_eq!(15, r.actual_size());
        assert_eq!(15, r.aligned_size());
        assert_eq!(0..15, r.actual_range);
        assert_eq!(0..15, r.aligned_range);
        assert_eq!("{ field_u32: Number, field_u16: Number, field_u8: Number, field_u32_little: Number }", r.display);
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
                H2Number::new_aligned(
                    Alignment::Loose(4),
                    GenericReader::U16(Endian::Big),
                    HexFormatter::pretty(),
                )
            ),
            (
                "struct".to_string(),
                H2Struct::new(vec![
                    (
                        "A".to_string(),
                        H2Number::new(
                            GenericReader::U8,
                            HexFormatter::pretty(),
                        )
                    ),
                    (
                        "B".to_string(),
                        H2Number::new(
                            GenericReader::U8,
                            HexFormatter::pretty(),
                        )
                    ),
                    (
                        "C".to_string(),
                        H2Number::new(
                            GenericReader::U16(Endian::Big),
                            HexFormatter::pretty(),
                        )
                    ),
                    (
                        "char_array".to_string(),
                        H2Array::new_aligned(
                            Alignment::Loose(8),
                            5,
                            H2Number::new_ascii(),
                        )?,
                    )
                ])?,
            ),
            (
                "ipv4".to_string(),
                IPv4::new(Endian::Big)
            ),
        ])?;

        // Start at 3 to test offsets and alignment
        let offset = Offset::Dynamic(Context::new_at(&data, 3));
        assert_eq!(true, t.is_static());
        assert_eq!(20, t.actual_size(offset)?);
        assert_eq!(20, t.aligned_size(offset)?);
        assert_eq!(3..23, t.actual_range(offset)?);
        assert_eq!(3..23, t.aligned_range(offset)?);
        assert_eq!("{ hex: 0x0001, struct: { A: 0x41, B: 0x42, C: 0x4343, char_array: [ 'a', 'b', 'c', 'd', 'e' ] }, ipv4: 127.0.0.1 }", t.to_display(offset)?);
        assert_eq!(0, t.related(offset)?.len());
        assert_eq!(3, t.children(offset)?.len());

        // Make sure it resolves sanely
        let r = t.resolve(offset, None)?;
        assert_eq!(20, r.actual_size());
        assert_eq!(20, r.aligned_size());
        assert_eq!(3..23, r.actual_range);
        assert_eq!(3..23, r.aligned_range);
        assert_eq!("{ hex: 0x0001, struct: { A: 0x41, B: 0x42, C: 0x4343, char_array: [ 'a', 'b', 'c', 'd', 'e' ] }, ipv4: 127.0.0.1 }", r.display);
        assert_eq!(0, r.related.len());
        assert_eq!(3, r.children.len());

        // Check the first child
        assert_eq!(2, r.children[0].actual_size());
        assert_eq!(4, r.children[0].aligned_size());
        assert_eq!("0x0001", r.children[0].display);
        assert_eq!(0, r.children[0].children.len());
        assert_eq!("hex", r.children[0].field_name.as_ref().unwrap());

        // Check the second child
        assert_eq!(12, r.children[1].actual_size());
        assert_eq!(12, r.children[1].aligned_size());
        assert_eq!("{ A: 0x41, B: 0x42, C: 0x4343, char_array: [ 'a', 'b', 'c', 'd', 'e' ] }", r.children[1].display);
        assert_eq!(4, r.children[1].children.len());
        assert_eq!("struct", r.children[1].field_name.as_ref().unwrap());

        // Check the character array
        assert_eq!(5, r.children[1].children[3].actual_size());
        assert_eq!(8, r.children[1].children[3].aligned_size());
        assert_eq!(5, r.children[1].children[3].children.len());
        assert_eq!("char_array", r.children[1].children[3].field_name.as_ref().unwrap());

        Ok(())
    }
}
