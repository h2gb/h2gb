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

    fn is_two_way(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use crate::transformation::Transformation;

    #[test]
    fn test_null() -> SimpleResult<()> {
        assert_eq!(true, Transformation::Null.is_two_way());

        let tests: Vec<(Vec<u8>, SimpleResult<Vec<u8>>)> = vec![
            (vec![1],             Ok(vec![1])),
            (vec![1, 2, 3],       Ok(vec![1, 2, 3])),
            (vec![1, 2, 3, 4, 5], Ok(vec![1, 2, 3, 4, 5])),
        ];

        for (test, expected) in tests {
            assert!(Transformation::Null.can_transform(&test));

            let result = Transformation::Null.transform(&test);
            assert_eq!(expected, result);

            let result = Transformation::Null.untransform(&result?);
            assert_eq!(Ok(test), result);
        }

        Ok(())
    }
}
