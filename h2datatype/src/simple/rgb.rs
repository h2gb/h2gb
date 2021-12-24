use serde::{Serialize, Deserialize};

use colored::Colorize;
use simple_error::SimpleResult;

use generic_number::Context;
use crate::{Alignment, H2Type, H2Types, H2TypeTrait};

/// Defines a numerical value.
///
/// This represents any standard numerical value - [`u8`], [`i32`], stuff like
/// that. The way it's defined, read, and displayed heavily leverages the
/// [`generic_number`] module.
///
/// The size a given numeric type is always known in advance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rgb {
    colorize_output: bool
}

impl Rgb {
    pub fn new_aligned(alignment: Alignment, colorize_output: bool) -> H2Type {
        H2Type::new(alignment, H2Types::Rgb(Self {
            colorize_output: colorize_output,
        }))
    }

    pub fn new(colorize_output: bool) -> H2Type {
        Self::new_aligned(Alignment::None, colorize_output)
    }
}

impl H2TypeTrait for Rgb {
    fn base_size(&self, _context: Context) -> SimpleResult<usize> {
        Ok(3)
    }

    fn to_display(&self, context: Context) -> SimpleResult<String> {
        // Read the 24-bit value
        let colors = context.read_bytes(3)?;

        let red = colors[0];
        let green = colors[1];
        let blue = colors[2];
        let value = ((red as u32) << 16) | ((green as u32) << 8) | (blue as u32);

        if self.colorize_output {
            if ((red as u32 + green as u32 + blue as u32) / 3) > 0x80 {
                // Use a light background
                Ok(format!("#{:06x}", value).truecolor(red, green, blue).on_black().to_string())
            } else {
                // Use a dark background
                Ok(format!("#{:06x}", value).truecolor(red, green, blue).on_white().to_string())
            }
        } else {
            Ok(format!("#{:06x}", value))
        }
    }
}

#[cfg(test)]
mod tests {
    // TODO: Tests
    // use super::*;
    // use simple_error::SimpleResult;
    // use generic_number::{Context, Endian, GenericReader, HexFormatter, DefaultFormatter};

}
