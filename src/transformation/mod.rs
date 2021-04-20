//! A library for transforming raw data between encodings.
//!
//! As part of [h2gb](https://github.com/h2gb), it's common to extract a buffer
//! from a binary that's encoded in some format - Base64, hex string, etc.
//!
//! This library can detect and transform common formats. It can also
//! transform back to the original data with a constant length and without
//! saving any context (while the length is constant, the data isn't always
//! identical - like the case of Base32 and hex strings). If proper undo/redo
//! is needed, this won't fit the bill.
//!
//! Check out the definition of the [`Transformation`] enum for full details on
//! everything it can do!
//!
//! # Usage
//!
//! The public API is pretty straight forward. Here's an example that transforms
//! then untransforms some hex data:
//!
//!
//! ```
//! use libh2gb::transformation::TransformHex;
//!
//! // Input (note that some are uppercase and some are lower - that's allowed)
//! let i: Vec<u8> = b"48656c6C6F2c20776f726c64".to_vec();
//!
//! // Output
//! let o = TransformHex::new().transform(&i).unwrap();
//!
//! // It's "Hello, world"
//! assert_eq!(b"Hello, world".to_vec(), o);
//!
//! // Transform back to the original
//! let i = TransformHex::new().untransform(&o).unwrap();
//!
//! // Get the original back - note that it's the same length, but the case has
//! // been normalized
//! assert_eq!(b"48656c6c6f2c20776f726c64".to_vec(), i);
//! ```

use simple_error::{SimpleResult, bail};

use serde::{Serialize, Deserialize};

mod transform_null;
pub use transform_null::TransformNull;

mod transform_base64;
pub use transform_base64::TransformBase64;

mod transform_base32;
pub use transform_base32::TransformBase32;

mod transform_xor_by_constant;
pub use transform_xor_by_constant::TransformXorByConstant;
pub use transform_xor_by_constant::XorSettings;

mod transform_deflate;
pub use transform_deflate::TransformDeflate;

mod transform_hex;
pub use transform_hex::TransformHex;

mod transform_block_cipher;
pub use transform_block_cipher::{TransformBlockCipher, BlockCipherPadding, BlockCipherType, BlockCipherMode};

mod transform_stream_cipher;
pub use transform_stream_cipher::{TransformStreamCipher, StreamCipherType};

mod helpers;
use helpers::transformer_trait::TransformerTrait;

/// Which transformation to perform.
#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Serialize, Deserialize)]
pub enum Transformation {
    /// No transformation - simply returns the same value. Mostly here for
    /// testing.
    ///
    /// # Example
    ///
    /// ```
    /// use libh2gb::transformation::TransformNull;
    ///
    /// // Input: "abcdef"
    /// let i: Vec<u8> = b"abcdef".to_vec();
    ///
    /// // Output: "abcdef"
    /// let o = TransformNull::new().transform(&i);
    /// assert_eq!(Ok(b"abcdef".to_vec()), o);
    /// ```
    ///
    /// # Restrictions / errors
    ///
    /// n/a
    Null(TransformNull),

    /// Xor each byte / word / dword / qword by a constant. Operates on eight,
    /// 16, 32, or 64-bit chunks.
    ///
    /// # Examples
    ///
    /// ## Eight bit
    ///
    /// ```
    /// use libh2gb::transformation::{TransformXorByConstant, XorSettings};
    ///
    /// // Input: "\x00\x01\x02\x03", XorSettings::EightBit(0xFF)
    /// let i: Vec<u8> = b"\x00\x01\x02\x03".to_vec();
    ///
    /// // Output: "\xff\xfe\xfd\xfc"
    /// let o = TransformXorByConstant::new(XorSettings::EightBit(0xFF)).transform(&i);
    /// assert_eq!(Ok(b"\xff\xfe\xfd\xfc".to_vec()), o);
    /// ```
    ///
    /// ## Sixteen bit
    ///
    /// ```
    /// use libh2gb::transformation::{TransformXorByConstant, XorSettings};
    ///
    /// // Input: "\x00\x01\x02\x03", XorSettings::SixteenBit(0xFF00)
    /// let i: Vec<u8> = b"\x00\x01\x02\x03".to_vec();
    ///
    /// // Output: "\xFF\x01\xFD\x03"
    /// let o = TransformXorByConstant::new(XorSettings::SixteenBit(0xFF00)).transform(&i);
    /// assert_eq!(Ok(b"\xff\x01\xfd\x03".to_vec()), o);
    /// ```
    ///
    /// # Restrictions / errors
    ///
    /// The size of the input buffer must be a multiple of the XOR bit size.
    ///
    /// ```
    /// use libh2gb::transformation::{TransformXorByConstant, XorSettings};
    ///
    /// let i: Vec<u8> = b"\x00".to_vec();
    ///
    /// // Error
    /// assert!(TransformXorByConstant::new(XorSettings::SixteenBit(0xFF00)).transform(&i).is_err());
    /// ```
    XorByConstant(TransformXorByConstant),

