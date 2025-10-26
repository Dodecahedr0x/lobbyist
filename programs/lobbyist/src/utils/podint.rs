#[macro_export]
macro_rules! impl_podint {
    ($name:ident, $replacement:ty, $len:expr) => {
        #[derive(
            Clone, Copy, Debug, Default, PartialEq, ::bytemuck::AnyBitPattern, ::bytemuck::NoUninit,
        )]
        #[repr(transparent)]
        pub struct $name(pub [u8; $len]);

        impl $name {
            pub const fn from_primitive(n: $replacement) -> Self {
                Self(n.to_le_bytes())
            }
        }

        impl From<$replacement> for $name {
            fn from(n: $replacement) -> Self {
                Self::from_primitive(n)
            }
        }

        impl From<$name> for $replacement {
            fn from(pod: $name) -> Self {
                <$replacement>::from_le_bytes(pod.0)
            }
        }

        impl std::ops::Add for $name {
            type Output = $name;
            fn add(self, rhs: $name) -> $name {
                Self::from(
                    <$replacement>::from_le_bytes(self.0)
                        + <$name as Into<$replacement>>::into(rhs),
                )
            }
        }

        impl std::ops::Sub for $name {
            type Output = $name;
            fn sub(self, rhs: $name) -> $name {
                Self::from(
                    <$replacement>::from_le_bytes(self.0)
                        - <$name as Into<$replacement>>::into(rhs),
                )
            }
        }

        impl std::ops::AddAssign<$name> for $name {
            fn add_assign(&mut self, rhs: $name) {
                *self = *self + rhs;
            }
        }

        impl std::ops::SubAssign<$name> for $name {
            fn sub_assign(&mut self, rhs: $name) {
                *self = *self - rhs;
            }
        }

        impl std::ops::AddAssign<$replacement> for $name {
            fn add_assign(&mut self, rhs: $replacement) {
                *self = *self + <$replacement as Into<$name>>::into(rhs);
            }
        }

        impl std::ops::SubAssign<$replacement> for $name {
            fn sub_assign(&mut self, rhs: $replacement) {
                *self = *self - rhs.into();
            }
        }

        impl std::ops::AddAssign<$name> for $replacement {
            fn add_assign(&mut self, rhs: $name) {
                *self = *self + <$name as Into<$replacement>>::into(rhs);
            }
        }

        impl std::ops::SubAssign<$name> for $replacement {
            fn sub_assign(&mut self, rhs: $name) {
                *self = *self - <$name as Into<$replacement>>::into(rhs);
            }
        }
    };
}

impl_podint!(PodI16, i16, 2);
impl_podint!(PodI32, i32, 4);
impl_podint!(PodI64, i64, 8);

impl_podint!(PodU16, u16, 2);
impl_podint!(PodU32, u32, 4);
impl_podint!(PodU64, u64, 8);
