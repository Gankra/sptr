//! Pointers Pretending To Be Integers For Crimes -- [uptr][] and [iptr][].

#![allow(unstable_name_collisions)]
use crate::Strict;

/// A pointer that pretends to be an integer, for API Crimes.
///
/// **Please don't use this type.**
///
/// If you can't possibly satisfy strict provenance for whatever reason, you can at least
/// use this type to make sure the compiler still understands that Pointers Are Happening.
///
/// All operations on this type will derive provenance from the left-hand-size (lhs).
/// So `x + y` has `x`'s provenance. *Many* operations are nonsensical if the pointer
/// inside is a real pointer, but hey, you've reached for the "I Know What I'm Doing"
/// lever, so we'll let you *say* whatever gibberish you want.
///
/// Please submit a PR if you need some operation defined on usize to be exposed here.

#[repr(transparent)]
#[allow(non_camel_case_types)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct uptr(*mut ());

/// A pointer that pretends to be an integer, for API Crimes.
///
/// **Please don't use this type.**
///
/// If you can't possibly satisfy strict provenance for whatever reason, you can at least
/// use this type to make sure the compiler still understands that Pointers Are Happening.
///
/// All operations on this type will derive provenance from the left-hand-size (lhs).
/// So `x + y` has `x`'s provenance. *Many* operations are nonsensical if the pointer
/// inside is a real pointer, but hey, you've reached for the "I Know What I'm Doing"
/// lever, so we'll let you *say* whatever gibberish you want.
///
/// Please submit a PR if you need some operation defined on isize to be exposed here.
#[repr(transparent)]
#[allow(non_camel_case_types)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct iptr(*mut ());

