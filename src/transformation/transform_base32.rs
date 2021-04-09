use base32;
use simple_error::{SimpleResult, bail};
use serde::{Serialize, Deserialize};

use crate::transformation::{Transformation, TransformerTrait};

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Serialize, Deserialize)]
pub struct TransformBase32 {
    no_padding: bool,
    permissive: bool,
    crockford: bool,
}

impl TransformBase32 {
    pub fn new(no_padding: bool, permissive: bool, crockford: bool) -> Transformation {
        Transformation::FromBase32(Self {
            no_padding: no_padding,
            permissive: permissive,
            crockford: crockford,
        })
    }

    pub fn standard() -> Transformation {
        Self::new(false, false, false)
    }

    pub fn no_padding() -> Transformation {
        Self::new(true, false, false)
    }

    pub fn permissive() -> Transformation {
        Self::new(false, true, false)
    }

    pub fn crockford() -> Transformation {
        Self::new(false, false, true)
    }

    pub fn crockford_no_padding() -> Transformation {
        Self::new(true, false, true)
    }

    pub fn crockford_permissive() -> Transformation {
        Self::new(false, true, true)
    }

    /// Internal function to get the correct alphabet based on settings
    fn get_alphabet(&self) -> base32::Alphabet {
        match self.crockford {
            false => base32::Alphabet::RFC4648 { padding: !self.no_padding },
            true  => base32::Alphabet::Crockford,
        }
    }

    fn transform_standard(self, buffer: &Vec<u8>) -> SimpleResult<Vec<u8>> {
        let original_length = buffer.len();

        let s = match std::str::from_utf8(buffer) {
            Ok(s) => s,
            Err(e) => bail!("Couldn't convert the buffer into a string: {}", e),
        };

        // Decode
        let out = match base32::decode(self.get_alphabet(), &s) {
            Some(r) => r,
            None => bail!("Couldn't decode base32"),
        };

        // Ensure it encodes to the same length - we can't handle length changes
        if base32::encode(self.get_alphabet(), &out).into_bytes().len() != original_length {
            bail!("Base32 didn't decode correctly");
        }

        Ok(out)
    }

    fn untransform_standard(self, buffer: &Vec<u8>) -> SimpleResult<Vec<u8>> {
        Ok(base32::encode(self.get_alphabet(), buffer).into_bytes())
    }

    fn check_standard(self, buffer: &Vec<u8>) -> bool {
        // The only reasonable way to check is by just doing it
        self.transform(buffer).is_ok()
    }

    fn transform_permissive(self, buffer: &Vec<u8>) -> SimpleResult<Vec<u8>> {
        // Filter out any obviously impossible characters
        let buffer: Vec<u8> = buffer.clone().into_iter().filter(|b| {
            (*b >= 0x30 && *b <= 0x39) || (*b >= 0x41 && *b <= 0x5a) || (*b >= 0x61 && *b <= 0x7a)
        }).collect();

        let s = match String::from_utf8(buffer) {
            Ok(s) => s,
            Err(e) => bail!("Couldn't convert the buffer into a string: {}", e),
        };

        // Decode
        match base32::decode(self.get_alphabet(), &s) {
            Some(r) => Ok(r),
            None => bail!("Couldn't decode base32"),
        }
    }

    fn untransform_permissive(self, _buffer: &Vec<u8>) -> SimpleResult<Vec<u8>> {
        bail!("Cannot untransform Base32 Permissive");
    }

    fn check_permissive(self, buffer: &Vec<u8>) -> bool {
        // The only reasonable way to check is by just doing it
        self.transform_permissive(buffer).is_ok()
    }
}

impl TransformerTrait for TransformBase32 {
    fn transform(&self, buffer: &Vec<u8>) -> SimpleResult<Vec<u8>> {
        match self.permissive {
            false => self.transform_standard(buffer),
            true  => self.transform_permissive(buffer),
        }
    }

    fn untransform(&self, buffer: &Vec<u8>) -> SimpleResult<Vec<u8>> {
        match self.permissive {
            false => self.untransform_standard(buffer),
            true  => self.untransform_permissive(buffer),
        }
    }

    fn can_transform(&self, buffer: &Vec<u8>) -> bool {
        match self.permissive {
            false => self.check_standard(buffer),
            true  => self.check_permissive(buffer),
        }
    }

    fn is_two_way(&self) -> bool {
        // It's two-way if it's not permissive (permissive allows illegal stuff,
        // which would only be one way).
        !self.permissive
    }

