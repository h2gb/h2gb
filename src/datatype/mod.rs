use serde::{Serialize, Deserialize};
use simple_error::{bail, SimpleResult};
use std::ops::Range;

use sized_number::Context;

pub mod alignment;
use alignment::Alignment;

pub mod basic_type;
pub mod complex_type;
// pub mod dynamic_type;

// Allow us to resolve either statically or dynamically, depending on what's
// needed. One or the other might throw an error, though.
pub enum ResolveOffset<'a> {
    Static(u64),
    Dynamic(Context<'a>),
}

impl<'a> From<u64> for ResolveOffset<'a> {
    fn from(o: u64) -> ResolveOffset<'a> {
        ResolveOffset::Static(o)
    }
}

impl<'a> From<Context<'a>> for ResolveOffset<'a> {
    fn from(o: Context<'a>) -> ResolveOffset<'a> {
        ResolveOffset::Dynamic(o)
    }
}

impl<'a> ResolveOffset<'a> {
    pub fn position(&self) -> u64 {
        match self {
            Self::Static(n) => *n,
            Self::Dynamic(c) => c.position(),
        }
    }

    pub fn at(&self, offset: u64) -> ResolveOffset {
        match self {
            Self::Static(_) => Self::Static(offset),
            Self::Dynamic(c) => Self::Dynamic(c.at(offset)),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum H2Types {
    // Basic
    H2Number(basic_type::h2number::H2Number),
    H2Pointer(basic_type::h2pointer::H2Pointer),
    Character(basic_type::character::Character),
    IPv4(basic_type::ipv4::IPv4),
    IPv6(basic_type::ipv6::IPv6),
    Unicode(basic_type::unicode::Unicode),

    // Complex
    H2Array(complex_type::h2array::H2Array),

    // Dynamic
    // NTString(dynamic_type::ntstring::NTString),
}

pub trait H2TypeTrait {
    // Is the size known ahead of time?
    fn is_static(&self) -> bool;

    // Get the static size, if possible
    fn size(&self, offset: &ResolveOffset) -> SimpleResult<u64>;

    // Get "child" nodes (array elements, struct body, etc), if possible
    // Empty vector = a leaf node
    fn resolve_partial(&self, _offset: &ResolveOffset) -> SimpleResult<Vec<ResolvedType>> {
        Ok(vec![])
    }

    // Get the user-facing name of the type
    fn to_string(&self, offset: &ResolveOffset) -> SimpleResult<String>;

    // Get "related" nodes - ie, what a pointer points to
    fn related(&self, _offset: &ResolveOffset) -> SimpleResult<Vec<(u64, H2Type)>> {
        Ok(vec![])
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResolvedType {
    offset: Range<u64>,
    field_name: Option<String>,
    field_type: H2Type,
}

impl ResolvedType {
    // This is a simpler way to display the type for the right part of the
    // context
    pub fn to_string(&self, offset: &ResolveOffset) -> SimpleResult<String> {
        self.field_type.to_string(&offset.at(self.offset.start))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct H2Type {
    field: H2Types,
    alignment: Alignment,
}

impl H2Type {
    pub fn new(alignment: Alignment, field: H2Types) -> Self {
        Self {
            field: field,
            alignment: alignment,
        }
    }

    pub fn field_type(&self) -> &dyn H2TypeTrait {
        match &self.field {
            // Basic
            H2Types::H2Number(t)  => t,
            H2Types::H2Pointer(t) => t,
            H2Types::Character(t) => t,
            H2Types::IPv4(t)      => t,
            H2Types::IPv6(t)      => t,
            H2Types::Unicode(t)   => t,

            // Complex
            H2Types::H2Array(t)   => t,

            // Dynamic
            // H2Types::NTString(t)  => t,
        }
    }

    // Is the size known ahead of time?
    fn is_static(&self) -> bool {
        self.field_type().is_static()
    }

    /// Size of just the field - no padding
    fn actual_size(&self, offset: &ResolveOffset) -> SimpleResult<u64> {
        self.field_type().size(offset)
    }

    /// Range of values this covers, with alignment padding built-in
    fn actual_range(&self, offset: &ResolveOffset) -> SimpleResult<Range<u64>> {
        // Get the start and end
        let start = offset.position();
        let end   = offset.position() + self.actual_size(offset)?;

        // Do the rounding
        Ok(start..end)
    }

    /// Range of values this covers, with alignment padding built-in
    fn aligned_range(&self, offset: &ResolveOffset) -> SimpleResult<Range<u64>> {
        // Get the start and end
        let start = offset.position();
        let end   = offset.position() + self.actual_size(offset)?;

        // Do the rounding
        self.alignment.align(start..end)
    }

    /// Size including padding either before or after
    fn aligned_size(&self, offset: &ResolveOffset) -> SimpleResult<u64> {
        let range = self.aligned_range(offset)?;

        Ok(range.end - range.start)
    }

    fn resolve_partial(&self, offset: &ResolveOffset) -> SimpleResult<Vec<ResolvedType>> {
        self.field_type().resolve_partial(offset)
    }

    // Render as a string
    fn to_string(&self, offset: &ResolveOffset) -> SimpleResult<String> {
        self.field_type().to_string(offset)
    }

    // Get "related" nodes - ie, what a pointer points to
    fn related(&self, offset: &ResolveOffset) -> SimpleResult<Vec<(u64, H2Type)>> {
        self.field_type().related(offset)
    }

    fn resolve_full(&self, offset: &ResolveOffset) -> SimpleResult<Vec<ResolvedType>> {
        let children = self.resolve_partial(offset)?;
        let mut result: Vec<ResolvedType> = Vec::new();

        if children.len() == 0 {
            // No children? Return ourself!
            result.push(ResolvedType {
                offset: offset.position()..(offset.position() + self.actual_size(offset)?),
                field_name: None,
                field_type: self.clone(),
            });
        } else {
            // Children? Gotta get 'em all!
            for child in children.iter() {
                result.append(&mut child.field_type.resolve_full(&offset.at(child.offset.start))?);
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;
    use sized_number::Context;
    use basic_type::character::Character;

    #[test]
    fn test_character() -> SimpleResult<()> {
        let t = Character::new();
        let data = b"ABCD".to_vec();
        let s_offset = ResolveOffset::Static(0);
        let d_offset = ResolveOffset::Dynamic(Context::new(&data));

        assert_eq!(1, t.actual_size(&s_offset)?);
        assert_eq!(1, t.actual_size(&d_offset)?);

        assert_eq!("A", t.to_string(&d_offset.at(0))?);
        assert_eq!("B", t.to_string(&d_offset.at(1))?);
        assert_eq!("C", t.to_string(&d_offset.at(2))?);
        assert_eq!("D", t.to_string(&d_offset.at(3))?);

        assert_eq!(0, t.resolve_partial(&s_offset)?.len());
        assert_eq!(0, t.resolve_partial(&d_offset)?.len());

        let resolved = t.resolve_full(&s_offset)?;
        assert_eq!(1, resolved.len());
        assert_eq!(0..1, resolved[0].offset);
        assert_eq!("Character", resolved[0].to_string(&s_offset)?);

        let resolved = t.resolve_full(&s_offset.at(1))?;
        assert_eq!(1, resolved.len());
        assert_eq!(1..2, resolved[0].offset);
        assert_eq!("Character", resolved[0].to_string(&s_offset)?);

        let resolved = t.resolve_full(&d_offset)?;
        assert_eq!(1, resolved.len());
        assert_eq!(0..1, resolved[0].offset);
        assert_eq!("A", resolved[0].to_string(&d_offset)?);

        let resolved = t.resolve_full(&d_offset.at(1))?;
        assert_eq!(1, resolved.len());
        assert_eq!(1..2, resolved[0].offset);
        assert_eq!("B", resolved[0].to_string(&d_offset)?);

        Ok(())
    }

    // #[test]
    // fn test_align() -> SimpleResult<()> {
    //     // Align to 4-byte boundaries
    //     let t = H2Type::from((4, Character::new()));
    //     let data = b"ABCD".to_vec();
    //     let context = Context::new(&data);

    //     assert_eq!(1, t.size()?);
    //     assert_eq!(1, t.size(&Context::new(&data).at(0))?);
    //     assert_eq!("A", t.to_string(&Context::new(&data).at(0))?);
    //     assert_eq!("B", t.to_string(&Context::new(&data).at(1))?);
    //     assert_eq!("C", t.to_string(&Context::new(&data).at(2))?);
    //     assert_eq!("D", t.to_string(&Context::new(&data).at(3))?);

    //     assert_eq!(0, t.children_static(0)?.len());
    //     assert_eq!(0, t.resolve_partial(&Context::new(&data).at(0))?.len());

    //     let resolved = t.resolve(&Context::new(&data).at(0))?;
    //     assert_eq!(1, resolved.len());
    //     assert_eq!(0..1, resolved[0].offset);
    //     assert_eq!("A", resolved[0].to_string(&Context::new(&data))?);

    //     let resolved = t.resolve(&Context::new(&data).at(1))?;
    //     assert_eq!(1, resolved.len());
    //     assert_eq!(1..2, resolved[0].offset);
    //     assert_eq!("B", resolved[0].to_string(&Context::new(&data))?);

    //     Ok(())
    // }

    #[test]
    fn test_padding() -> SimpleResult<()> {
        Ok(())
    }

    #[test]
    fn test_pointer() -> SimpleResult<()> {
        Ok(())
    }

    #[test]
    fn test_static_array() -> SimpleResult<()> {
        Ok(())
    }

    #[test]
    fn test_dynamic_array() -> SimpleResult<()> {
        Ok(())
    }

    #[test]
    fn test_aligned_array() -> SimpleResult<()> {
        Ok(())
    }

    #[test]
    fn test_static_struct() -> SimpleResult<()> {
        Ok(())
    }

    #[test]
    fn test_dynamic_struct() -> SimpleResult<()> {
        Ok(())
    }

    #[test]
    fn test_enum() -> SimpleResult<()> {
        Ok(())
    }

    #[test]
    fn test_ntstring() -> SimpleResult<()> {
        Ok(())
    }
}
