macro_rules! impl_address {
    ($T:ident, $V:ty) => {
        const _: fn($V) -> $T = $T::from_bits;
        const _: fn($T) -> $V = $T::into_bits;

        impl $crate::arch::traits::TargetAddress for $T {
            fn byte_offset(self, count: isize) -> Self {
                if count < 0 {
                    self.byte_sub(count as usize)
                } else {
                    self.byte_add(count as usize)
                }
            }

            fn byte_add(self, count: usize) -> Self {
                Self::from_bits(self.into_bits() + count as $V)
            }

            fn byte_sub(self, count: usize) -> Self {
                Self::from_bits(self.into_bits() - count as $V)
            }

            fn byte_offset_from(&self, origin: Self) -> isize {
                self.into_bits().wrapping_sub(origin.into_bits()) as isize
            }

            fn byte_offset_from_unsigned(self, origin: Self) -> usize {
                self.into_bits().wrapping_sub(origin.into_bits()) as usize
            }
        }

        impl $T {
            /// Convenience method for checking if an address is null.
            #[inline]
            pub const fn is_null(self) -> bool {
                self.into_bits() == 0
            }

            /// Creates an address that points to `0`.
            #[inline]
            pub const fn zero() -> $T {
                $T::from_bits(0)
            }
        }

        impl PartialEq for $T {
            fn eq(&self, other: &Self) -> bool {
                self.into_bits() == other.into_bits()
            }
        }

        impl Eq for $T {}

        impl PartialOrd for $T {
            fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
                self.into_bits().partial_cmp(&other.into_bits())
            }
        }

        impl Ord for $T {
            fn cmp(&self, other: &Self) -> core::cmp::Ordering {
                self.into_bits().cmp(&other.into_bits())
            }
        }

        impl TryFrom<usize> for $T {
            type Error = core::num::TryFromIntError;

            fn try_from(value: usize) -> Result<Self, Self::Error> {
                let inner = value.try_into()?;
                Ok($T::from_bits(inner))
            }
        }

        impl TryInto<usize> for $T {
            type Error = core::num::TryFromIntError;

            fn try_into(self) -> Result<usize, Self::Error> {
                let outer = self.into_bits().try_into()?;
                Ok(outer)
            }
        }

        impl core::fmt::Binary for $T {
            #[inline]
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                core::fmt::Binary::fmt(&self.into_bits(), f)
            }
        }

        impl core::fmt::LowerHex for $T {
            #[inline]
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                core::fmt::LowerHex::fmt(&self.into_bits(), f)
            }
        }

        impl core::fmt::Octal for $T {
            #[inline]
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                core::fmt::Octal::fmt(&self.into_bits(), f)
            }
        }

        impl core::fmt::UpperHex for $T {
            #[inline]
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                core::fmt::UpperHex::fmt(&self.into_bits(), f)
            }
        }

        impl core::fmt::Pointer for $T {
            #[inline]
            fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                core::fmt::Pointer::fmt(&(self.into_bits() as *const ()), f)
            }
        }
    };
}

pub(crate) use impl_address;
