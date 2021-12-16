use serde::{Serialize, Deserialize};

use crate::{CharacterRenderer, Character, CharacterRendererTrait};

/// Format options for unprintable characters
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CharacterUnprintableOption {
    /// Encode unprintable characters as hex, - `'\xYY'`.
    ///
    /// This uses UTF-8, which isn't necessarily the same as the original.
    HexEncode,

    /// Encode like a C string.
    ///
    /// When possible, control characters (such as `\n`) are used, but
    /// otherwise hex-encode - `'\xYY'`.
    ///
    /// This uses UTF-8, which isn't necessarily the same as the original.
    CString,


    /// Replace with the Unicode Replacement character - '�'.
    UnicodeReplacementCharacter,

    /// URL encode - `%xx` and `+` for spaces.
    ///
    /// This uses UTF-8, which isn't necessarily the same as the original.
    URLEncode,

    /// Just try to render as best as possible.. probably not a good option
    PrintAnyways,
}

/// Which characters are considered unprintable?
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CharacterReplacementPolicy {
    /// Try to display everything - probably not a good idea
    ReplaceNone,

    /// Replace control and unprintable characters
    ReplaceControl,

    /// Replace non-ASCII (including any UTF-8 / UNICODE / etc)
    ReplaceNonAscii,

    /// Replace every character
    ReplaceEverything,
}

/// Render a [`Character`] as a character value.
///
/// # Example
///
/// ```
/// use generic_number::*;
///
/// // Create a Character directly - normally you'd use a CharacterReader
/// let number = Character::from(('a', 1)); // (the size field doesn't matter when used directly like this)
/// let othernumber = Character::from(('☃', 1));
///
/// // Default 'pretty' formatter
/// assert_eq!("'a'", CharacterFormatter::pretty_character().render(number));
/// assert_eq!("'☃'", CharacterFormatter::pretty_character().render(othernumber));
///
/// // Default 'pretty string' formatter
/// assert_eq!("a", CharacterFormatter::pretty_str_character().render(number));
/// assert_eq!("☃", CharacterFormatter::pretty_str_character().render(othernumber));
///
/// // Specify options: replace everything with hex encoding
/// assert_eq!("\\x61", CharacterFormatter::new_character(false, CharacterReplacementPolicy::ReplaceEverything, CharacterUnprintableOption::HexEncode).render(number));
/// assert_eq!("\\xe2\\x98\\x83", CharacterFormatter::new_character(false, CharacterReplacementPolicy::ReplaceEverything, CharacterUnprintableOption::HexEncode).render(othernumber));
///
/// // Specify different options: replace non-ascii characters with URL encoding
/// assert_eq!("a", CharacterFormatter::new_character(false, CharacterReplacementPolicy::ReplaceNonAscii, CharacterUnprintableOption::URLEncode).render(number));
/// assert_eq!("%e2%98%83", CharacterFormatter::new_character(false, CharacterReplacementPolicy::ReplaceNonAscii, CharacterUnprintableOption::URLEncode).render(othernumber));
/// ```
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CharacterFormatter {
    /// Put single quotes before and after the character; eg: `'c'`
    show_single_quotes: bool,

    /// If set, don't attempt to render non-ASCII characters.
    ///
    /// If this is set, all non-ASCII sequences will be rendered as hex using
    /// the `unprintable_option` setting.
    character_replacement_policy: CharacterReplacementPolicy,

    /// How are unprintable characters handled?
    unprintable_option: CharacterUnprintableOption,
}

impl CharacterFormatter {
    pub fn new_character(show_single_quotes: bool, character_replacement_policy: CharacterReplacementPolicy, unprintable_option: CharacterUnprintableOption) -> CharacterRenderer {
        CharacterRenderer::Character(Self {
            show_single_quotes:           show_single_quotes,
            character_replacement_policy: character_replacement_policy,
            unprintable_option:           unprintable_option,
        })
    }

    /// Choose decent options to look nice
    pub fn pretty_character() -> CharacterRenderer {
        Self::new_character(true, CharacterReplacementPolicy::ReplaceControl, CharacterUnprintableOption::CString)
    }