    fn detect(buffer: &Vec<u8>) -> Vec<Transformation> where Self: Sized {
        // These are all the detect-able transformations
        let transformations = vec![
            Self::standard(),
            Self::no_padding(),
            Self::crockford(),
        ];

        // Filter down to the ones that work
        transformations.into_iter().filter(|t| t.can_transform(buffer)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use crate::transformation::Transformation;

    #[test]
    fn test_base32_standard() -> SimpleResult<()> {
        let t = TransformBase32::standard();
        assert_eq!(true, t.is_two_way());

        // Short string: "\x00"
        let t = TransformBase32::standard();
        let result = t.transform(&b"IE======".to_vec())?;
        assert_eq!(b"A".to_vec(), result);
        let original = t.untransform(&result)?;
        assert_eq!(b"IE======".to_vec(), original);

        // Longer string: "ABCDEF"
        let t = TransformBase32::standard();
        let result = t.transform(&b"IFBEGRCFIY======".to_vec())?;
        assert_eq!(b"ABCDEF".to_vec(), result);
        let original = t.untransform(&result)?;
        assert_eq!(b"IFBEGRCFIY======".to_vec(), original);

        // It's okay to be case insensitive
        let t = TransformBase32::standard();
        let result = t.transform(&b"ifbegrcfiy======".to_vec())?;
        assert_eq!(b"ABCDEF".to_vec(), result);
        let original = t.untransform(&result)?;
        assert_eq!(b"IFBEGRCFIY======".to_vec(), original);

        // Do padding wrong
        let t = TransformBase32::standard();
        assert!(t.transform(&b"IE".to_vec()).is_err());
        assert!(t.transform(&b"IE=".to_vec()).is_err());
        assert!(t.transform(&b"IE==".to_vec()).is_err());
        assert!(t.transform(&b"IE===".to_vec()).is_err());
        assert!(t.transform(&b"IE====".to_vec()).is_err());
        assert!(t.transform(&b"IE=====".to_vec()).is_err());
        assert!(t.transform(&b"IE=======".to_vec()).is_err());
        assert!(t.transform(&b"IE========".to_vec()).is_err());

        // Wrong characters
        let t = TransformBase32::standard();
        assert!(t.transform(&b"I.======".to_vec()).is_err());

        Ok(())
    }

    #[test]
    fn test_base32_no_padding() -> SimpleResult<()> {
        let t = TransformBase32::no_padding();
        assert_eq!(true, t.is_two_way());

        // Short string: "\x00"
        let t = TransformBase32::no_padding();
        let result = t.transform(&b"IE".to_vec())?;
        assert_eq!(b"A".to_vec(), result);
        let original = t.untransform(&result)?;
        assert_eq!(b"IE".to_vec(), original);

        // Longer string: "ABCDEF"
        let t = TransformBase32::no_padding();
        let result = t.transform(&b"IFBEGRCFIY".to_vec())?;
        assert_eq!(b"ABCDEF".to_vec(), result);
        let original = t.untransform(&result)?;
        assert_eq!(b"IFBEGRCFIY".to_vec(), original);

        // It's okay to be case insensitive
        let t = TransformBase32::no_padding();
        let result = t.transform(&b"ifbegrcfiy".to_vec())?;
        assert_eq!(b"ABCDEF".to_vec(), result);
        let original = t.untransform(&result)?;
        assert_eq!(b"IFBEGRCFIY".to_vec(), original);

        // Do padding wrong
        let t = TransformBase32::no_padding();
        assert!(t.transform(&b"IE=".to_vec()).is_err());
        assert!(t.transform(&b"IE==".to_vec()).is_err());
        assert!(t.transform(&b"IE===".to_vec()).is_err());
        assert!(t.transform(&b"IE====".to_vec()).is_err());
        assert!(t.transform(&b"IE=====".to_vec()).is_err());
        assert!(t.transform(&b"IE======".to_vec()).is_err());
        assert!(t.transform(&b"IE=======".to_vec()).is_err());
        assert!(t.transform(&b"IE========".to_vec()).is_err());

        // Wrong characters
        let t = TransformBase32::no_padding();
        assert!(t.transform(&b"A.".to_vec()).is_err());

        Ok(())
    }

    #[test]
    fn test_base32_crockford() -> SimpleResult<()> {
        let t = TransformBase32::crockford();
        assert_eq!(true, t.is_two_way());

        // Short string: "\x00"
        let t = TransformBase32::crockford();
        let result = t.transform(&b"84".to_vec())?;
        assert_eq!(b"A".to_vec(), result);
        let original = t.untransform(&result)?;
        assert_eq!(b"84".to_vec(), original);

        // Longer string: "ABCDEF"
        let t = TransformBase32::crockford();
        let result = t.transform(&b"85146H258R".to_vec())?;
        assert_eq!(b"ABCDEF".to_vec(), result);
        let original = t.untransform(&result)?;
        assert_eq!(b"85146H258R".to_vec(), original);

        // It's okay to be case insensitive
        let t = TransformBase32::crockford();
        let result = t.transform(&b"85146h258r".to_vec())?;
        assert_eq!(b"ABCDEF".to_vec(), result);
        let original = t.untransform(&result)?;
        assert_eq!(b"85146H258R".to_vec(), original);

        // Do padding wrong
        let t = TransformBase32::crockford();
        assert!(t.transform(&b"84=".to_vec()).is_err());
        assert!(t.transform(&b"84==".to_vec()).is_err());
        assert!(t.transform(&b"84===".to_vec()).is_err());
        assert!(t.transform(&b"84====".to_vec()).is_err());
        assert!(t.transform(&b"84=====".to_vec()).is_err());
        assert!(t.transform(&b"84======".to_vec()).is_err());
        assert!(t.transform(&b"84=======".to_vec()).is_err());
        assert!(t.transform(&b"84========".to_vec()).is_err());

        // Wrong characters
        let t = TransformBase32::crockford();
        assert!(t.transform(&b"A.".to_vec()).is_err());

        Ok(())
    }

    #[test]
    fn test_base32_permissive() -> SimpleResult<()> {
        let t = TransformBase32::permissive();
        assert_eq!(false, t.is_two_way());

        // Short string: "\x00"
        let t = TransformBase32::permissive();
        let result = t.transform(&b"IE======".to_vec())?;
        assert_eq!(b"A".to_vec(), result);

        // Longer string: "ABCDEF"
        let t = TransformBase32::permissive();
        let result = t.transform(&b"IFBEGRCFIY======".to_vec())?;
        assert_eq!(b"ABCDEF".to_vec(), result);

        // It's okay to be case insensitive
        let t = TransformBase32::permissive();
        let result = t.transform(&b"ifbegrcfiy======".to_vec())?;
        assert_eq!(b"ABCDEF".to_vec(), result);

        // Do padding wrong
        let t = TransformBase32::permissive();
        assert_eq!(b"A".to_vec(), t.transform(&b"IE".to_vec())?);
        assert_eq!(b"A".to_vec(), t.transform(&b"IE=".to_vec())?);
        assert_eq!(b"A".to_vec(), t.transform(&b"IE==".to_vec())?);
        assert_eq!(b"A".to_vec(), t.transform(&b"IE===".to_vec())?);
        assert_eq!(b"A".to_vec(), t.transform(&b"IE====".to_vec())?);
        assert_eq!(b"A".to_vec(), t.transform(&b"IE=====".to_vec())?);
        assert_eq!(b"A".to_vec(), t.transform(&b"IE=============".to_vec())?);
        assert_eq!(b"A".to_vec(), t.transform(&b"I=============E".to_vec())?);
        assert_eq!(b"A".to_vec(), t.transform(&b"IE=============".to_vec())?);
        assert_eq!(b"A".to_vec(), t.transform(&b"I.@#$...E...======".to_vec())?);

        // We can still error with bad characters
        assert!(t.transform(&b"1234567890".to_vec()).is_err());

        Ok(())
    }

    #[test]
    fn test_base32_crockford_permissive() -> SimpleResult<()> {
        let t = TransformBase32::crockford_permissive();
        assert_eq!(false, t.is_two_way());

        // Short string: "\x00"
        let t = TransformBase32::crockford_permissive();
        let result = t.transform(&b"84======".to_vec())?;
        assert_eq!(b"A".to_vec(), result);

        // Longer string: "ABCDEF"
        let t = TransformBase32::crockford_permissive();
        let result = t.transform(&b"85146H258R======".to_vec())?;
        assert_eq!(b"ABCDEF".to_vec(), result);

        // It's okay to be case insensitive
        let t = TransformBase32::crockford_permissive();
        let result = t.transform(&b"85146h258r======".to_vec())?;
        assert_eq!(b"ABCDEF".to_vec(), result);

        // Do padding wrong
        let t = TransformBase32::crockford_permissive();
        assert_eq!(b"A".to_vec(), t.transform(&b"84".to_vec())?);
        assert_eq!(b"A".to_vec(), t.transform(&b"84=".to_vec())?);
        assert_eq!(b"A".to_vec(), t.transform(&b"84==".to_vec())?);
        assert_eq!(b"A".to_vec(), t.transform(&b"84===".to_vec())?);
        assert_eq!(b"A".to_vec(), t.transform(&b"84====".to_vec())?);
        assert_eq!(b"A".to_vec(), t.transform(&b"84=====".to_vec())?);
        assert_eq!(b"A".to_vec(), t.transform(&b"84=============".to_vec())?);
        assert_eq!(b"A".to_vec(), t.transform(&b"8==---========4".to_vec())?);
        assert_eq!(b"A".to_vec(), t.transform(&b"84=============".to_vec())?);
        assert_eq!(b"A".to_vec(), t.transform(&b"8.@#$...4...======".to_vec())?);

        // We can still error with bad characters
        assert!(t.transform(&b"no u".to_vec()).is_err());

        Ok(())
    }
}