    /// Base64 decode the buffer.
    ///
    /// [`Base64Transform`] has a number of constructors to configure the
    /// padding, character set, and strictness.
    ///
    /// The validation, padding, and ability to do a two-way conversation
    /// all depends on the options.
    ///
    /// # Example 1 - Standard
    ///
    /// ```
    /// use libh2gb::transformation::TransformBase64;
    ///
    /// // Input: "AQIDBA=="
    /// let i: Vec<u8> = b"AQIDBA==".to_vec();
    ///
    /// // Output: "\x01\x02\x03\x04"
    /// let o = TransformBase64::standard().transform(&i).unwrap();
    ///
    /// assert_eq!(b"\x01\x02\x03\x04".to_vec(), o);
    /// ```
    ///
    /// # Example 2 - Error
    /// ```
    /// use libh2gb::transformation::TransformBase64;
    ///
    /// let i: Vec<u8> = b"Not valid base64~".to_vec();
    ///
    /// // Error
    /// assert!(TransformBase64::standard().transform(&i).is_err());
    /// ```
    ///
    /// # Example 3 - No padding
    ///
    /// ```
    /// use libh2gb::transformation::TransformBase64;
    ///
    /// // Input: "AQIDBA"
    /// let i: Vec<u8> = b"AQIDBA".to_vec();
    ///
    /// // Output: "\x01\x02\x03\x04"
    /// let o = TransformBase64::no_padding().transform(&i).unwrap();
    ///
    /// assert_eq!(b"\x01\x02\x03\x04".to_vec(), o);
    /// ```
    ///
    /// # Example 4 - URL character set
    ///
    /// ```
    /// use libh2gb::transformation::TransformBase64;
    ///
    /// // Input: "aa--_z8="
    /// let i: Vec<u8> = b"aa--_z8=".to_vec();
    ///
    /// // Output: "\x69\xaf\xbe\xff\x3f"
    /// let o = TransformBase64::url().transform(&i).unwrap();
    ///
    /// assert_eq!(b"\x69\xaf\xbe\xff\x3f".to_vec(), o);
    /// ```
    ///
    /// # Example 5 - Permissive
    ///
    /// ```
    /// use libh2gb::transformation::TransformBase64;
    ///
    /// // Input: "AQIDBA="
    /// let i: Vec<u8> = b"AQIDBA=".to_vec();
    ///
    /// // Output: "\x01\x02\x03\x04"
    /// let o = TransformBase64::permissive().transform(&i).unwrap();
    ///
    /// assert_eq!(b"\x01\x02\x03\x04".to_vec(), o);
    /// ```
    FromBase64(TransformBase64),

