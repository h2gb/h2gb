use simple_error::SimpleResult;
use serde::{Serialize, Deserialize};

use crate::transformation::TransformerTrait;

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Serialize, Deserialize)]
pub struct TransformNull {
}

impl TransformNull {
    pub fn new() -> Self {
        TransformNull {}
    }
}

impl TransformerTrait for TransformNull {
    fn transform(&self, buffer: &Vec<u8>) -> SimpleResult<Vec<u8>> {
        Ok(buffer.clone())
    }

    fn untransform(&self, buffer: &Vec<u8>) -> SimpleResult<Vec<u8>> {
        Ok(buffer.clone())
    }

    fn check(&self, _buffer: &Vec<u8>) -> bool {
        true
    }
}
