use aes::{Aes128, Aes192, Aes256};
use des::Des;
use block_modes::{BlockMode, Ecb, Cbc, Cfb};
// use block_modes::{Cfb8, Ofb, Pcbc}
use block_modes::block_padding::{NoPadding, Pkcs7, ZeroPadding};

use simple_error::{SimpleResult, bail};
use serde::{Serialize, Deserialize};

use crate::transformation::{Transformation, TransformerTrait};
use crate::transformation::helpers::key_or_iv::KeyOrIV;

/// A macro to simplify decryption - lets us pass a class name as an argument
/// (and greatly simplifies the code)
macro_rules! decrypt {
    ($buffer:expr, $key:expr, $iv:expr, $mode:ident, $algorithm:ident, $padding:ident) => {
        match $mode::<$algorithm, $padding>::new_var($key, $iv) {
            Ok(c) => {
                match c.decrypt_vec($buffer) {
                    Ok(d) => d,
                    Err(e) => bail!("Error decrypting buffer: {}", e),
                }
            },
            Err(e) => bail!("Error setting up cipher: {}", e),
        }
    };
}

/// Similar to [`decrypt!`], simplify encryption with a macro
macro_rules! encrypt {
    ($buffer:expr, $key:expr, $iv:expr, $mode:ident, $algorithm:ident, $padding:ident) => {
        match $mode::<$algorithm, $padding>::new_var($key, $iv) {
            Ok(c) => c.encrypt_vec($buffer),
            Err(e) => bail!("Error setting up cipher: {}", e),
        }
    };
}

/// How should the ciphertext's padding be validated?
///
/// If in doubt, use [`BlockCipherPadding::NoPadding`]. You'll see the padding in
/// the output, then you can match it up to the correct padding if NoPadding
/// was incorrect
#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Serialize, Deserialize)]
pub enum BlockCipherPadding {
    /// Don't touch padding at all.
    ///
    /// The advantage is that you'll see the output in all its glory.
    ///
    /// The disadvantage is that it can't validate whether the key was correct,
    /// anything with a valid length will successfully decrypt to something!
    NoPadding,

    /// Use Pkcs7 padding.
    ///
    /// Pkcs7 is by far the most popular padding on block ciphers, so it's a
    /// good guess in most cases.
    ///
    /// The padding is made up single bytes with the value of the number of
    /// bytes. That is, a single byte of padding is `"\x01"`, two bytes are
    /// `"\x02\x02"`, three are "`\x03\x03\x03`", and so on.
    Pkcs7,

    /// Pad zero bytes.
    ///
    /// I've never actually seen this used, but it's easy to implement! As a
    /// special bonus, it's even ambiguous.
    ZeroPadding,
}

/// Which block cipher should we use?
///
/// The key length is determined at runtime by the length of key. So whether
/// you need `AES128`, `AES192`, or `AES256` will be sorted out at runtime.
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Serialize, Deserialize)]
pub enum BlockCipherType {
    /// AES (128, 192, or 256-bit keys, 128-bit IV)
    AES,

    /// DES (64-bit keys, 64-bit IVs)
    DES,
}

/// Which mode of operation should we use?
///
/// A mode of operation is the method used to encrypt multiple blocks. If you
/// end up in a situation where the first block is correct and the rest are
/// wrong, you might be using the wrong mode!
///
/// If in doubt, try [`BlockCipherMode::CBC`] or [`BlockCipherMode::ECB`].
#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Serialize, Deserialize)]
pub enum BlockCipherMode {
    /// Electronic Codebook
    ///
    /// Each block is encrypted independently of all the others. Really bad!
    ECB,

    /// Cipher Block Chaining
    ///
    /// Each block is XORed with the ciphertext of the previous. Super common.
    CBC,

    /// Cipher Feedback
    ///
    /// Each block of ciphertext is encrypted then XORed with the plaintext.
    CFB,
}

/// Configures a block cipher.
///
/// This configures all the settings for a block cipher together, in a single
/// serializable place.
#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Serialize, Deserialize)]
pub struct TransformBlockCipher {
    cipher: BlockCipherType,
    mode: BlockCipherMode,
    padding: BlockCipherPadding,
    key: KeyOrIV,
    iv: Option<KeyOrIV>,
}

impl TransformBlockCipher {
    /// Create a new instance of [`TransformBlockCipher`].
    ///
    /// The settings are validated as much as possible (key lengths and such),
    /// then they are "written in stone", so to speak - you can't change them
    /// without creating a new instance.
    pub fn new(cipher: BlockCipherType, mode: BlockCipherMode, padding: BlockCipherPadding, key: Vec<u8>, iv: Option<Vec<u8>>) -> SimpleResult<Transformation> {
        // Validate and store the key
        let key = KeyOrIV::new(key)?;

        // Validate and store the IV (or a blank one)
        let iv = match iv {
            Some(iv) => Some(KeyOrIV::new(iv)?),
            None => None,
        };

        // Create the result so we can validate it
        let result = TransformBlockCipher {
            cipher: cipher,
            mode: mode,
            padding: padding,
            key: key,
            iv: iv,
        };

        // This validates the key length and iv and other characteristics
        result.validate_settings()?;

        Ok(Transformation::FromBlockCipher(result))
    }

    /// A helper function - ensure that the DES ciphertext length is sane.
    fn des_check_length(length: usize) -> SimpleResult<()> {
        if length % 8 != 0 {
            bail!("DES length must be a multiple of 8 bytes / 64 bits");
        }

        Ok(())
    }

    /// A helper function - ensure that the AES ciphertext length is sane.
    fn aes_check_length(length: usize) -> SimpleResult<()> {
        if length % 16 != 0 {
            bail!("AES length must be a multiple of 16 bytes / 128-bits");
        }

        Ok(())
    }

