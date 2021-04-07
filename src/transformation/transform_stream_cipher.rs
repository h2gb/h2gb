use cipher::{NewStreamCipher, SyncStreamCipher, SyncStreamCipherSeek};
use salsa20::Salsa20;
use chacha20::ChaCha20Legacy;

// These aren't implemented in the standard cipher crate
use nettle::cipher::Salsa20_128;
use nettle::cipher::insecure_do_not_use::ArcFour;

use simple_error::{SimpleResult, bail};
use serde::{Serialize, Deserialize};

use crate::transformation::TransformerTrait;
use crate::transformation::key_or_iv::KeyOrIV;

/// Which stream cipher should we use?
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Serialize, Deserialize)]
pub enum StreamCipherType {
    // Salsa20 (128 or 256-bit key, 64-bit IV)
    Salsa20,

    // ChaCha (256-bit key, 64-bit IV, 20 rounds)
    ChaCha,

    // RC4 (40 - 256-bit key (though we only support powers of 2), no nonce)
    Arc4,
}

/// Configures a stream cipher.
#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Serialize, Deserialize)]
pub struct StreamCipherSettings {
    cipher: StreamCipherType,
    key: KeyOrIV,
    iv: Option<KeyOrIV>,
    offset: u64,
}

impl StreamCipherSettings {
    /// Create a new instance of [`StreamCipherSettings`].
    pub fn new(cipher: StreamCipherType, key: Vec<u8>, iv: Option<Vec<u8>>) -> SimpleResult<Self> {
        // Validate and store the key / iv
        let key = KeyOrIV::new(key)?;
        let iv = match iv {
            Some(iv) => Some(KeyOrIV::new(iv)?),
            None     => None,
        };

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
        let nonce = match self.iv {
            Some(iv) => iv.get64()?,
            None => bail!("Salsa20 requires an IV/Nonce"),
        };

        match self.key {
            // The stream-cipher library pack doesn't seem to support
            // Salsa20-128, so we use the nettle library (which is simpler to
            // use, but seems to have some bugs)
            KeyOrIV::Bits128(k) => {
                // Create a buffer for the output
                let mut encrypted = vec![0; buffer.len()];

                let mut c = match Salsa20_128::with_key_and_nonce(&k, &nonce) {
                    Ok(c) => c,
                    Err(e) => bail!("Salsa20_128 cipher failed: {}", e),
                };
                c.crypt(&mut encrypted, buffer);

                Ok(encrypted)
            },
            KeyOrIV::Bits256(k) => {
                let key = salsa20::Key::from_slice(&k);

                let nonce = salsa20::Nonce::from_slice(&nonce);

                let mut cipher = Salsa20::new(&key, &nonce);

                // Clone the buffer to something mutable
                let mut buffer = buffer.clone();

                // Apply the keystream to decrypt it
                cipher.apply_keystream(&mut buffer);

                // And I guess that's it
                Ok(buffer)
            },
            _ => bail!("Invalid key size for Salsa20"),
        }
    }

    /// Internal function to encrypt
    fn encrypt_salsa20(self, buffer: &Vec<u8>) -> SimpleResult<Vec<u8>> {
        // Encrypting is literally identical to decrypting
        self.decrypt_salsa20(buffer)
    }

    /// Internal function to decrypt
    fn decrypt_chacha(self, buffer: &Vec<u8>) -> SimpleResult<Vec<u8>> {
        // Get the nonce
        // Make sure the nonce is sane
        let nonce = match self.iv {
            Some(iv) => iv.get64()?,
            None => bail!("ChaCha requires an IV/Nonce"),
        };
        let nonce = chacha20::LegacyNonce::from_slice(&nonce);

        // Get the key
        let key = self.key.get256()?;
        let key = chacha20::Key::from_slice(&key);

        let mut cipher = ChaCha20Legacy::new(&key, &nonce);

        // Clone the buffer to something mutable
        let mut buffer = buffer.clone();

        // Apply the keystream to decrypt it
        cipher.apply_keystream(&mut buffer);

        // And I guess that's it
        Ok(buffer)
    }

    /// Internal function to encrypt
    fn encrypt_chacha(self, buffer: &Vec<u8>) -> SimpleResult<Vec<u8>> {
        // Encrypting is literally identical to decrypting
        self.decrypt_chacha(buffer)
    }

    /// Internal function to decrypt
    fn decrypt_arc4(self, buffer: &Vec<u8>) -> SimpleResult<Vec<u8>> {
        let mut c = match self.key {
            KeyOrIV::Bits64(k)  => ArcFour::with_key(&k),
            KeyOrIV::Bits128(k) => ArcFour::with_key(&k),
            KeyOrIV::Bits192(k) => ArcFour::with_key(&k),
            KeyOrIV::Bits256(k) => ArcFour::with_key(&k),
        };

        // Create a buffer for the output
        let mut encrypted = vec![0; buffer.len()];

        // Do the encryption
        c.crypt(&mut encrypted, buffer);

        Ok(encrypted)
    }

