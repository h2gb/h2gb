use serde::{Serialize, Deserialize};

use crate::{Character, DefaultFormatter, CharacterFormatter};

/// Define the interface for rendering an integer
pub trait CharacterRendererTrait {
    fn render_character(&self, number: Character) -> String;
}

/// Configure how a [`Character`] is rendered.
///
/// You don't generally want to create this enum directly; instead, use the
/// `new_character()` or `pretty_character()` methods in the formatter you want.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CharacterRenderer {
    Default(DefaultFormatter),
    Character(CharacterFormatter),
}

impl CharacterRenderer {
    /// Render the given number
    pub fn render(&self, number: Character) -> String {
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

    use crate::Character;

    #[test]
    fn test_pretty() -> SimpleResult<()> {
        assert_eq!("'A'",   CharacterFormatter::pretty_character().render(Character::from(('A', 1))));
        assert_eq!("'\\n'", CharacterFormatter::pretty_character().render(Character::from(('\n', 1))));

        Ok(())
    }
}
