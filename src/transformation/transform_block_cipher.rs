use aes::{Aes128, Aes192, Aes256};
use block_modes::{BlockMode, Ecb, Cbc, Cfb, Cfb8, Ofb, Pcbc};
use block_modes::block_padding::{NoPadding, Pkcs7};

use simple_error::{SimpleResult, bail};
use serde::{Serialize, Deserialize};

use crate::transformation::TransformerTrait;

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Serialize, Deserialize)]
enum KeyOrIV {
    Bits64([u8; 8]),
    Bits128([u8; 16]),
    Bits192([u8; 24]),
    Bits256([u8; 32]),
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Serialize, Deserialize)]
enum Padding {
    /// Don't pad
    NoPadding,

    /// Pad with bytes that are each the number of bytes
    Pkcs7,

    // Other things we *can* do if we want?
    // AnsiX923	Pad block with zeros except the last byte which will be set to the number bytes.
    // Iso7816	Pad block with byte sequence \x80 00...00 00.
    // ZeroPadding	Pad block with zeros.
}

impl KeyOrIV {
    fn new(key: Vec<u8>) -> SimpleResult<Self> {
        Ok(match key.len() {
            8 => {
                let mut a = [0; 8];
                a.copy_from_slice(&key);
                KeyOrIV::Bits64(a)
            }
            16 => {
                let mut a = [0; 16];
                a.copy_from_slice(&key);
                KeyOrIV::Bits128(a)
            }
            24 => {
                let mut a = [0; 24];
                a.copy_from_slice(&key);
                KeyOrIV::Bits192(a)
            }
            32 => {
                let mut a = [0; 32];
                a.copy_from_slice(&key);
                KeyOrIV::Bits256(a)
            }
            _  => bail!("Invalid BlockCipher key or iv length: {} bytes / {} bits", key.len(), key.len() * 8),
        })
    }
}

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Serialize, Deserialize)]
pub enum CipherType {
    // AES (128, 192, or 256-bit) with Electronic Codebook chaining
    AES_ECB,

    // AES (128, 192, or 256-bit) with Cipher Block Chaining
    AES_CBC,

    // AES (128, 192, or 256-bit) with Cipher Feedback Chaining
    AES_CFB,
}

impl CipherType {
    fn aes_check_length(length: usize) -> SimpleResult<()> {
        if length % 16 != 0 {
            bail!("AES length must be a multiple of 16 bytes");
        }

        Ok(())
    }

    fn validate_aes_ecb(key: KeyOrIV, iv: Option<KeyOrIV>) -> SimpleResult<()> {
        // Key is 128, 192, or 256
        match key {
            KeyOrIV::Bits128(_) => (),
            KeyOrIV::Bits192(_) => (),
            KeyOrIV::Bits256(_) => (),
            _ => bail!("Invalid key length for AES-ECB"),
        };

        // IV is not allowed
        if iv.is_some() {
            bail!("AES-ECB cannot have an IV");
        }

        Ok(())
    }

    fn validate_aes_cbc(key: KeyOrIV, iv: Option<KeyOrIV>) -> SimpleResult<()> {
        // Key is 128, 192, or 256
        match key {
            KeyOrIV::Bits128(_) => (),
            KeyOrIV::Bits192(_) => (),
            KeyOrIV::Bits256(_) => (),
            _ => bail!("Invalid key length for AES_CBC"),
        };

        // IV is optional, 128 bits
        match iv {
            Some(iv) => {
                match iv {
                    KeyOrIV::Bits128(_) => (),
                    _ => bail!("Invalid IV length for AES_CBC"),
                }
            },
            None => (),
        };

        Ok(())
    }

    fn validate_aes_cfb(key: KeyOrIV, iv: Option<KeyOrIV>) -> SimpleResult<()> {
        // Key is 128, 192, or 256
        match key {
            KeyOrIV::Bits128(_) => (),
            KeyOrIV::Bits192(_) => (),
            KeyOrIV::Bits256(_) => (),
            _ => bail!("Invalid key length for AES_CFB"),
        };

        // IV is optional, 128 bits
        match iv {
            Some(iv) => {
                match iv {
                    KeyOrIV::Bits128(_) => (),
                    _ => bail!("Invalid IV length for AES_CFB"),
                }
            },
            None => (),
        };

        Ok(())
    }

