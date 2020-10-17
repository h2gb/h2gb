use serde::{Serialize, Deserialize};
use simple_error::SimpleResult;

use sized_number::Context;

pub mod h2array;
use h2array::H2Array;

pub mod h2enum;
use h2enum::H2Enum;

pub mod h2simple;
use h2simple::H2Simple;

pub mod h2struct;
use h2struct::H2Struct;

use crate::datatype::{ResolvedType, PartiallyResolvedType};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum StaticType {
    H2Array(H2Array),
    H2Enum(H2Enum),
    H2Simple(H2Simple),
    H2Struct(H2Struct),
}

impl StaticType {
    // Resolve "one layer" - ie, to one or more `StaticType`s
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
