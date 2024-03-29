use serde::{Serialize, Deserialize};

use simple_error::SimpleResult;
use generic_number::{Float, FloatReader, FloatRenderer};

use crate::{Alignment, H2Type, H2Types, H2TypeTrait, Offset};

/// Defines a numerical value.
///
/// This represents any standard numerical value - [`u8`], [`i32`], stuff like
/// that. The way it's defined, read, and displayed heavily leverages the
/// [`generic_number`] module.
///
/// The size a given numeric type is always known in advance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct H2Float {
    /// The sign, signedness, and endianness of the value.
    reader: FloatReader,

    /// How the value is to be displayed.
    ///
    /// This is created by the various --Formatter modules in GenericNumber.
    /// For example, [`DefaultFormatter::new()`] or [`HexFormatter::pretty()`].
    renderer: FloatRenderer,
}

impl H2Float {
    pub fn new_aligned(alignment: Alignment, reader: FloatReader, renderer: FloatRenderer) -> H2Type {
        H2Type::new(alignment, H2Types::H2Float(Self {
            reader: reader,
            renderer: renderer,
        }))
    }

    pub fn new(reader: FloatReader, renderer: FloatRenderer) -> H2Type {
        Self::new_aligned(Alignment::None, reader, renderer)
    }
}

impl H2TypeTrait for H2Float {
    fn is_static(&self) -> bool {
        true
    }

    fn actual_size(&self, _offset: Offset) -> SimpleResult<u64> {
        Ok(self.reader.size() as u64)
    }

    fn to_display(&self, offset: Offset) -> SimpleResult<String> {
        match offset {
            Offset::Static(_) => Ok("Float".to_string()),
            Offset::Dynamic(context) => {
                Ok(format!("{:?}", self.reader.read(context)?))
            }
        }
    }

    fn can_be_float(&self) -> bool {
        true
    }

    fn to_float(&self, offset: Offset) -> SimpleResult<Float> {
        self.reader.read(offset.get_dynamic()?)
    }
}
