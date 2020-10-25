use serde::{Serialize, Deserialize};
use simple_error::{bail, SimpleResult};
use std::ops::Range;

use sized_number::Context;

pub mod basic_type;
use basic_type::H2BasicType;

pub mod complex_type;
use complex_type::H2ComplexType;

pub mod dynamic_type;
use dynamic_type::H2DynamicType;

pub mod helpers;

pub trait H2TypeTrait {
    // Is the size known ahead of time?
    fn is_static(&self) -> bool;

    // Get the static size, if possible
    fn static_size(&self) -> Option<u64>;

    // Get "child" nodes (array elements, struct body, etc), if possible
    // Empty vector = a leaf node
    fn children_static(&self, start: u64) -> Option<Vec<PartiallyResolvedType>> {
        match self.is_static() {
            true  => Some(vec![]),
            false => None,
        }
    }

    // Get the user-facing name of the type
    fn name(&self) -> String;

    // Get the actual size, including dynamic parts
    fn size(&self, context: &Context) -> SimpleResult<u64> {
        match self.static_size() {
            Some(s) => Ok(s),
            None => bail!("No size() implementation on {}", self.name())
        }
    }

    // Get the children - this will work for static or dynamic types
    fn children(&self, context: &Context) -> SimpleResult<Option<Vec<PartiallyResolvedType>>> {
        match self.is_static() {
            true  => Ok(self.children_static(context.position())),
            false => Ok(None),
        }
    }

    // Get "related" nodes - ie, what a pointer points to
    fn related(&self, _context: &Context) -> SimpleResult<Vec<(u64, H2Type)>> {
        Ok(vec![])
    }

    // Render as a string
    fn to_string(&self, context: &Context) -> SimpleResult<String>;
}

// #[derive(Serialize, Deserialize, Debug, Clone)]
// pub struct ResolvedType {
//     offset: Range<u64>,
//     breadcrumbs: Option<Vec<String>>,
//     basic_type: H2BasicType,
// }

// impl ResolvedType {
//     // This is a simpler way to display the type for the right part of the
//     // context
//     pub fn to_string(&self, context: &Context) -> SimpleResult<String> {
//         self.basic_type.to_string(&context.at(self.offset.start))
//     }
// }

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
pub enum H2Types {
    Basic(H2BasicType),
    Complex(H2ComplexType),
    Dynamic(H2DynamicType),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct H2Type {
    field: H2Types,
    byte_alignment: Option<u64>,
}

impl From<H2BasicType> for H2Type {
    fn from(o: H2BasicType) -> H2Type {
        H2Type::new(H2Types::Basic(o))
    }
}

impl From<H2ComplexType> for H2Type {
    fn from(o: H2ComplexType) -> H2Type {
        H2Type::new(H2Types::Complex(o))
    }
}

impl From<H2DynamicType> for H2Type {
    fn from(o: H2DynamicType) -> H2Type {
        H2Type::new(H2Types::Dynamic(o))
    }
}

impl H2Type {
    pub fn new(field: H2Types) -> Self {
        Self {
            field: field,
            byte_alignment: None,
        }
    }

    pub fn as_trait(&self) -> Box<&dyn H2TypeTrait> {
        match &self.field {
            H2Types::Basic(t)   => t.as_trait(),
            H2Types::Complex(t) => t.as_trait(),
            H2Types::Dynamic(t) => t.as_trait(),
        }
    }

    pub fn resolve(&self) -> SimpleResult<()> {
        bail!("Not implemented");
    }
}

impl H2TypeTrait for H2Type {
    // Is the size known ahead of time?
    fn is_static(&self) -> bool {
        self.as_trait().is_static()
    }

    // Get the static size, if possible
    fn static_size(&self) -> Option<u64> {
        match self.as_trait().static_size() {
            Some(s) => Some(helpers::maybe_round_up(s, self.byte_alignment)),
            None    => None,
        }
    }

    // Get "child" nodes (array elements, struct body, etc), if possible
    // Empty vector = a leaf node
    fn children_static(&self, start: u64) -> Option<Vec<PartiallyResolvedType>> {
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
    fn children(&self, context: &Context) -> SimpleResult<Option<Vec<PartiallyResolvedType>>> {
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