    fn decrypt_aes_ecb(buffer: &Vec<u8>, key: KeyOrIV) -> SimpleResult<Vec<u8>> {
        Self::aes_check_length(buffer.len())?;

        // Pick the implementation based on the key
        let out = match key {
            KeyOrIV::Bits128(k) => {
                match Ecb::<Aes128, Pkcs7>::new_var(&k, Default::default()) {
                    Ok(c) => {
                        match c.decrypt_vec(&buffer) {
                            Ok(d) => d,
                            Err(e) => bail!("Error decrypting buffer: {}", e),
                        }
                    }
                    Err(e) => bail!("Error setting up cipher: {}", e),
                }
            },

            KeyOrIV::Bits192(k) => {
                match Ecb::<Aes192, Pkcs7>::new_var(&k, Default::default()) {
                    Ok(c) => {
                        match c.decrypt_vec(&buffer) {
                            Ok(d) => d,
                            Err(e) => bail!("Error decrypting buffer: {}", e),
                        }
                    }
                    Err(e) => bail!("Error setting up cipher: {}", e),
                }
            },

            KeyOrIV::Bits256(k) => {
                match Ecb::<Aes256, Pkcs7>::new_var(&k, Default::default()) {
                    Ok(c) => {
                        match c.decrypt_vec(&buffer) {
                            Ok(d) => d,
                            Err(e) => bail!("Error decrypting buffer: {}", e),
                        }
                    }
                    Err(e) => bail!("Error setting up cipher: {}", e),
                }
            },

            _ => {
                bail!("Invalid key size for AES-ECB");
            },
        };

        Ok(out.to_vec())
    }

    fn decrypt_aes_cbc(buffer: &Vec<u8>, key: KeyOrIV, iv: Option<KeyOrIV>) -> SimpleResult<Vec<u8>> {
        Self::aes_check_length(buffer.len())?;

        // Get the iv, or a default blank one
        let iv = match iv {
            Some(iv) => {
                match iv {
                    KeyOrIV::Bits128(iv) => iv,
                    _ => bail!("Invalid IV length"),
                }
            },
            None => [0; 16],
        };

        // Pick the implementation based on the key
        let out = match key {
            KeyOrIV::Bits128(k) => {
                match Cbc::<Aes128, Pkcs7>::new_var(&k, &iv) {
                    Ok(c) => {
                        match c.decrypt_vec(&buffer) {
                            Ok(d) => d,
                            Err(e) => bail!("Error decrypting buffer: {}", e),
                        }
                    }
                    Err(e) => bail!("Error setting up cipher: {}", e),
                }
            },

            KeyOrIV::Bits192(k) => {
                match Cbc::<Aes192, Pkcs7>::new_var(&k, &iv) {
                    Ok(c) => {
                        match c.decrypt_vec(&buffer) {
                            Ok(d) => d,
                            Err(e) => bail!("Error decrypting buffer: {}", e),
                        }
                    }
                    Err(e) => bail!("Error setting up cipher: {}", e),
                }
            },

            KeyOrIV::Bits256(k) => {
                match Cbc::<Aes256, Pkcs7>::new_var(&k, &iv) {
                    Ok(c) => {
                        match c.decrypt_vec(&buffer) {
                            Ok(d) => d,
                            Err(e) => bail!("Error decrypting buffer: {}", e),
                        }
                    }
                    Err(e) => bail!("Error setting up cipher: {}", e),
                }
            },

            _ => {
                bail!("Invalid key size for AES-CBC");
            },
        };

        Ok(out.to_vec())
    }

    fn decrypt_aes_cfb(buffer: &Vec<u8>, key: KeyOrIV, iv: Option<KeyOrIV>) -> SimpleResult<Vec<u8>> {
        Self::aes_check_length(buffer.len())?;

        // Get the iv, or a default blank one
        let iv = match iv {
            Some(iv) => {
                match iv {
                    KeyOrIV::Bits128(iv) => iv,
                    _ => bail!("Invalid IV length"),
                }
            },
            None => [0; 16],
        };

        // Pick the implementation based on the key
        let out = match key {
            KeyOrIV::Bits128(k) => {
                match Cfb::<Aes128, Pkcs7>::new_var(&k, &iv) {
                    Ok(c) => {
                        match c.decrypt_vec(&buffer) {
                            Ok(d) => d,
                            Err(e) => bail!("Error decrypting buffer: {}", e),
                        }
                    }
                    Err(e) => bail!("Error setting up cipher: {}", e),
                }
            },

            KeyOrIV::Bits192(k) => {
                match Cfb::<Aes192, Pkcs7>::new_var(&k, &iv) {
                    Ok(c) => {
                        match c.decrypt_vec(&buffer) {
                            Ok(d) => d,
                            Err(e) => bail!("Error decrypting buffer: {}", e),
                        }
                    }
                    Err(e) => bail!("Error setting up cipher: {}", e),
                }
            },

            KeyOrIV::Bits256(k) => {
                match Cfb::<Aes256, Pkcs7>::new_var(&k, &iv) {
                    Ok(c) => {
                        match c.decrypt_vec(&buffer) {
                            Ok(d) => d,
                            Err(e) => bail!("Error decrypting buffer: {}", e),
                        }
                    }
                    Err(e) => bail!("Error setting up cipher: {}", e),
                }
            },

            _ => {
                bail!("Invalid key size for AES-CFB");
            },
        };

        Ok(out.to_vec())
    }

