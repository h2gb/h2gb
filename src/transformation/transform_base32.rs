use base32;
use simple_error::{SimpleResult, bail};
use serde::{Serialize, Deserialize};

use crate::transformation::TransformerTrait;

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Serialize, Deserialize)]
pub struct Base32Settings {
    no_padding: bool,
    permissive: bool,
    crockford: bool,
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Serialize, Deserialize)]
pub struct TransformBase32 {
    settings: Base32Settings,
}

impl Base32Settings {
    pub fn standard() -> Self {
        Base32Settings {
            no_padding: false,
            permissive: false,
            crockford: false,
        }
    }

    pub fn no_padding() -> Self {
        Base32Settings {
            no_padding: true,
            permissive: false,
            crockford: false,
        }
    }

    pub fn permissive() -> Self {
        Base32Settings {
            no_padding: false,
            permissive: true,
            crockford: false,
        }
    }

    pub fn crockford() -> Self {
        Base32Settings {
            no_padding: false,
            permissive: false,
            crockford: true,
        }
    }

    pub fn crockford_no_padding() -> Self {
        Base32Settings {
            no_padding: true,
            permissive: false,
            crockford: true,
        }
    }

    pub fn crockford_permissive() -> Self {
        Base32Settings {
            no_padding: false,
            permissive: true,
            crockford: true,
        }
    }

    fn get_alphabet(&self) -> base32::Alphabet {
        match self.crockford {
            false => base32::Alphabet::RFC4648 { padding: !self.no_padding },
            true  => base32::Alphabet::Crockford,
        }
    }
}

impl TransformBase32 {
    pub fn new(settings: Base32Settings) -> Self {
        TransformBase32 {
            settings: settings,
        }
    }

    fn transform_standard(self, buffer: &Vec<u8>) -> SimpleResult<Vec<u8>> {
        let original_length = buffer.len();

        let s = match std::str::from_utf8(buffer) {
            Ok(s) => s,
            Err(e) => bail!("Couldn't convert the buffer into a string: {}", e),
        };

        // Decode
        let out = match base32::decode(self.settings.get_alphabet(), &s) {
            Some(r) => r,
            None => bail!("Couldn't decode base32"),
        };

        // Ensure it encodes to the same length - we can't handle length changes
        if base32::encode(self.settings.get_alphabet(), &out).into_bytes().len() != original_length {
            bail!("Base32 didn't decode correctly");
        }

        Ok(out)
    }

    fn untransform_standard(self, buffer: &Vec<u8>) -> SimpleResult<Vec<u8>> {
        Ok(base32::encode(self.settings.get_alphabet(), buffer).into_bytes())
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
        match base32::decode(self.settings.get_alphabet(), &s) {
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
        match self.settings.permissive {
            false => self.transform_standard(buffer),
            true  => self.transform_permissive(buffer),
        }
    }

    fn untransform(&self, buffer: &Vec<u8>) -> SimpleResult<Vec<u8>> {
        match self.settings.permissive {
            false => self.untransform_standard(buffer),
            true  => self.untransform_permissive(buffer),
        }
    }

    fn check(&self, buffer: &Vec<u8>) -> bool {
        match self.settings.permissive {
            false => self.check_standard(buffer),
            true  => self.check_permissive(buffer),
        }
    }
}
