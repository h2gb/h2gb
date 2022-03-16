use serde::{Serialize, Deserialize};
use simple_error::{bail, SimpleResult};

use generic_number::Context;

use crate::{Alignment, Data, H2Type, H2Types, H2TypeTrait};

/// Defines an array of values.
///
/// An array is one or more elements of the same type that are sequential in
/// memory. The type can be either static (the size is known in advance) or
/// dynamic (the size is calculated at runtime and can vary element-to-element).
///
/// Arrays can be nested, can contain
/// [`crate::composite::H2Struct`]s/[`crate::composite::H2Array`]s,
/// and can be as complex or simple as you need.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct H2Array {
    field_type: Box<H2Type>,
    length: usize,
}

impl H2Array {
    pub fn new_aligned(alignment: Alignment, length: usize, field_type: H2Type) -> SimpleResult<H2Type> {
        if length == 0 {
            bail!("Arrays must be at least one element long");
        }

        Ok(H2Type::new(alignment, H2Types::H2Array(Self {
            field_type: Box::new(field_type),
            length: length,
        })))
    }

    pub fn new(length: usize, field_type: H2Type) -> SimpleResult<H2Type> {
        Self::new_aligned(Alignment::None, length, field_type)
    }
}

impl H2TypeTrait for H2Array {
    fn children(&self, _context: Context) -> SimpleResult<Vec<(Option<String>, H2Type)>> {
        // Just clone the child type over and over
        Ok((0..self.length).into_iter().map(|_index| {
            (None, self.field_type.as_ref().clone())
        }).collect())
    }

