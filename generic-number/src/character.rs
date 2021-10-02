use std::fmt;
use serde::{Serialize, Deserialize};

/// A number that can be any of the primitive types.
///
/// The goal of creating this enum is to wrap around *any* generic type, with
/// serialize, deserialize, and transparent conversion to [`u64`] and [`i64`].
///
/// Typically, you'd use a [`crate::GenericReader`] to create a
/// [`Character`], then a [`crate::GenericFormatter`] to
/// render it. All three of those classes can be serialized, so this operation
/// is always repeatable!
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Character {
    character: char,

    // Explicitly store the size of the character, because the source (whether
    // it was UTF8, UTF16, ASCII, etc.) is lost
    size: usize,
}

// Simplify converting from various basic types - generally, these shouldn't be
// used directly except for testing!
impl From<(char, usize)> for Character { fn from(o: (char, usize)) -> Self { Self { character: o.0, size: o.1 } } }

impl Character {
    /// The size - in bytes - of the type.
    pub fn size(self) -> usize {
        self.size
    }

    pub fn as_char(self) -> char {
        self.character
    }
}

impl fmt::Display for Character {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.character, f)
    }
}
