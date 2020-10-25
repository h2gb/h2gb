use serde::{Serialize, Deserialize};

use crate::datatype::H2TypeTrait;

pub mod h2number;
use h2number::H2Number;

pub mod h2pointer;
use h2pointer::H2Pointer;

// pub mod ipv4;
// use ipv4::IPv4;

// pub mod ipv6;
// use ipv6::IPv6;

// pub mod character;
// use character::Character;

// pub mod unicode;
// use unicode::Unicode;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum H2BasicType {
    Number(H2Number),
    Pointer(H2Pointer),
    // IPv4(IPv4),
    // IPv6(IPv6),
    // Character(Character),
    // Unicode(Unicode),
}

impl H2BasicType {
    pub fn as_trait(&self) -> Box<&dyn H2TypeTrait> {
        match self {
            H2BasicType::Number(t)  => Box::new(t),
            H2BasicType::Pointer(t) => Box::new(t),
        }
    }
}
