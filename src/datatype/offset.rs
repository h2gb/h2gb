use simple_error::{bail, SimpleResult};
use sized_number::Context;

/// Represents data that is being parsed.
///
/// For some types, such as an integer, a lot can be done without actually
/// having a buffer: hence, [`Offset::Static`], which works with simply an
/// offset.
///
/// To actually read and parse data, a [`Context`] is required. A [`Context`] is
/// basically a [`std::io::Cursor`] - a pointer to a buffer of data and a
/// position.
#[derive(Debug, Clone, Copy)]
pub enum Offset<'a> {
    Static(u64),
    Dynamic(Context<'a>),
}

impl<'a> From<u64> for Offset<'a> {
    fn from(o: u64) -> Offset<'a> {
        Offset::Static(o)
    }
}

impl<'a> From<Context<'a>> for Offset<'a> {
    fn from(o: Context<'a>) -> Offset<'a> {
        Offset::Dynamic(o)
    }
}

impl<'a> Offset<'a> {
    pub fn position(self) -> u64 {
        match self {
            Self::Static(n) => n,
            Self::Dynamic(c) => c.position(),
        }
    }

    pub fn at(self, offset: u64) -> Offset<'a> {
        match self {
            Self::Static(_) => Self::Static(offset),
            Self::Dynamic(c) => Self::Dynamic(c.at(offset)),
        }
    }

    pub fn get_dynamic(self) -> SimpleResult<Context<'a>> {
        match self {
            Self::Static(_) => bail!("This operation cannot be performed on a static context"),
            Self::Dynamic(c) => Ok(c),
        }
    }
}

