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
}
