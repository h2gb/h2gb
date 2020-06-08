#![allow(dead_code)]
use serde::{Serialize, Deserialize};

pub mod h2transformation;
pub mod h2buffer;
pub mod h2datatype;
pub mod h2entry;
pub mod h2layer;
pub mod h2project;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct H2Combinator {
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct H2Analyzer {
}

#[cfg(test)]
mod tests {
    // use super::*;
    use bumpy_vector::BumpyVector;

    #[test]
    fn test_vector() {
        let h: BumpyVector<u32> = BumpyVector::new(1);

        let s = ron::ser::to_string(&h).unwrap();
        println!("Serialized: {}", s);
    }
}
