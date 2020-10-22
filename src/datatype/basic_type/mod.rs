use serde::{Serialize, Deserialize};
use simple_error::{bail, SimpleResult};
use sized_number::Context;
use std::ops::Range;

use crate::datatype::{helpers, H2Type, PartiallyResolvedType};

pub mod h2number;
use h2number::H2Number;

pub mod h2pointer;
use h2pointer::H2Pointer;

pub mod ipv4;
use ipv4::IPv4;

pub mod ipv6;
use ipv6::IPv6;

pub mod character;
use character::Character;

pub mod unicode;
use unicode::Unicode;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum H2BasicTypes {
    Number(H2Number),
    Pointer(H2Pointer),
    IPv4(IPv4),
    IPv6(IPv6),
    Character(Character),
    Unicode(Unicode),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct H2BasicType {
    field: H2BasicTypes,
    byte_alignment: Option<u64>,
}

pub trait H2BasicTrait {
    fn to_string(&self, context: &Context) -> SimpleResult<String>;
    fn size(&self) -> u64;
    fn related(&self, _context: &Context) -> SimpleResult<Vec<(u64, H2Type)>>;
}

impl H2BasicType {
    pub fn new(field: H2BasicTypes) -> Self {
        Self {
            field: field,
            byte_alignment: None,
        }
    }

    pub fn new_aligned(field: H2BasicTypes, byte_alignment: Option<u64>) -> Self {
        Self {
            field: field,
            byte_alignment: byte_alignment,
        }
    }

    pub fn as_basic_trait(&self) -> Box<&dyn H2BasicTrait> {
        match &self.field {
            H2BasicTypes::Number(t)    => Box::new(t),
            H2BasicTypes::Pointer(t)   => Box::new(t),
            H2BasicTypes::IPv4(t)      => Box::new(t),
            H2BasicTypes::IPv6(t)      => Box::new(t),
            H2BasicTypes::Character(t) => Box::new(t),
            H2BasicTypes::Unicode(t)   => Box::new(t),
        }
    }

    pub fn to_string(&self, context: &Context) -> SimpleResult<String> {
        self.as_basic_trait().to_string(context)
    }

    pub fn related(&self, context: &Context) -> SimpleResult<Vec<(u64, H2Type)>> {
        self.as_basic_trait().related(context)
    }

    pub fn size(&self) -> u64 {
        let size = self.as_basic_trait().size();

        // Round up to alignment, if it's set
        helpers::maybe_round_up(size, self.byte_alignment)
    }

    pub fn range(&self, start: u64) -> Range<u64> {
        start..(start + self.size())
    }

    pub fn partially_resolve(&self, start: u64) -> SimpleResult<Vec<PartiallyResolvedType>> {
        let partially_resolved = PartiallyResolvedType {
            offset: self.range(start),
            field_name: None,
            field_type: H2Type::from(self.clone()),
        };

        Ok(vec![partially_resolved])
    }
}
