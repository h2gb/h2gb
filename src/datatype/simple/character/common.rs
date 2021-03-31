pub fn display_character(c: char) -> String {
    match c as u32 {
        0x00        => "'\\0'".to_string(),
        0x01..=0x06 => format!("'\\x{:02x}'", c as u32),
        0x07        => "'\\a'".to_string(),
        0x08        => "'\\b'".to_string(),
        0x09        => "'\\t'".to_string(),
        0x0a        => "'\\n'".to_string(),
        0x0b        => "'\\v'".to_string(),
        0x0c        => "'\\f'".to_string(),
        0x0d        => "'\\r'".to_string(),
        0x0e..=0x1f => format!("'\\x{:02x}'", c as u32),

        _ => format!("'{}'", c)
    }
}
