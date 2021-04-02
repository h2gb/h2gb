use aes::{Aes128, Aes192, Aes256};
use block_modes::{BlockMode, Cbc};
use block_modes::block_padding::Pkcs7;

use simple_error::{SimpleResult, bail};
use serde::{Serialize, Deserialize};

use crate::transformation::TransformerTrait;

#[allow(non_camel_case_types)]
#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Serialize, Deserialize)]
enum CipherType {
    AES_CBC,
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Serialize, Deserialize)]
enum KeyOrIV {
    Bits128([u8; 16]),
    Bits192([u8; 24]),
    Bits256([u8; 32]),
}

impl KeyOrIV {
    fn new(key: Vec<u8>) -> SimpleResult<Self> {
        Ok(match key.len() {
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

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Serialize, Deserialize)]
pub struct BlockCipherSettings {
    cipher: CipherType,
    key: KeyOrIV,
    iv: KeyOrIV,
}

impl BlockCipherSettings {
    pub fn new(key: Vec<u8>, iv: Option<Vec<u8>>) -> SimpleResult<Self> {
        // Validate and store the key
        let key = KeyOrIV::new(key)?;

        // Validate and store the IV (or a blank one)
        let iv = match iv {
            Some(iv) => {
                if iv.len() != 16 {
                    bail!("Invalid IV length ({} bytes / {} bits) - AES requires a 16-byte / 128-bit IV", iv.len(), iv.len());
                }

                KeyOrIV::new(iv)?
            }
            None     => KeyOrIV::new(b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0".to_vec())?,
        };

        Ok(BlockCipherSettings {
            cipher: CipherType::AES_CBC,
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
        // Get the iv, or a default blank one
        let iv = match self.settings.iv {
            KeyOrIV::Bits128(iv) => iv,
            _ => bail!("Invalid IV length"),
        };

        // Pick the implementation based on the key
        let out = match self.settings.key {
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
        };

        Ok(out.to_vec())
    }

    fn untransform(&self, _buffer: &Vec<u8>) -> SimpleResult<Vec<u8>> {
        bail!("Not implemented yet!");
    }

    fn check(&self, _buffer: &Vec<u8>) -> bool {
       true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    use crate::transformation::Transformation;

    #[test]
    fn test_aes() -> SimpleResult<()> {
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
            let result = Transformation::FromBlockCipher(BlockCipherSettings::new(
                key,
                iv,
            )?).transform(&ciphertext)?;
            assert_eq!(plaintext, result);
        }

        Ok(())
    }

    // #[test]
    // fn test_aes_ecb() -> SimpleResult<()> {
    //     bail!("Not implemented");
    // }
    // #[test]
    // fn test_aes_errors() -> SimpleResult<()> {
    //     bail!("Not implemented");
    // }
}
