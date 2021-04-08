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
//! use libh2gb::transformation::Transformation;
//!
//! // Input (note that some are uppercase and some are lower - that's allowed)
//! let i: Vec<u8> = b"48656c6C6F2c20776f726c64".to_vec();
//!
//! // Output
//! let o = Transformation::FromHex.transform(&i).unwrap();
//!
//! // It's "Hello, world"
//! assert_eq!(b"Hello, world".to_vec(), o);
//!
//! // Transform back to the original
//! let i = Transformation::FromHex.untransform(&o).unwrap();
//!
//! // Get the original back - note that it's the same length, but the case has
//! // been normalized
//! assert_eq!(b"48656c6c6f2c20776f726c64".to_vec(), i);
//! ```

use simple_error::{SimpleResult, bail};

use serde::{Serialize, Deserialize};

mod transform_null;
use transform_null::TransformNull;

mod transform_base64;
use transform_base64::TransformBase64;
pub use transform_base64::Base64Settings;

mod transform_base32;
use transform_base32::TransformBase32;
pub use transform_base32::Base32Settings;

mod transform_xor_by_constant;
use transform_xor_by_constant::TransformXorByConstant;
pub use transform_xor_by_constant::XorSettings;

mod transform_deflate;
use transform_deflate::TransformDeflate;
pub use transform_deflate::DeflateSettings;

mod transform_hex;
use transform_hex::TransformHex;

mod transform_block_cipher;
use transform_block_cipher::TransformBlockCipher;
pub use transform_block_cipher::{BlockCipherSettings, BlockCipherPadding, BlockCipherType, BlockCipherMode};

