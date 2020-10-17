use serde::{Serialize, Deserialize};
use simple_error::SimpleResult;

use sized_number::Context;

pub mod ntstring;
use ntstring::NTString;

use crate::datatype::{ResolvedType, PartiallyResolvedType};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DynamicType {
    NTString(NTString),
}

impl DynamicType {
    // Resolve "one layer" - ie, to one or more `DynamicType`s
    pub fn partially_resolve(&self, context: &Context) -> SimpleResult<Vec<PartiallyResolvedType>> {
        match self {
            Self::NTString(t) => t.partially_resolve(context),
        }
    }

    // Resolve right down to `H2BasicType`s
    pub fn fully_resolve(&self, _starting_offset: u64, _breadcrumbs: Option<Vec<String>>) -> Vec<ResolvedType> {
        vec![]
    }

    pub fn size(&self, context: &Context) -> SimpleResult<u64> {
        match self {
            Self::NTString(t)  => t.size(context),
        }
    }

    pub fn to_string(&self, context: &Context) -> SimpleResult<String> {
        match self {
            Self::NTString(t)  => t.to_string(context),
        }
    }
}
