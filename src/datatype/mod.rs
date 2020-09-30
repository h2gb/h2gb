pub mod basic;
pub mod composite;
pub mod helpers;

use serde::{Serialize, Deserialize};
use simple_error::SimpleResult;

use crate::datatype::basic::H2BasicType;
use crate::datatype::helpers::h2context::H2Context;

use composite::h2struct::H2Struct;
use composite::h2array::H2Array;
use composite::h2simple::H2Simple;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum H2Type {
    H2Struct(H2Struct),
    H2Array(H2Array),
    H2Simple(H2Simple),
}

pub struct ResolvedType {
    offset: usize,
    field_names: Option<Vec<String>>,
    basic_type: H2BasicType,
}

impl H2Type {
    pub fn resolve_from_offset(&self, starting_offset: Option<usize>, field_names: Option<Vec<String>>) -> (Vec<ResolvedType>, usize) {
        match self {
            Self::H2Struct(t) => t.resolve(starting_offset.unwrap_or(0), field_names),
            Self::H2Array(t)  => t.resolve(starting_offset.unwrap_or(0), field_names),
            Self::H2Simple(t) => t.resolve(starting_offset.unwrap_or(0), field_names),
        }
    }

    pub fn resolve(&self) -> Vec<ResolvedType> {
        self.resolve_from_offset(None, None).0
    }

    pub fn size(&self) -> usize {
        match self {
            Self::H2Struct(t) => t.size(),
            Self::H2Array(t)  => t.size(),
            Self::H2Simple(t) => t.size(),
        }
    }

    pub fn to_strings(&self, context: &H2Context) -> SimpleResult<Vec<String>> {
        // This is a simple datatype to clone
        let mut context = context.clone();

        Ok(self.resolve().iter().map(|r| {
            context.set_index(r.offset);

            match &r.field_names {
                Some(f) => format!("{} {} [{}]", r.offset, r.basic_type.to_string(&context).unwrap_or("Invalid".to_string()), f.join(".")),
                None => format!("{} {}", r.offset, r.basic_type.to_string(&context).unwrap_or("Invalid".to_string())),
            }
        }).collect())
    }
}