mod transform_stream_cipher;
use transform_stream_cipher::TransformStreamCipher;
pub use transform_stream_cipher::{StreamCipherType, StreamCipherSettings};

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
    /// use libh2gb::transformation::Transformation;
    ///
    /// // Input: "abcdef"
    /// let i: Vec<u8> = b"abcdef".to_vec();
    ///
    /// // Output: "abcdef"
    /// let o = Transformation::Null.transform(&i);
    /// assert_eq!(Ok(b"abcdef".to_vec()), o);
    /// ```
    ///
    /// # Restrictions / errors
    ///
    /// n/a
    Null,

    /// Xor each byte / word / dword / qword by a constant. Operates on eight,
    /// 16, 32, or 64-bit chunks.
    ///
    /// # Examples
    ///
    /// ## Eight bit
    ///
    /// ```
    /// use libh2gb::transformation::{Transformation, XorSettings};
    ///
    /// // Input: "\x00\x01\x02\x03", XorSettings::EightBit(0xFF)
    /// let i: Vec<u8> = b"\x00\x01\x02\x03".to_vec();
    ///
    /// // Output: "\xff\xfe\xfd\xfc"
    /// let o = Transformation::XorByConstant(XorSettings::EightBit(0xFF)).transform(&i);
    /// assert_eq!(Ok(b"\xff\xfe\xfd\xfc".to_vec()), o);
    /// ```
    ///
    /// ## Sixteen bit
    ///
    /// ```
    /// use libh2gb::transformation::{Transformation, XorSettings};
    ///
    /// // Input: "\x00\x01\x02\x03", XorSettings::SixteenBit(0xFF00)
    /// let i: Vec<u8> = b"\x00\x01\x02\x03".to_vec();
    ///
    /// // Output: "\xFF\x01\xFD\x03"
    /// let o = Transformation::XorByConstant(XorSettings::SixteenBit(0xFF00)).transform(&i);
    /// assert_eq!(Ok(b"\xff\x01\xfd\x03".to_vec()), o);
    /// ```
    ///
    /// # Restrictions / errors
    ///
    /// The size of the input buffer must be a multiple of the XOR bit size.
    ///
    /// ```
    /// use libh2gb::transformation::{Transformation, XorSettings};
    ///
    /// let i: Vec<u8> = b"\x00".to_vec();
    ///
    /// // Error
    /// assert!(Transformation::XorByConstant(XorSettings::SixteenBit(0xFF00)).transform(&i).is_err());
    /// ```
    XorByConstant(XorSettings),

    /// Generic Base64 implementation.
    ///
    /// You can use this and set up your [`Base64Settings`] by hand, if you
    /// like, or you can use one of the other Base64 types from this enum.
    FromBase64(Base64Settings),

    /// Convert from standard Base64 with padding (the '=' signs at the end).
    ///
    /// # Example
    ///
    /// ```
    /// use libh2gb::transformation::Transformation;
    ///
    /// // Input: "AQIDBA=="
    /// let i: Vec<u8> = b"AQIDBA==".to_vec();
    ///
    /// // Output: "\x01\x02\x03\x04"
    /// let o = Transformation::FromBase64Standard.transform(&i).unwrap();
    ///
    /// assert_eq!(b"\x01\x02\x03\x04".to_vec(), o);
    /// ```
    ///
    /// # Restrictions / errors
    ///
    /// Must be valid Base64 with correct padding and decode to full bytes.
    ///
    /// ```
    /// use libh2gb::transformation::Transformation;
    ///
    /// let i: Vec<u8> = b"Not valid base64~".to_vec();
    ///
    /// // Error
    /// assert!(Transformation::FromBase64Standard.transform(&i).is_err());
    /// ```
    FromBase64Standard,

    /// Convert from standard Base64 with NO padding.
    ///
    /// # Example
    ///
    /// ```
    /// use libh2gb::transformation::Transformation;
    ///
    /// // Input: "AQIDBA"
    /// let i: Vec<u8> = b"AQIDBA".to_vec();
    ///
    /// // Output: "\x01\x02\x03\x04"
    /// let o = Transformation::FromBase64NoPadding.transform(&i).unwrap();
    ///
    /// assert_eq!(b"\x01\x02\x03\x04".to_vec(), o);
    /// ```
    ///
    /// # Restrictions / errors
    ///
    /// Must be valid Base64 with NO padding whatsoever, and decode to full bytes.
    ///
    /// ```
    /// use libh2gb::transformation::Transformation;
    ///
    /// let i: Vec<u8> = b"Not valid base64~".to_vec();
    ///
    /// // Error
    /// assert!(Transformation::FromBase64NoPadding.transform(&i).is_err());
    /// ```
    FromBase64NoPadding,

    /// Convert from standard Base64 with optional padding, with some attempt
    /// to ignore problems.
    ///
    /// This is a ONE-WAY transformation!
    ///
    /// # Example
    ///
    /// ```
    /// use libh2gb::transformation::Transformation;
    ///
    /// // Input: "AQIDBA="
    /// let i: Vec<u8> = b"AQIDBA=".to_vec();
    ///
    /// // Output: "\x01\x02\x03\x04"
    /// let o = Transformation::FromBase64Permissive.transform(&i).unwrap();
    ///
    /// assert_eq!(b"\x01\x02\x03\x04".to_vec(), o);
    /// ```
    ///
    /// # Restrictions / errors
    ///
    /// Must be valid enough Base64.
    ///
    /// ```
    /// use libh2gb::transformation::Transformation;
    ///
    /// let i: Vec<u8> = b"Not valid base64~".to_vec();
    ///
    /// // Error
    /// assert!(Transformation::FromBase64Permissive.transform(&i).is_err());
    /// ```
    FromBase64Permissive,

    /// Convert from URL-safe Base64 with padding - that is, `+` becomes `-`
    /// and `/` becomes `_`.
    ///
    /// # Example
    ///
    /// ```
    /// use libh2gb::transformation::Transformation;
    ///
    /// // Input: "aa--_z8="
    /// let i: Vec<u8> = b"aa--_z8=".to_vec();
    ///
    /// // Output: "\x69\xaf\xbe\xff\x3f"
    /// let o = Transformation::FromBase64URL.transform(&i).unwrap();
    ///
    /// assert_eq!(b"\x69\xaf\xbe\xff\x3f".to_vec(), o);
    /// ```
    ///
    /// # Restrictions / errors
    ///
    /// Must be valid Base64 with correct padding and decode to full bytes.
    ///
    /// ```
    /// use libh2gb::transformation::Transformation;
    ///
    /// let i: Vec<u8> = b"Not valid base64~".to_vec();
    ///
    /// // Error
    /// assert!(Transformation::FromBase64URL.transform(&i).is_err());
    /// ```
    FromBase64URL,

    /// Convert from URL-safe Base64 with NO padding.
    ///
    /// # Example
    ///
    /// ```
    /// use libh2gb::transformation::Transformation;
    ///
    /// // Input: "aa--_z8"
    /// let i: Vec<u8> = b"aa--_z8".to_vec();
    ///
    /// // Output: "\x69\xaf\xbe\xff\x3f"
    /// let o = Transformation::FromBase64URLNoPadding.transform(&i).unwrap();
    ///
    /// assert_eq!(b"\x69\xaf\xbe\xff\x3f".to_vec(), o);
    /// ```
    ///
    /// # Restrictions / errors
    ///
    /// Must be valid Base64 with NO padding whatsoever, and decode to full bytes.
    ///
    /// ```
    /// use libh2gb::transformation::Transformation;
    ///
    /// let i: Vec<u8> = b"Not valid base64~".to_vec();
    ///
    /// // Error
    /// assert!(Transformation::FromBase64URLNoPadding.transform(&i).is_err());
    /// ```
    FromBase64URLNoPadding,

    /// Convert from URL-safe Base64URL with optional padding, with some attempt
    /// to ignore problems.
    ///
    /// This is a ONE-WAY transformation!
    ///
    /// # Example
    ///
    /// ```
    /// use libh2gb::transformation::Transformation;
    ///
    /// // Input: "aa--_z8"
    /// let i: Vec<u8> = b"aa--_z8".to_vec();
    ///
    /// // Output: "\x69\xaf\xbe\xff\x3f"
    /// let o = Transformation::FromBase64URLPermissive.transform(&i).unwrap();
    ///
    /// assert_eq!(b"\x69\xaf\xbe\xff\x3f".to_vec(), o);
    /// ```
    ///
    /// # Restrictions / errors
    ///
    /// Must be valid enough Base64.
    ///
    /// ```
    /// use libh2gb::transformation::Transformation;
    ///
    /// let i: Vec<u8> = b"Not valid base64~".to_vec();
    ///
    /// // Error
    /// assert!(Transformation::FromBase64URLPermissive.transform(&i).is_err());
    /// ```
    FromBase64URLPermissive,

    /// General Base32 class.
    ///
    /// You can use this class and set up the [`Base32Settings`] by hand, or
    /// just use one of the pre-configured variants below.
    FromBase32(Base32Settings),

    /// Convert from standard Base32 with padding. Case is ignored.
    ///
    /// # Example
    ///
    /// ```
    /// use libh2gb::transformation::Transformation;
    ///
    /// // Input: "AEBAGBA="
    /// let i: Vec<u8> = b"AEBAGBA=".to_vec();
    ///
    /// // Output: "\x01\x02\x03\x04"
    /// let o = Transformation::FromBase32Standard.transform(&i).unwrap();
    ///
    /// assert_eq!(b"\x01\x02\x03\x04".to_vec(), o);
    /// ```
    ///
    /// # Restrictions / errors
    ///
    /// Must be valid Base32 with correct padding and decode to full bytes.
    ///
    /// ```
    /// use libh2gb::transformation::Transformation;
    ///
    /// let i: Vec<u8> = b"Not valid base32~".to_vec();
    ///
    /// // Error
    /// assert!(Transformation::FromBase32Standard.transform(&i).is_err());
    /// ```
    FromBase32Standard,

    /// Convert from standard Base32 with no padding. Case is ignored.
    ///
    /// # Example
    ///
    /// ```
    /// use libh2gb::transformation::Transformation;
    ///
    /// // Input: "AEBAGBA"
    /// let i: Vec<u8> = b"AEBAGBA".to_vec();
    ///
    /// // Output: "\x01\x02\x03\x04"
    /// let o = Transformation::FromBase32NoPadding.transform(&i).unwrap();
    ///
    /// assert_eq!(b"\x01\x02\x03\x04".to_vec(), o);
    /// ```
    ///
    /// # Restrictions / errors
    ///
    /// Must be valid Base32 with no padding and decode to full bytes.
    ///
    /// ```
    /// use libh2gb::transformation::Transformation;
    ///
    /// let i: Vec<u8> = b"Not valid base32~".to_vec();
    ///
    /// // Error
    /// assert!(Transformation::FromBase32NoPadding.transform(&i).is_err());
    /// ```
    FromBase32NoPadding,

    /// Convert from Base32 using the Crockford alphabet, which does not allow
    /// padding. Case is ignored, and ambiguous letters (like i/l/L) are
    /// treated the same. Untransforming is possible, but will be normalized.
    ///
    /// # Example
    ///
    /// ```
    /// use libh2gb::transformation::Transformation;
    ///
    /// // Input: "91JPRV3F"
    /// let i: Vec<u8> = b"91JPRV3F".to_vec();
    ///
    /// // Output: "Hello"
    /// let o = Transformation::FromBase32Crockford.transform(&i).unwrap();
    ///
    /// assert_eq!(b"Hello".to_vec(), o);
    /// ```
    ///
    /// # Restrictions / errors
    ///
    /// Must be valid Base32 Crockford with no padding and decode to full bytes.
    ///
    /// ```
    /// use libh2gb::transformation::Transformation;
    ///
    /// let i: Vec<u8> = b"Not valid base32~".to_vec();
    ///
    /// // Error
    /// assert!(Transformation::FromBase32Crockford.transform(&i).is_err());
    /// ```
    FromBase32Crockford,

    /// Convert from standard Base32 with optional padding. Any non-Base32
    /// characters are ignored and discarded.
    ///
    /// This is a ONE-WAY transformation!
    ///
    /// # Example
    ///
    /// ```
    /// use libh2gb::transformation::Transformation;
    ///
    /// // Input: "AEBAGBA="
    /// let i: Vec<u8> = b"AEBAGBA=".to_vec();
    ///
    /// // Output: "\x01\x02\x03\x04"
    /// let o = Transformation::FromBase32Standard.transform(&i).unwrap();
    ///
    /// assert_eq!(b"\x01\x02\x03\x04".to_vec(), o);
    /// ```
    ///
    /// # Restrictions / errors
    ///
    /// Must be close enough to Base32 and decode to full bytes.
    ///
    /// ```
    /// use libh2gb::transformation::Transformation;
    ///
    /// let i: Vec<u8> = b"Not valid base32~0123456789".to_vec();
    ///
    /// // Error
    /// assert!(Transformation::FromBase32Permissive.transform(&i).is_err());
    /// ```
    FromBase32Permissive,

    /// Convert from Base32 using the Crockford alphabet, but allow optional
    /// padding. Case is ignored, and ambiguous letters (like i/l/L) are
    /// treated the same. All non-Base32 characters are ignored.
    ///
    /// This is a ONE-WAY transformation!
    ///
    /// # Example
    ///
    /// ```
    /// use libh2gb::transformation::Transformation;
    ///
    /// // Input: "91JPRV3F=="
    /// let i: Vec<u8> = b"91JPRV3F==".to_vec();
    ///
    /// // Output: "Hello"
    /// let o = Transformation::FromBase32CrockfordPermissive.transform(&i).unwrap();
    ///
    /// assert_eq!(b"Hello".to_vec(), o);
    /// ```
    ///
    /// # Restrictions / errors
    ///
    /// Must be valid enough Base32 Crockford and decode to full bytes (the
    /// letter 'u', for example, is not allowed)
    ///
    /// ```
    /// use libh2gb::transformation::Transformation;
    ///
    /// let i: Vec<u8> = b"uuuuu".to_vec();
    ///
    /// // Error
    /// assert!(Transformation::FromBase32CrockfordPermissive.transform(&i).is_err());
    /// ```
    FromBase32CrockfordPermissive,

    /// Generic from deflated
    FromDeflated(DeflateSettings),

    /// Convert from Zlib "Deflated" format with no header. Uses the
    /// [inflate](https://github.com/image-rs/inflate) library.
    ///
    /// This is a ONE-WAY transformation!
    ///
    /// # Restrictions / errors
    ///
    /// Must be valid deflated data.
    FromDeflatedNoZlibHeader,

    /// Convert from Zlib "Deflated" format with a header. Uses the
    /// [inflate](https://github.com/image-rs/inflate) library.
    ///
    /// This is a ONE-WAY transformation!
    ///
    /// # Restrictions / errors
    ///
    /// Must be valid deflated data with a valid checksum.
    FromDeflatedZlibHeader,

    /// Convert from a hex string. Case is ignored.
    ///
    /// # Example
    ///
    /// ```
    /// use libh2gb::transformation::Transformation;
    ///
    /// // Input: "41424344"
    /// let i: Vec<u8> = b"41424344".to_vec();
    ///
    /// // Output: "ABCD"
    /// let o = Transformation::FromHex.transform(&i).unwrap();
    ///
    /// assert_eq!(b"ABCD".to_vec(), o);
    /// ```
    ///
    /// # Restrictions / errors
    ///
    /// Must be a hex string with an even length, made up of the digits 0-9
    /// and a-f.
    FromHex,

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
    /// let transformation = Transformation::FromBlockCipher(BlockCipherSettings::new(
    ///     BlockCipherType::AES, // AES algorithm
    ///     BlockCipherMode::CBC, // Cipher block chaining
    ///     BlockCipherPadding::Pkcs7, // Pkcs7 is the usual padding
    ///     b"AAAAAAAAAAAAAAAA".to_vec(), // A 128-bit key
    ///     Some(b"BBBBBBBBBBBBBBBB".to_vec()), // A 128-bit IV
    /// ).unwrap());
    ///
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
    FromBlockCipher(BlockCipherSettings),

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
    /// let transformation = Transformation::FromStreamCipher(StreamCipherSettings::new(
    ///     StreamCipherType::Salsa20,    // Salsa20
    ///     b"AAAAAAAAAAAAAAAA".to_vec(), // A 128-bit key
    ///     Some(b"BBBBBBBB".to_vec()),   // A 64-bit IV
    /// ).unwrap());
    ///
    /// // Here's some encrypted data that I generated with Ruby:
    /// // irb(main):002:0> require 'salsa20'
    /// // irb(main):003:0> puts (Salsa20.new("A"*16, "B"*8).encrypt("Salsa20 Demo")).bytes.map { |b| '\x%02x' % b }.join
    /// // \xef\xc6\x5d\x82\x35\x1c\xcc\xa6\x11\xe2\x82\xfc
    ///
    /// let result = transformation.transform(&b"\xef\xc6\x5d\x82\x35\x1c\xcc\xa6\x11\xe2\x82\xfc".to_vec()).unwrap();
    /// assert_eq!(b"Salsa20 Demo".to_vec(), result);
    /// ```
    FromStreamCipher(StreamCipherSettings),
}

