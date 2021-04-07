use nettle::cipher::{Salsa20_128, Salsa20_256};

use simple_error::{SimpleResult, bail};
use serde::{Serialize, Deserialize};

use crate::transformation::TransformerTrait;
use crate::transformation::key_or_iv::KeyOrIV;

/// Which stream cipher should we use?
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Serialize, Deserialize)]
pub enum StreamCipherType {
    Salsa20,
}

/// Configures a stream cipher.
#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Serialize, Deserialize)]
pub struct StreamCipherSettings {
    cipher: StreamCipherType,
    key: KeyOrIV,
    iv: KeyOrIV,
    offset: u64,
}

impl StreamCipherSettings {
    /// Create a new instance of [`StreamCipherSettings`].
    pub fn new(cipher: StreamCipherType, key: Vec<u8>, iv: Vec<u8>) -> SimpleResult<Self> {
        // Validate and store the key / iv
        let key = KeyOrIV::new(key)?;
        let iv = KeyOrIV::new(iv)?;

        // Create the result so we can validate it
        let result = StreamCipherSettings {
            cipher: cipher,
            key: key,
            iv: iv,
            offset: 0,
        };

        // This validates the key length and iv and other characteristics
        result.validate_settings()?;

        Ok(result)
    }

    /// Internal function to decrypt
    fn decrypt_salsa20(self, buffer: &Vec<u8>) -> SimpleResult<Vec<u8>> {
        // Make sure the nonce is sane
        let nonce = self.iv.get64()?;

        // Create a buffer for the output
        let mut encrypted = vec![0; buffer.len()];

        match self.key {
            KeyOrIV::Bits128(k) => {
                let mut c = match Salsa20_128::with_key_and_nonce(&k, &nonce) {
                    Ok(c) => c,
                    Err(e) => bail!("Salsa20_128 cipher failed: {}", e),
                };
                c.crypt(&mut encrypted, buffer);
            },
            KeyOrIV::Bits256(k) => {
                let mut c = match Salsa20_256::with_key_and_nonce(&k, &nonce) {
                    Ok(c) => c,
                    Err(e) => bail!("Salsa20_256 cipher failed: {}", e),
                };
                c.crypt(&mut encrypted, buffer);
            },
            _ => bail!("Invalid key size for Salsa20"),
        };

        Ok(encrypted)
    }

    /// Internal function to encrypt
    fn encrypt_salsa20(self, buffer: &Vec<u8>) -> SimpleResult<Vec<u8>> {
        // Encrypting is literally identical to decrypting
        self.decrypt_salsa20(buffer)
    }

    // /// Internal function to decrypt
    // fn decrypt_chacha(self, buffer: &Vec<u8>) -> SimpleResult<Vec<u8>> {
    //     // Make sure the nonce is sane
    //     let nonce = self.iv.get64()?;

    //     // Create a buffer for the output
    //     let mut encrypted = vec![0; buffer.len()];

    //     match self.key {
    //         KeyOrIV::Bits256(k) => {
    //             let mut c = match ChaCha::with_key_and_nonce(&k, &nonce) {
    //                 Ok(c) => c,
    //                 Err(e) => bail!("ChaCha cipher failed: {}", e),
    //             };
    //             c.crypt(&mut encrypted, buffer);
    //         },
    //         _ => bail!("Invalid key size for ChaCha"),
    //     };

    //     Ok(encrypted)
    // }

    // /// Internal function to encrypt
    // fn encrypt_chacha(self, buffer: &Vec<u8>) -> SimpleResult<Vec<u8>> {
    //     // Encrypting is literally identical to decrypting
    //     self.decrypt_chacha(buffer)
    // }

    /// Sanity check settings (key size, IV, etc).
    fn validate_settings(self) -> SimpleResult<()> {
        match (self.cipher, self.iv, self.key) {
            (StreamCipherType::Salsa20, KeyOrIV::Bits64(_), KeyOrIV::Bits128(_)) => (),
            (StreamCipherType::Salsa20, KeyOrIV::Bits64(_), KeyOrIV::Bits256(_)) => (),
            (StreamCipherType::Salsa20, _, _) => bail!("Invalid stream cipher settings for Salsa20"),

            // (StreamCipherType::ChaCha,  KeyOrIV::Bits64(_), KeyOrIV::Bits256(_)) => (),
            // (StreamCipherType::ChaCha, _, _) => bail!("Invalid stream cipher settings for ChaCha"),
        }

        Ok(())
    }
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Serialize, Deserialize)]
pub struct TransformStreamCipher {
    settings: StreamCipherSettings,
}