    /// Choose decent options to look nice (as part of a string)
    pub fn pretty_str_character() -> CharacterRenderer {
        Self::new_character(false, CharacterReplacementPolicy::ReplaceControl, CharacterUnprintableOption::CString)
    }

    fn handle_unprintable(self, c: char) -> String {
        match self.unprintable_option {
            CharacterUnprintableOption::HexEncode => {
                // Create a buffer (mandatory) - this function literally panics
                // if the buffer is too small, but maximum ever needed is 4
                // bytes
                let mut b = [0; 4];
                let result = c.encode_utf8(&mut b);
                result.as_bytes().into_iter().map(|b| format!("\\x{:02x}", b)).collect()
            },

            CharacterUnprintableOption::CString => {
                // Create a buffer (mandatory) - this function literally panics
                // if the buffer is too small, but maximum ever needed is 4
                // bytes
                let mut b = [0; 4];
                let result = c.encode_utf8(&mut b);
                result.bytes().into_iter().map(|b| {
                    match b {
                        0x00   => "\\0".to_string(),
                        0x07   => "\\a".to_string(),
                        0x08   => "\\b".to_string(),
                        0x09   => "\\t".to_string(),
                        0x0a   => "\\n".to_string(),
                        0x0b   => "\\v".to_string(),
                        0x0c   => "\\f".to_string(),
                        0x0d   => "\\r".to_string(),
                        _      => format!("\\x{:02x}", b),
                    }
                }).collect()
            },

            CharacterUnprintableOption::UnicodeReplacementCharacter => "�".to_string(),

            CharacterUnprintableOption::URLEncode => {
                let mut b = [0; 4];
                let result = c.encode_utf8(&mut b);
                result.as_bytes().into_iter().map(|b| {
                    if *b == (' ' as u8) {
                        "+".to_string()
                    } else {
                        format!("%{:02x}", b)
                    }
                }).collect()
            },

            CharacterUnprintableOption::PrintAnyways => {
                c.to_string()
            },
        }
    }

    fn do_render(self, c: char) -> String {
        let out = match self.character_replacement_policy {
            CharacterReplacementPolicy::ReplaceNone => c.to_string(),

            CharacterReplacementPolicy::ReplaceControl => {
                match c.is_control() {
                    true  => self.handle_unprintable(c),
                    false => c.to_string(),
                }
            },

            CharacterReplacementPolicy::ReplaceNonAscii => {
                match c.is_ascii() && !c.is_control() {
                    true  => c.to_string(),
                    false => self.handle_unprintable(c),
                }
            },

            CharacterReplacementPolicy::ReplaceEverything => {
                self.handle_unprintable(c)
            },
        };

        // Add single quotes, if requested
        match self.show_single_quotes {
            true => format!("'{}'", out),
            false => out,
        }
    }
}