    /// Internal function to decrypt AES with any settings.
    fn decrypt_aes(self, buffer: &Vec<u8>) -> SimpleResult<Vec<u8>> {
        // A very quick sanity check
        Self::aes_check_length(buffer.len())?;

        // Get the iv, or a default blank one
        let iv = match self.iv {
            Some(iv) => iv.get128()?,
            None     => [0; 16],
        };

        Ok(match (self.key, self.mode, self.padding) {
            (KeyOrIV::Bits128(k), BlockCipherMode::ECB, BlockCipherPadding::NoPadding)   => decrypt!(&buffer, &k, &iv, Ecb, Aes128, NoPadding),
            (KeyOrIV::Bits192(k), BlockCipherMode::ECB, BlockCipherPadding::NoPadding)   => decrypt!(&buffer, &k, &iv, Ecb, Aes192, NoPadding),
            (KeyOrIV::Bits256(k), BlockCipherMode::ECB, BlockCipherPadding::NoPadding)   => decrypt!(&buffer, &k, &iv, Ecb, Aes256, NoPadding),

            (KeyOrIV::Bits128(k), BlockCipherMode::CBC, BlockCipherPadding::NoPadding)   => decrypt!(&buffer, &k, &iv, Cbc, Aes128, NoPadding),
            (KeyOrIV::Bits192(k), BlockCipherMode::CBC, BlockCipherPadding::NoPadding)   => decrypt!(&buffer, &k, &iv, Cbc, Aes192, NoPadding),
            (KeyOrIV::Bits256(k), BlockCipherMode::CBC, BlockCipherPadding::NoPadding)   => decrypt!(&buffer, &k, &iv, Cbc, Aes256, NoPadding),

            (KeyOrIV::Bits128(k), BlockCipherMode::CFB, BlockCipherPadding::NoPadding)   => decrypt!(&buffer, &k, &iv, Cfb, Aes128, NoPadding),
            (KeyOrIV::Bits192(k), BlockCipherMode::CFB, BlockCipherPadding::NoPadding)   => decrypt!(&buffer, &k, &iv, Cfb, Aes192, NoPadding),
            (KeyOrIV::Bits256(k), BlockCipherMode::CFB, BlockCipherPadding::NoPadding)   => decrypt!(&buffer, &k, &iv, Cfb, Aes256, NoPadding),

            (KeyOrIV::Bits128(k), BlockCipherMode::ECB, BlockCipherPadding::Pkcs7)       => decrypt!(&buffer, &k, &iv, Ecb, Aes128, Pkcs7),
            (KeyOrIV::Bits192(k), BlockCipherMode::ECB, BlockCipherPadding::Pkcs7)       => decrypt!(&buffer, &k, &iv, Ecb, Aes192, Pkcs7),
            (KeyOrIV::Bits256(k), BlockCipherMode::ECB, BlockCipherPadding::Pkcs7)       => decrypt!(&buffer, &k, &iv, Ecb, Aes256, Pkcs7),

            (KeyOrIV::Bits128(k), BlockCipherMode::CBC, BlockCipherPadding::Pkcs7)       => decrypt!(&buffer, &k, &iv, Cbc, Aes128, Pkcs7),
            (KeyOrIV::Bits192(k), BlockCipherMode::CBC, BlockCipherPadding::Pkcs7)       => decrypt!(&buffer, &k, &iv, Cbc, Aes192, Pkcs7),
            (KeyOrIV::Bits256(k), BlockCipherMode::CBC, BlockCipherPadding::Pkcs7)       => decrypt!(&buffer, &k, &iv, Cbc, Aes256, Pkcs7),

            (KeyOrIV::Bits128(k), BlockCipherMode::CFB, BlockCipherPadding::Pkcs7)       => decrypt!(&buffer, &k, &iv, Cfb, Aes128, Pkcs7),
            (KeyOrIV::Bits192(k), BlockCipherMode::CFB, BlockCipherPadding::Pkcs7)       => decrypt!(&buffer, &k, &iv, Cfb, Aes192, Pkcs7),
            (KeyOrIV::Bits256(k), BlockCipherMode::CFB, BlockCipherPadding::Pkcs7)       => decrypt!(&buffer, &k, &iv, Cfb, Aes256, Pkcs7),

            (KeyOrIV::Bits128(k), BlockCipherMode::ECB, BlockCipherPadding::ZeroPadding) => decrypt!(&buffer, &k, &iv, Ecb, Aes128, ZeroPadding),
            (KeyOrIV::Bits192(k), BlockCipherMode::ECB, BlockCipherPadding::ZeroPadding) => decrypt!(&buffer, &k, &iv, Ecb, Aes192, ZeroPadding),
            (KeyOrIV::Bits256(k), BlockCipherMode::ECB, BlockCipherPadding::ZeroPadding) => decrypt!(&buffer, &k, &iv, Ecb, Aes256, ZeroPadding),

            (KeyOrIV::Bits128(k), BlockCipherMode::CBC, BlockCipherPadding::ZeroPadding) => decrypt!(&buffer, &k, &iv, Cbc, Aes128, ZeroPadding),
            (KeyOrIV::Bits192(k), BlockCipherMode::CBC, BlockCipherPadding::ZeroPadding) => decrypt!(&buffer, &k, &iv, Cbc, Aes192, ZeroPadding),
            (KeyOrIV::Bits256(k), BlockCipherMode::CBC, BlockCipherPadding::ZeroPadding) => decrypt!(&buffer, &k, &iv, Cbc, Aes256, ZeroPadding),

            (KeyOrIV::Bits128(k), BlockCipherMode::CFB, BlockCipherPadding::ZeroPadding) => decrypt!(&buffer, &k, &iv, Cfb, Aes128, ZeroPadding),
            (KeyOrIV::Bits192(k), BlockCipherMode::CFB, BlockCipherPadding::ZeroPadding) => decrypt!(&buffer, &k, &iv, Cfb, Aes192, ZeroPadding),
            (KeyOrIV::Bits256(k), BlockCipherMode::CFB, BlockCipherPadding::ZeroPadding) => decrypt!(&buffer, &k, &iv, Cfb, Aes256, ZeroPadding),

            (_, _, _) => bail!("Invalid key size, mode, or padding"),
        }.to_vec())
    }

