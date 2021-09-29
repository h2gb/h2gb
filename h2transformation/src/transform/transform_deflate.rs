use inflate;
use simple_error::{SimpleResult, bail};
use serde::{Serialize, Deserialize};
use std::fmt;

use crate::{Transformation, TransformerTrait};

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Serialize, Deserialize)]
pub struct TransformDeflate {
    zlib_header: bool,
}

impl fmt::Display for TransformDeflate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl TransformDeflate {
    pub fn new(zlib_header: bool) -> Transformation {
        Transformation::FromDeflated(Self {
            zlib_header: zlib_header,
        })
    }

    pub fn with_header() -> Transformation {
        Self::new(true)
    }

    pub fn without_header() -> Transformation {
        Self::new(false)
    }

    fn transform_deflated(self, buffer: &Vec<u8>) -> SimpleResult<Vec<u8>> {
        match inflate::inflate_bytes(buffer) {
            Ok(b) => Ok(b),
            Err(e) => bail!("Couldn't inflate: {}", e),
        }
    }

    fn check_deflated(self, buffer: &Vec<u8>) -> bool {
        // Extra short strings kinda sorta decode, but a zero-length string is
        // a minimum 6 characters so just enforce that
        buffer.len() > 5 && self.transform_deflated(buffer).is_ok()
    }

    fn transform_deflated_zlib(self, buffer: &Vec<u8>) -> SimpleResult<Vec<u8>> {
        match inflate::inflate_bytes_zlib(buffer) {
            Ok(b) => Ok(b),
            Err(e) => bail!("Couldn't inflate: {}", e),
        }
    }

    fn check_deflated_zlib(self, buffer: &Vec<u8>) -> bool {
        // The only reasonable way to check is by just doing it
        self.transform_deflated_zlib(buffer).is_ok()
    }
}

impl TransformerTrait for TransformDeflate {
    fn transform(&self, buffer: &Vec<u8>) -> SimpleResult<Vec<u8>> {
        match self.zlib_header {
            true => self.transform_deflated_zlib(buffer),
            false => self.transform_deflated(buffer),
        }
    }

    fn untransform(&self, _buffer: &Vec<u8>) -> SimpleResult<Vec<u8>> {
        bail!("Deflate cannot be untransformed");
    }

    fn can_transform(&self, buffer: &Vec<u8>) -> bool {
        match self.zlib_header {
            true => self.check_deflated_zlib(buffer),
            false => self.check_deflated(buffer),
        }
    }

    fn is_two_way(&self) -> bool {
        // Deflate can't reliably go backwards
        false
    }

    fn detect(buffer: &Vec<u8>) -> Vec<Transformation> where Self: Sized {
        let mut out: Vec<_> = Vec::new();

        let t = TransformDeflate::with_header();
        if t.can_transform(buffer) {
            out.push(t);
        }

        let t = TransformDeflate::without_header();
        if t.can_transform(buffer) {
            out.push(t);
        }

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_deflate() -> SimpleResult<()> {
        let t = TransformDeflate::without_header();

        let result = t.transform(&b"\x03\x00\x00\x00\x00\x01".to_vec())?;
        assert_eq!(0, result.len());

        let result = t.transform(&b"\x63\x00\x00\x00\x01\x00\x01".to_vec())?;
        assert_eq!(vec![0x00], result);

        let result = t.transform(&b"\x63\x60\x80\x01\x00\x00\x0a\x00\x01".to_vec())?;
        assert_eq!(vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], result);

        let result = t.transform(&b"\x63\x60\x64\x62\x66\x61\x65\x63\xe7\xe0\x04\x00\x00\xaf\x00\x2e".to_vec())?;
        assert_eq!(vec![0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09], result);

        // Best compression
        let result = t.transform(&b"\x73\x74\x72\x76\x01\x00\x02\x98\x01\x0b".to_vec())?;
        assert_eq!(vec![0x41, 0x42, 0x43, 0x44], result);

        // No compression
        let result = t.transform(&b"\x01\x04\x00\xfb\xff\x41\x42\x43\x44\x02\x98\x01\x0b".to_vec())?;
        assert_eq!(vec![0x41, 0x42, 0x43, 0x44], result);

        // Try an intentional error
        assert!(t.transform(&b"\xFF".to_vec()).is_err());

        Ok(())
    }

    #[test]
    fn test_deflate_zlib() -> SimpleResult<()> {
        let t = TransformDeflate::with_header();

        let result = t.transform(&b"\x78\x9c\x03\x00\x00\x00\x00\x01".to_vec())?;
        assert_eq!(0, result.len());

        let result = t.transform(&b"\x78\x9c\x63\x00\x00\x00\x01\x00\x01".to_vec())?;
        assert_eq!(vec![0x00], result);

        let result = t.transform(&b"\x78\x9c\x63\x60\x80\x01\x00\x00\x0a\x00\x01".to_vec())?;
        assert_eq!(vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], result);

        let result = t.transform(&b"\x78\x9c\x63\x60\x64\x62\x66\x61\x65\x63\xe7\xe0\x04\x00\x00\xaf\x00\x2e".to_vec())?;
        assert_eq!(vec![0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09], result);

        // Best compression
        let result = t.transform(&b"\x78\x9c\x73\x74\x72\x76\x01\x00\x02\x98\x01\x0b".to_vec())?;
        assert_eq!(vec![0x41, 0x42, 0x43, 0x44], result);

        // No compression
        let result = t.transform(&b"\x78\x01\x01\x04\x00\xfb\xff\x41\x42\x43\x44\x02\x98\x01\x0b".to_vec())?;
        assert_eq!(vec![0x41, 0x42, 0x43, 0x44], result);

        // Try an intentional error
        assert!(t.transform(&b"\xFF".to_vec()).is_err());

        Ok(())
    }
}
