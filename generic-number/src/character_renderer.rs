use serde::{Serialize, Deserialize};

use crate::{Character, DefaultFormatter, CharacterFormatter};

/// Define the interface for rendering an integer
pub trait CharacterRendererTrait {
    fn render_character(&self, number: impl Into<Character>) -> String;
}

/// Configure how a [`Character`] is rendered.
///
/// You don't generally want to create this enum directly; instead, use a
/// formatter instead.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CharacterRenderer {
    Default(DefaultFormatter),
    Character(CharacterFormatter),
}

impl Default for CharacterRenderer {
    fn default() -> Self {
        Self::Default(Default::default())
    }
}

impl CharacterRenderer {
    /// Render the given number
    pub fn render_character(&self, number: impl Into<Character>) -> String {
        match self {
            Self::Default(f)    => f.render_character(number),
            Self::Character(f)  => f.render_character(number),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use simple_error::SimpleResult;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_pretty() -> SimpleResult<()> {
        assert_eq!("'A'",   CharacterFormatter::new_pretty().render_character(('A', 1)));
        assert_eq!("'\\n'", CharacterFormatter::new_pretty().render_character(('\n', 1)));

        Ok(())
    }
}
