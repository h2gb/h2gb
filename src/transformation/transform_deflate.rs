use inflate;
use simple_error::{SimpleResult, bail};
use serde::{Serialize, Deserialize};

use crate::transformation::TransformerTrait;

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Serialize, Deserialize)]
pub struct DeflateSettings {
    zlib_header: bool,
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Serialize, Deserialize)]
pub struct TransformDeflate {
    settings: DeflateSettings,
}

impl DeflateSettings {
    pub fn zlib_header() -> Self {
        DeflateSettings {
            zlib_header: true,
        }
    }

    pub fn no_zlib_header() -> Self {
        DeflateSettings {
            zlib_header: false,
        }
    }
}

impl TransformDeflate {
    pub fn new(settings: DeflateSettings) -> Self {
        TransformDeflate {
            settings: settings,
        }
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
        match self.settings.zlib_header {
            true => self.transform_deflated_zlib(buffer),
            false => self.transform_deflated(buffer),
        }
    }

    fn untransform(&self, _buffer: &Vec<u8>) -> SimpleResult<Vec<u8>> {
        bail!("Deflate cannot be untransformed");
    }

    fn check(&self, buffer: &Vec<u8>) -> bool {
        match self.settings.zlib_header {
            true => self.check_deflated_zlib(buffer),
            false => self.check_deflated(buffer),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use crate::transformation::Transformation;

    #[test]
    fn test_deflate() -> SimpleResult<()> {
        let t = Transformation::FromDeflatedNoZlibHeader;

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
        let t = Transformation::FromDeflatedZlibHeader;

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