impl TransformStreamCipher {
    pub fn new(settings: StreamCipherSettings) -> Self {
        TransformStreamCipher {
            settings: settings,
        }
    }

}

impl TransformerTrait for TransformStreamCipher {
    /// transform() =~ decrypt
    fn transform(&self, buffer: &Vec<u8>) -> SimpleResult<Vec<u8>> {
        self.settings.validate_settings()?;

        match self.settings.cipher {
            StreamCipherType::Salsa20 => self.settings.decrypt_salsa20(buffer),
            // StreamCipherType::ChaCha  => self.settings.decrypt_chacha(buffer),
        }
    }

    /// transform() =~ encrypt
    fn untransform(&self, buffer: &Vec<u8>) -> SimpleResult<Vec<u8>> {
        self.settings.validate_settings()?;

        match self.settings.cipher {
            StreamCipherType::Salsa20 => self.settings.encrypt_salsa20(buffer),
            // StreamCipherType::ChaCha  => self.settings.encrypt_chacha(buffer),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    use crate::transformation::Transformation;

    #[test]
    fn test_salsa20() -> SimpleResult<()> {
        let tests: Vec<(Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>)> = vec![
            (
                b"Testing Salsa20".to_vec(),                  // Plaintext
                b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_vec(), // Key
                b"BBBBBBBB".to_vec(),                         // IV
                // Ciphertext
                b"\x96\xb9\x31\xa2\x3b\xff\x65\x11\xe4\xba\x52\x79\xb1\xaa\x57".to_vec(),
            ),
            (
                b"Testing Salsa20".to_vec(),                  // Plaintext
                b"AAAAAAAAAAAAAAAA".to_vec(),                 // Key
                b"BBBBBBBB".to_vec(),                         // IV
                // Ciphertext
                b"\xe8\xc2\x42\x85\x3d\x40\x9b\xa6\x06\xe6\x83\xe0\x0d\x37\xd6".to_vec(),
            ),
        ];

        for (plaintext, key, iv, ciphertext) in tests {
            let transformation = Transformation::FromStreamCipher(StreamCipherSettings::new(
                StreamCipherType::Salsa20,
                key,
                iv,
            )?);

            let result = transformation.transform(&ciphertext)?;
            assert_eq!(plaintext, result, "salsa20 transform {}", std::str::from_utf8(&plaintext).unwrap());

            let result = transformation.untransform(&result)?;
            assert_eq!(ciphertext, result, "salsa20 untransform {}", std::str::from_utf8(&plaintext).unwrap());
        }

        Ok(())
    }

    // #[test]
    // fn test_chacha() -> SimpleResult<()> {
    //     let tests: Vec<(Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>)> = vec![
    //         (
    //             b"\xe2\x8a\x5f\xa4\xa6\x7f\x8c\x5d\xef\xed\x3e\x6f\xb7\x30\x34\x86".to_vec(),
    //             b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\
    //               \x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".to_vec(), // Key
    //             b"\x00\x00\x00\x00\x00\x00\x00\x00".to_vec(),                                 // IV
    //             // Ciphertext
    //             b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".to_vec(), // Plaintext
    //         ),
    //         // (
    //         //     b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".to_vec(), // Plaintext
    //         //     b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\
    //         //       \x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".to_vec(), // Key
    //         //     b"\x01\x01\x01\x01\x01\x01\x01\x01".to_vec(),                                 // IV
    //         //     // Ciphertext
    //         //     b"".to_vec()
    //         // ),
    //     ];

    //     for (plaintext, key, iv, ciphertext) in tests {
    //         let transformation = Transformation::FromStreamCipher(StreamCipherSettings::new(
    //             StreamCipherType::Salsa20,
    //             key,
    //             iv,
    //         )?);

    //         let result = transformation.transform(&ciphertext)?;
    //         assert_eq!(plaintext, result);

    //         // let result = transformation.untransform(&result)?;
    //         // assert_eq!(ciphertext, result, "chacha untransform {}", std::str::from_utf8(&plaintext).unwrap());
    //     }

    //     Ok(())
    // }
}
