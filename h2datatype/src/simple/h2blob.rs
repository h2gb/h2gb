use serde::{Serialize, Deserialize};
use simple_error::{bail, SimpleResult};

use generic_number::{Context, IntegerRenderer, Integer};

use crate::{H2Type, H2TypeTrait, Alignment, Data};

/// Defines a string with a configured length.
///
/// The length (in characters) is chosen when creating the type. The length in
/// bytes may be longer if the character type is non-ASCII, however. See
/// [`generic_number::GenericReader`] for a list of possible character
/// types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct H2Blob {
    length: usize,

    #[serde(default)]
    length_display: IntegerRenderer,

    #[serde(skip_serializing_if = "Option::is_none")]
    alignment: Option<Alignment>,
}

impl From<H2Blob> for H2Type {
    fn from(t: H2Blob) -> H2Type {
        H2Type::H2Blob(t)
    }
}

impl H2Blob {
    pub fn new_aligned(alignment: Option<Alignment>, length_in_bytes: usize, length_display: impl Into<IntegerRenderer>) -> SimpleResult<Self> {
        if length_in_bytes == 0 {
            bail!("Length must be at least 1 character long");
        }

        Ok(Self {
            length: length_in_bytes,
            length_display: length_display.into(),
            alignment: alignment,
        })
    }

    pub fn new(length_in_bytes: usize, length_display: impl Into<IntegerRenderer>) -> SimpleResult<Self> {
        Self::new_aligned(None, length_in_bytes, length_display)
    }
}

impl H2TypeTrait for H2Blob {
    fn base_size(&self, _context: Context) -> SimpleResult<usize> {
        Ok(self.length)
    }

    fn to_display(&self, _context: Context, _data: &Data) -> SimpleResult<String> {
        Ok(format!("Binary blob ({} bytes)", self.length_display.render_integer(Integer::from(self.length))))
    }

    fn alignment(&self) -> Option<Alignment> {
        self.alignment
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;
    use pretty_assertions::assert_eq;
    use generic_number::{Context, HexFormatter, DefaultFormatter};

    #[test]
    fn test_blob() -> SimpleResult<()> {
        let data = b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00".to_vec();
        let context = Context::new(&data);

        assert_eq!("Binary blob (16 bytes)", H2Blob::new(16, DefaultFormatter::new())?.to_display(context, &Data::default())?);
        assert_eq!(16, H2Blob::new(16, DefaultFormatter::new())?.base_size(context)?);

        assert_eq!("Binary blob (0x10 bytes)", H2Blob::new(16, HexFormatter::new(false, true, false))?.to_display(context, &Data::default())?);
        assert_eq!(16, H2Blob::new(16, DefaultFormatter::new())?.base_size(context)?);

        Ok(())
    }

    #[test]
    fn test_zero_length_blob() -> SimpleResult<()> {
        assert!(H2Blob::new(0, DefaultFormatter::new()).is_err());
        assert!(H2Blob::new(1, DefaultFormatter::new()).is_ok());

        Ok(())
    }
}