    fn to_display(&self, context: Context, _data: &Data) -> SimpleResult<String> {
        // Because the collect() expects a result, this will end and bubble
        // up errors automatically!
        let strings: Vec<String> = self.children_with_range(context)?.iter().map(|(range, _name, child)| {
            child.to_display(context.at(range.start), &Data::default())
        }).collect::<SimpleResult<Vec<String>>>()?;

        Ok(format!("[ {} ]", strings.join(", ")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;
    use pretty_assertions::assert_eq;

    use generic_number::{CharacterFormatter, CharacterReader, Context};
    use crate::simple::numeric::H2Character;

    #[test]
    fn test_array_type() -> SimpleResult<()> {
        let data = b"ABCD".to_vec();
        let context = Context::new(&data);

        // Check the basics
        let a = H2Array::new(4, H2Character::new_ascii())?;
        assert_eq!(4, a.base_size(context)?);
        assert_eq!(4, a.aligned_size(context)?);
        assert_eq!(0..4, a.actual_range(context)?);
        assert_eq!(0..4, a.aligned_range(context)?);
        assert_eq!("[ 'A', 'B', 'C', 'D' ]", a.to_display(context, &Data::default())?);
        assert_eq!(0, a.related(context)?.len());
        assert_eq!(4, a.children(context)?.len());

        // Check the resolved version
        let r = a.resolve(context, None, &Data::default())?;
        assert_eq!(4, r.base_size());
        assert_eq!(4, r.aligned_size());
        assert_eq!(0..4, r.actual_range);
        assert_eq!(0..4, r.aligned_range);
        assert_eq!("[ 'A', 'B', 'C', 'D' ]", r.display);
        assert_eq!(0, r.related.len());
        assert_eq!(4, r.children.len());

        // Check the resolved children ranges
        assert_eq!(0..1, r.children[0].aligned_range);
        assert_eq!(1..2, r.children[1].aligned_range);
        assert_eq!(2..3, r.children[2].aligned_range);
        assert_eq!(3..4, r.children[3].aligned_range);

        // Check the resolved children values
        assert_eq!("'A'", r.children[0].display);
        assert_eq!("'B'", r.children[1].display);
        assert_eq!("'C'", r.children[2].display);
        assert_eq!("'D'", r.children[3].display);

        Ok(())
    }

    #[test]
    fn test_array_type_aligned() -> SimpleResult<()> {
        let data = b"ABCD".to_vec();
        let context = Context::new(&data);

        // Check the basics
        let a = H2Array::new_aligned(Alignment::Loose(8), 4, H2Character::new_ascii())?;
        assert_eq!(4, a.base_size(context)?);
        assert_eq!(8, a.aligned_size(context)?);
        assert_eq!(0..4, a.actual_range(context)?);
        assert_eq!(0..8, a.aligned_range(context)?);
        assert_eq!("[ 'A', 'B', 'C', 'D' ]", a.to_display(context, &Data::default())?);
        assert_eq!(0, a.related(context)?.len());
        assert_eq!(4, a.children(context)?.len());

        // Check the resolved version
        let r = a.resolve(context, None, &Data::default())?;
        assert_eq!(4, r.base_size());
        assert_eq!(8, r.aligned_size());
        assert_eq!(0..4, r.actual_range);
        assert_eq!(0..8, r.aligned_range);
        assert_eq!("[ 'A', 'B', 'C', 'D' ]", r.display);
        assert_eq!(0, r.related.len());
        assert_eq!(4, r.children.len());

        // Check the resolved children ranges
        assert_eq!(0..1, r.children[0].aligned_range);
        assert_eq!(1..2, r.children[1].aligned_range);
        assert_eq!(2..3, r.children[2].aligned_range);
        assert_eq!(3..4, r.children[3].aligned_range);

        // Check the resolved children values
        assert_eq!("'A'", r.children[0].display);
        assert_eq!("'B'", r.children[1].display);
        assert_eq!("'C'", r.children[2].display);
        assert_eq!("'D'", r.children[3].display);

        Ok(())
    }

    #[test]
    fn test_array_type_aligned_elements() -> SimpleResult<()> {
        let data = b"AxxxBxxxCxxxDxxx".to_vec();
        let context = Context::new(&data);

        // Check the basics
        let a = H2Array::new(
            4,
            H2Character::new_aligned(
                Alignment::Loose(4),
                CharacterReader::ASCII,
                CharacterFormatter::new_pretty(),
            ),
        )?;
        assert_eq!(16,  a.base_size(context)?);
        assert_eq!(16, a.aligned_size(context)?);
        assert_eq!(0..16,  a.actual_range(context)?);
        assert_eq!(0..16, a.aligned_range(context)?);
        assert_eq!("[ 'A', 'B', 'C', 'D' ]", a.to_display(context, &Data::default())?);
        assert_eq!(0, a.related(context)?.len());
        assert_eq!(4, a.children(context)?.len());

        // Check the resolved version
        let r = a.resolve(context, None, &Data::default())?;
        assert_eq!(16, r.base_size());
        assert_eq!(16, r.aligned_size());
        assert_eq!(0..16, r.actual_range);
        assert_eq!(0..16, r.aligned_range);
        assert_eq!("[ 'A', 'B', 'C', 'D' ]", r.display);
        assert_eq!(0, r.related.len());
        assert_eq!(4, r.children.len());

        // Check the resolved children ranges
        assert_eq!(0..1,   r.children[0].actual_range);
        assert_eq!(4..5,   r.children[1].actual_range);
        assert_eq!(8..9,   r.children[2].actual_range);
        assert_eq!(12..13, r.children[3].actual_range);

        // Make sure the aligned range is right
        assert_eq!(0..4,   r.children[0].aligned_range);
        assert_eq!(4..8,   r.children[1].aligned_range);
        assert_eq!(8..12,  r.children[2].aligned_range);
        assert_eq!(12..16, r.children[3].aligned_range);

        // Check the resolved children values
        assert_eq!("'A'", r.children[0].display);
        assert_eq!("'B'", r.children[1].display);
        assert_eq!("'C'", r.children[2].display);
        assert_eq!("'D'", r.children[3].display);

        Ok(())
    }

    #[test]
    fn test_array_type_aligned_and_aligned_elements() -> SimpleResult<()> {
        let data = b"AxxxBxxxCxxxDxxx".to_vec();
        let context = Context::new(&data);

        // Check the basics (align to 5, which is awkward but easy to check)
        let a = H2Array::new_aligned(
            Alignment::Loose(5),
            4,
            H2Character::new_aligned(
                Alignment::Loose(4),
                CharacterReader::ASCII,
                CharacterFormatter::new_pretty(),
            ),
        )?;
        assert_eq!(16,  a.base_size(context)?);
        assert_eq!(20, a.aligned_size(context)?);
        assert_eq!(0..16,  a.actual_range(context)?);
        assert_eq!(0..20, a.aligned_range(context)?);
        assert_eq!("[ 'A', 'B', 'C', 'D' ]", a.to_display(context, &Data::default())?);
        assert_eq!(0, a.related(context)?.len());
        assert_eq!(4, a.children(context)?.len());

        // Check the resolved version
        let r = a.resolve(context, None, &Data::default())?;
        assert_eq!(16, r.base_size());
        assert_eq!(20, r.aligned_size());
        assert_eq!(0..16, r.actual_range);
        assert_eq!(0..20, r.aligned_range);
        assert_eq!("[ 'A', 'B', 'C', 'D' ]", r.display);
        assert_eq!(0, r.related.len());
        assert_eq!(4, r.children.len());

        // Check the resolved children ranges
        assert_eq!(0..1,   r.children[0].actual_range);
        assert_eq!(4..5,   r.children[1].actual_range);
        assert_eq!(8..9,   r.children[2].actual_range);
        assert_eq!(12..13, r.children[3].actual_range);

        // Make sure the aligned range is right
        assert_eq!(0..4,   r.children[0].aligned_range);
        assert_eq!(4..8,   r.children[1].aligned_range);
        assert_eq!(8..12,  r.children[2].aligned_range);
        assert_eq!(12..16, r.children[3].aligned_range);

        // Check the resolved children values
        assert_eq!("'A'", r.children[0].display);
        assert_eq!("'B'", r.children[1].display);
        assert_eq!("'C'", r.children[2].display);
        assert_eq!("'D'", r.children[3].display);

        Ok(())
    }

    #[test]
    fn test_array_type_aligned_and_context_elements() -> SimpleResult<()> {
        let data = b"xAxxxBxxxCxxxDxx".to_vec();
        let context = Context::new(&data).at(1);

        let a = H2Array::new(
            4,
            H2Character::new_aligned(
                Alignment::Loose(4),
                CharacterReader::ASCII,
                CharacterFormatter::new_pretty(),
            ),
        )?;
        assert_eq!(16,  a.base_size(context)?);
        assert_eq!(16, a.aligned_size(context)?);
        assert_eq!(1..17,  a.actual_range(context)?);
        assert_eq!(1..17, a.aligned_range(context)?);
        assert_eq!("[ 'A', 'B', 'C', 'D' ]", a.to_display(context, &Data::default())?);
        assert_eq!(0, a.related(context)?.len());
        assert_eq!(4, a.children(context)?.len());

        // Check the resolved version
        let r = a.resolve(context, None, &Data::default())?;
        assert_eq!(16, r.base_size());
        assert_eq!(16, r.aligned_size());
        assert_eq!(1..17, r.actual_range);
        assert_eq!(1..17, r.aligned_range);
        assert_eq!("[ 'A', 'B', 'C', 'D' ]", r.display);
        assert_eq!(0, r.related.len());
        assert_eq!(4, r.children.len());

        // Check the resolved children ranges
        assert_eq!(1..2,   r.children[0].actual_range);
        assert_eq!(5..6,   r.children[1].actual_range);
        assert_eq!(9..10,  r.children[2].actual_range);
        assert_eq!(13..14, r.children[3].actual_range);

        // Make sure the aligned range is right
        assert_eq!(1..5,   r.children[0].aligned_range);
        assert_eq!(5..9,   r.children[1].aligned_range);
        assert_eq!(9..13,  r.children[2].aligned_range);
        assert_eq!(13..17, r.children[3].aligned_range);

        // Check the resolved children values
        assert_eq!("'A'", r.children[0].display);
        assert_eq!("'B'", r.children[1].display);
        assert_eq!("'C'", r.children[2].display);
        assert_eq!("'D'", r.children[3].display);

        Ok(())
    }

    #[test]
    fn test_dynamic_utf8_array() -> SimpleResult<()> {
        //             --  --  ----------  ----------  --------------  --------------  ------
        let data = b"\x41\x42\xE2\x9D\x84\xE2\x98\xA2\xF0\x9D\x84\x9E\xF0\x9F\x98\x88\xc3\xb7".to_vec();
        let context = Context::new(&data);

        let a = H2Array::new(7, H2Character::new_utf8())?;
        assert_eq!(18, a.base_size(context)?);
        assert_eq!("[ 'A', 'B', '❄', '☢', '𝄞', '😈', '÷' ]", a.to_display(context, &Data::default())?);

        Ok(())
    }
}