impl CharacterRendererTrait for CharacterFormatter {
    fn render_character(&self, number: Character) -> String {
        self.do_render(number.as_char())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;
    use simple_error::SimpleResult;

    #[test]
    fn test_char_formatter() -> SimpleResult<()> {
        let tests = vec![
            // character  single_quotes   replacement_policy                           unprintable_option                                          expected

            // Test single quotes on/off
            (  'a',       false,          CharacterReplacementPolicy::ReplaceControl,  CharacterUnprintableOption::UnicodeReplacementCharacter,    "a"  ),
            (  'a',       true,           CharacterReplacementPolicy::ReplaceControl,  CharacterUnprintableOption::UnicodeReplacementCharacter,    "'a'"),

            // Test ReplaceNonAscii
            (  '💣',      false,          CharacterReplacementPolicy::ReplaceControl,  CharacterUnprintableOption::UnicodeReplacementCharacter,    "💣"),
            (  '💣',      false,          CharacterReplacementPolicy::ReplaceNonAscii, CharacterUnprintableOption::UnicodeReplacementCharacter,    "�" ),
            (  '\n',      false,          CharacterReplacementPolicy::ReplaceNonAscii, CharacterUnprintableOption::UnicodeReplacementCharacter,    "�" ),
            (  '~',       false,          CharacterReplacementPolicy::ReplaceNonAscii, CharacterUnprintableOption::UnicodeReplacementCharacter,    "~" ),

            // Test ReplaceEverything
            (  'a',       false,          CharacterReplacementPolicy::ReplaceEverything, CharacterUnprintableOption::UnicodeReplacementCharacter,  "�" ),
            (  '\n',      false,          CharacterReplacementPolicy::ReplaceEverything, CharacterUnprintableOption::UnicodeReplacementCharacter,  "�" ),
            (  '💣',      false,          CharacterReplacementPolicy::ReplaceEverything, CharacterUnprintableOption::UnicodeReplacementCharacter,  "�" ),

            // Test ReplaceControl
            (  'a',       false,          CharacterReplacementPolicy::ReplaceControl, CharacterUnprintableOption::UnicodeReplacementCharacter,  "a" ),
            (  '\n',      false,          CharacterReplacementPolicy::ReplaceControl, CharacterUnprintableOption::UnicodeReplacementCharacter,  "�" ),
            (  '\x7e',    false,          CharacterReplacementPolicy::ReplaceControl, CharacterUnprintableOption::UnicodeReplacementCharacter,  "~" ),
            (  '\x7f',    false,          CharacterReplacementPolicy::ReplaceControl, CharacterUnprintableOption::UnicodeReplacementCharacter,  "�" ),
            (  '💣',      false,          CharacterReplacementPolicy::ReplaceControl, CharacterUnprintableOption::UnicodeReplacementCharacter,  "💣" ),

            // Test HexEncode
            (  'a',       false,          CharacterReplacementPolicy::ReplaceEverything, CharacterUnprintableOption::HexEncode,  "\\x61" ),
            (  '\n',      false,          CharacterReplacementPolicy::ReplaceEverything, CharacterUnprintableOption::HexEncode,  "\\x0a" ),
            (  '💣',      false,          CharacterReplacementPolicy::ReplaceEverything, CharacterUnprintableOption::HexEncode,  "\\xf0\\x9f\\x92\\xa3" ),

            // Test UrlEncode
            (  'a',       false,          CharacterReplacementPolicy::ReplaceEverything, CharacterUnprintableOption::URLEncode,  "%61" ),
            (  '\n',      false,          CharacterReplacementPolicy::ReplaceEverything, CharacterUnprintableOption::URLEncode,  "%0a" ),
            (  '💣',      false,          CharacterReplacementPolicy::ReplaceEverything, CharacterUnprintableOption::URLEncode,  "%f0%9f%92%a3" ),
            (  ' ',       false,          CharacterReplacementPolicy::ReplaceEverything, CharacterUnprintableOption::URLEncode,  "+" ),
            (  '%',       false,          CharacterReplacementPolicy::ReplaceEverything, CharacterUnprintableOption::URLEncode,  "%25" ),

            // Test CString
            (  'a',       false,          CharacterReplacementPolicy::ReplaceEverything, CharacterUnprintableOption::CString,  "\\x61" ),
            (  '\n',      false,          CharacterReplacementPolicy::ReplaceEverything, CharacterUnprintableOption::CString,  "\\n" ),
            (  '💣',      false,          CharacterReplacementPolicy::ReplaceEverything, CharacterUnprintableOption::CString,  "\\xf0\\x9f\\x92\\xa3" ),
            (  ' ',       false,          CharacterReplacementPolicy::ReplaceEverything, CharacterUnprintableOption::CString,  "\\x20" ),
            (  '%',       false,          CharacterReplacementPolicy::ReplaceEverything, CharacterUnprintableOption::CString,  "\\x25" ),
        ];

        for (c, show_quotes, replacement_policy, unprintable, expected) in tests {
            let number = Character::from((c, 1)); // (the size doesn't matter here)

            assert_eq!(
                expected,
                CharacterFormatter::new_character(show_quotes, replacement_policy, unprintable).render(number),
            );
        }

        Ok(())
    }
}
