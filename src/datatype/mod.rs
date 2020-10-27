use serde::{Serialize, Deserialize};
use simple_error::{bail, SimpleResult};
use std::ops::Range;

use sized_number::Context;

pub mod basic_type;
pub mod complex_type;
pub mod dynamic_type;

pub mod helpers;

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

    fn as_trait(&self) -> Box<&dyn H2TypeTrait>
    where Self: Sized
    {
        Box::new(self)
    }

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

    pub fn as_trait(&self) -> Box<&dyn H2TypeTrait> {
        match &self.field {
            // Basic
            H2Types::H2Number(t)  => t.as_trait(),
            H2Types::H2Pointer(t) => t.as_trait(),
            H2Types::Character(t) => t.as_trait(),
            H2Types::IPv4(t)      => t.as_trait(),
            H2Types::IPv6(t)      => t.as_trait(),
            H2Types::Unicode(t)   => t.as_trait(),

            // Complex
            H2Types::H2Array(t)   => t.as_trait(),

            // Dynamic
            H2Types::NTString(t)  => t.as_trait(),
        }
    }

    pub fn resolve(&self, context: &Context) -> SimpleResult<Vec<PartiallyResolvedType>> {
        let mut children = self.children(context)?;
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
            result.append(&mut children);
        }

        Ok(result)
    }
}

impl H2TypeTrait for H2Type {
    // Is the size known ahead of time?
    fn is_static(&self) -> bool {
        self.as_trait().is_static()
    }

    // Get the static size, if possible
    fn static_size(&self) -> SimpleResult<u64> {
        match self.as_trait().static_size() {
            Ok(s)   => Ok(helpers::maybe_round_up(s, self.byte_alignment)),
            Err(e)  => Err(e),
        }
    }

    // Get "child" nodes (array elements, struct body, etc), if possible
    // Empty vector = a leaf node
    fn children_static(&self, start: u64) -> SimpleResult<Vec<PartiallyResolvedType>> {
        self.as_trait().children_static(start)
    }

    // Get the user-facing name of the type
    fn name(&self) -> String {
        self.as_trait().name()
    }

    // Get the actual size, including dynamic parts
    fn size(&self, context: &Context) -> SimpleResult<u64> {
        match self.as_trait().size(context) {
            Ok(s)  => Ok(helpers::maybe_round_up(s, self.byte_alignment)),
            Err(e) => Err(e),
        }
    }

    // Get the children - this will work for static or dynamic types
    fn children(&self, context: &Context) -> SimpleResult<Vec<PartiallyResolvedType>> {
        self.as_trait().children(context)
    }

    // Get "related" nodes - ie, what a pointer points to
    fn related(&self, context: &Context) -> SimpleResult<Vec<(u64, H2Type)>> {
        self.as_trait().related(context)
    }

    // Render as a string
    fn to_string(&self, context: &Context) -> SimpleResult<String> {
        self.as_trait().to_string(context)
    }
}