impl Transformation {
    fn get_transformer(&self) -> Box<dyn TransformerTrait> {
        match self {
            Self::Null                          => Box::new(TransformNull::new()),
            Self::XorByConstant(c)              => Box::new(TransformXorByConstant::new(*c)),

            Self::FromBase64(s)                 => Box::new(TransformBase64::new(*s)),
            Self::FromBase64Standard            => Box::new(TransformBase64::new(Base64Settings::standard())),
            Self::FromBase64NoPadding           => Box::new(TransformBase64::new(Base64Settings::no_padding())),
            Self::FromBase64Permissive          => Box::new(TransformBase64::new(Base64Settings::permissive())),
            Self::FromBase64URL                 => Box::new(TransformBase64::new(Base64Settings::url())),
            Self::FromBase64URLNoPadding        => Box::new(TransformBase64::new(Base64Settings::url_no_padding())),
            Self::FromBase64URLPermissive       => Box::new(TransformBase64::new(Base64Settings::url_permissive())),

            Self::FromBase32(s)                 => Box::new(TransformBase32::new(*s)),
            Self::FromBase32Standard            => Box::new(TransformBase32::new(Base32Settings::standard())),
            Self::FromBase32NoPadding           => Box::new(TransformBase32::new(Base32Settings::no_padding())),
            Self::FromBase32Crockford           => Box::new(TransformBase32::new(Base32Settings::crockford())),
            Self::FromBase32Permissive          => Box::new(TransformBase32::new(Base32Settings::permissive())),
            Self::FromBase32CrockfordPermissive => Box::new(TransformBase32::new(Base32Settings::crockford_permissive())),

            Self::FromDeflated(s)               => Box::new(TransformDeflate::new(*s)),
            Self::FromDeflatedNoZlibHeader      => Box::new(TransformDeflate::new(DeflateSettings::no_zlib_header())),
            Self::FromDeflatedZlibHeader        => Box::new(TransformDeflate::new(DeflateSettings::zlib_header())),

            Self::FromHex                       => Box::new(TransformHex::new()),

            Self::FromBlockCipher(s)            => Box::new(TransformBlockCipher::new(*s)),
            Self::FromStreamCipher(s)           => Box::new(TransformStreamCipher::new(*s)),
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

        self.get_transformer().check(buffer)
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
                    Transformation::FromBase64NoPadding,
                    Transformation::FromBase64URLNoPadding,
                    Transformation::FromHex,
                    Transformation::FromBase32NoPadding,
                    Transformation::FromBase32Crockford,
                ],
            ),