    /// Internal function to encrypt AES with any settings.
    fn encrypt_aes(self, buffer: &Vec<u8>) -> SimpleResult<Vec<u8>> {
        // Get the iv, or a default blank one
        let iv = match self.iv {
            Some(iv) => iv.get128()?,
            None     => [0; 16],
        };

        Ok(match (self.key, self.mode, self.padding) {
            (KeyOrIV::Bits128(k), BlockCipherMode::ECB, BlockCipherPadding::NoPadding) => encrypt!(&buffer, &k, &iv, Ecb, Aes128, NoPadding),
            (KeyOrIV::Bits192(k), BlockCipherMode::ECB, BlockCipherPadding::NoPadding) => encrypt!(&buffer, &k, &iv, Ecb, Aes192, NoPadding),
            (KeyOrIV::Bits256(k), BlockCipherMode::ECB, BlockCipherPadding::NoPadding) => encrypt!(&buffer, &k, &iv, Ecb, Aes256, NoPadding),

            (KeyOrIV::Bits128(k), BlockCipherMode::CBC, BlockCipherPadding::NoPadding) => encrypt!(&buffer, &k, &iv, Cbc, Aes128, NoPadding),
            (KeyOrIV::Bits192(k), BlockCipherMode::CBC, BlockCipherPadding::NoPadding) => encrypt!(&buffer, &k, &iv, Cbc, Aes192, NoPadding),
            (KeyOrIV::Bits256(k), BlockCipherMode::CBC, BlockCipherPadding::NoPadding) => encrypt!(&buffer, &k, &iv, Cbc, Aes256, NoPadding),

            (KeyOrIV::Bits128(k), BlockCipherMode::CFB, BlockCipherPadding::NoPadding) => encrypt!(&buffer, &k, &iv, Cfb, Aes128, NoPadding),
            (KeyOrIV::Bits192(k), BlockCipherMode::CFB, BlockCipherPadding::NoPadding) => encrypt!(&buffer, &k, &iv, Cfb, Aes192, NoPadding),
            (KeyOrIV::Bits256(k), BlockCipherMode::CFB, BlockCipherPadding::NoPadding) => encrypt!(&buffer, &k, &iv, Cfb, Aes256, NoPadding),

            (KeyOrIV::Bits128(k), BlockCipherMode::ECB, BlockCipherPadding::Pkcs7) => encrypt!(&buffer, &k, &iv, Ecb, Aes128, Pkcs7),
            (KeyOrIV::Bits192(k), BlockCipherMode::ECB, BlockCipherPadding::Pkcs7) => encrypt!(&buffer, &k, &iv, Ecb, Aes192, Pkcs7),
            (KeyOrIV::Bits256(k), BlockCipherMode::ECB, BlockCipherPadding::Pkcs7) => encrypt!(&buffer, &k, &iv, Ecb, Aes256, Pkcs7),

            (KeyOrIV::Bits128(k), BlockCipherMode::CBC, BlockCipherPadding::Pkcs7) => encrypt!(&buffer, &k, &iv, Cbc, Aes128, Pkcs7),
            (KeyOrIV::Bits192(k), BlockCipherMode::CBC, BlockCipherPadding::Pkcs7) => encrypt!(&buffer, &k, &iv, Cbc, Aes192, Pkcs7),
            (KeyOrIV::Bits256(k), BlockCipherMode::CBC, BlockCipherPadding::Pkcs7) => encrypt!(&buffer, &k, &iv, Cbc, Aes256, Pkcs7),

            (KeyOrIV::Bits128(k), BlockCipherMode::CFB, BlockCipherPadding::Pkcs7) => encrypt!(&buffer, &k, &iv, Cfb, Aes128, Pkcs7),
            (KeyOrIV::Bits192(k), BlockCipherMode::CFB, BlockCipherPadding::Pkcs7) => encrypt!(&buffer, &k, &iv, Cfb, Aes192, Pkcs7),
            (KeyOrIV::Bits256(k), BlockCipherMode::CFB, BlockCipherPadding::Pkcs7) => encrypt!(&buffer, &k, &iv, Cfb, Aes256, Pkcs7),

            (KeyOrIV::Bits128(k), BlockCipherMode::ECB, BlockCipherPadding::ZeroPadding) => encrypt!(&buffer, &k, &iv, Ecb, Aes128, ZeroPadding),
            (KeyOrIV::Bits192(k), BlockCipherMode::ECB, BlockCipherPadding::ZeroPadding) => encrypt!(&buffer, &k, &iv, Ecb, Aes192, ZeroPadding),
            (KeyOrIV::Bits256(k), BlockCipherMode::ECB, BlockCipherPadding::ZeroPadding) => encrypt!(&buffer, &k, &iv, Ecb, Aes256, ZeroPadding),

            (KeyOrIV::Bits128(k), BlockCipherMode::CBC, BlockCipherPadding::ZeroPadding) => encrypt!(&buffer, &k, &iv, Cbc, Aes128, ZeroPadding),
            (KeyOrIV::Bits192(k), BlockCipherMode::CBC, BlockCipherPadding::ZeroPadding) => encrypt!(&buffer, &k, &iv, Cbc, Aes192, ZeroPadding),
            (KeyOrIV::Bits256(k), BlockCipherMode::CBC, BlockCipherPadding::ZeroPadding) => encrypt!(&buffer, &k, &iv, Cbc, Aes256, ZeroPadding),

            (KeyOrIV::Bits128(k), BlockCipherMode::CFB, BlockCipherPadding::ZeroPadding) => encrypt!(&buffer, &k, &iv, Cfb, Aes128, ZeroPadding),
            (KeyOrIV::Bits192(k), BlockCipherMode::CFB, BlockCipherPadding::ZeroPadding) => encrypt!(&buffer, &k, &iv, Cfb, Aes192, ZeroPadding),
            (KeyOrIV::Bits256(k), BlockCipherMode::CFB, BlockCipherPadding::ZeroPadding) => encrypt!(&buffer, &k, &iv, Cfb, Aes256, ZeroPadding),

            (_, _, _) => bail!("Invalid key size, mode, or padding"),
        }.to_vec())
    }

