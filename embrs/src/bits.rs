//! Support for converting between Rust types and bitwise representations,
//! including registers with packed bitfields.
//!
//! This module is an attempt to reduce the boilerplate in interacting with
//! packed registers, without going all the way to a compiler plugin.

#![macro_use]

use core;

/// Error type indicating that some bits read from the hardware weren't valid
/// for the expected type.  This usually indicates a driver bug, but can also
/// indicate misbehaving hardware.
pub struct BadBits(pub u32);

/// Result type for `BadBits`.
pub type BitsResult<T> = Result<T, BadBits>;

/// Construct `Self` from a small bitwise representation, without assuming that
/// every possible bit pattern can be represented.
///
/// This trait is similar to `core::convert::From`, but less general and can
/// fail.
///
/// This trait is similar to `FromBitsTotal` but allows illegal bit patterns to
/// be processed at runtime.
pub trait FromBits: Sized {
    /// Constructs `Self` from `bits`.  If `bits` is not valid (e.g. is out of
    /// range for an enum) returns `BadBits`.
    fn from_bits(bits: u32) -> BitsResult<Self>;
}

/// Maps 0 to `false` and 1 to `true`.
impl FromBits for bool {
    fn from_bits(bits: u32) -> BitsResult<Self> {
        match bits {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(BadBits(bits)),
        }
    }
}

/// Construct `Self` from a small bitwise representation, panicking if the bits
/// are invalid.
///
/// This trait is similar to `core::convert::From` but less general.
///
/// This trait is similar to `FromBits` but is used when the application is
/// willing to panic if it encounters an illegal bit pattern (e.g. when reading
/// from a byte register into a 256-valued enum, illegal bit patterns are really
/// unlikely).
pub trait FromBitsTotal {
    fn from_bits_total(bits: u32) -> Self;
}

/// Maps 0 to `false` and 1 to `true`.
impl FromBitsTotal for bool {
    fn from_bits_total(bits: u32) -> Self {
        match bits {
            0 => false,
            1 => true,
            _ => unreachable!(),
        }
    }
}

/// Maps 0..255
impl FromBitsTotal for u8 {
    fn from_bits_total(bits: u32) -> Self {
        if bits > core::u8::MAX as u32 {
            unreachable!()
        } else {
            bits as u8
        }
    }
}

/// Identity map.
impl FromBitsTotal for u32 {
    fn from_bits_total(bits: u32) -> Self {
        bits
    }
}

/// Converts `self` into a small bitwise representation.  For small integers and
/// C-like enumerations, this is equivalent to widening casts using `as`.  It
/// should not panic.
pub trait IntoBits {
    fn into_bits(self) -> u32;
}

impl IntoBits for bool {
    fn into_bits(self) -> u32 {
        match self {
            false => 0,
            true => 1,
        }
    }
}

impl IntoBits for u8 {
    fn into_bits(self) -> u32 {
        self as u32
    }
}

impl IntoBits for u32 {
    fn into_bits(self) -> u32 {
        self
    }
}

/// Associates a wrapped bits type (e.g. the typesafe contents of a packed
/// register) with both its underlying `Raw` type, and a function for
/// constructing from that type.
pub trait BitsWrapper {
    /// Underlying bitwise type (often `u32`).
    type Raw;

    /// Constructor from bitwise representation.
    fn from_raw(v: Self::Raw) -> Self;
}

/// Given a value `v`, extracts bits `hi` through `lo` (inclusive).
#[inline(always)]
pub fn bitfield_extract(v: u32, hi: usize, lo: usize) -> u32 {
    let width = hi - lo + 1;
    let mask : u32 = if width < core::mem::size_of::<u32>() * 8 {
        (1 << width) - 1
    } else {
        !0
    };

    (v >> lo) & mask
}

/// Given a value `v`, replaces bits `hi` through `lo` (inclusive) with the
/// same number of low-order bits from `new`.
#[inline(always)]
pub fn bitfield_replace(orig: u32, hi: usize, lo: usize, new: u32) -> u32 {
    let width = hi - lo + 1;
    let mask : u32 = if width < core::mem::size_of::<u32>() * 8 {
        (1 << width) - 1
    } else {
        !0
    };

    (orig & !(mask << lo)) | ((new & mask) << lo)
}
    
