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

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use crate::transformation::Transformation;

    #[test]
    fn test_hex() -> SimpleResult<()> {
        let t = Transformation::FromHex;

        assert!(t.is_two_way());
        assert!(t.can_transform(&b"00".to_vec()));
        assert!(t.can_transform(&b"0001".to_vec()));
        assert!(t.can_transform(&b"000102feff".to_vec()));
        assert!(!t.can_transform(&b"0".to_vec()));
        assert!(!t.can_transform(&b"001".to_vec()));
        assert!(!t.can_transform(&b"00102FEff".to_vec()));
        assert!(!t.can_transform(&b"fg".to_vec()));
        assert!(!t.can_transform(&b"+=".to_vec()));

        assert_eq!(vec![0x00], t.transform(&b"00".to_vec())?);
        assert_eq!(vec![0x00, 0x01], t.transform(&b"0001".to_vec())?);
        assert_eq!(vec![0x00, 0x01, 0x02, 0xfe, 0xff], t.transform(&b"000102fEFf".to_vec())?);

        assert_eq!(b"00".to_vec(), t.untransform(&vec![0x00])?);
        assert_eq!(b"0001".to_vec(), t.untransform(&vec![0x00, 0x01])?);
        assert_eq!(b"000102feff".to_vec(), t.untransform(&vec![0x00, 0x01, 0x02, 0xfe, 0xff])?);

        assert!(t.transform(&b"abababag".to_vec()).is_err());

        Ok(())
    }
}