    /// Internal function to encrypt
    fn encrypt_arc4(self, buffer: &Vec<u8>) -> SimpleResult<Vec<u8>> {
        // Encrypting is literally identical to decrypting
        self.decrypt_arc4(buffer)
    }
    /// Sanity check settings (key size, IV, etc).
    fn validate_settings(self) -> SimpleResult<()> {
        match (self.cipher,             self.iv,                  self.key) {
            (StreamCipherType::Salsa20, Some(KeyOrIV::Bits64(_)), KeyOrIV::Bits128(_)) => (),
            (StreamCipherType::Salsa20, Some(KeyOrIV::Bits64(_)), KeyOrIV::Bits256(_)) => (),
            (StreamCipherType::Salsa20, _, _) => bail!("Invalid stream cipher settings for Salsa20"),

            (StreamCipherType::ChaCha,  Some(KeyOrIV::Bits64(_)), KeyOrIV::Bits256(_)) => (),
            (StreamCipherType::ChaCha,  Some(KeyOrIV::Bits64(_)), _                  ) => bail!("Invalid key size for ChaCha"),
            (StreamCipherType::ChaCha,  _,                        KeyOrIV::Bits256(_)) => bail!("Invalid iv size for ChaCha"),
            (StreamCipherType::ChaCha,  _,                        _                  ) => bail!("Invalid key and iv sizes for ChaCha"),

            (StreamCipherType::Arc4,    None,                     _                  ) => (),
            (StreamCipherType::Arc4,    _,                        _                  ) => bail!("Arc4 does not support an IV"),
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
            StreamCipherType::ChaCha  => self.settings.decrypt_chacha(buffer),
            StreamCipherType::Arc4    => self.settings.decrypt_arc4(buffer),
        }
    }

    /// transform() =~ encrypt
    fn untransform(&self, buffer: &Vec<u8>) -> SimpleResult<Vec<u8>> {
        self.settings.validate_settings()?;

        match self.settings.cipher {
            StreamCipherType::Salsa20 => self.settings.encrypt_salsa20(buffer),
            StreamCipherType::ChaCha  => self.settings.encrypt_chacha(buffer),
            StreamCipherType::Arc4    => self.settings.encrypt_arc4(buffer),
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
                Some(iv),
            )?);

            let result = transformation.transform(&ciphertext)?;
            assert_eq!(plaintext, result, "salsa20 transform {}", std::str::from_utf8(&plaintext).unwrap());

            let result = transformation.untransform(&result)?;
            assert_eq!(ciphertext, result, "salsa20 untransform {}", std::str::from_utf8(&plaintext).unwrap());
        }

        Ok(())
    }

    #[test]
    fn test_chacha() -> SimpleResult<()> {
        // Test vectors from https://tools.ietf.org/html/draft-strombergson-chacha-test-vectors-00
        let tests: Vec<(Vec<u8>, Vec<u8>, Vec<u8>)> = vec![
            (
                b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\
                  \x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".to_vec(), // Key
                b"\x00\x00\x00\x00\x00\x00\x00\x00".to_vec(),                                 // IV
                // Ciphertext
                b"\x76\xb8\xe0\xad\xa0\xf1\x3d\x90".to_vec(),
            ),
            (
                b"\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\
                  \x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".to_vec(), // Key
                b"\x00\x00\x00\x00\x00\x00\x00\x00".to_vec(),                                 // IV
                // Ciphertext
                b"\xc5\xd3\x0a\x7c\xe1\xec\x11\x93".to_vec(),
            ),
        ];

        for (key, iv, ciphertext) in tests {
            let transformation = Transformation::FromStreamCipher(StreamCipherSettings::new(
                StreamCipherType::ChaCha,
                key,
                Some(iv),
            )?);

            let result = transformation.transform(&ciphertext)?;
            assert_eq!(vec![0; ciphertext.len()], result);

            let result = transformation.untransform(&result)?;
            assert_eq!(ciphertext, result);
        }

        Ok(())
    }

    #[test]
    fn test_arc4() -> SimpleResult<()> {
        let tests: Vec<(Vec<u8>, Vec<u8>, Vec<u8>)> = vec![
            (
                b"My fun RC4 test".to_vec(),                  // Plaintext
                b"AAAAAAAA".to_vec(),                         // Key
                // Ciphertext
                b"\x8f\xb2\xc6\x5b\xb5\xcd\xed\xf3\xf9\x9f\x49\x28\x81\x83\x41".to_vec(),
            ),
            (
                b"Testing RC4 with longer key".to_vec(),      // Plaintext
                b"AAAAAAAAAAAAAAAA".to_vec(),                 // Key
                // Ciphertext
                b"\x96\xae\x95\x49\xa9\xcd\xaa\x81\xe8\xe8\x5d\x7c\x93\x99\x41\x43\x57\x29\xfd\xdc\x73\xcc\x77\x59\x01\x0e\xd0".to_vec(),
            ),
            (
                b"Testing 192-bit key".to_vec(),              // Plaintext
                b"AAAAAAAAAAAAAAAAAAAAAAAA".to_vec(),         // Key
                // Ciphertext
                b"\x96\xae\x95\x49\xa9\xcd\xaa\x81\x8b\x92\x5b\x71\x86\x99\x41\x0b\x1c\x20\xeb".to_vec(),
            ),
        ];

        for (plaintext, key, ciphertext) in tests {
            let transformation = Transformation::FromStreamCipher(StreamCipherSettings::new(
                StreamCipherType::Arc4,
                key,
                None,
            )?);

            let result = transformation.transform(&ciphertext)?;
            assert_eq!(plaintext, result, "Arc4 transform {}", std::str::from_utf8(&plaintext).unwrap());

            let result = transformation.untransform(&result)?;
            assert_eq!(ciphertext, result, "Arc4 untransform {}", std::str::from_utf8(&plaintext).unwrap());
        }

        Ok(())
    }
}