/// Declares wrapped bits types.  A wrapped bits type declaration looks like a
/// newtype around an integer:
///
///     pub struct MyType(pub u32);
///
/// This macro automatically derives `Copy`, `Clone`, and `BitsWrapper`.
macro_rules! bit_wrappers {
    () => {};
    ($(#[$m:meta])* pub struct $name:ident(pub $ty:ty); $($rest:tt)*) => {
        #[derive(Copy, Clone)]
        $(#[$m])*
        pub struct $name(pub $ty);

        impl $crate::bits::BitsWrapper for $name {
            type Raw = $ty;

            fn from_raw(v: Self::Raw) -> Self {
                $name(v)
            }
        }

        bit_wrappers!{$($rest)*}
    };
}

/// Declares accessors for packed bitfields.  This macro should be used within
/// an `impl` block for a `BitsWrapper` type (possibly declared using the
/// `bit_wrappers` macro).
///
/// Packed bitfield accessor declarations look like this:
///
///     pub total [31] get_sign / with_sign: bool,
///     pub total [30:15] get_value / with_value: u32,
///     pub       [3:0] get_mode / with_mode: MyMode,
///
/// From left to right:
/// - Access modifier(s) (`pub` is currently required).
/// - Bit range, given as either a single bit index, or high and low indices
///   (inclusive).
/// - Getter name and builder name, separated by a slash.
/// - Rust type.
///
/// The special modifier `total` says that every bit pattern that may appear in
/// that field is valid for the Rust type.  Without `total`, packed field values
/// are returned as `BitsResult<T>` so that invalid values can be handled.  With
/// `total`, they are returned as simply `T`, and invalid values will panic.
/// (Under the hood, `total` uses a `FromBitsTotal` impl, while otherwise
/// `FromBits` is used.)
///
/// The declarations above will produce the following methods:
///
///     pub fn get_sign(self) -> bool { ... }
///     pub fn with_sign(self, v: bool) -> Self { ... }
///
///     pub fn get_value(self) -> u32  { ... }
///     pub fn with_value(self, v: u32) -> Self { ... }
///
///     pub fn get_mode(self) -> BitsResult<Mode> { ... }
///     pub fn with_value(self, v: Mode) -> Self { ... }
macro_rules! bitfield_accessors {
    () => {};

    (
        $(#[$m:meta])*
        pub [$bit:tt] $get:ident / $with:ident : $ty:ty,
        $($rest:tt)*
    ) => {
        bitfield_accessors! {
            @_impl
            $(#[$m])*
            [pub] partial [$bit : $bit] $get / $with : $ty
        }

        bitfield_accessors!{ $($rest)* }
    };

    (
        $(#[$m:meta])*
        pub total [$bit:tt] $get:ident / $with:ident : $ty:ty,
        $($rest:tt)*
    ) => {
        bitfield_accessors! {
            @_impl
            $(#[$m])*
            [pub] total [$bit : $bit] $get / $with : $ty
        }

        bitfield_accessors!{ $($rest)* }
    };

    (
        $(#[$m:meta])*
        pub [$hi:tt : $lo:tt] $get:ident / $with:ident : $ty:ty,
        $($rest:tt)*
    ) => {
        bitfield_accessors! {
            @_impl
            $(#[$m])*
            [pub] partial [$hi : $lo] $get / $with : $ty
        }

        bitfield_accessors!{ $($rest)* }
    };

    (
        $(#[$m:meta])*
        pub total [$hi:tt : $lo:tt] $get:ident / $with:ident : $ty:ty,
        $($rest:tt)*
    ) => {
        bitfield_accessors! {
            @_impl
            $(#[$m])*
            [pub] total [$hi : $lo] $get / $with : $ty
        }

        bitfield_accessors!{ $($rest)* }
    };

    (
        @_impl
        $(#[$m:meta])*
        [$($access:ident)*] partial [$hi:tt : $lo:tt]
        $get:ident / $with:ident : $ty:ty
    ) => {
        $(#[$m])*
        #[inline]
        $($access)* fn $get(self) -> $crate::bits::BitsResult<$ty> {
            <$ty as $crate::bits::FromBits>::from_bits(
                $crate::bits::bitfield_extract(self.0, $hi, $lo))
        }

        $(#[$m])*
        #[inline]
        $($access)* fn $with(self, v: $ty) -> Self {
            $crate::bits::BitsWrapper::from_raw(
                $crate::bits::bitfield_replace(
                    self.0, $hi, $lo,
                    <$ty as $crate::bits::IntoBits>::into_bits(v)))
        }
    };

    (
        @_impl
        $(#[$m:meta])*
        [$($access:ident)*] total [$hi:tt : $lo:tt]
        $get:ident / $with:ident : $ty:ty
    ) => {
        $(#[$m])*
        #[inline]
        $($access)* fn $get(self) -> $ty {
            <$ty as $crate::bits::FromBitsTotal>::from_bits_total(
                $crate::bits::bitfield_extract(self.0, $hi, $lo))
        }

        $(#[$m])*
        #[inline]
        $($access)* fn $with(self, v: $ty) -> Self {
            $crate::bits::BitsWrapper::from_raw(
                $crate::bits::bitfield_replace(
                    self.0, $hi, $lo,
                    <$ty as $crate::bits::IntoBits>::into_bits(v)))
        }
    };
}

/// Declares `bit_enum` types.  These are Rust enums with bidirectional mapping
/// to small bit patterns.
///
/// The declaration should be a simple `enum` with every value given an explicit
/// numeric equivalent, and with the keyword `enum` replaced by `bit_enum`, like
/// so:
///
///     bit_enums! {
///         pub bit_enum Mode {
///             Stun = 0,
///             Coddle = 1,
///             Blanche = 2,
///         }
///     }
///
/// This macro will generate an equivalent `enum` type and automatically derive
/// instances of `Copy`, `Clone`, `Eq`, `PartialEq`, `IntoBits`, `FromBits`, and
/// `FromBitsTotal`.
macro_rules! bit_enums {
    () => {};
    (
        $(#[$m:meta])*
        pub bit_enum $name:ident {
            $($e_name:ident = $e_val:expr,)+
        }
        $($rest:tt)*
    ) => {
        #[derive(Copy, Clone, Eq, PartialEq)]
        $(#[$m])*
        pub enum $name {
            $($e_name = $e_val),+
        }

        impl $crate::bits::IntoBits for $name {
            fn into_bits(self) -> u32 {
                self as u32
            }
        }

        impl $crate::bits::FromBits for $name {
            fn from_bits(bits: u32) -> $crate::bits::BitsResult<Self> {
                match bits {
                    $($e_val => Ok($name::$e_name),)+
                    _ => Err($crate::bits::BadBits(bits)),
                }
            }
        }

        impl $crate::bits::FromBitsTotal for $name {
            fn from_bits_total(bits: u32) -> Self {
                match bits {
                    $($e_val => $name::$e_name,)+
                    _ => unreachable!(),
                }
            }
        }

        bit_enums!{$($rest)*}
    };
}
