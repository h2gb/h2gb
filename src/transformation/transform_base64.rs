use base64;
use simple_error::{SimpleResult, bail};
use serde::{Serialize, Deserialize};

use crate::transformation::TransformerTrait;

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Serialize, Deserialize)]
pub struct Base64Settings {
    no_padding: bool,
    permissive: bool,
    url: bool,
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Serialize, Deserialize)]
pub struct TransformBase64 {
    settings: Base64Settings,
}

impl Base64Settings {
    pub fn standard() -> Self {
        Base64Settings {
            no_padding: false,
            permissive: false,
            url: false,
        }
    }

    pub fn no_padding() -> Self {
        Base64Settings {
            no_padding: true,
            permissive: false,
            url: false,
        }
    }

    pub fn permissive() -> Self {
        Base64Settings {
            no_padding: false,
            permissive: true,
            url: false,
        }
    }

    pub fn url() -> Self {
        Base64Settings {
            no_padding: false,
            permissive: false,
            url: true,
        }
    }

    pub fn url_no_padding() -> Self {
        Base64Settings {
            no_padding: true,
            permissive: false,
            url: true,
        }
    }

    pub fn url_permissive() -> Self {
        Base64Settings {
            no_padding: false,
            permissive: true,
            url: true,
        }
    }

    fn get_config(&self) -> base64::Config {
        match (self.no_padding, self.url) {
            (false, false) => base64::STANDARD,
            (true,  false) => base64::STANDARD_NO_PAD,
            (false, true)  => base64::URL_SAFE,
            (true,  true)  => base64::URL_SAFE_NO_PAD,
        }
    }
}

impl TransformBase64 {
    pub fn new(settings: Base64Settings) -> Self {
        TransformBase64 {
            settings: settings,
        }
    }

    fn transform_standard(&self, buffer: &Vec<u8>) -> SimpleResult<Vec<u8>> {
        let original_length = buffer.len();

        // Decode
        let out = match base64::decode_config(buffer, self.settings.get_config()) {
            Ok(r) => r,
            Err(e) => bail!("Couldn't decode base64: {}", e),
        };

        // Ensure it encodes to the same length - we can't handle length changes
        if base64::encode_config(&out, self.settings.get_config()).len() != original_length {
            bail!("Base64 didn't decode correctly (the length changed with decode->encode, check padding)");
        }

        Ok(out)
    }

    fn untransform_standard(&self, buffer: &Vec<u8>) -> SimpleResult<Vec<u8>> {
        Ok(base64::encode_config(buffer, self.settings.get_config()).into_bytes())
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
        let out = match base64::decode_config(buffer, self.settings.get_config()) {
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
