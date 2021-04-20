use simple_error::SimpleResult;
use serde::{Serialize, Deserialize};

use crate::transformation::{Transformation, TransformerTrait};

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Serialize, Deserialize)]
pub struct TransformNull {
}

impl TransformNull {
    pub fn new() -> Transformation {
        Transformation::Null(TransformNull {})
    }
}

impl TransformerTrait for TransformNull {
    fn transform(&self, buffer: &Vec<u8>) -> SimpleResult<Vec<u8>> {
        Ok(buffer.clone())
    }

    fn untransform(&self, buffer: &Vec<u8>) -> SimpleResult<Vec<u8>> {
        Ok(buffer.clone())
    }

    fn can_transform(&self, _buffer: &Vec<u8>) -> bool {
        true
    }

    fn is_two_way(&self) -> bool {
        true
    }

    fn detect(_buffer: &Vec<u8>) -> Vec<Transformation> where Self: Sized {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_null() -> SimpleResult<()> {
        assert_eq!(true, TransformNull::new().is_two_way());

        let tests: Vec<(Vec<u8>, SimpleResult<Vec<u8>>)> = vec![
            (vec![1],             Ok(vec![1])),
            (vec![1, 2, 3],       Ok(vec![1, 2, 3])),
            (vec![1, 2, 3, 4, 5], Ok(vec![1, 2, 3, 4, 5])),
        ];

        for (test, expected) in tests {
            assert!(TransformNull::new().can_transform(&test));

            let result = TransformNull::new().transform(&test);
            assert_eq!(expected, result);

            let result = TransformNull::new().untransform(&result?);
            assert_eq!(Ok(test), result);
        }

        Ok(())
    }
}