macro_rules! int_impls {
    ($self_ty: ident, $int_ty: ident) => {
        impl $self_ty {
            // Inherent MIN/MAX requires 1.43
            // pub const MIN: $self_ty = Self::from_int(<$int_ty>::MIN);
            // pub const MAX: $self_ty = Self::from_int(<$int_ty>::MAX);
            pub const MIN: $self_ty = Self::from_int(core::$int_ty::MIN);
            pub const MAX: $self_ty = Self::from_int(core::$int_ty::MAX);

            // Inherent BITS requires 1.53
            // pub const BITS: u32 = <$int_ty>::BITS;
            pub const BITS: u32 = core::mem::size_of::<$int_ty>() as u32 * 8;

            #[inline]
            #[must_use]
            pub const fn from_int(val: $int_ty) -> Self {
                $self_ty(crate::invalid_mut(val as usize))
            }

            #[inline]
            #[must_use]
            pub const fn from_ptr_mut<T>(val: *mut T) -> Self {
                $self_ty(val as *mut ())
            }

            #[inline]
            #[must_use]
            pub const fn from_ptr<T>(val: *const T) -> Self {
                $self_ty(val as *const () as *mut ())
            }

            pub const fn to_ptr(self) -> *mut () {
                self.0
            }

            #[inline]
            #[must_use]
            pub fn wrapping_add(self, rhs: Self) -> Self {
                $self_ty(
                    self.0.map_addr(|a| {
                        ((a as $int_ty).wrapping_add(rhs.0.addr() as $int_ty)) as usize
                    }),
                )
            }
            #[inline]
            #[must_use]
            pub fn wrapping_sub(self, rhs: Self) -> Self {
                $self_ty(
                    self.0.map_addr(|a| {
                        ((a as $int_ty).wrapping_sub(rhs.0.addr() as $int_ty)) as usize
                    }),
                )
            }
            #[inline]
            #[must_use]
            pub fn wrapping_mul(self, rhs: Self) -> Self {
                $self_ty(
                    self.0.map_addr(|a| {
                        ((a as $int_ty).wrapping_mul(rhs.0.addr() as $int_ty)) as usize
                    }),
                )
            }
            #[inline]
            #[must_use]
            pub fn wrapping_div(self, rhs: Self) -> Self {
                $self_ty(
                    self.0.map_addr(|a| {
                        ((a as $int_ty).wrapping_div(rhs.0.addr() as $int_ty)) as usize
                    }),
                )
            }
        }

        impl From<$int_ty> for $self_ty {
            #[inline]
            #[must_use]
            fn from(val: $int_ty) -> Self {
                $self_ty(crate::invalid_mut(val as usize))
            }
        }
        impl<T> From<*mut T> for $self_ty {
            #[inline]
            #[must_use]
            fn from(val: *mut T) -> Self {
                $self_ty(val as *mut ())
            }
        }
        impl<T> From<*const T> for $self_ty {
            #[inline]
            #[must_use]
            fn from(val: *const T) -> Self {
                $self_ty(val as *const () as *mut ())
            }
        }

        impl core::ops::Add<Self> for $self_ty {
            type Output = Self;
            #[inline]
            #[must_use]
            fn add(self, rhs: Self) -> Self::Output {
                $self_ty(
                    self.0
                        .map_addr(|a| ((a as $int_ty) + (rhs.0.addr() as $int_ty)) as usize),
                )
            }
        }
        impl core::ops::Sub<Self> for $self_ty {
            type Output = Self;
            #[inline]
            #[must_use]
            fn sub(self, rhs: Self) -> Self::Output {
                $self_ty(
                    self.0
                        .map_addr(|a| ((a as $int_ty) - (rhs.0.addr() as $int_ty)) as usize),
                )
            }
        }
        impl core::ops::Mul<Self> for $self_ty {
            type Output = Self;
            #[inline]
            #[must_use]
            fn mul(self, rhs: Self) -> Self::Output {
                $self_ty(
                    self.0
                        .map_addr(|a| ((a as $int_ty) * (rhs.0.addr() as $int_ty)) as usize),
                )
            }
        }
        impl core::ops::Div<Self> for $self_ty {
            type Output = Self;
            #[inline]
            #[must_use]
            fn div(self, rhs: Self) -> Self::Output {
                $self_ty(
                    self.0
                        .map_addr(|a| ((a as $int_ty) / (rhs.0.addr() as $int_ty)) as usize),
                )
            }
        }
        impl core::ops::Rem<Self> for $self_ty {
            type Output = Self;
            #[inline]
            #[must_use]
            fn rem(self, rhs: Self) -> Self::Output {
                $self_ty(
                    self.0
                        .map_addr(|a| ((a as $int_ty) % (rhs.0.addr() as $int_ty)) as usize),
                )
            }
        }
        impl core::ops::BitAnd<Self> for $self_ty {
            type Output = Self;
            #[inline]
            #[must_use]
            fn bitand(self, rhs: Self) -> Self::Output {
                $self_ty(
                    self.0
                        .map_addr(|a| ((a as $int_ty) & (rhs.0.addr() as $int_ty)) as usize),
                )
            }
        }
        impl core::ops::BitOr<Self> for $self_ty {
            type Output = Self;
            #[inline]
            #[must_use]
            fn bitor(self, rhs: Self) -> Self::Output {
                $self_ty(
                    self.0
                        .map_addr(|a| ((a as $int_ty) | (rhs.0.addr() as $int_ty)) as usize),
                )
            }
        }
        impl core::ops::BitXor<Self> for $self_ty {
            type Output = Self;
            #[inline]
            #[must_use]
            fn bitxor(self, rhs: Self) -> Self::Output {
                $self_ty(
                    self.0
                        .map_addr(|a| ((a as $int_ty) ^ (rhs.0.addr() as $int_ty)) as usize),
                )
            }
        }
        impl core::ops::Shl<usize> for $self_ty {
            type Output = Self;
            #[inline]
            #[must_use]
            fn shl(self, rhs: usize) -> Self::Output {
                $self_ty(self.0.map_addr(|a| ((a as $int_ty) << rhs) as usize))
            }
        }
        impl core::ops::Shr<usize> for $self_ty {
            type Output = Self;
            #[inline]
            #[must_use]
            fn shr(self, rhs: usize) -> Self::Output {
                $self_ty(self.0.map_addr(|a| ((a as $int_ty) >> rhs) as usize))
            }
        }

        impl core::ops::Not for $self_ty {
            type Output = Self;
            #[inline]
            #[must_use]
            fn not(self) -> Self::Output {
                $self_ty(self.0.map_addr(|a| (!(a as $int_ty)) as usize))
            }
        }

        impl core::ops::AddAssign<Self> for $self_ty {
            #[inline]
            #[must_use]
            fn add_assign(&mut self, rhs: Self) {
                self.0 = self
                    .0
                    .map_addr(|a| ((a as $int_ty) + (rhs.0.addr() as $int_ty)) as usize);
            }
        }
        impl core::ops::SubAssign<Self> for $self_ty {
            #[inline]
            #[must_use]
            fn sub_assign(&mut self, rhs: Self) {
                self.0 = self
                    .0
                    .map_addr(|a| ((a as $int_ty) - (rhs.0.addr() as $int_ty)) as usize);
            }
        }
        impl core::ops::MulAssign<Self> for $self_ty {
            #[inline]
            #[must_use]
            fn mul_assign(&mut self, rhs: Self) {
                self.0 = self
                    .0
                    .map_addr(|a| ((a as $int_ty) * (rhs.0.addr() as $int_ty)) as usize);
            }
        }
        impl core::ops::DivAssign<Self> for $self_ty {
            #[inline]
            #[must_use]
            fn div_assign(&mut self, rhs: Self) {
                self.0 = self
                    .0
                    .map_addr(|a| ((a as $int_ty) / (rhs.0.addr() as $int_ty)) as usize);
            }
        }
        impl core::ops::RemAssign<Self> for $self_ty {
            #[inline]
            #[must_use]
            fn rem_assign(&mut self, rhs: Self) {
                self.0 = self
                    .0
                    .map_addr(|a| ((a as $int_ty) % (rhs.0.addr() as $int_ty)) as usize);
            }
        }
        impl core::ops::BitAndAssign<Self> for $self_ty {
            #[inline]
            #[must_use]
            fn bitand_assign(&mut self, rhs: Self) {
                self.0 = self
                    .0
                    .map_addr(|a| ((a as $int_ty) & (rhs.0.addr() as $int_ty)) as usize);
            }
        }
        impl core::ops::BitOrAssign<Self> for $self_ty {
            #[inline]
            #[must_use]
            fn bitor_assign(&mut self, rhs: Self) {
                self.0 = self
                    .0
                    .map_addr(|a| ((a as $int_ty) | (rhs.0.addr() as $int_ty)) as usize);
            }
        }
        impl core::ops::BitXorAssign<Self> for $self_ty {
            #[inline]
            #[must_use]
            fn bitxor_assign(&mut self, rhs: Self) {
                self.0 = self
                    .0
                    .map_addr(|a| ((a as $int_ty) ^ (rhs.0.addr() as $int_ty)) as usize);
            }
        }
        impl core::ops::ShlAssign<usize> for $self_ty {
            #[inline]
            #[must_use]
            fn shl_assign(&mut self, rhs: usize) {
                self.0 = self.0.map_addr(|a| ((a as $int_ty) << rhs) as usize);
            }
        }
        impl core::ops::ShrAssign<usize> for $self_ty {
            #[inline]
            #[must_use]
            fn shr_assign(&mut self, rhs: usize) {
                self.0 = self.0.map_addr(|a| ((a as $int_ty) >> rhs) as usize);
            }
        }

        impl core::fmt::Display for $self_ty {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{}", self.0.addr() as $int_ty)
            }
        }

        impl core::fmt::Debug for $self_ty {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{:?}", self.0.addr() as $int_ty)
            }
        }
    };
}

int_impls!(uptr, usize);
int_impls!(iptr, isize);

// usize can't be negated
impl core::ops::Neg for iptr {
    type Output = Self;
    #[inline]
    #[must_use]
    fn neg(self) -> Self::Output {
        iptr(self.0.map_addr(|a| (-(a as isize)) as usize))
    }
}
