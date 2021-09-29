use base64;
use simple_error::{SimpleResult, bail};
use serde::{Serialize, Deserialize};
use std::fmt;

use crate::{Transformation, TransformerTrait};

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Serialize, Deserialize)]
pub struct TransformBase64 {
    no_padding: bool,
    permissive: bool,
    url: bool,
}

impl fmt::Display for TransformBase64 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl TransformBase64 {
    pub fn new(no_padding: bool, permissive: bool, url: bool) -> Transformation {
        Transformation::FromBase64(Self {
            no_padding: no_padding,
            permissive: permissive,
            url: url,
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

    pub fn url() -> Transformation {
        Self::new(false, false, true)
    }

    pub fn url_no_padding() -> Transformation {
        Self::new(true, false, true)
    }

    pub fn url_permissive() -> Transformation {
        Self::new(false, true, true)
    }

    /// Convert my padding and URL booleans into the built-in [`base64`]
    /// constants
    fn get_config(&self) -> base64::Config {
        match (self.no_padding, self.url) {
            (false, false) => base64::STANDARD,
            (true,  false) => base64::STANDARD_NO_PAD,
            (false, true)  => base64::URL_SAFE,
            (true,  true)  => base64::URL_SAFE_NO_PAD,
        }
    }

    // TODO: Permit newlines / whitespace in base64 (seems fairly common)
    fn transform_standard(&self, buffer: &Vec<u8>) -> SimpleResult<Vec<u8>> {
        let original_length = buffer.len();

        // Decode
        let out = match base64::decode_config(buffer, self.get_config()) {
            Ok(r) => r,
            Err(e) => bail!("Couldn't decode base64: {}", e),
        };

        // Ensure it encodes to the same length - we can't handle length changes
        if base64::encode_config(&out, self.get_config()).len() != original_length {
            bail!("Base64 didn't decode correctly (the length changed with decode->encode, check padding)");
        }

        Ok(out)
    }

    fn untransform_standard(&self, buffer: &Vec<u8>) -> SimpleResult<Vec<u8>> {
        Ok(base64::encode_config(buffer, self.get_config()).into_bytes())
    }

    fn check_standard(&self, buffer: &Vec<u8>) -> bool {
        // The only reasonable way to check is by just doing it (since the
        // config is opaque to us)
        self.transform_standard(buffer).is_ok()
    }

    fn transform_permissive(&self, buffer: &Vec<u8>) -> SimpleResult<Vec<u8>> {
        // Filter out any control characters and spaces
        let buffer: Vec<u8> = buffer.clone().into_iter().filter(|b| {
            *b > 0x20 && *b < 0x80
        }).collect();

        // Decode
        let out = match base64::decode_config(buffer, self.get_config()) {
            Ok(r) => r,
            Err(e) => bail!("Couldn't decode base64: {}", e),
        };

        Ok(out)
    }

    fn untransform_permissive(&self, _buffer: &Vec<u8>) -> SimpleResult<Vec<u8>> {
        bail!("Cannot untransform base64_permissive");
    }

    fn check_permissive(&self, buffer: &Vec<u8>) -> bool {
        // The only reasonable way to check is by just doing it (since the config is opaque to us)
        self.transform_permissive(buffer).is_ok()
    }
}

impl TransformerTrait for TransformBase64 {
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
            Self::url(),
            Self::url_no_padding()
        ];

        // Filter down to the ones that work
        transformations.into_iter().filter(|t| t.can_transform(buffer)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_base64_standard() -> SimpleResult<()> {
        let t = TransformBase64::standard();
        assert_eq!(true, t.is_two_way());

        // Short string: "\x00"
        assert!(t.can_transform(&b"AA==".to_vec()));
        let result = t.transform(&b"AA==".to_vec())?;
        assert_eq!(b"\x00".to_vec(), result);
        let original = t.untransform(&result)?;
        assert_eq!(b"AA==".to_vec(), original);

        // Longer string: "\x00\x01\x02\x03\x04\x05\x06"
        assert!(t.can_transform(&b"AAECAwQFBg==".to_vec()));
        let result = t.transform(&b"AAECAwQFBg==".to_vec())?;
        assert_eq!(b"\x00\x01\x02\x03\x04\x05\x06".to_vec(), result);
        let original = t.untransform(&result)?;
        assert_eq!(b"AAECAwQFBg==".to_vec(), original);

        // Weird string: "\x69\xaf\xbe\xff\x3f"
        assert!(t.can_transform(&b"aa++/z8=".to_vec()));
        let result = t.transform(&b"aa++/z8=".to_vec())?;
        assert_eq!(b"\x69\xaf\xbe\xff\x3f".to_vec(), result);
        let original = t.untransform(&result)?;
        assert_eq!(b"aa++/z8=".to_vec(), original);

        // Do padding wrong
        assert!(!t.can_transform(&b"AA".to_vec()));
        assert!(!t.can_transform(&b"AA=".to_vec()));
        assert!(!t.can_transform(&b"AA===".to_vec()));
        assert!(!t.can_transform(&b"AA====".to_vec()));

        assert!(t.transform(&b"AA".to_vec()).is_err());
        assert!(t.transform(&b"AA=".to_vec()).is_err());
        assert!(t.transform(&b"AA===".to_vec()).is_err());
        assert!(t.transform(&b"AA====".to_vec()).is_err());

        // Wrong characters
        assert!(t.transform(&b"aa--_z8=".to_vec()).is_err());

        Ok(())
    }

    #[test]
    fn test_base64_standard_no_padding() -> SimpleResult<()> {
        let t = TransformBase64::no_padding();
        assert_eq!(true, t.is_two_way());

        // Short string: "\x00"
        assert!(t.can_transform(&b"AA".to_vec()));
        let result = t.transform(&b"AA".to_vec())?;
        assert_eq!(b"\x00".to_vec(), result);
        let original = t.untransform(&result)?;
        assert_eq!(b"AA".to_vec(), original);

        // Longer string: "\x00\x01\x02\x03\x04\x05\x06"
        assert!(t.can_transform(&b"AAECAwQFBg".to_vec()));
        let result = t.transform(&b"AAECAwQFBg".to_vec())?;
        assert_eq!(b"\x00\x01\x02\x03\x04\x05\x06".to_vec(), result);
        let original = t.untransform(&result)?;
        assert_eq!(b"AAECAwQFBg".to_vec(), original);

        // Weird string: "\x69\xaf\xbe\xff\x3f"
        let result = t.transform(&b"aa++/z8".to_vec())?;
        assert_eq!(b"\x69\xaf\xbe\xff\x3f".to_vec(), result);
        let original = t.untransform(&result)?;
        assert_eq!(b"aa++/z8".to_vec(), original);

        // Do padding wrong
        assert!(t.transform(&b"AA=".to_vec()).is_err());
        assert!(t.transform(&b"AA==".to_vec()).is_err());
        assert!(t.transform(&b"AA===".to_vec()).is_err());
        assert!(t.transform(&b"AA====".to_vec()).is_err());

        // Wrong characters
        assert!(t.transform(&b"aa--_z8".to_vec()).is_err());

        Ok(())
    }

    #[test]
    fn test_base64_permissive() -> SimpleResult<()> {
        let t = TransformBase64::permissive();
        assert_eq!(false, t.is_two_way());

        // Short string: "\x00" with various padding
        assert!(t.can_transform(&b"AA".to_vec()));
        assert!(t.can_transform(&b"AA=".to_vec()));
        assert!(t.can_transform(&b"AA==".to_vec()));
        assert_eq!(b"\x00".to_vec(), t.transform(&b"AA".to_vec())?);
        assert_eq!(b"\x00".to_vec(), t.transform(&b"AA=".to_vec())?);
        assert_eq!(b"\x00".to_vec(), t.transform(&b"AA==".to_vec())?);

        // Add a bunch of control characters
        assert_eq!(b"\x00\x00\x00\x00".to_vec(), t.transform(&b"A A\nAAA\n    \t\rA=\n=".to_vec())?);

        Ok(())
    }

    #[test]
    fn test_base64_url() -> SimpleResult<()> {
        let t = TransformBase64::url();
        assert_eq!(true, t.is_two_way());

        // Short string: "\x00"
        let result = t.transform(&b"AA==".to_vec())?;
        assert_eq!(b"\x00".to_vec(), result);
        let original = t.untransform(&result)?;
        assert_eq!(b"AA==".to_vec(), original);

        // Longer string: "\x00\x01\x02\x03\x04\x05\x06"
        let result = t.transform(&b"AAECAwQFBg==".to_vec())?;
        assert_eq!(b"\x00\x01\x02\x03\x04\x05\x06".to_vec(), result);
        let original = t.untransform(&result)?;
        assert_eq!(b"AAECAwQFBg==".to_vec(), original);

        // Weird string: "\x69\xaf\xbe\xff\x3f"
        let result = t.transform(&b"aa--_z8=".to_vec())?;
        assert_eq!(b"\x69\xaf\xbe\xff\x3f".to_vec(), result);
        let original = t.untransform(&result)?;
        assert!(t.can_transform(&b"aa--_z8=".to_vec()));
        assert_eq!(b"aa--_z8=".to_vec(), original);

        // Do padding wrong
        assert!(t.transform(&b"AA".to_vec()).is_err());
        assert!(t.transform(&b"AA=".to_vec()).is_err());
        assert!(t.transform(&b"AA===".to_vec()).is_err());
        assert!(t.transform(&b"AA====".to_vec()).is_err());

        // Wrong characters
        assert!(!t.can_transform(&b"aa++/z8=".to_vec()));
        assert!(t.transform(&b"aa++/z8=".to_vec()).is_err());

        Ok(())
    }

    #[test]
    fn test_base64_standard_url_no_padding() -> SimpleResult<()> {
        let t = TransformBase64::url_no_padding();
        assert_eq!(true, t.is_two_way());

        // Short string: "\x00"
        let result = t.transform(&b"AA".to_vec())?;
        assert_eq!(b"\x00".to_vec(), result);
        let original = t.untransform(&result)?;
        assert_eq!(b"AA".to_vec(), original);

        // Longer string: "\x00\x01\x02\x03\x04\x05\x06"
        let result = t.transform(&b"AAECAwQFBg".to_vec())?;
        assert_eq!(b"\x00\x01\x02\x03\x04\x05\x06".to_vec(), result);
        let original = t.untransform(&result)?;
        assert_eq!(b"AAECAwQFBg".to_vec(), original);

        // Weird string: "\x69\xaf\xbe\xff\x3f"
        let result = t.transform(&b"aa--_z8".to_vec())?;
        assert_eq!(b"\x69\xaf\xbe\xff\x3f".to_vec(), result);
        let original = t.untransform(&result)?;
        assert_eq!(b"aa--_z8".to_vec(), original);

        // Do padding wrong
        assert!(t.transform(&b"AA=".to_vec()).is_err());
        assert!(t.transform(&b"AA==".to_vec()).is_err());
        assert!(t.transform(&b"AA===".to_vec()).is_err());
        assert!(t.transform(&b"AA====".to_vec()).is_err());

        // Wrong characters
        assert!(t.transform(&b"aa++/z8".to_vec()).is_err());

        Ok(())
    }

    #[test]
    fn test_base64_url_permissive() -> SimpleResult<()> {
        let t = TransformBase64::url_permissive();
        assert_eq!(false, t.is_two_way());

        // Short string: "\x00" with various padding
        assert_eq!(b"\x00".to_vec(), t.transform(&b"AA".to_vec())?);
        assert_eq!(b"\x00".to_vec(), t.transform(&b"AA=".to_vec())?);
        assert_eq!(b"\x00".to_vec(), t.transform(&b"AA==".to_vec())?);

        // Add a bunch of control characters
        assert_eq!(b"\x00\x00\x00\x00".to_vec(), t.transform(&b"A A\nAAA\n    \t\rA=\n=".to_vec())?);

        Ok(())
    }
}
