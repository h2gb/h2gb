use serde::{Serialize, Deserialize};
use simple_error::{bail, SimpleResult};
use std::ops::Range;

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum Alignment {
    /// Don't align at all
    None,

    /// Align after.
    ///
    /// Each field is padded until its length is a multiple of the padding
    /// Length.. so 0..1 aligned to 4 will be 0..4, and 1..2 aligned to 4 will
    /// be 1..5
    After(u64),

    /// Align before and after.
    ///
    /// Each field must start and end on a multiple of the alignment value.
    Full(u64),

    /// Only pad after, but error out if the start isn't aligned.
    AfterStrict(u64),
}

impl Alignment {
    fn round_up(number: u64, multiple: u64) -> u64 {
        if multiple == 0 {
            return number;
        }

        let remainder = number % multiple;
        if remainder == 0 {
            return number;
        }

        number - remainder + multiple
    }

    fn round_down(number: u64, multiple: u64) -> u64 {
        if multiple == 0 {
            return number;
        }

        number - (number % multiple)
    }

    pub fn align(self, range: Range<u64>) -> SimpleResult<Range<u64>> {
        if range.end < range.start {
            bail!("Range ends before it starts");
        }

        match self {
            Self::None => Ok(range),
            Self::After(m) => {
                Ok(range.start..Self::round_up(range.end, m))
            },
            Self::Full(m) => {
                Ok(Self::round_down(range.start, m)..Self::round_up(range.end, m))
            },
            Self::AfterStrict(m) => {
                if m != 0 && (range.start % m != 0) {
                    bail!("Alignment error");
                }

                Ok(range.start..Self::round_up(range.end, m))
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use simple_error::SimpleResult;

    #[test]
    fn test_none() -> SimpleResult<()> {
        let tests: Vec<(Range<u64>, Range<u64>)> = vec![
            //       value       expected
            (         0..0,          0..0),
            (        0..10,         0..10),
            (         1..5,          1..5),
            (         2..8,          2..8),
            (10200..100000, 10200..100000),
        ];

        for (value, expected) in tests {
            assert_eq!(expected, Alignment::None.align(value)?);
        }

        Ok(())
    }

    #[test]
    fn test_after() -> SimpleResult<()> {
        let tests: Vec<(Range<u64>, u64, Range<u64>)> = vec![
            //  value  multiple  expected
            (    0..0,        0,     0..0),
            (    0..0,        4,     0..0),
            (    0..1,        4,     0..4),
            (    0..2,        4,     0..4),
            (    1..3,        4,     1..4),
            (    3..4,        4,     3..4),
            (    5..5,        4,     5..8),
            (   1..10,      789,   1..789),
            (4..10200,    10000, 4..20000),
        ];

        for (value, multiple, expected) in tests {
            assert_eq!(expected, Alignment::After(multiple).align(value)?);
        }

        Ok(())
    }

    #[test]
    fn test_full() -> SimpleResult<()> {
        let tests: Vec<(Range<u64>, u64, Range<u64>)> = vec![
            //  value  multiple  expected
            (    0..0,        0,     0..0),
            (    2..4,        4,     0..4),
            (    1..1,        4,     0..4),
            (    0..2,        4,     0..4),
            (    1..3,        4,     0..4),
            (    3..4,        4,     0..4),
            (    5..5,        4,     4..8),
            (   5..10,      789,   0..789),
            (4..10200,    10000, 0..20000),
        ];

        for (value, multiple, expected) in tests {
            assert_eq!(expected, Alignment::Full(multiple).align(value)?);
        }

        Ok(())
    }

    #[test]
    fn test_after_strict() -> SimpleResult<()> {
        let good_tests: Vec<(Range<u64>, u64, Range<u64>)> = vec![
            //      value  multiple  expected
            (    0..0,        0,         0..0),
            (    0..1,        4,         0..4),
            (    0..2,        4,         0..4),
            (    0..3,        4,         0..4),
            (    4..4,        4,         4..4),
            (    0..5,        4,         0..8),
            (    0..10,       789,       0..789),
            (    0..10200,    10000,     0..20000),
            (10000..10200,    10000, 10000..20000),
        ];

        for (value, multiple, expected) in good_tests {
            assert_eq!(expected, Alignment::AfterStrict(multiple).align(value)?);
        }

        let bad_tests: Vec<(Range<u64>, u64)> = vec![
            //   value      multiple
            (    2..3,            4),
            (    1..1,            4),
            (    5..100,          4),
            (    4..5,            8),
            (    3..4,            4),
            (    5..5,            4),
            (    5..10,         789),
            ( 9999..10200,    10000),
            (10001..10200,    10000),
        ];

        for (value, multiple) in bad_tests {
            assert!(Alignment::AfterStrict(multiple).align(value).is_err());
        }

        Ok(())
    }

}
