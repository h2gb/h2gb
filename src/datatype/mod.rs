use serde::{Serialize, Deserialize};
use simple_error::{bail, SimpleResult};
use std::ops::Range;

use sized_number::Context;

pub mod basic_type;
pub mod complex_type;
pub mod dynamic_type;

pub mod helpers;

// Allow us to resolve either statically or dynamically, depending on what's
// needed. One or the other might throw an error, though.
// pub enum ResolveOffset<'a> {
//     Static(u64),
//     Dynamic(&'a Context<'a>),
// }

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
    NTString(dynamic_type::ntstring::NTString),
}

pub trait H2TypeTrait {
    // Is the size known ahead of time?
    fn is_static(&self) -> bool;

    // Get the static size, if possible
    fn static_size(&self) -> SimpleResult<u64>;

    // Get "child" nodes (array elements, struct body, etc), if possible
    // Empty vector = a leaf node
    fn children_static(&self, _start: u64) -> SimpleResult<Vec<PartiallyResolvedType>> {
        match self.is_static() {
            true  => Ok(vec![]),
            false => bail!("Can't get children_static() for a non-static type"),
        }
    }

    // Get the user-facing name of the type
    fn name(&self) -> String;

    // Get the actual size, including dynamic parts
    fn size(&self, _context: &Context) -> SimpleResult<u64> {
        self.static_size()
    }

    // Get the children - this will work for static or dynamic types, but is
    // only implemented here for static
    fn children(&self, context: &Context) -> SimpleResult<Vec<PartiallyResolvedType>> {
        match self.is_static() {
            true  => self.children_static(context.position()),
            false => bail!("children() must be implemented on a dynamic type"),
        }
    }

    // Get "related" nodes - ie, what a pointer points to
    fn related(&self, _context: &Context) -> SimpleResult<Vec<(u64, H2Type)>> {
        Ok(vec![])
    }