    /// Decode Base32.
    ///
    /// Like Base64, many options can be configured depending on which
    /// constructor / variant you choose. The Permissive variations will
    /// try hard to decode even broken data (ignoring illegal characters), but
    /// will almost certainly output garbage.
    ///
    /// # Example 1 - Standard
    ///
    /// ```
    /// use libh2gb::transformation::TransformBase32;
    ///
    /// // Input: "AEBAGBA="
    /// let i: Vec<u8> = b"AEBAGBA=".to_vec();
    ///
    /// // Output: "\x01\x02\x03\x04"
    /// let o = TransformBase32::standard().transform(&i).unwrap();
    ///
    /// assert_eq!(b"\x01\x02\x03\x04".to_vec(), o);
    /// ```
    ///
    /// # Example 2 - Invalid transformation
    ///
    /// ```
    /// use libh2gb::transformation::TransformBase32;
    ///
    /// let i: Vec<u8> = b"Not valid base32~".to_vec();
    ///
    /// // Error
    /// assert!(TransformBase32::standard().transform(&i).is_err());
    /// ```
    ///
    /// # Example 3 - Crockford character set
    ///
    /// ```
    /// use libh2gb::transformation::TransformBase32;
    ///
    /// // Input: "91JPRV3F"
    /// let i: Vec<u8> = b"91JPRV3F".to_vec();
    ///
    /// // Output: "Hello"
    /// let o = TransformBase32::crockford().transform(&i).unwrap();
    ///
    /// assert_eq!(b"Hello".to_vec(), o);
    /// ```
    ///
    /// # Example 4 - Permissive
    ///
    /// ```
    /// use libh2gb::transformation::TransformBase32;
    ///
    /// // Input: "AEBA??*GBA=" (bad characters will be ignored)
    /// let i: Vec<u8> = b"AEBA??*GBA=".to_vec();
    ///
    /// // Output: "\x01\x02\x03\x04"
    /// let o = TransformBase32::permissive().transform(&i).unwrap();
    ///
    /// assert_eq!(b"\x01\x02\x03\x04".to_vec(), o);
    /// ```
    FromBase32(TransformBase32),

    /// Convert from Zlib "Deflated" format with or without a header. Uses the
    /// [inflate](https://github.com/image-rs/inflate) library.
    ///
    /// This is a ONE-WAY transformation!
    ///
    /// # Restrictions / errors
    ///
    /// Must be valid deflated data.
    FromDeflated(TransformDeflate),

    /// Convert from a hex string. Case is ignored.
    ///
    /// # Example
    ///
    /// ```
    /// use libh2gb::transformation::TransformHex;
    ///
    /// // Input: "41424344"
    /// let i: Vec<u8> = b"41424344".to_vec();
    ///
    /// // Output: "ABCD"
    /// let o = TransformHex::new().transform(&i).unwrap();
    ///
    /// assert_eq!(b"ABCD".to_vec(), o);
    /// ```
    ///
    /// # Restrictions / errors
    ///
    /// Must be a hex string with an even length, made up of the digits 0-9
    /// and a-f.
    FromHex(TransformHex),

    /// Convert from a block cipher such as AES and DES.
    ///
    /// Block ciphers have a lot of knobs, such as the algorithm, the key,
    /// the IV, mode of operation, and so on. These options are all included
    /// in the [`BlockCipherSettings`] struct.
    ///
    /// # Example
    ///
    /// This is AES-128-CBC with Pkcs7 padding and a key/IV (this is by far the
    /// most common operation I see).
    ///
    /// ```
    /// use libh2gb::transformation::*;
    ///
    /// let transformation = TransformBlockCipher::new(
    ///     BlockCipherType::AES, // AES algorithm
    ///     BlockCipherMode::CBC, // Cipher block chaining
    ///     BlockCipherPadding::Pkcs7, // Pkcs7 is the usual padding
    ///     b"AAAAAAAAAAAAAAAA".to_vec(), // A 128-bit key
    ///     Some(b"BBBBBBBBBBBBBBBB".to_vec()), // A 128-bit IV
    /// ).unwrap();
    ///
    /// // Here's some encrypted data that I generated with Ruby:
    /// // irb(main):056:0> require 'openssl'
    /// // irb(main):057:0> c = OpenSSL::Cipher.new('AES-128-CBC')
    /// // irb(main):058:0> c.encrypt
    /// // irb(main):059:0> c.key = 'A' * 16
    /// // irb(main):060:0> c.iv = 'B' * 16
    /// // irb(main):061:0> puts (c.update("Hello example section!") + c.final()).bytes.map { |b| '\x%02x' % b }.join
    /// // => \x9c\xf2\x65\x82\xa2\xa7\x8b\x65\xcb\x41\xbc\x2d\x02\x1a\xe4\x18\xaf\xf4\xbc\x9e\xf4\x0c\x8a\x26\xc4\x97\x22\x26\x3e\xc2\x34\x95
    ///
    /// let result = transformation.transform(&b"\x9c\xf2\x65\x82\xa2\xa7\x8b\x65\xcb\x41\xbc\x2d\x02\x1a\xe4\x18\xaf\xf4\xbc\x9e\xf4\x0c\x8a\x26\xc4\x97\x22\x26\x3e\xc2\x34\x95".to_vec()).unwrap();
    /// assert_eq!(b"Hello example section!".to_vec(), result);
    /// ```
    FromBlockCipher(TransformBlockCipher),

