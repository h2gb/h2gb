use serde::{Serialize, Deserialize};

use simple_error::SimpleResult;
use std::ops::Range;

use generic_number::{Context, Integer, Float, Character};

use crate::{H2TypeTrait, Alignment, ResolvedType};
use crate::simple::*;
use crate::simple::network::*;
use crate::simple::numeric::*;
use crate::simple::string::*;
use crate::composite::*;

/// An enum used to multiplex between the various types.
///
/// Consumers of this library probably won't have to use this directly.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum H2Types {
    // Simple
    //H2Pointer(H2Pointer),
    Rgb(Rgb),
    H2Bitmask(H2Bitmask),
    H2Enum(H2Enum),
    H2UUID(H2UUID),
    H2Blob(H2Blob),

    // Numeric
    H2Character(H2Character),
    H2Float(H2Float),
    H2Integer(H2Integer),

    // Network
    IPv4(IPv4),
    IPv6(IPv6),
    MacAddress(MacAddress),
    MacAddress8(MacAddress8),

    // Strings
    H2String(H2String),
    NTString(NTString),
    LPString(LPString),

    // Composite
    H2Array(H2Array),
    H2Struct(H2Struct),

}

/// The core of this crate - defines any type of value abstractly.
///
/// In general, when consuming this crate, you probably won't be creating an
/// `H2Type` directly; rather, use the `new()` or `new_aligned()` function of
/// any of the various types defined in [`crate::simple`],
/// [`crate::composite`], or [`crate::composite::string`].
/// Those `new()` functions return an `H2Type`.
///
/// Please note that many of the functions here are very expensive, because
/// they have to read the object and iterate every time they're called. If you
/// call `resolve()`, a static version will be created with the fields pre-
/// calculated.
///
/// In terms of implementation, this basically passes everything through to
/// [`H2TypeTrait`]. The biggest reason for having this layer above the trait
/// is to store an alignment value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct H2Type {
    pub field: H2Types,
    pub alignment: Alignment,
}

impl H2Type {
    pub fn new(alignment: Alignment, field: H2Types) -> Self {
        Self {
            field: field,
            alignment: alignment,
        }
    }

    fn field_type(&self) -> &dyn H2TypeTrait {
        match &self.field {
            // Simple
            //H2Types::H2Pointer(t) => t,
            H2Types::Rgb(t)       => t,
            H2Types::H2Bitmask(t) => t,
            H2Types::H2Enum(t)    => t,
            H2Types::H2UUID(t)    => t,
            H2Types::H2Blob(t)    => t,

            // Numeric
            H2Types::H2Float(t)     => t,
            H2Types::H2Character(t) => t,
            H2Types::H2Integer(t)   => t,

            // Network
            H2Types::IPv4(t)        => t,
            H2Types::IPv6(t)        => t,
            H2Types::MacAddress(t)  => t,
            H2Types::MacAddress8(t) => t,

            // Complex
            H2Types::H2Array(t)   => t,
            H2Types::H2Struct(t)  => t,

            // Strings
            H2Types::H2String(t)   => t,
            H2Types::NTString(t)  => t,
            H2Types::LPString(t)  => t,
        }
    }

    /// Is the size known ahead of time?
    pub fn is_static(&self) -> bool {
        self.field_type().is_static()
    }

    /// Get the size of just the field - no alignment included.
    ///
    /// Note that if the type has children (such as a
    /// [`crate::composite::H2Array`], the alignment on THAT is
    /// included since that's part of the actual object.
    pub fn actual_size(&self, context: Context) -> SimpleResult<u64> {
        self.field_type().actual_size(context)
    }

    /// Get the size of the field, including the alignment.
    pub fn aligned_size(&self, context: Context) -> SimpleResult<u64> {
        self.field_type().aligned_size(context, self.alignment)
    }

    /// Get the [`Range<u64>`] that the type will cover, starting at the
    /// given [`Context`], if it can be known, without adding padding.
    pub fn actual_range(&self, context: Context) -> SimpleResult<Range<u64>> {
        self.field_type().range(context, Alignment::None)
    }

    /// Get the [`Range<u64>`] that the type will cover, with padding.
    pub fn aligned_range(&self, context: Context) -> SimpleResult<Range<u64>> {
        self.field_type().range(context, self.alignment)
    }

    /// Get *related* nodes - ie, other fields that a pointer points to
    pub fn related(&self, context: Context) -> SimpleResult<Vec<(u64, H2Type)>> {
        self.field_type().related(context)
    }

    /// Get the types that make up the given type.
    ///
    /// Some types don't have children, they are essentially leaf notes. Others
    /// (such as [`H2Array`] and
    /// [`NTString`]) do.
    pub fn children(&self, context: Context) -> SimpleResult<Vec<(Option<String>, H2Type)>> {
        self.field_type().children(context)
    }

    /// Resolve this type into a concrete type.
    ///
    /// Once a type is resolved, the size, range, data, string value, and so on
    /// are "written in stone", so to speak, which means they no longer need to
    /// be calculated.
    pub fn resolve(&self, context: Context, name: Option<String>) -> SimpleResult<ResolvedType> {
        self.field_type().resolve(context, self.alignment, name)
    }

    /// Get a user-consumeable string
    pub fn to_display(&self, context: Context) -> SimpleResult<String> {
        self.field_type().to_display(context)
    }

    /// Can this value represent a [`String`]?
    pub fn can_be_string(&self) -> bool {
        self.field_type().can_be_string()
    }

    /// Try to convert to a [`String`].
    pub fn to_string(&self, context: Context) -> SimpleResult<String> {
        self.field_type().to_string(context)
    }

    pub fn can_be_integer(&self) -> bool {
        self.field_type().can_be_integer()
    }

    pub fn to_integer(&self, context: Context) -> SimpleResult<Integer> {
        self.field_type().to_integer(context)
    }

    pub fn can_be_float(&self) -> bool {
        self.field_type().can_be_float()
    }

    pub fn to_float(&self, context: Context) -> SimpleResult<Float> {
        self.field_type().to_float(context)
    }

    pub fn can_be_character(&self) -> bool {
        self.field_type().can_be_character()
    }

    pub fn to_character(&self, context: Context) -> SimpleResult<Character> {
        self.field_type().to_character(context)
    }
}
