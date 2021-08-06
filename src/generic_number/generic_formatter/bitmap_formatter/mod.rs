use serde::{Serialize, Deserialize};
use simple_error::{SimpleResult, bail};

use crate::data::{bitmap_exists, from_bitmap};
use crate::generic_number::{GenericNumber, GenericFormatter, GenericFormatterImpl};

/// Render a [`GenericNumber`] as a bitmap.
///
/// # Example
///
/// ```
/// // TODO: Example of a bitmap
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BitmapFormatter {
    bitmap_type: String,
    show_negative: bool,
}

impl BitmapFormatter {
    pub fn new(bitmap_type: &str, show_negative: bool) -> SimpleResult<GenericFormatter> {
        // Make sure the bitmap type exists
        if !bitmap_exists(bitmap_type) {
            bail!("No such Bitmap: {}", bitmap_type);
        }

        Ok(GenericFormatter::Bitmap(Self {
            bitmap_type: bitmap_type.to_string(),
            show_negative: show_negative,
        }))
    }
}

impl GenericFormatterImpl for BitmapFormatter {
    fn render(&self, number: GenericNumber) -> SimpleResult<String> {
        let as_u64 = number.as_u64()?;

        let mut out: Vec<String> = Vec::new();

        // TODO: Add in the remainder
        let (output, _remainder) = from_bitmap(&self.bitmap_type, as_u64)?;

        output.into_iter().for_each(|(_value, name, present)| {
            match (present, self.show_negative) {
                // The flag is present
                (true, _) => out.push(name.to_string()),

                // The flag is not present, but we want to see it
                (false, true) => out.push(format!("~{}", name)),

                // The flag is not present, and we don't want to see it
                (false, false) => (),
            }
        });

        if out.len() == 0 {
            Ok("(n/a)".to_string())
        } else {
            Ok(out.join(" | "))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use simple_error::SimpleResult;

    #[test]
    fn test_bitmap_formatter() -> SimpleResult<()> {
        let tests = vec![
          // number                      show_negative  expected
            (GenericNumber::from(0u32),  false,         "(n/a)"),
            (GenericNumber::from(1u32),  false,         "HIDE_SLOT_HEAD"),
            (GenericNumber::from(2u32),  false,         "HIDE_SLOT_BODY"),
            (GenericNumber::from(3u32),  false,         "HIDE_SLOT_HEAD | HIDE_SLOT_BODY"),

            (GenericNumber::from(1u32),  true,          "HIDE_SLOT_HEAD | ~HIDE_SLOT_BODY | ~HIDE_SLOT_LEGS | ~HIDE_SLOT_ACCESSORY1 | ~HIDE_SLOT_ACCESSORY2 | ~HIDE_SLOT_ACCESSORY3 | ~HIDE_SLOT_ACCESSORY4 | ~HIDE_SLOT_ACCESSORY5 | ~HIDE_SLOT_ACCESSORY6 | ~HIDE_SLOT_ACCESSORY8"),
        ];

        for (number, show_negative, expected) in tests {
            assert_eq!(
                expected,
                BitmapFormatter::new("TerrariaVisibility", show_negative)?.render(number)?,
            );
        }

        Ok(())
    }
}