    /// Convert from a stream cipher such as Salsa20, ChaCha, or Arc4.
    ///
    /// Stream ciphers have even more knobs than block ciphers. I tried to
    /// implement some of the most common algorithms with common settings, but
    /// that meant skipping others. The most popular libraries are also somewhat
    /// opinionated (for example, they don't support Salsa20 with a 128-bit
    /// key), so I had to carefully pick and choose what to implement. I'll
    /// likely go back and fill in gaps in the future.
    ///
    /// ```
    /// use libh2gb::transformation::*;
    ///
    /// let transformation = TransformStreamCipher::new(
    ///     StreamCipherType::Salsa20,    // Salsa20
    ///     b"AAAAAAAAAAAAAAAA".to_vec(), // A 128-bit key
    ///     Some(b"BBBBBBBB".to_vec()),   // A 64-bit IV
    /// ).unwrap();
    ///
    /// // Here's some encrypted data that I generated with Ruby:
    /// // irb(main):002:0> require 'salsa20'
    /// // irb(main):003:0> puts (Salsa20.new("A"*16, "B"*8).encrypt("Salsa20 Demo")).bytes.map { |b| '\x%02x' % b }.join
    /// // \xef\xc6\x5d\x82\x35\x1c\xcc\xa6\x11\xe2\x82\xfc
    ///
    /// let result = transformation.transform(&b"\xef\xc6\x5d\x82\x35\x1c\xcc\xa6\x11\xe2\x82\xfc".to_vec()).unwrap();
    /// assert_eq!(b"Salsa20 Demo".to_vec(), result);
    /// ```
    FromStreamCipher(TransformStreamCipher),
}

impl Transformation {
    fn get_transformer(&self) -> Box<dyn TransformerTrait> {
        match self {
            Self::Null(s)             => Box::new(*s),
            Self::XorByConstant(s)    => Box::new(*s),
            Self::FromBase64(s)       => Box::new(*s),
            Self::FromBase32(s)       => Box::new(*s),
            Self::FromDeflated(s)     => Box::new(*s),
            Self::FromHex(s)          => Box::new(*s),
            Self::FromBlockCipher(s)  => Box::new(*s),
            Self::FromStreamCipher(s) => Box::new(*s),
        }
    }

    /// Transform a buffer into another buffer, without changing the original.
    pub fn transform(&self, buffer: &Vec<u8>) -> SimpleResult<Vec<u8>> {
        // We can never handle 0-length buffers
        if buffer.len() == 0 {
            bail!("Cannot transform 0-length buffer");
        }

        self.get_transformer().transform(buffer)
    }

    /// Transform a buffer backwards, if possible. The length of the result will
    /// match the length of the original buffer, but the data may be normalized.
    /// The original buffer is not changed.
    pub fn untransform(&self, buffer: &Vec<u8>) -> SimpleResult<Vec<u8>> {
        // We can never handle 0-length buffers
        if buffer.len() == 0 {
            bail!("Cannot untransform 0-length buffer");
        }

        self.get_transformer().untransform(buffer)
    }

    /// Check whether a buffer can be transformed by this variant.
    ///
    /// Warning: This is a semi-expensive operation for most variants; unless
    /// the transformation is based on length or another easy-to-check factor,
    /// we simply clone the data and attempt to transform it.
    pub fn can_transform(&self, buffer: &Vec<u8>) -> bool {
        // We can never handle 0-length buffers
        if buffer.len() == 0 {
            return false;
        }

        self.get_transformer().can_transform(buffer)
    }

    /// Determines if the transformation can be undone.
    ///
    /// Does not require a buffer, because the variant itself is enough to
    /// make this determination.
    pub fn is_two_way(&self) -> bool {
        self.get_transformer().is_two_way()
    }

