use serde::{Serialize, Deserialize};
use simple_error::SimpleResult;
use std::ops::Range;

use sized_number::Context;

pub mod composite;
use composite::h2array::H2Array;
use composite::h2enum::H2Enum;
use composite::h2simple::H2Simple;
use composite::h2struct::H2Struct;

pub mod basic;
use basic::H2BasicType;

pub mod helpers;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum H2StaticType {
    H2Array(H2Array),
    H2Enum(H2Enum),
    H2Simple(H2Simple),
    H2Struct(H2Struct),
}

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
    field_type: H2StaticType,
}

impl PartiallyResolvedType {
    // This is a simpler way to display the type for the right part of the
    // context
    pub fn to_string(&self, context: &Context) -> SimpleResult<String> {
        self.field_type.to_string(&context.at(self.offset.start))
    }
}

impl H2StaticType {
    // Resolve "one layer" - ie, to one or more `H2StaticType`s
    pub fn partially_resolve(&self, start: u64) -> Vec<PartiallyResolvedType> {
        match self {
            Self::H2Array(t)  => t.partially_resolve(start),
            Self::H2Enum(t)   => t.partially_resolve(start),
            Self::H2Simple(t) => t.partially_resolve(start),
            Self::H2Struct(t) => t.partially_resolve(start),
        }
    }

    // Resolve right down to `H2BasicType`s
    pub fn fully_resolve(&self, starting_offset: u64, breadcrumbs: Option<Vec<String>>) -> Vec<ResolvedType> {
        match self {
            // If we've made it to H2Simple, we can just return it
            Self::H2Simple(t) => {
                vec![t.to_resolved_type(starting_offset, breadcrumbs)]
            },
            // If it's anything else, we're going to do some recursion
            _ => {
                // Ensure this is always initialized
                let breadcrumbs = breadcrumbs.unwrap_or(Vec::new());

                let mut result: Vec<ResolvedType> = Vec::new();

                for partial in self.partially_resolve(starting_offset).into_iter() {
                    // Update the breadcrumbs
                    let mut new_breadcrumbs = breadcrumbs.clone();

                    if let Some(f) = partial.field_name {
                        new_breadcrumbs.push(f.clone());
                    }

                    // Recurse with each result (until we get to H2Simple)
                    result.append(&mut partial.field_type.fully_resolve(partial.offset.start, Some(new_breadcrumbs)));
                }

                result
            },
        }
    }

    pub fn size(&self) -> u64 {
        match self {
            Self::H2Array(t)  => t.size(),
            Self::H2Enum(t)   => t.size(),
            Self::H2Simple(t) => t.size(),
            Self::H2Struct(t) => t.size(),
        }
    }

    pub fn to_string(&self, context: &Context) -> SimpleResult<String> {
        match self {
            Self::H2Array(t)  => t.to_string(context),
            Self::H2Enum(t)   => t.to_string(context),
            Self::H2Simple(t) => t.to_string(context),
            Self::H2Struct(t) => t.to_string(context),
        }
    }
}
