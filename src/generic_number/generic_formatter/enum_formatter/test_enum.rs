use serde::{Serialize, Deserialize};
use num_derive::FromPrimitive;
use std::fmt;

#[derive(FromPrimitive, Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TestEnum {
    Zero = 0,
    NegativeOne = -1,
    One = 1,
    I32Max = 0x7FFFFFFF,
    U32Max = 0xFFFFFFFF,
    I64Max = 0x7FFFFFFFFFFFFFFF,

    // This is the same as -1
    //u64_max = 0xFFFFFFFFFFFFFFFF,
}

impl fmt::Display for TestEnum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self))
    }
}