    /// Internal function to decrypt DES with any settings.
    fn decrypt_des(self, buffer: &Vec<u8>) -> SimpleResult<Vec<u8>> {
        Self::des_check_length(buffer.len())?;

        // Get the iv, or a default blank one
        let iv = match self.iv {
            Some(iv) => iv.get64()?,
            None     => [0; 8],
        };

        Ok(match (self.key, self.mode, self.padding) {
            (KeyOrIV::Bits64(k), BlockCipherMode::ECB, BlockCipherPadding::NoPadding) => decrypt!(&buffer, &k, &iv, Ecb, Des, NoPadding),
            (KeyOrIV::Bits64(k), BlockCipherMode::CBC, BlockCipherPadding::NoPadding) => decrypt!(&buffer, &k, &iv, Cbc, Des, NoPadding),
            (KeyOrIV::Bits64(k), BlockCipherMode::CFB, BlockCipherPadding::NoPadding) => decrypt!(&buffer, &k, &iv, Cfb, Des, NoPadding),

            (KeyOrIV::Bits64(k), BlockCipherMode::ECB, BlockCipherPadding::Pkcs7) => decrypt!(&buffer, &k, &iv, Ecb, Des, Pkcs7),
            (KeyOrIV::Bits64(k), BlockCipherMode::CBC, BlockCipherPadding::Pkcs7) => decrypt!(&buffer, &k, &iv, Cbc, Des, Pkcs7),
            (KeyOrIV::Bits64(k), BlockCipherMode::CFB, BlockCipherPadding::Pkcs7) => decrypt!(&buffer, &k, &iv, Cfb, Des, Pkcs7),

            (KeyOrIV::Bits64(k), BlockCipherMode::ECB, BlockCipherPadding::ZeroPadding) => decrypt!(&buffer, &k, &iv, Ecb, Des, ZeroPadding),
            (KeyOrIV::Bits64(k), BlockCipherMode::CBC, BlockCipherPadding::ZeroPadding) => decrypt!(&buffer, &k, &iv, Cbc, Des, ZeroPadding),
            (KeyOrIV::Bits64(k), BlockCipherMode::CFB, BlockCipherPadding::ZeroPadding) => decrypt!(&buffer, &k, &iv, Cfb, Des, ZeroPadding),

            (_, _, _) => bail!("Invalid key size, mode, or padding"),
        }.to_vec())
    }

    /// Internal function to encrypt DES with any settings.
    fn encrypt_des(self, buffer: &Vec<u8>) -> SimpleResult<Vec<u8>> {
        // Get the iv, or a default blank one
        let iv = match self.iv {
            Some(iv) => iv.get64()?,
            None     => [0; 8],
        };

        Ok(match (self.key, self.mode, self.padding) {
            (KeyOrIV::Bits64(k), BlockCipherMode::ECB, BlockCipherPadding::NoPadding) => encrypt!(&buffer, &k, &iv, Ecb, Des, NoPadding),
            (KeyOrIV::Bits64(k), BlockCipherMode::CBC, BlockCipherPadding::NoPadding) => encrypt!(&buffer, &k, &iv, Cbc, Des, NoPadding),
            (KeyOrIV::Bits64(k), BlockCipherMode::CFB, BlockCipherPadding::NoPadding) => encrypt!(&buffer, &k, &iv, Cfb, Des, NoPadding),

            (KeyOrIV::Bits64(k), BlockCipherMode::ECB, BlockCipherPadding::Pkcs7) => encrypt!(&buffer, &k, &iv, Ecb, Des, Pkcs7),
            (KeyOrIV::Bits64(k), BlockCipherMode::CBC, BlockCipherPadding::Pkcs7) => encrypt!(&buffer, &k, &iv, Cbc, Des, Pkcs7),
            (KeyOrIV::Bits64(k), BlockCipherMode::CFB, BlockCipherPadding::Pkcs7) => encrypt!(&buffer, &k, &iv, Cfb, Des, Pkcs7),

            (KeyOrIV::Bits64(k), BlockCipherMode::ECB, BlockCipherPadding::ZeroPadding) => encrypt!(&buffer, &k, &iv, Ecb, Des, ZeroPadding),
            (KeyOrIV::Bits64(k), BlockCipherMode::CBC, BlockCipherPadding::ZeroPadding) => encrypt!(&buffer, &k, &iv, Cbc, Des, ZeroPadding),
            (KeyOrIV::Bits64(k), BlockCipherMode::CFB, BlockCipherPadding::ZeroPadding) => encrypt!(&buffer, &k, &iv, Cfb, Des, ZeroPadding),

            (_, _, _) => bail!("Invalid key size, mode, or padding"),
        }.to_vec())
    }

    /// Sanity check settings (key size, IV, etc).
    fn validate_settings(self) -> SimpleResult<()> {
        // Validate the iv for ECB mode
        match (self.iv, self.mode) {
            // Don't allow an IV with ECB ever
            (Some(_), BlockCipherMode::ECB) => bail!("ECB is not compatible with IVs"),

            // If the iv is set, make sure it's the correct length
            (Some(iv), _) => {
                match (self.cipher, iv) {
                    (BlockCipherType::AES, KeyOrIV::Bits128(_)) => (),
                    (BlockCipherType::AES, _) => bail!("Invalid IV size for AES (must be 128 bits)"),

                    (BlockCipherType::DES, KeyOrIV::Bits64(_)) => (),
                    (BlockCipherType::DES, _) => bail!("Invalid IV size for DES (must be 64 bits)"),
                }
            },

            // If no IV is set, we're probably okay
            (None, _) => (),

        };

        // Validate the key length
        match (self.cipher, self.key) {
            (BlockCipherType::AES, KeyOrIV::Bits128(_)) => (),
            (BlockCipherType::AES, KeyOrIV::Bits192(_)) => (),
            (BlockCipherType::AES, KeyOrIV::Bits256(_)) => (),
            (BlockCipherType::AES, _) => bail!("Invalid key size for AES (must be 128, 192, or 256 bits)"),

            (BlockCipherType::DES, KeyOrIV::Bits64(_)) => (),
            (BlockCipherType::DES, _) => bail!("Invalid key size for DES (must be 64 bits)"),
        };

        Ok(())
    }
}

