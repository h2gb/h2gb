pub fn round_up(number: u64, multiple: u64) -> u64 {
    if multiple == 0 {
        return number;
    }

    let remainder = number % multiple;
    if remainder == 0 {
        return number;
    }

    return number - remainder + multiple;
}

pub fn maybe_round_up(number: u64, multiple: Option<u64>) -> u64 {
    match multiple {
        Some(m) => round_up(number, m),
        None => number,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_round_up() {
        let tests: Vec<(u64, u64, u64)> = vec![
            // number   multiple  expected
            (    0,        0,        0),
            (    0,        4,        0),
            (    1,        4,        4),
            (    2,        4,        4),
            (    3,        4,        4),
            (    4,        4,        4),
            (    5,        4,        8),
            (   10,      789,      789),
            (10200,    10000,    20000),
        ];

        for (number, multiple, expected) in tests {
            assert_eq!(expected, round_up(number, multiple));
        }
    }

    #[test]
    fn test_maybe_round_up() {
        let tests: Vec<(u64, Option<u64>, u64)> = vec![
            // number   multiple        expected
            (    0,     Some(    0),        0),
            (    0,     Some(    4),        0),
            (    1,     Some(    4),        4),
            (    2,     Some(    4),        4),
            (    3,     Some(    4),        4),
            (    4,     Some(    4),        4),
            (    5,     Some(    4),        8),
            (   10,     Some(  789),      789),
            (10200,     Some(10000),    20000),

            (    0,     None,               0),
            (    1,     None,               1),
            (    2,     None,               2),
            (    3,     None,               3),
            (    4,     None,               4),
            (    5,     None,               5),
            (   10,     None,              10),
            (10200,     None,           10200),
        ];

        for (number, multiple, expected) in tests {
            assert_eq!(expected, maybe_round_up(number, multiple));
        }
    }
}
