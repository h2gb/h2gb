use simple_error::{SimpleResult, bail};
use serde::{Serialize, Deserialize};

use crate::transformation::TransformerTrait;

/// When performing an XorByConstant transformation, this represents the size
#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Serialize, Deserialize)]
pub enum XorSettings {
    /// One byte / 8 bits - eg, `0x12`
    EightBit(u8),

    /// Two bytes / 16 bits - eg, `0x1234`
    SixteenBit(u16),

    /// Four bytes / 32 bits - eg, `0x12345678`
    ThirtyTwoBit(u32),

    /// Eight bytes / 64 bits - eg, `0x123456789abcdef0`
    SixtyFourBit(u64),
}

#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Copy, Serialize, Deserialize)]
pub struct TransformXorByConstant {
    settings: XorSettings,
}

impl TransformXorByConstant {
    pub fn new(settings: XorSettings) -> Self {
        Self {
            settings: settings,
        }
    }
}

impl TransformerTrait for TransformXorByConstant {
    fn transform(&self, buffer: &Vec<u8>) -> SimpleResult<Vec<u8>> {
        if !self.check(buffer) {
            bail!("Xor failed: Xor isn't a multiple of the buffer size");
        }

        // Clone the buffer so we can edit in place
        let mut buffer = buffer.clone();

        match self.settings {
            XorSettings::EightBit(c) => {
                // Transform in-place, since we can
                for n in &mut buffer {
                    *n = *n ^ c;
                }
            },
            XorSettings::SixteenBit(c) => {
                let xorer: Vec<u8> = vec![
                    ((c >> 8) & 0x00FF) as u8,
                    ((c >> 0) & 0x00FF) as u8,
                ];

                let mut xor_position: usize = 0;
                for n in &mut buffer {
                    *n = *n ^ (xorer[xor_position]);
                    xor_position = (xor_position + 1) % 2;
                }
            },
            XorSettings::ThirtyTwoBit(c) => {
                let xorer: Vec<u8> = vec![
                    ((c >> 24) & 0x00FF) as u8,
                    ((c >> 16) & 0x00FF) as u8,
                    ((c >> 8)  & 0x00FF) as u8,
                    ((c >> 0)  & 0x00FF) as u8,
                ];

                let mut xor_position: usize = 0;
                for n in &mut buffer {
                    *n = *n ^ (xorer[xor_position]);
                    xor_position = (xor_position + 1) % 4;
                }
            },
            XorSettings::SixtyFourBit(c) => {
                let xorer: Vec<u8> = vec![
                    ((c >> 56) & 0x00FF) as u8,
                    ((c >> 48) & 0x00FF) as u8,
                    ((c >> 40) & 0x00FF) as u8,
                    ((c >> 32) & 0x00FF) as u8,
                    ((c >> 24) & 0x00FF) as u8,
                    ((c >> 16) & 0x00FF) as u8,
                    ((c >> 8)  & 0x00FF) as u8,
                    ((c >> 0)  & 0x00FF) as u8,
                ];

                let mut xor_position: usize = 0;
                for n in &mut buffer {
                    *n = *n ^ (xorer[xor_position]);
                    xor_position = (xor_position + 1) % 8;
                }
            },
        };

        Ok(buffer)
    }

    fn untransform(&self, buffer: &Vec<u8>) -> SimpleResult<Vec<u8>> {
        // Untransform is identical to transform
        self.transform(buffer)
    }

    fn check(&self, buffer: &Vec<u8>) -> bool {
        match self.settings {
            XorSettings::EightBit(_)     => true,
            XorSettings::SixteenBit(_)   => {
                (buffer.len() % 2) == 0
            },
            XorSettings::ThirtyTwoBit(_) => {
                (buffer.len() % 4) == 0
            },
            XorSettings::SixtyFourBit(_) => {
                (buffer.len() % 8) == 0
            },
        }
    }

}