    // Render as a string
    fn to_string(&self, context: &Context) -> SimpleResult<String>;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PartiallyResolvedType {
    offset: Range<u64>,
    field_name: Option<String>,
    field_type: H2Type,
}

impl PartiallyResolvedType {
    // This is a simpler way to display the type for the right part of the
    // context
    pub fn to_string(&self, context: &Context) -> SimpleResult<String> {
        self.field_type.to_string(&context.at(self.offset.start))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct H2Type {
    field: H2Types,
    byte_alignment: Option<u64>,
}

impl H2Type {
    pub fn new(field: H2Types) -> Self {
        Self {
            field: field,
            byte_alignment: None,
        }
    }

    pub fn new_aligned(byte_alignment: Option<u64>, field: H2Types) -> Self {
        Self {
            byte_alignment: byte_alignment,
            field: field,
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
            H2Types::NTString(t)  => t,
        }
    }

    pub fn resolve(&self, context: &Context) -> SimpleResult<Vec<PartiallyResolvedType>> {
        let children = self.children(context)?;
        let mut result: Vec<PartiallyResolvedType> = Vec::new();

        if children.len() == 0 {
            // No children? Return ourself!
            result.push(PartiallyResolvedType {
                offset: context.position()..(context.position() + self.size(context)?),
                field_name: None,
                field_type: self.clone(),
            });
        } else {
            // Children? Gotta get 'em all!
            for child in children.iter() {
                result.append(&mut child.field_type.resolve(&context.at(child.offset.start))?);
            }
        }

        Ok(result)
    }

    // Get the static size, if possible
    fn static_size(&self) -> SimpleResult<u64> {
        self.field_type().static_size()
        // match self.field_type().static_size() {
        //     Ok(s)   => Ok(helpers::maybe_round_up(s, self.byte_alignment)),
        //     Err(e)  => Err(e),
        // }
    }

    fn aligned_static_size(&self) -> SimpleResult<u64> {
        Ok(helpers::maybe_round_up(self.static_size()?, self.byte_alignment))
    }

    // Get the actual size, including dynamic parts
    fn size(&self, context: &Context) -> SimpleResult<u64> {
        self.field_type().size(context)
        // match self.field_type().size(context) {
        //     Ok(s)  => Ok(helpers::maybe_round_up(s, self.byte_alignment)),
        //     Err(e) => Err(e),
        // }
    }

    fn aligned_size(&self, context: &Context) -> SimpleResult<u64> {
        Ok(helpers::maybe_round_up(self.size(context)?, self.byte_alignment))
    }

    // Is the size known ahead of time?
    fn is_static(&self) -> bool {
        self.field_type().is_static()
    }

    // Get "child" nodes (array elements, struct body, etc), if possible
    // Empty vector = a leaf node
    fn children_static(&self, start: u64) -> SimpleResult<Vec<PartiallyResolvedType>> {
        self.field_type().children_static(start)
    }

    // Get the user-facing name of the type
    fn name(&self) -> String {
        self.field_type().name()
    }

    // Get the children - this will work for static or dynamic types
    fn children(&self, context: &Context) -> SimpleResult<Vec<PartiallyResolvedType>> {
        self.field_type().children(context)
    }

    // Get "related" nodes - ie, what a pointer points to
    fn related(&self, context: &Context) -> SimpleResult<Vec<(u64, H2Type)>> {
        self.field_type().related(context)
    }

    // Render as a string
    fn to_string(&self, context: &Context) -> SimpleResult<String> {
        self.field_type().to_string(context)
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
        let t = H2Type::from(Character::new());
        let data = b"ABCD".to_vec();

        assert_eq!(1, t.static_size()?);
        assert_eq!(1, t.size(&Context::new(&data).at(0))?);
        assert_eq!("A", t.to_string(&Context::new(&data).at(0))?);
        assert_eq!("B", t.to_string(&Context::new(&data).at(1))?);
        assert_eq!("C", t.to_string(&Context::new(&data).at(2))?);
        assert_eq!("D", t.to_string(&Context::new(&data).at(3))?);

        assert_eq!(0, t.children_static(0)?.len());
        assert_eq!(0, t.children(&Context::new(&data).at(0))?.len());

        let resolved = t.resolve(&Context::new(&data).at(0))?;
        assert_eq!(1, resolved.len());
        assert_eq!(0..1, resolved[0].offset);
        assert_eq!("A", resolved[0].to_string(&Context::new(&data))?);

        let resolved = t.resolve(&Context::new(&data).at(1))?;
        assert_eq!(1, resolved.len());
        assert_eq!(1..2, resolved[0].offset);
        assert_eq!("B", resolved[0].to_string(&Context::new(&data))?);

        Ok(())
    }

    #[test]
    fn test_align() -> SimpleResult<()> {
        // Align to 4-byte boundaries
        let t = H2Type::from((4, Character::new()));
        let data = b"ABCD".to_vec();

        assert_eq!(1, t.static_size()?);
        assert_eq!(1, t.size(&Context::new(&data).at(0))?);
        assert_eq!("A", t.to_string(&Context::new(&data).at(0))?);
        assert_eq!("B", t.to_string(&Context::new(&data).at(1))?);
        assert_eq!("C", t.to_string(&Context::new(&data).at(2))?);
        assert_eq!("D", t.to_string(&Context::new(&data).at(3))?);

        assert_eq!(0, t.children_static(0)?.len());
        assert_eq!(0, t.children(&Context::new(&data).at(0))?.len());

        let resolved = t.resolve(&Context::new(&data).at(0))?;
        assert_eq!(1, resolved.len());
        assert_eq!(0..1, resolved[0].offset);
        assert_eq!("A", resolved[0].to_string(&Context::new(&data))?);

        let resolved = t.resolve(&Context::new(&data).at(1))?;
        assert_eq!(1, resolved.len());
        assert_eq!(1..2, resolved[0].offset);
        assert_eq!("B", resolved[0].to_string(&Context::new(&data))?);

        Ok(())
    }

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
