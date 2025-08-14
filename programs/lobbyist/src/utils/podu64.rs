//! A wrapper around `u64` that is serialized as a 8-byte little-endian integer.
//!
//! This is useful for storing `u64` values in a program's data accounts.

use {
    bytemuck::{AnyBitPattern, NoUninit},
    std::ops::{AddAssign, SubAssign},
};

#[derive(Clone, Copy, Debug, Default, PartialEq, AnyBitPattern, NoUninit)]
#[repr(transparent)]
pub struct PodU64(pub [u8; 8]);

impl PodU64 {
    pub const fn from_primitive(n: u64) -> Self {
        Self(n.to_le_bytes())
    }
}
impl From<u64> for PodU64 {
    fn from(n: u64) -> Self {
        Self::from_primitive(n)
    }
}
impl From<PodU64> for u64 {
    fn from(pod: PodU64) -> Self {
        Self::from_le_bytes(pod.0)
    }
}

impl AddAssign<PodU64> for u64 {
    fn add_assign(&mut self, rhs: PodU64) {
        *self = *self + <PodU64 as Into<u64>>::into(rhs);
    }
}

impl SubAssign<PodU64> for u64 {
    fn sub_assign(&mut self, rhs: PodU64) {
        *self = *self - <PodU64 as Into<u64>>::into(rhs);
    }
}
