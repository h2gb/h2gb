use serde::{Serialize, Deserialize};
use simple_error::{bail, SimpleResult};
use std::ops::Range;

use sized_number::Context;

pub mod basic_type;
use basic_type::H2BasicType;

// pub mod static_type;
// use static_type::StaticType;

// pub mod dynamic_type;
// use dynamic_type::DynamicType;

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
    // Static(StaticType),
    // Dynamic(DynamicType),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct H2Type {
    field: H2Types,
}

impl From<H2BasicType> for H2Type {
    fn from(o: H2BasicType) -> H2Type {
        H2Type::new(H2Types::Basic(o))
    }
}

impl H2Type {
    pub fn new(field: H2Types) -> Self {
        Self {
            field: field,
        }
    }

    pub fn partially_resolve(&self, context: &Context) -> SimpleResult<Vec<PartiallyResolvedType>> {
        match &self.field {
            H2Types::Basic(t)   => t.partially_resolve(context.position()),
            // H2Types::Static(t)  => t.partially_resolve(context.position()),
            // H2Types::Dynamic(t) => t.partially_resolve(context),
        }
    }

    // Resolve right down to `Basic`s
    pub fn fully_resolve(&self, _starting_offset: u64, _breadcrumbs: Option<Vec<String>>) -> SimpleResult<Vec<ResolvedType>> {
        bail!("Not implemented");
    }

    pub fn size(&self, context: &Context) -> SimpleResult<u64> {
        bail!("Not implemented");
    }

    pub fn to_string(&self, context: &Context) -> SimpleResult<String> {
        bail!("Not implemented");
    }
}
