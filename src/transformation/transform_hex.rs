use simple_error::{SimpleResult, bail};
use serde::{Serialize, Deserialize};

use crate::transformation::TransformerTrait;

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Serialize, Deserialize)]
pub struct TransformHex {
}

impl TransformHex {
    pub fn new() -> Self {
        TransformHex {}
    }
}

impl TransformerTrait for TransformHex {
    fn transform(&self, buffer: &Vec<u8>) -> SimpleResult<Vec<u8>> {
        let s = match std::str::from_utf8(buffer) {
            Ok(s) => s,
            Err(e) => bail!("Couldn't convert the buffer into a string: {}", e),
        };

        match hex::decode(s) {
            Ok(s) => Ok(s),
            Err(e) => bail!("Couldn't decode hex: {}", e),
        }
    }

    fn untransform(&self, buffer: &Vec<u8>) -> SimpleResult<Vec<u8>> {
        Ok(hex::encode(buffer).into_bytes())
    }

    fn check(&self, buffer: &Vec<u8>) -> bool {
        self.transform(buffer).is_ok()
    }

}