    fn encrypt_aes_ecb(buffer: &Vec<u8>, key: KeyOrIV) -> SimpleResult<Vec<u8>> {
        // Pick the implementation based on the key
        let out = match key {
            KeyOrIV::Bits128(k) => {
                match Ecb::<Aes128, Pkcs7>::new_var(&k, Default::default()) {
                    Ok(c) => c.encrypt_vec(&buffer),
                    Err(e) => bail!("Error setting up cipher: {}", e),
                }
            },

            KeyOrIV::Bits192(k) => {
                match Ecb::<Aes192, Pkcs7>::new_var(&k, Default::default()) {
                    Ok(c) => c.encrypt_vec(&buffer),
                    Err(e) => bail!("Error setting up cipher: {}", e),
                }
            },

            KeyOrIV::Bits256(k) => {
                match Ecb::<Aes256, Pkcs7>::new_var(&k, Default::default()) {
                    Ok(c) => c.encrypt_vec(&buffer),
                    Err(e) => bail!("Error setting up cipher: {}", e),
                }
            },

            _ => {
                bail!("Invalid key size for AES-ECB");
            },
        };

        Ok(out.to_vec())
    }

    fn encrypt_aes_cbc(buffer: &Vec<u8>, key: KeyOrIV, iv: Option<KeyOrIV>) -> SimpleResult<Vec<u8>> {
        // Get the iv, or a default blank one
        let iv = match iv {
            Some(iv) => {
                match iv {
                    KeyOrIV::Bits128(iv) => iv,
                    _ => bail!("Invalid IV length"),
                }
            },
            None => [0; 16],
        };

        // Pick the implementation based on the key
        let out = match key {
            KeyOrIV::Bits128(k) => {
                match Cbc::<Aes128, Pkcs7>::new_var(&k, &iv) {
                    Ok(c) => c.encrypt_vec(&buffer),
                    Err(e) => bail!("Error setting up cipher: {}", e),
                }
            },

            KeyOrIV::Bits192(k) => {
                match Cbc::<Aes192, Pkcs7>::new_var(&k, &iv) {
                    Ok(c) => c.encrypt_vec(&buffer),
                    Err(e) => bail!("Error setting up cipher: {}", e),
                }
            },

            KeyOrIV::Bits256(k) => {
                match Cbc::<Aes256, Pkcs7>::new_var(&k, &iv) {
                    Ok(c) => c.encrypt_vec(&buffer),
                    Err(e) => bail!("Error setting up cipher: {}", e),
                }
            },

            _ => {
                bail!("Invalid key size for AES-CBC");
            },
        };

        Ok(out.to_vec())
    }

    fn encrypt_aes_cfb(buffer: &Vec<u8>, key: KeyOrIV, iv: Option<KeyOrIV>) -> SimpleResult<Vec<u8>> {
        // Get the iv, or a default blank one
        let iv = match iv {
            Some(iv) => {
                match iv {
                    KeyOrIV::Bits128(iv) => iv,
                    _ => bail!("Invalid IV length"),
                }
            },
            None => [0; 16],
        };

        // Pick the implementation based on the key
        let out = match key {
            KeyOrIV::Bits128(k) => {
                match Cfb::<Aes128, Pkcs7>::new_var(&k, &iv) {
                    Ok(c) => c.encrypt_vec(&buffer),
                    Err(e) => bail!("Error setting up cipher: {}", e),
                }
            },

            KeyOrIV::Bits192(k) => {
                match Cfb::<Aes192, Pkcs7>::new_var(&k, &iv) {
                    Ok(c) => c.encrypt_vec(&buffer),
                    Err(e) => bail!("Error setting up cipher: {}", e),
                }
            },

            KeyOrIV::Bits256(k) => {
                match Cfb::<Aes256, Pkcs7>::new_var(&k, &iv) {
                    Ok(c) => c.encrypt_vec(&buffer),
                    Err(e) => bail!("Error setting up cipher: {}", e),
                }
            },

            _ => {
                bail!("Invalid key size for AES-CFB");
            },
        };

        Ok(out.to_vec())
    }