    /// Returns a list of possible transformations that will work on this
    /// buffer.
    ///
    /// This is VERY expensive, as it attempts to transform using every
    /// potential variant.
    pub fn detect(buffer: &Vec<u8>) -> Vec<Transformation> {
        let mut out: Vec<Transformation> = Vec::new();

        out.extend(TransformNull::detect(buffer));
        out.extend(TransformHex::detect(buffer));
        out.extend(TransformXorByConstant::detect(buffer));
        out.extend(TransformBase64::detect(buffer));
        out.extend(TransformBase32::detect(buffer));
        out.extend(TransformDeflate::detect(buffer));
        out.extend(TransformBlockCipher::detect(buffer));
        out.extend(TransformStreamCipher::detect(buffer));

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_detect() -> SimpleResult<()> {
        let tests: Vec<_> = vec![
            (
                "Testcase: 'A'",
                b"A".to_vec(),
                vec![
                ],
            ),

            (
                "Testcase: 'AA'",
                b"AA".to_vec(),
                vec![
                    TransformBase32::no_padding(),
                    TransformBase32::crockford(),
                    TransformBase64::no_padding(),
                    TransformBase64::url_no_padding(),
                    TransformHex::new(),
                ],
            ),

            (
                "Testcase: 'AA=='",
                b"AA==".to_vec(),
                vec![
                    TransformBase64::standard(),
                    TransformBase64::url(),
                ],
            ),

            (
                "Testcase: '/+AAAA=='",
                b"/+AAAA==".to_vec(),
                vec![
                    TransformBase64::standard(),
                ],
            ),

            (
                "Testcase: '-_AAAA=='",
                b"-_AAAA==".to_vec(),
                vec![
                    TransformBase64::url(),
                    TransformDeflate::without_header(),
                ],
            ),

            (
                "Testcase: Simple deflated",
                b"\x03\x00\x00\x00\x00\x01".to_vec(),
                vec![
                    TransformDeflate::without_header(),
                ]
            ),

            (
                "Testcase: Zlib deflated",
                b"\x78\x9c\x03\x00\x00\x00\x00\x01".to_vec(),
                vec![
                    TransformDeflate::with_header(),
                ]
            ),

            (
                "Testcase: Base32",
                b"ORSXG5BRGIZSA2DFNRWG6===".to_vec(),
                vec![
                    TransformBase32::standard(),
                ]
            ),

            (
                "Testcase: Base32 no padding",
                b"ORSXG5BRGIZSA2DFNRWG6".to_vec(),
                vec![
                    TransformBase32::no_padding(),
                    TransformBase32::crockford(),
                ]
            ),

            (
                "Testcase: Base32 crockford",
                b"EHJQ6X1H68SJ0T35DHP6Y".to_vec(),
                vec![
                    TransformBase32::crockford(),
                ]
            ),

            (
                "Testcase: AES-128-CBC or ECB with a NULL key + IV",
                b"\x1c\xdc\x23\xdb\xc0\xc5\x5a\x68\xd6\xed\x4e\x5b\x1d\x36\xdf\xf5".to_vec(),
                vec![
                    TransformBlockCipher::new(
                        BlockCipherType::AES,
                        BlockCipherMode::CBC,
                        BlockCipherPadding::Pkcs7,
                        b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0".to_vec(),
                        None,
                    ).unwrap(),
                    TransformBlockCipher::new(
                        BlockCipherType::AES,
                        BlockCipherMode::ECB,
                        BlockCipherPadding::Pkcs7,
                        b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0".to_vec(),
                        None,
                    ).unwrap(),
                ]
            ),
            (
                "Testcase: AES-128-CBC with a NULL key + IV",
                b"\xfb\x35\x97\x36\x05\x1a\xf5\x90\x18\xe4\xdd\x95\x31\xcd\x6a\x27\
                  \x68\x74\xd0\xd8\x48\x6b\x0b\xc8\x1f\x3c\xc4\xd1\x95\x14\xcc\x1a\
                  \x16\xc3\x0d\x5a\x74\xa9\x1a\x3c\x01\xa3\x34\x52\x69\x46\x0e\x52".to_vec(),
                vec![
                    TransformBlockCipher::new(
                        BlockCipherType::AES,
                        BlockCipherMode::CBC,
                        BlockCipherPadding::Pkcs7,
                        b"\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0".to_vec(),
                        None,
                    ).unwrap(),
                ]
            ),
        ];

        // Do this in a loop since we have to sort both vectors
        for (desc, s, r) in tests {
            let mut t = Transformation::detect(&s);
            t.sort();

            let mut r = r.clone();
            r.sort();

            assert_eq!(t, r, "{}", desc);
        }

        Ok(())
    }
}
