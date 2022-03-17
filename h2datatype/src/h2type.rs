use serde::{Serialize, Deserialize};

use simple_error::SimpleResult;
use std::ops::Range;

use generic_number::{Context, Integer, Float, Character};

use crate::{H2TypeTrait, Data, ResolvedType};
use crate::simple::*;
use crate::simple::network::*;
use crate::simple::numeric::*;
use crate::simple::string::*;
use crate::composite::*;

/// The core of this crate - defines any type of value abstractly.
///
/// In general, when consuming this crate, you probably won't be creating an
/// `H2Type` directly; rather, create one of the [`crate::simple`] or
/// [`crate::composite`] types, then use `.into()` to get H2Type.
///
/// Please note that many of the functions here are very expensive, because
/// they have to read the object and iterate every time they're called. If you
/// call `resolve()`, a static version will be created with the fields pre-
/// calculated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum H2Type {
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

impl H2Type {
    /// XXX Can this be done by an into thing?
    fn field_type(&self) -> &dyn H2TypeTrait {
        match self {
            // Simple
            H2Type::Rgb(t)       => t,
            H2Type::H2Bitmask(t) => t,
            H2Type::H2Enum(t)    => t,
            H2Type::H2UUID(t)    => t,
            H2Type::H2Blob(t)    => t,

            // Numeric
            H2Type::H2Float(t)     => t,
            H2Type::H2Character(t) => t,
            H2Type::H2Integer(t)   => t,

            // Network
            H2Type::IPv4(t)        => t,
            H2Type::IPv6(t)        => t,
            H2Type::MacAddress(t)  => t,
            H2Type::MacAddress8(t) => t,

            // Complex
            H2Type::H2Array(t)   => t,
            H2Type::H2Struct(t)  => t,

            // Strings
            H2Type::H2String(t)  => t,
            H2Type::NTString(t)  => t,
            H2Type::LPString(t)  => t,
        }
    }

    /// Get the size of just the field - no alignment included.
    ///
    /// Note that if the type has children (such as a
    /// [`crate::composite::H2Array`], the alignment on THAT is
    /// included since that's part of the actual object.
    pub fn base_size(&self, context: Context) -> SimpleResult<usize> {
        self.field_type().base_size(context)
    }

    /// Get the size of the field, including the alignment.
    pub fn aligned_size(&self, context: Context) -> SimpleResult<usize> {
        self.field_type().aligned_size(context)
    }

    /// Get the [`Range<usize>`] that the type will cover, starting at the
    /// given [`Context`], if it can be known, without adding padding.
    pub fn base_range(&self, context: Context) -> SimpleResult<Range<usize>> {
        self.field_type().base_range(context)
    }

    /// Get the [`Range<usize>`] that the type will cover, with padding.
    pub fn aligned_range(&self, context: Context) -> SimpleResult<Range<usize>> {
        self.field_type().aligned_range(context)
    }

    /// Get *related* nodes - ie, other fields that a pointer points to
    pub fn related(&self, context: Context) -> SimpleResult<Vec<(usize, H2Type)>> {
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
    pub fn resolve(&self, context: Context, name: Option<String>, data: &Data) -> SimpleResult<ResolvedType> {
        self.field_type().resolve(context, name, data)
    }

    /// Get a user-consumeable string
    pub fn to_display(&self, context: Context, data: &Data) -> SimpleResult<String> {
        self.field_type().to_display(context, data)
    }

    /// Can this value represent a [`String`]?
    pub fn can_be_string(&self) -> bool {
        self.field_type().can_be_string()
    }

    /// Try to convert to a [`String`].
    pub fn to_string(&self, context: Context, data: &Data) -> SimpleResult<String> {
        self.field_type().to_string(context, data)
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