impl TransformerTrait for TransformBlockCipher {
    /// transform() =~ decrypt
    fn transform(&self, buffer: &Vec<u8>) -> SimpleResult<Vec<u8>> {
        self.validate_settings()?;

        match self.cipher {
            BlockCipherType::AES => self.decrypt_aes(buffer),
            BlockCipherType::DES => self.decrypt_des(buffer),
        }
    }

    /// transform() =~ encrypt
    fn untransform(&self, buffer: &Vec<u8>) -> SimpleResult<Vec<u8>> {
        self.validate_settings()?;

        match self.cipher {
            BlockCipherType::AES => self.encrypt_aes(buffer),
            BlockCipherType::DES => self.encrypt_des(buffer),
        }
    }

    fn is_two_way(&self) -> bool {
        // Block ciphers can always go back and forth!
        true
    }

    fn detect(_buffer: &Vec<u8>) -> Vec<Transformation> where Self: Sized {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_aes_ecb() -> SimpleResult<()> {
        let tests: Vec<(Vec<u8>, Vec<u8>, BlockCipherPadding, Vec<u8>)> = vec![
            (
                b"Test for AES-128 with ECB padding and a couple blocks".to_vec(),            // Plaintext
                b"AAAAAAAAAAAAAAAA".to_vec(),                                                 // Key
                BlockCipherPadding::Pkcs7,                                                    // Padding
                // Ciphertext
                b"\x8f\x43\x5a\x89\xf4\xda\x6b\x67\xe2\x2f\x43\xaf\x71\xbf\x93\xb0\
                  \xdc\x7e\x2f\x80\xcc\x6d\x67\xd9\xaa\xea\xda\x4f\xf3\xe6\x54\x52\
                  \x13\x3e\xdd\x7b\x52\x5b\x60\x5a\x1d\xe7\x3b\x1a\xd9\x6b\xb3\x45\
                  \x3e\x81\xd7\xbf\x1f\xc6\xdb\x7b\x12\xfd\xf3\x13\xf7\xe0\xba\xc4".to_vec(),
            ),

            (
                b"Test for AES-192 with EBC chaining!".to_vec(),                               // Plaintext
                b"AAAAAAAAAAAAAAAAAAAAAAAA".to_vec(),                                          // Key
                BlockCipherPadding::Pkcs7,                                                     // Padding
                // Ciphertext
                b"\x4d\x44\x10\x2e\x61\x88\xe9\xa0\xc5\xf0\x60\xd9\xb7\x0c\xc6\x75\
                  \x26\x91\x98\x01\x45\x06\xf5\x95\x99\xb2\x9e\x3c\x13\xb5\xee\xb5\
                  \xff\x91\x0a\xf6\x5e\xe4\x9e\x35\x30\x21\x07\x06\x43\xc8\x45\x3d".to_vec(),
            ),

            (
                b"Final test for AES-256 ECB with a much longer plaintext and many blocks".to_vec(), // Plaintext
                b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_vec(),                                        // Key
                BlockCipherPadding::Pkcs7,                                                           // Padding
                // Ciphertext
                b"\xed\x79\xa2\x28\x21\x55\x65\xc9\x50\xbc\x93\xc8\xa3\xed\x6a\xc4\
                  \x10\x48\xc6\x47\xac\x30\xf0\x55\x96\xd1\xd6\xfc\x51\x5b\x6b\x04\
                  \x25\x6c\x1a\xa6\x22\x46\x32\x0f\x1c\xea\x8d\x1d\xbf\x0a\xcd\x0a\
                  \xa3\xda\xa8\x47\xaf\x27\xf6\x71\x85\x5d\x1f\xe9\x14\xb4\x4d\x6d\
                  \x50\x29\x40\xd2\x73\xfb\xd7\xa8\x61\xdd\xfb\x88\x6b\xa0\xa8\x07".to_vec(),
            ),

            (
                b"Test for AES-128 with ECB padding and a couple blocks\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b".to_vec(), // Plaintext
                b"AAAAAAAAAAAAAAAA".to_vec(),                                                                                  // Key
                BlockCipherPadding::NoPadding,                                                                                 // Padding
                // Ciphertext
                b"\x8f\x43\x5a\x89\xf4\xda\x6b\x67\xe2\x2f\x43\xaf\x71\xbf\x93\xb0\
                  \xdc\x7e\x2f\x80\xcc\x6d\x67\xd9\xaa\xea\xda\x4f\xf3\xe6\x54\x52\
                  \x13\x3e\xdd\x7b\x52\x5b\x60\x5a\x1d\xe7\x3b\x1a\xd9\x6b\xb3\x45\
                  \x3e\x81\xd7\xbf\x1f\xc6\xdb\x7b\x12\xfd\xf3\x13\xf7\xe0\xba\xc4".to_vec(),
            ),

            (
                b"Test for AES-192 with EBC chaining!\x0d\x0d\x0d\x0d\x0d\x0d\x0d\x0d\x0d\x0d\x0d\x0d\x0d".to_vec(), // Plaintext
                b"AAAAAAAAAAAAAAAAAAAAAAAA".to_vec(),                                                                // Key
                BlockCipherPadding::NoPadding,                                                                       // Padding
                // Ciphertext
                b"\x4d\x44\x10\x2e\x61\x88\xe9\xa0\xc5\xf0\x60\xd9\xb7\x0c\xc6\x75\
                  \x26\x91\x98\x01\x45\x06\xf5\x95\x99\xb2\x9e\x3c\x13\xb5\xee\xb5\
                  \xff\x91\x0a\xf6\x5e\xe4\x9e\x35\x30\x21\x07\x06\x43\xc8\x45\x3d".to_vec(),
            ),

            (
                b"Final test for AES-256 ECB with a much longer plaintext and many blocks\x09\x09\x09\x09\x09\x09\x09\x09\x09".to_vec(), // Plaintext
                b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_vec(),                                                                            // Key
                BlockCipherPadding::NoPadding,                                                                                           // Padding
                // Ciphertext
                b"\xed\x79\xa2\x28\x21\x55\x65\xc9\x50\xbc\x93\xc8\xa3\xed\x6a\xc4\
                  \x10\x48\xc6\x47\xac\x30\xf0\x55\x96\xd1\xd6\xfc\x51\x5b\x6b\x04\
                  \x25\x6c\x1a\xa6\x22\x46\x32\x0f\x1c\xea\x8d\x1d\xbf\x0a\xcd\x0a\
                  \xa3\xda\xa8\x47\xaf\x27\xf6\x71\x85\x5d\x1f\xe9\x14\xb4\x4d\x6d\
                  \x50\x29\x40\xd2\x73\xfb\xd7\xa8\x61\xdd\xfb\x88\x6b\xa0\xa8\x07".to_vec(),
            ),
        ];

        for (plaintext, key, padding, ciphertext) in tests {
            let transformation = TransformBlockCipher::new(
                BlockCipherType::AES,
                BlockCipherMode::ECB,
                padding,
                key,
                None,
            )?;

            let result = transformation.transform(&ciphertext)?;
            assert_eq!(plaintext, result, "aes transform {}", std::str::from_utf8(&plaintext).unwrap());

            let result = transformation.untransform(&result)?;
            assert_eq!(ciphertext, result, "aes untransform {}", std::str::from_utf8(&plaintext).unwrap());
        }

        Ok(())
    }

    #[test]
    fn test_aes_cbc() -> SimpleResult<()> {
        let tests: Vec<(Vec<u8>, Vec<u8>, Option<Vec<u8>>, BlockCipherPadding, Vec<u8>)> = vec![
            (
                b"Test for AES-128 with CBC padding and a couple blocks".to_vec(),            // Plaintext
                b"AAAAAAAAAAAAAAAA".to_vec(),                                                 // Key
                None,                                                                         // IV
                BlockCipherPadding::Pkcs7,                                                    // Padding
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
                BlockCipherPadding::Pkcs7,                                                    // Padding
                // Ciphertext
                b"\x86\x62\x63\x07\x47\x5d\x2e\x61\x8e\x3d\xed\x1a\xff\x00\xef\xc3\
                  \x95\x8b\x83\x3d\xc8\x30\x6e\x50\x36\x4e\x6d\x29\x9e\x19\xd2\xc9".to_vec(),
            ),

            (
                b"Test for AES-192 with CBC padding!".to_vec(),                               // Plaintext
                b"AAAAAAAAAAAAAAAAAAAAAAAA".to_vec(),                                         // Key
                None,                                                                         // IV
                BlockCipherPadding::Pkcs7,                                                    // Padding
                // Ciphertext
                b"\x4d\x44\x10\x2e\x61\x88\xe9\xa0\xc5\xf0\x60\xd9\xb7\x0c\xc6\x75\
                  \xed\xcf\x7c\xf3\xaa\xe0\xdb\xcc\x39\xd7\x7f\x24\x02\x6d\x6c\x98\
                  \x5d\x34\x46\xc6\x76\x19\xaa\xca\x21\xdc\x7d\x09\x3e\x23\x56\x4a".to_vec(),
            ),

            (
                b"Final test for AES-256 with a longer plaintext".to_vec(),                   // Plaintext
                b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_vec(),                                 // Key
                None,                                                                         // IV
                BlockCipherPadding::Pkcs7,                                                    // Padding
                // Ciphertext
                b"\xed\x79\xa2\x28\x21\x55\x65\xc9\x50\xbc\x93\xc8\xa3\xed\x6a\xc4\
                  \xac\x6c\x8c\x56\x56\xea\x83\x29\x22\x43\x76\xa1\xe2\x2d\x74\xe3\
                  \x1d\xcf\x51\x5c\xdc\xc6\x34\x4c\xa6\x5b\xb4\x46\x8b\x52\x3c\xa2".to_vec(),
            ),

            (
                b"AES256 with an all-C IV!".to_vec(),                                         // Plaintext
                b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_vec(),                                 // Key
                Some(b"CCCCCCCCCCCCCCCC".to_vec()),                                           // IV
                BlockCipherPadding::Pkcs7,                                                    // Padding
                // Ciphertext
                b"\x99\x70\x25\x50\x5c\xd5\x9e\x9d\xc7\x73\x19\x94\x5c\xae\xc9\x9f\
                  \xd5\x28\x00\xf1\x34\xcd\xcf\xf9\xbf\x15\x08\x52\x2b\xd4\x09\xa2".to_vec(),
            ),

            (
                b"Test for AES-128 with CBC padding and a couple blocks\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b".to_vec(), // Plaintext
                b"AAAAAAAAAAAAAAAA".to_vec(),                                                 // Key
                None,                                                                         // IV
                BlockCipherPadding::NoPadding,                                                // Padding
                // Ciphertext
                b"\x8f\x43\x5a\x89\xf4\xda\x6b\x67\xe2\x2f\x43\xaf\x71\xbf\x93\xb0\
                  \x21\x2c\x88\x77\x01\x5c\x28\xe9\xa6\xac\x34\xb8\xb4\x3c\x15\x21\
                  \xa3\x36\x13\xbd\x14\x25\xe3\x47\x37\x87\xa0\x0d\x65\xd0\x15\xe9\
                  \x5e\xf8\x64\x3d\x91\x86\x21\xc4\x3e\x97\x24\x4f\xb6\xa3\x27\xf5".to_vec(),
            ),

            (
                b"Test for AES-192 with CBC padding!\x0e\x0e\x0e\x0e\x0e\x0e\x0e\x0e\x0e\x0e\x0e\x0e\x0e\x0e".to_vec(), // Plaintext
                b"AAAAAAAAAAAAAAAAAAAAAAAA".to_vec(),                                         // Key
                None,                                                                         // IV
                BlockCipherPadding::NoPadding,                                                // Padding
                // Ciphertext
                b"\x4d\x44\x10\x2e\x61\x88\xe9\xa0\xc5\xf0\x60\xd9\xb7\x0c\xc6\x75\
                  \xed\xcf\x7c\xf3\xaa\xe0\xdb\xcc\x39\xd7\x7f\x24\x02\x6d\x6c\x98\
                  \x5d\x34\x46\xc6\x76\x19\xaa\xca\x21\xdc\x7d\x09\x3e\x23\x56\x4a".to_vec(),
            ),

            (
                b"AES256 with an all-C IV!\x08\x08\x08\x08\x08\x08\x08\x08".to_vec(),         // Plaintext
                b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_vec(),                                 // Key
                Some(b"CCCCCCCCCCCCCCCC".to_vec()),                                           // IV
                BlockCipherPadding::NoPadding,                                                // Padding
                // Ciphertext
                b"\x99\x70\x25\x50\x5c\xd5\x9e\x9d\xc7\x73\x19\x94\x5c\xae\xc9\x9f\
                  \xd5\x28\x00\xf1\x34\xcd\xcf\xf9\xbf\x15\x08\x52\x2b\xd4\x09\xa2".to_vec(),
            ),
        ];

        for (plaintext, key, iv, padding, ciphertext) in tests {
            let transformation = TransformBlockCipher::new(
                BlockCipherType::AES,
                BlockCipherMode::CBC,
                padding,
                key,
                iv,
            )?;

            let result = transformation.transform(&ciphertext)?;
            assert_eq!(plaintext, result, "aes transform {}", std::str::from_utf8(&plaintext).unwrap());

            let result = transformation.untransform(&result)?;
            assert_eq!(ciphertext, result, "aes untransform {}", std::str::from_utf8(&plaintext).unwrap());
        }

        Ok(())
    }

    #[test]
    fn test_aes_cfb() -> SimpleResult<()> {
        let tests: Vec<(Vec<u8>, Vec<u8>, Option<Vec<u8>>, BlockCipherPadding, Vec<u8>)> = vec![
            (
                b"Test for AES-128 with CFB padding and a couple blocks".to_vec(),            // Plaintext
                b"AAAAAAAAAAAAAAAA".to_vec(),                                                 // Key
                Some(b"BBBBBBBBBBBBBBBB".to_vec()),
                BlockCipherPadding::Pkcs7,                                                    // Padding
                b"\x65\x86\x49\x1a\x72\x36\xff\xe8\x5e\x10\xc9\xb4\x40\x1d\xad\x41\
                  \xd0\x55\x2f\x5c\xa9\x5b\xcb\xcf\x8b\x6e\xc8\x09\x73\xa7\x03\x3d\
                  \xb0\x10\x8c\x66\xa3\x18\xda\x1d\x46\x55\xb9\x61\xfa\xb2\xc9\x2e\
                  \x74\x60\xcf\x59\x2e\xd4\x28\x99\x38\xc3\x01\x1f\xf4\x95\x9a\x51".to_vec()
            ),

            (
                b"AES-192 + CFB!".to_vec(),                                                   // Plaintext
                b"AAAAAAAAAAAAAAAAAAAAAAAA".to_vec(),                                         // Key
                None,                                                                         // IV
                BlockCipherPadding::Pkcs7,                                                    // Padding
                // Ciphertext
                b"\x4e\x7c\x30\x7f\x6e\x64\xb0\x01\x11\x59\xaf\x39\xb2\xc6\x8f\xfe".to_vec(),
            ),

            (
                b"Final test for AES-256 with a longer plaintext".to_vec(),                   // Plaintext
                b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_vec(),                                 // Key
                Some(b"BBBBBBBBBBBBBBBB".to_vec()),                                           // IV
                BlockCipherPadding::Pkcs7,                                                    // Padding
                // Ciphertext
                b"\x89\x80\x0e\xd3\x0f\x53\x26\x36\xac\x10\xc7\x0c\x3e\x9d\x62\xc9\
                  \xf2\x34\xde\x6f\xf4\x6c\xcb\x68\xbb\xaa\x13\x8d\x89\xe8\x76\xb5\
                  \xf0\xc2\x4c\x41\x3f\xa6\x2e\xfc\x8c\xe4\x5d\x2f\x1a\x9f\xc7\x8d".to_vec(),
            ),

            (
                b"Test for AES-128 with CFB padding and a couple blocks\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b\x0b".to_vec(), // Plaintext
                b"AAAAAAAAAAAAAAAA".to_vec(),                                                 // Key
                Some(b"BBBBBBBBBBBBBBBB".to_vec()),                                           // IV
                BlockCipherPadding::NoPadding,                                                // Padding
                b"\x65\x86\x49\x1a\x72\x36\xff\xe8\x5e\x10\xc9\xb4\x40\x1d\xad\x41\
                  \xd0\x55\x2f\x5c\xa9\x5b\xcb\xcf\x8b\x6e\xc8\x09\x73\xa7\x03\x3d\
                  \xb0\x10\x8c\x66\xa3\x18\xda\x1d\x46\x55\xb9\x61\xfa\xb2\xc9\x2e\
                  \x74\x60\xcf\x59\x2e\xd4\x28\x99\x38\xc3\x01\x1f\xf4\x95\x9a\x51".to_vec()
            ),

            (
                b"AES-192 + CFB!\x02\x02".to_vec(),                                           // Plaintext
                b"AAAAAAAAAAAAAAAAAAAAAAAA".to_vec(),                                         // Key
                None,                                                                         // IV
                BlockCipherPadding::NoPadding,                                                // Padding
                // Ciphertext
                b"\x4e\x7c\x30\x7f\x6e\x64\xb0\x01\x11\x59\xaf\x39\xb2\xc6\x8f\xfe".to_vec(),
            ),

            (
                b"Final test for AES-256 with a longer plaintext\x02\x02".to_vec(),           // Plaintext
                b"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA".to_vec(),                                 // Key
                Some(b"BBBBBBBBBBBBBBBB".to_vec()),                                           // IV
                BlockCipherPadding::NoPadding,                                                // Padding
                // Ciphertext
                b"\x89\x80\x0e\xd3\x0f\x53\x26\x36\xac\x10\xc7\x0c\x3e\x9d\x62\xc9\
                  \xf2\x34\xde\x6f\xf4\x6c\xcb\x68\xbb\xaa\x13\x8d\x89\xe8\x76\xb5\
                  \xf0\xc2\x4c\x41\x3f\xa6\x2e\xfc\x8c\xe4\x5d\x2f\x1a\x9f\xc7\x8d".to_vec(),
            ),
        ];

        for (plaintext, key, iv, padding, ciphertext) in tests {
            let transformation = TransformBlockCipher::new(
                BlockCipherType::AES,
                BlockCipherMode::CFB,
                padding,
                key,
                iv,
            )?;

            let result = transformation.transform(&ciphertext)?;
            assert_eq!(plaintext, result, "aes transform {}", std::str::from_utf8(&plaintext).unwrap());

            let result = transformation.untransform(&result)?;
            assert_eq!(ciphertext, result, "aes untransform {}", std::str::from_utf8(&plaintext).unwrap());
        }

        Ok(())
    }

    #[test]
    fn test_des() -> SimpleResult<()> {
        let tests: Vec<(Vec<u8>, Vec<u8>, Option<Vec<u8>>, BlockCipherMode, BlockCipherPadding, Vec<u8>)> = vec![
            (
                b"DES-ECB!!".to_vec(),                                                        // Plaintext
                b"AAAAAAAA".to_vec(),                                                         // Key
                None,                                                                         // IV
                BlockCipherMode::ECB,
                BlockCipherPadding::Pkcs7,                                                    // Padding
                // Ciphertext
                b"\x08\x39\x7c\x04\xb5\xbc\x8f\x3f\x01\x58\xb7\xc1\x70\x0e\xd6\x92".to_vec(),
            ),
            (
                b"DES-CBC!!".to_vec(),                                                        // Plaintext
                b"AAAAAAAA".to_vec(),                                                         // Key
                None,                                                                         // IV
                BlockCipherMode::CBC,
                BlockCipherPadding::Pkcs7,                                                    // Padding
                // Ciphertext
                b"\x50\x1d\x75\x4f\x12\x6d\xa5\x8b\x8d\x19\x20\xf6\xb9\x24\x9e\xed".to_vec(),
            ),
            (
                b"DES-CBC!!".to_vec(),                                                        // Plaintext
                b"AAAAAAAA".to_vec(),                                                         // Key
                Some(b"BBBBBBBB".to_vec()),                                                   // IV
                BlockCipherMode::CBC,
                BlockCipherPadding::Pkcs7,                                                    // Padding
                // Ciphertext
                b"\x71\x55\xb0\xdc\x7b\xed\xcd\x81\x3b\x81\xfa\x9a\xc7\x7c\x8a\x8e".to_vec(),
            ),
            (
                b"DES-CFB!!".to_vec(),                                                        // Plaintext
                b"AAAAAAAA".to_vec(),                                                         // Key
                None,                                                                         // IV
                BlockCipherMode::CFB,
                BlockCipherPadding::Pkcs7,                                                    // Padding
                // Ciphertext
                b"\x80\xf2\x6b\xe0\x92\xcb\x2c\x69\xd0\xc4\x55\xea\x50\x98\xfd\x55".to_vec(),
            ),
            (
                b"DES-CFB!!".to_vec(),                                                        // Plaintext
                b"AAAAAAAA".to_vec(),                                                         // Key
                Some(b"BBBBBBBB".to_vec()),                                                   // IV
                BlockCipherMode::CFB,
                BlockCipherPadding::Pkcs7,                                                    // Padding
                // Ciphertext
                b"\x5c\xbb\x74\x22\xc2\x46\x5b\x3d\x73\xf2\x3c\xdf\xdf\x4d\x10\x37".to_vec(),
            ),
        ];

        for (plaintext, key, iv, mode, padding, ciphertext) in tests {
            let transformation = TransformBlockCipher::new(
                BlockCipherType::DES,
                mode,
                padding,
                key,
                iv,
            )?;

            let result = transformation.transform(&ciphertext)?;
            assert_eq!(plaintext, result, "des transform {}", std::str::from_utf8(&plaintext).unwrap());

            let result = transformation.untransform(&result)?;
            assert_eq!(ciphertext, result, "des untransform {}", std::str::from_utf8(&plaintext).unwrap());
        }

        Ok(())
    }

    #[test]
    fn test_zero_padding() -> SimpleResult<()> {
        let tests: Vec<(Vec<u8>, Vec<u8>, Option<Vec<u8>>, BlockCipherMode, BlockCipherPadding, Vec<u8>)> = vec![
            (
                b"TEST Zero Pad".to_vec(),                                     // Plaintext
                b"AAAAAAAAAAAAAAAA".to_vec(),                                  // Key
                None,                                                          // IV
                BlockCipherMode::ECB,
                BlockCipherPadding::ZeroPadding,                               // Padding
                // Ciphertext
                b"\xd2\xc0\xaf\x98\xae\xf3\xce\x2d\x95\x93\x37\xe6\x9a\xcb\x7f\x31".to_vec(),
            ),
        ];

        for (plaintext, key, iv, mode, padding, ciphertext) in tests {
            let transformation = TransformBlockCipher::new(
                BlockCipherType::AES,
                mode,
                padding,
                key,
                iv,
            )?;

            let result = transformation.transform(&ciphertext)?;
            assert_eq!(plaintext, result, "des transform {}", std::str::from_utf8(&plaintext).unwrap());

            let result = transformation.untransform(&result)?;
            assert_eq!(ciphertext, result, "des untransform {}", std::str::from_utf8(&plaintext).unwrap());
        }

        Ok(())
    }
}
