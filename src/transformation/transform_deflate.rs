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