    fn validate_settings(self, key: KeyOrIV, iv: Option<KeyOrIV>) -> SimpleResult<()> {
        match self {
            Self::AES_ECB => Self::validate_aes_ecb(key, iv),
            Self::AES_CBC => Self::validate_aes_cbc(key, iv),
            Self::AES_CFB => Self::validate_aes_cfb(key, iv),
        }
    }

    fn decrypt(self, buffer: &Vec<u8>, key: KeyOrIV, iv: Option<KeyOrIV>) -> SimpleResult<Vec<u8>> {
        match self {
            Self::AES_ECB => Self::decrypt_aes_ecb(buffer, key),
            Self::AES_CBC => Self::decrypt_aes_cbc(buffer, key, iv),
            Self::AES_CFB => Self::decrypt_aes_cfb(buffer, key, iv),
        }
    }

    fn encrypt(self, buffer: &Vec<u8>, key: KeyOrIV, iv: Option<KeyOrIV>) -> SimpleResult<Vec<u8>> {
        match self {
            Self::AES_ECB => Self::encrypt_aes_ecb(buffer, key),
            Self::AES_CBC => Self::encrypt_aes_cbc(buffer, key, iv),
            Self::AES_CFB => Self::encrypt_aes_cfb(buffer, key, iv),
        }
    }
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Serialize, Deserialize)]
pub struct BlockCipherSettings {
    cipher: CipherType,
    key: KeyOrIV,
    iv: Option<KeyOrIV>,
}

impl BlockCipherSettings {
    pub fn new(cipher: CipherType, key: Vec<u8>, iv: Option<Vec<u8>>) -> SimpleResult<Self> {
        // Validate and store the key
        let key = KeyOrIV::new(key)?;

        // Validate and store the IV (or a blank one)
        let iv = match iv {
            Some(iv) => Some(KeyOrIV::new(iv)?),
            None => None,
        };

        // Sanity check
        cipher.validate_settings(key, iv)?;

        Ok(BlockCipherSettings {
            cipher: cipher,
            key: key,
            iv: iv,
        })
    }
}

pub struct TransformBlockCipher {
    settings: BlockCipherSettings,
}

impl TransformBlockCipher {
    pub fn new(settings: BlockCipherSettings) -> Self {
        TransformBlockCipher {
            settings: settings,
        }
    }
}

impl TransformerTrait for TransformBlockCipher {
    fn transform(&self, buffer: &Vec<u8>) -> SimpleResult<Vec<u8>> {
        self.settings.cipher.decrypt(buffer, self.settings.key, self.settings.iv)
    }

    fn untransform(&self, buffer: &Vec<u8>) -> SimpleResult<Vec<u8>> {
        self.settings.cipher.encrypt(buffer, self.settings.key, self.settings.iv)
    }