            (
                "Testcase: 'AA=='",
                b"AA==".to_vec(),
                vec![
                    Transformation::FromBase64Standard,
                    Transformation::FromBase64URL,
                ],
            ),

            (
                "Testcase: '/+AAAA=='",
                b"/+AAAA==".to_vec(),
                vec![
                    Transformation::FromBase64Standard,
                ],
            ),

            (
                "Testcase: '-_AAAA=='",
                b"-_AAAA==".to_vec(),
                vec![
                    Transformation::FromBase64URL,
                    Transformation::FromDeflatedNoZlibHeader,
                ],
            ),

            (
                "Testcase: Simple deflated",
                b"\x03\x00\x00\x00\x00\x01".to_vec(),
                vec![
                    Transformation::FromDeflatedNoZlibHeader,
                ]
            ),

            (
                "Testcase: Zlib deflated",
                b"\x78\x9c\x03\x00\x00\x00\x00\x01".to_vec(),
                vec![
                    Transformation::FromDeflatedZlibHeader,
                ]
            ),

            (
                "Testcase: Base32",
                b"ORSXG5BRGIZSA2DFNRWG6===".to_vec(),
                vec![
                    Transformation::FromBase32Standard,
                ]
            ),

            (
                "Testcase: Base32 no padding",
                b"ORSXG5BRGIZSA2DFNRWG6".to_vec(),
                vec![
                    Transformation::FromBase32NoPadding,
                    Transformation::FromBase32Crockford,
                ]
            ),

            (
                "Testcase: Base32 crockford",
                b"EHJQ6X1H68SJ0T35DHP6Y".to_vec(),
                vec![
                    Transformation::FromBase32Crockford,
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
