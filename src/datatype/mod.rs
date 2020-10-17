use serde::{Serialize, Deserialize};
use simple_error::SimpleResult;
use std::ops::Range;

use sized_number::Context;

pub mod basic_type;
use basic_type::H2BasicType;

pub mod static_type;
use static_type::StaticType;

pub mod dynamic_type;

pub mod helpers;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResolvedType {
    offset: Range<u64>,
    breadcrumbs: Option<Vec<String>>,
    basic_type: H2BasicType,
}

impl ResolvedType {
    // This is a simpler way to display the type for the right part of the
    // context
    pub fn to_string(&self, context: &Context) -> SimpleResult<String> {
        self.basic_type.to_string(&context.at(self.offset.start))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PartiallyResolvedType {
    offset: Range<u64>,
    field_name: Option<String>,
    field_type: StaticType,
}

impl PartiallyResolvedType {
    fn new(start: u64, field_name: Option<String>, field_type: StaticType) -> Self {
        PartiallyResolvedType {
            offset: start..(start + field_type.size()),
            field_name: field_name,
            field_type: field_type,
        }
    }
}


impl PartiallyResolvedType {
    // This is a simpler way to display the type for the right part of the
    // context
    pub fn to_string(&self, context: &Context) -> SimpleResult<String> {
        self.field_type.to_string(&context.at(self.offset.start))
    }
}
