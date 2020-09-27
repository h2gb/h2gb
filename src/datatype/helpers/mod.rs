// use serde::{Serialize, Deserialize};
pub mod number;

pub struct H2Context<'a> {
    pub data: &'a Vec<u8>,
    pub index: usize,
}

impl<'a> From<(&'a Vec<u8>, usize)> for H2Context<'a> {
    fn from(o: (&'a Vec<u8>, usize)) -> H2Context {
        Self {
            data: o.0,
            index: o.1,
        }
    }
}
