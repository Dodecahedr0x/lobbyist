//! A wrapper around `u64` that is serialized as a 8-byte little-endian integer.
//!
//! This is useful for storing `u64` values in a program's data accounts.

use bytemuck::{AnyBitPattern, NoUninit};

#[derive(Clone, Copy, Debug, Default, PartialEq, AnyBitPattern, NoUninit)]
#[repr(transparent)]
pub struct PodBool8(pub u8);

impl PodBool8 {
    pub const fn from_primitive(n: bool) -> Self {
        Self(n as u8)
    }
}

impl From<PodBool8> for bool {
    fn from(podbool8: PodBool8) -> Self {
        podbool8.0 != 0
    }
}

impl From<bool> for PodBool8 {
    fn from(n: bool) -> Self {
        Self::from_primitive(n)
    }
}

impl From<PodBool16> for PodBool8 {
    fn from(podbool16: PodBool16) -> Self {
        Self(podbool16.0 as u8)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, AnyBitPattern, NoUninit)]
#[repr(transparent)]
pub struct PodBool16(pub u16);

impl PodBool16 {
    pub const fn from_primitive(n: bool) -> Self {
        Self(n as u16)
    }
}

impl From<bool> for PodBool16 {
    fn from(n: bool) -> Self {
        Self::from_primitive(n)
    }
}

impl From<PodBool16> for bool {
    fn from(podbool16: PodBool16) -> Self {
        podbool16.0 != 0
    }
}
