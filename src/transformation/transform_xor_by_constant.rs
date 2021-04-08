use simple_error::{SimpleResult, bail};
use serde::{Serialize, Deserialize};

use crate::transformation::TransformerTrait;
use crate::transformation::Transformation;

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

    fn is_two_way(&self) -> bool {
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
    use crate::transformation::Transformation;

    #[test]
    fn test_xor8() -> SimpleResult<()> {
        assert_eq!(true, Transformation::XorByConstant(XorSettings::EightBit(0)).is_two_way());

        let tests: Vec<(u8, Vec<u8>, SimpleResult<Vec<u8>>)> = vec![
            (0, vec![1],             Ok(vec![1])),
            (0, vec![1, 2, 3],       Ok(vec![1, 2, 3])),
            (0, vec![1, 2, 3, 4, 5], Ok(vec![1, 2, 3, 4, 5])),

            (1, vec![1],             Ok(vec![0])),
            (1, vec![1, 2, 3],       Ok(vec![0, 3, 2])),
            (1, vec![1, 2, 3, 4, 5], Ok(vec![0, 3, 2, 5, 4])),

            (0xFF, vec![1],             Ok(vec![254])),
            (0xFF, vec![1, 2, 3],       Ok(vec![254, 253, 252])),
            (0xFF, vec![1, 2, 3, 4, 5], Ok(vec![254, 253, 252, 251, 250])),
        ];

        for (c, test, expected) in tests {
            assert!(Transformation::XorByConstant(XorSettings::EightBit(c)).can_transform(&test));

            let result = Transformation::XorByConstant(XorSettings::EightBit(c)).transform(&test);
            assert_eq!(expected, result);

            let result = Transformation::XorByConstant(XorSettings::EightBit(c)).untransform(&result?);
            assert_eq!(Ok(test), result);
        }

        Ok(())
    }

    #[test]
    fn test_xor16() -> SimpleResult<()> {
        let t = Transformation::XorByConstant(XorSettings::SixteenBit(0x0000));

        // It can transform even-length vectors
        assert!(t.can_transform(&vec![0x11, 0x22]));
        assert!(t.can_transform(&vec![0x11, 0x22, 0x33, 0x44]));

        // It cannot transform odd-length vectors
        assert!(!t.can_transform(&vec![0x11]));
        assert!(!t.can_transform(&vec![0x11, 0x22, 0x33]));

        // Simplest examples
        let t = Transformation::XorByConstant(XorSettings::SixteenBit(0x0000));
        assert_eq!(vec![0x11, 0x22, 0x33, 0x44, 0x55, 0x66], t.transform(&vec![0x11, 0x22, 0x33, 0x44, 0x55, 0x66])?);

        let t = Transformation::XorByConstant(XorSettings::SixteenBit(0xFFFF));
        assert_eq!(vec![0xEE, 0xDD, 0xCC, 0xBB, 0xAA, 0x99], t.transform(&vec![0x11, 0x22, 0x33, 0x44, 0x55, 0x66])?);

        // More complex examples
        let t = Transformation::XorByConstant(XorSettings::SixteenBit(0x1234));

        // First byte: 0x11 & 0x12 = 0x03
        // Second byte: 0x22 & 0x34 = 0x16
        assert_eq!(vec![0x03, 0x16], t.transform(&vec![0x11, 0x22])?);

        // Third byte: 0x33 & 0x12 = 0x21
        // Fourth byte: 0x44 & 0x34 = 0x70
        assert_eq!(vec![0x03, 0x16, 0x21, 0x70], t.transform(&vec![0x11, 0x22, 0x33, 0x44])?);

        // Fail on bad strings
        assert!(t.transform(&vec![0x11]).is_err());

        Ok(())
    }

    #[test]
    fn test_xor32() -> SimpleResult<()> {
        let t = Transformation::XorByConstant(XorSettings::ThirtyTwoBit(0x00000000));

        // It can transform multiple-of-4 vectors
        assert!(t.can_transform(&vec![0x11, 0x22, 0x33, 0x44]));
        assert!(t.can_transform(&vec![0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88]));

        // It cannot transform odd-length vectors
        assert!(!t.can_transform(&vec![0x11]));
        assert!(!t.can_transform(&vec![0x11, 0x33]));
        assert!(!t.can_transform(&vec![0x11, 0x22, 0x33]));
        assert!(!t.can_transform(&vec![0x11, 0x22, 0x33, 0x44, 0x55]));

        // Simplest examples
        let t = Transformation::XorByConstant(XorSettings::ThirtyTwoBit(0x00000000));
        assert_eq!(vec![0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88], t.transform(&vec![0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88])?);

        let t = Transformation::XorByConstant(XorSettings::ThirtyTwoBit(0xFFFFFFFF));
        assert_eq!(vec![0xEE, 0xDD, 0xCC, 0xBB, 0xAA, 0x99, 0x88, 0x77], t.transform(&vec![0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88])?);

        // More complex examples
        let t = Transformation::XorByConstant(XorSettings::ThirtyTwoBit(0x12345678));

        // First byte:  0x11 & 0x12 = 0x03
        // Second byte: 0x22 & 0x34 = 0x16
        // Third byte:  0x33 & 0x56 = 0x65
        // Fourth byte: 0x44 & 0x78 = 0x3c
        assert_eq!(vec![0x03, 0x16, 0x65, 0x3c], t.transform(&vec![0x11, 0x22, 0x33, 0x44])?);

        // Fifth byte:   0x55 & 0x12 = 0x47
        // Sixth byte:   0x66 & 0x34 = 0x52
        // Seventh byte: 0x77 & 0x56 = 0x21
        // Eighth byte:  0x88 & 0x78 = 0xf0
        assert_eq!(vec![0x03, 0x16, 0x65, 0x3c, 0x47, 0x52, 0x21, 0xf0], t.transform(&vec![0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88])?);

        //assert_eq!(vec![0x03, 0x16, 0x21, 0x70], t.transform(&vec![0x11, 0x22, 0x33, 0x44])?);

        Ok(())
    }

    #[test]
    fn test_xor64() -> SimpleResult<()> {
        let t = Transformation::XorByConstant(XorSettings::SixtyFourBit(0x0000000000000000));

        // It can transform multiple-of-8 vectors
        assert!(t.can_transform(&vec![0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77]));
        assert!(t.can_transform(&vec![0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff]));

        // It cannot transform anything else
        assert!(!t.can_transform(&vec![0x00]));
        assert!(!t.can_transform(&vec![0x00, 0x11]));
        assert!(!t.can_transform(&vec![0x00, 0x11, 0x22]));
        assert!(!t.can_transform(&vec![0x00, 0x11, 0x22, 0x33]));
        assert!(!t.can_transform(&vec![0x00, 0x11, 0x22, 0x33, 0x44]));
        assert!(!t.can_transform(&vec![0x00, 0x11, 0x22, 0x33, 0x44, 0x55]));
        assert!(!t.can_transform(&vec![0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66]));
        assert!(!t.can_transform(&vec![0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88]));

        // Simplest examples
        let t = Transformation::XorByConstant(XorSettings::SixtyFourBit(0x0000000000000000));
        assert_eq!(
            vec![0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff],
            t.transform(&vec![0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff])?
        );

        let t = Transformation::XorByConstant(XorSettings::SixtyFourBit(0xFFFFFFFFFFFFFFFF));
        assert_eq!(
            vec![0xff, 0xee, 0xdd, 0xcc, 0xbb, 0xaa, 0x99, 0x88, 0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, 0x00],
            t.transform(&vec![0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff])?
        );

        // // More complex examples
        let t = Transformation::XorByConstant(XorSettings::SixtyFourBit(0x0123456789abcdef));

        // First byte:   0x00 & 0x01 = 0x01
        // Second byte:  0x11 & 0x23 = 0x32
        // Third byte:   0x22 & 0x45 = 0x67
        // Fourth byte:  0x33 & 0x67 = 0x54
        // Fifth byte:   0x44 & 0x89 = 0xcd
        // Sixth byte:   0x55 & 0xab = 0xfe
        // Seventh byte: 0x66 & 0xcd = 0xab
        // Eighth byte:  0x77 & 0xef = 0x98
        assert_eq!(
            vec![0x01, 0x32, 0x67, 0x54, 0xcd, 0xfe, 0xab, 0x98],
            t.transform(&vec![0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77])?
        );

        // First byte:   0x88 & 0x01 = 0x89
        // Second byte:  0x99 & 0x23 = 0xba
        // Third byte:   0xaa & 0x45 = 0xef
        // Fourth byte:  0xbb & 0x67 = 0xdc
        // Fifth byte:   0xcc & 0x89 = 0x45
        // Sixth byte:   0xdd & 0xab = 0x76
        // Seventh byte: 0xee & 0xcd = 0x23
        // Eighth byte:  0xff & 0xef = 0x10
        assert_eq!(
           vec![0x01, 0x32, 0x67, 0x54, 0xcd, 0xfe, 0xab, 0x98, 0x89, 0xba, 0xef, 0xdc, 0x45, 0x76, 0x23, 0x10],
            t.transform(&vec![0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee, 0xff])?
        );

        Ok(())
    }
}