    fn check(&self, buffer: &Vec<u8>) -> bool {
        self.settings.cipher.decrypt(buffer, self.settings.key, self.settings.iv).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    use crate::transformation::Transformation;

    #[test]
    fn test_aes_ecb() -> SimpleResult<()> {
        let tests: Vec<(Vec<u8>, Vec<u8>, Vec<u8>)> = vec![
            (
                b"Test for AES-128 with ECB padding and a couple blocks".to_vec(),            // Plaintext
                b"AAAAAAAAAAAAAAAA".to_vec(),                                                 // Key
                // Ciphertext
                b"\x8f\x43\x5a\x89\xf4\xda\x6b\x67\xe2\x2f\x43\xaf\x71\xbf\x93\xb0\
                  \xdc\x7e\x2f\x80\xcc\x6d\x67\xd9\xaa\xea\xda\x4f\xf3\xe6\x54\x52\
                  \x13\x3e\xdd\x7b\x52\x5b\x60\x5a\x1d\xe7\x3b\x1a\xd9\x6b\xb3\x45\
                  \x3e\x81\xd7\xbf\x1f\xc6\xdb\x7b\x12\xfd\xf3\x13\xf7\xe0\xba\xc4".to_vec(),
            ),

            (
                b"Test for AES-192 with EBC chaining!".to_vec(),                               // Plaintext
                b"AAAAAAAAAAAAAAAAAAAAAAAA".to_vec(),                                          // Key
                // Ciphertext
                b"\x4d\x44\x10\x2e\x61\x88\xe9\xa0\xc5\xf0\x60\xd9\xb7\x0c\xc6\x75\
                  \x26\x91\x98\x01\x45\x06\xf5\x95\x99\xb2\x9e\x3c\x13\xb5\xee\xb5\
                  \xff\x91\x0a\xf6\x5e\xe4\x9e\x35\x30\x21\x07\x06\x43\xc8\x45\x3d".to_vec(),
            ),

            (
                b"Final test for AES-256 ECB with a much longer plaintext and many blocks".to_vec(), // Plaintext
                b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_vec(),                                 // Key
                // Ciphertext
                b"\xed\x79\xa2\x28\x21\x55\x65\xc9\x50\xbc\x93\xc8\xa3\xed\x6a\xc4\
                  \x10\x48\xc6\x47\xac\x30\xf0\x55\x96\xd1\xd6\xfc\x51\x5b\x6b\x04\
                  \x25\x6c\x1a\xa6\x22\x46\x32\x0f\x1c\xea\x8d\x1d\xbf\x0a\xcd\x0a\
                  \xa3\xda\xa8\x47\xaf\x27\xf6\x71\x85\x5d\x1f\xe9\x14\xb4\x4d\x6d\
                  \x50\x29\x40\xd2\x73\xfb\xd7\xa8\x61\xdd\xfb\x88\x6b\xa0\xa8\x07".to_vec(),
            ),
        ];

        for (plaintext, key, ciphertext) in tests {
            let transformation = Transformation::FromBlockCipher(BlockCipherSettings::new(
                CipherType::AES_ECB,
                key,
                None,
            )?);

            let result = transformation.transform(&ciphertext)?;
            assert_eq!(plaintext, result);

            let result = transformation.untransform(&result)?;
            assert_eq!(ciphertext, result);
        }

        Ok(())
    }

    #[test]
    fn test_aes_cbc() -> SimpleResult<()> {
        let tests: Vec<(Vec<u8>, Vec<u8>, Option<Vec<u8>>, Vec<u8>)> = vec![
            (
                b"Test for AES-128 with CBC padding and a couple blocks".to_vec(),            // Plaintext
                b"AAAAAAAAAAAAAAAA".to_vec(),                                                 // Key
                None,                                                                         // IV
                // Ciphertext
                b"\x8f\x43\x5a\x89\xf4\xda\x6b\x67\xe2\x2f\x43\xaf\x71\xbf\x93\xb0\
                  \x21\x2c\x88\x77\x01\x5c\x28\xe9\xa6\xac\x34\xb8\xb4\x3c\x15\x21\
                  \xa3\x36\x13\xbd\x14\x25\xe3\x47\x37\x87\xa0\x0d\x65\xd0\x15\xe9\
                  \x5e\xf8\x64\x3d\x91\x86\x21\xc4\x3e\x97\x24\x4f\xb6\xa3\x27\xf5".to_vec(),
            ),

            (
                b"AES128 with an IV!".to_vec(),                                               // Plaintext
                b"AAAAAAAAAAAAAAAA".to_vec(),                                                 // Key
                Some(b"BBBBBBBBBBBBBBBB".to_vec()),                                           // IV
                // Ciphertext
                b"\x86\x62\x63\x07\x47\x5d\x2e\x61\x8e\x3d\xed\x1a\xff\x00\xef\xc3\
                  \x95\x8b\x83\x3d\xc8\x30\x6e\x50\x36\x4e\x6d\x29\x9e\x19\xd2\xc9".to_vec(),
            ),

            (
                b"Test for AES-192 with CBC padding!".to_vec(),                               // Plaintext
                b"AAAAAAAAAAAAAAAAAAAAAAAA".to_vec(),                                         // Key
                None,                                                                         // IV
                // Ciphertext
                b"\x4d\x44\x10\x2e\x61\x88\xe9\xa0\xc5\xf0\x60\xd9\xb7\x0c\xc6\x75\
                  \xed\xcf\x7c\xf3\xaa\xe0\xdb\xcc\x39\xd7\x7f\x24\x02\x6d\x6c\x98\
                  \x5d\x34\x46\xc6\x76\x19\xaa\xca\x21\xdc\x7d\x09\x3e\x23\x56\x4a".to_vec(),
            ),

            (
                b"Final test for AES-256 with a longer plaintext".to_vec(),                   // Plaintext
                b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_vec(),                                 // Key
                None,                                                                         // IV
                // Ciphertext
                b"\xed\x79\xa2\x28\x21\x55\x65\xc9\x50\xbc\x93\xc8\xa3\xed\x6a\xc4\
                  \xac\x6c\x8c\x56\x56\xea\x83\x29\x22\x43\x76\xa1\xe2\x2d\x74\xe3\
                  \x1d\xcf\x51\x5c\xdc\xc6\x34\x4c\xa6\x5b\xb4\x46\x8b\x52\x3c\xa2".to_vec(),
            ),

            (
                b"AES256 with an all-C IV!".to_vec(),                                         // Plaintext
                b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_vec(),                                 // Key
                Some(b"CCCCCCCCCCCCCCCC".to_vec()),                                           // IV
                // Ciphertext
                b"\x99\x70\x25\x50\x5c\xd5\x9e\x9d\xc7\x73\x19\x94\x5c\xae\xc9\x9f\
                  \xd5\x28\x00\xf1\x34\xcd\xcf\xf9\xbf\x15\x08\x52\x2b\xd4\x09\xa2".to_vec(),
            ),
        ];

        for (plaintext, key, iv, ciphertext) in tests {
            let transformation = Transformation::FromBlockCipher(BlockCipherSettings::new(
                CipherType::AES_CBC,
                key,
                iv,
            )?);

            let result = transformation.transform(&ciphertext)?;
            assert_eq!(plaintext, result);

            let result = transformation.untransform(&result)?;
            assert_eq!(ciphertext, result);
        }

        Ok(())
    }

    #[test]
    fn test_aes_cfb() -> SimpleResult<()> {
        let tests: Vec<(Vec<u8>, Vec<u8>, Option<Vec<u8>>, Vec<u8>)> = vec![
            (
                b"Test for AES-128 with CFB padding and a couple blocks".to_vec(),            // Plaintext
                b"AAAAAAAAAAAAAAAA".to_vec(),                                                 // Key
                Some(b"BBBBBBBBBBBBBBBB".to_vec()),
                b"\x65\x86\x49\x1a\x72\x36\xff\xe8\x5e\x10\xc9\xb4\x40\x1d\xad\x41\
                  \xd0\x55\x2f\x5c\xa9\x5b\xcb\xcf\x8b\x6e\xc8\x09\x73\xa7\x03\x3d\
                  \xb0\x10\x8c\x66\xa3\x18\xda\x1d\x46\x55\xb9\x61\xfa\xb2\xc9\x2e\
                  \x74\x60\xcf\x59\x2e\xd4\x28\x99\x38\xc3\x01\x1f\xf4\x95\x9a\x51".to_vec()
            ),

            (
                b"AES-192 + CFB!".to_vec(),                                                   // Plaintext
                b"AAAAAAAAAAAAAAAAAAAAAAAA".to_vec(),                                         // Key
                None,                                                                         // IV
                // Ciphertext
                b"\x4e\x7c\x30\x7f\x6e\x64\xb0\x01\x11\x59\xaf\x39\xb2\xc6\x8f\xfe".to_vec(),
            ),

            (
                b"Final test for AES-256 with a longer plaintext".to_vec(),                   // Plaintext
                b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_vec(),                                 // Key
                Some(b"BBBBBBBBBBBBBBBB".to_vec()),                                           // IV
                // Ciphertext
                b"\x89\x80\x0e\xd3\x0f\x53\x26\x36\xac\x10\xc7\x0c\x3e\x9d\x62\xc9\
                  \xf2\x34\xde\x6f\xf4\x6c\xcb\x68\xbb\xaa\x13\x8d\x89\xe8\x76\xb5\
                  \xf0\xc2\x4c\x41\x3f\xa6\x2e\xfc\x8c\xe4\x5d\x2f\x1a\x9f\xc7\x8d".to_vec(),
            ),
        ];

        for (plaintext, key, iv, ciphertext) in tests {
            let transformation = Transformation::FromBlockCipher(BlockCipherSettings::new(
                CipherType::AES_CFB,
                key,
                iv,
            )?);

            let result = transformation.transform(&ciphertext)?;
            assert_eq!(plaintext, result);

            let result = transformation.untransform(&result)?;
            assert_eq!(ciphertext, result);
        }

        Ok(())
    }

    // #[test]
    // fn test_aes_errors() -> SimpleResult<()> {
    //     bail!("Not implemented");
    // }
}
