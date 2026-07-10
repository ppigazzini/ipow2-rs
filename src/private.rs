use core::ops::{Add, BitAnd, Not, Shl, Shr, Sub};

mod seals {
    pub trait SealedInt {}

    impl SealedInt for u8 {}
    impl SealedInt for u16 {}
    impl SealedInt for u32 {}
    impl SealedInt for u64 {}
    impl SealedInt for u128 {}
    impl SealedInt for usize {}

    impl SealedInt for i8 {}
    impl SealedInt for i16 {}
    impl SealedInt for i32 {}
    impl SealedInt for i64 {}
    impl SealedInt for i128 {}
    impl SealedInt for isize {}
}

/// Internal generics utility trait. While it is publicly exposed for the purpose
/// of specifying generic bounds it is highly discouraged to use it in any other way.
pub trait Int: seals::SealedInt
where
    Self: Sized
        + Copy
        + Clone
        + Eq
        + PartialEq
        + Ord
        + PartialOrd
        + Shr<u8, Output = Self>
        + Shl<u8, Output = Self>
        + Sub<Output = Self>
        + Add<Output = Self>
        + Not<Output = Self>
        + BitAnd<Output = Self>,
{
    type Signed: SignedInt<Signed = Self::Signed, Unsigned = Self::Unsigned>;
    type Unsigned: UnsignedInt<Signed = Self::Signed, Unsigned = Self::Unsigned>;

    const BITS: u32;
    const ZERO: Self;
    const ONE: Self;
    const MINUS_ONE: Self;
    const MIN: Self;
    const MAX: Self;
    const IS_SIGNED: bool;
    const IS_UNSIGNED: bool;
    const SAFE_SHIFT: u32;

    #[must_use]
    fn is_power_of_two(self) -> bool;

    #[must_use]
    fn ilog2(self) -> u32;

    #[must_use]
    fn cast_signed(self) -> Self::Signed;

    #[must_use]
    fn cast_unsigned(self) -> Self::Unsigned;

    #[must_use]
    fn from_bool(b: bool) -> Self;

    #[must_use]
    fn is_zero(self) -> bool;

    #[must_use]
    fn is_not_zero(self) -> bool;

    #[must_use]
    fn checked_shl(self, rhs: u32) -> Option<Self>;

    #[must_use]
    fn checked_add(self, rhs: Self) -> Option<Self>;

    #[must_use]
    fn mask(bits: u32) -> Self;

    #[must_use]
    fn highest_mask_bit(bits: u32) -> Self;

    #[must_use]
    #[allow(clippy::missing_safety_doc)]
    unsafe fn unchecked_mask(bits: u32) -> Self;

    #[must_use]
    #[allow(clippy::missing_safety_doc)]
    unsafe fn unchecked_highest_mask_bit(bits: u32) -> Self;

    #[must_use]
    fn trailing_zeros(self) -> u32;

    #[must_use]
    #[allow(clippy::missing_safety_doc)]
    unsafe fn unchecked_shr(self, rhs: u32) -> Self;

    #[must_use]
    #[allow(clippy::missing_safety_doc)]
    unsafe fn unchecked_shl(self, rhs: u32) -> Self;

    #[must_use]
    fn saturating_sub(self, rhs: Self) -> Self;

    #[must_use]
    fn self_from_signed(rhs: Self::Signed) -> Self;

    #[must_use]
    fn self_from_unsigned(rhs: Self::Unsigned) -> Self;
}

/// Internal generics utility trait. While it is publicly exposed for the purpose
/// of specifying generic bounds it is highly discouraged to use it in any other way.
pub trait UnsignedInt: Int<Unsigned = Self> {}

impl UnsignedInt for u8 {}
impl UnsignedInt for u16 {}
impl UnsignedInt for u32 {}
impl UnsignedInt for u64 {}
impl UnsignedInt for u128 {}
impl UnsignedInt for usize {}

/// Internal generics utility trait. While it is publicly exposed for the purpose
/// of specifying generic bounds it is highly discouraged to use it in any other way.
pub trait SignedInt: Int<Signed = Self> {}

impl SignedInt for i8 {}
impl SignedInt for i16 {}
impl SignedInt for i32 {}
impl SignedInt for i64 {}
impl SignedInt for i128 {}
impl SignedInt for isize {}

macro_rules! impl_common_int {
    ($t:ty) => {
        const BITS: u32 = <$t>::BITS;
        const ZERO: Self = 0;
        const ONE: Self = 1;
        const MIN: Self = <$t>::MIN;
        const MAX: Self = <$t>::MAX;

        #[inline(always)]
        fn ilog2(self) -> u32 {
            self.ilog2()
        }

        #[inline(always)]
        fn from_bool(b: bool) -> Self {
            b as Self
        }

        #[inline(always)]
        fn is_zero(self) -> bool {
            self == 0
        }

        #[inline(always)]
        fn is_not_zero(self) -> bool {
            self != 0
        }

        #[inline(always)]
        fn checked_shl(self, rhs: u32) -> Option<Self> {
            <$t>::checked_shl(self, rhs)
        }

        #[inline(always)]
        fn checked_add(self, rhs: Self) -> Option<Self> {
            <$t>::checked_add(self, rhs)
        }

        #[inline(always)]
        fn trailing_zeros(self) -> u32 {
            self.trailing_zeros()
        }

        #[inline(always)]
        #[allow(clippy::missing_safety_doc)]
        unsafe fn unchecked_shr(self, rhs: u32) -> Self {
            unsafe { self.unchecked_shr(rhs) }
        }

        #[inline(always)]
        #[allow(clippy::missing_safety_doc)]
        unsafe fn unchecked_shl(self, rhs: u32) -> Self {
            unsafe { self.unchecked_shl(rhs) }
        }

        #[inline(always)]
        fn saturating_sub(self, rhs: Self) -> Self {
            self.saturating_sub(rhs)
        }

        #[inline(always)]
        fn self_from_signed(rhs: Self::Signed) -> Self {
            rhs as Self
        }

        #[inline(always)]
        fn self_from_unsigned(rhs: Self::Unsigned) -> Self {
            rhs as Self
        }
    };
}

macro_rules! impl_uint {
    ($t:ty, $t_sint:ty) => {
        impl_common_int!($t);

        const MINUS_ONE: Self = <$t>::MAX;
        const IS_SIGNED: bool = false;
        const IS_UNSIGNED: bool = true;
        const SAFE_SHIFT: u32 = <$t>::BITS - 1;

        #[inline(always)]
        fn is_power_of_two(self) -> bool {
            self.is_power_of_two()
        }

        #[inline(always)]
        fn cast_signed(self) -> $t_sint {
            self.cast_signed()
        }

        #[inline(always)]
        fn cast_unsigned(self) -> $t {
            self
        }

        #[inline(always)]
        fn mask(bits: u32) -> Self {
            debug_assert!(bits <= Self::SAFE_SHIFT);
            ((1 as Self) << bits) - 1
        }

        #[inline(always)]
        fn highest_mask_bit(bits: u32) -> Self {
            debug_assert!(bits <= Self::SAFE_SHIFT);
            ((1 as Self) << bits) >> 1
        }

        #[inline(always)]
        #[allow(clippy::missing_safety_doc)]
        unsafe fn unchecked_mask(bits: u32) -> Self {
            debug_assert!(bits <= Self::SAFE_SHIFT);
            unsafe { (1 as Self).unchecked_shl(bits) - 1 }
        }

        #[inline(always)]
        #[allow(clippy::missing_safety_doc)]
        unsafe fn unchecked_highest_mask_bit(bits: u32) -> Self {
            debug_assert!(bits <= Self::SAFE_SHIFT);
            unsafe { (1 as Self).unchecked_shl(bits).unchecked_shr(1) }
        }
    };
}

macro_rules! impl_sint {
    ($t:ty, $t_uint:ty) => {
        impl_common_int!($t);

        const MINUS_ONE: Self = -1;
        const IS_SIGNED: bool = true;
        const IS_UNSIGNED: bool = false;
        const SAFE_SHIFT: u32 = <$t>::BITS - 2;

        #[inline(always)]
        fn is_power_of_two(self) -> bool {
            self > 0 && self.cast_unsigned().is_power_of_two()
        }

        #[inline(always)]
        fn cast_signed(self) -> $t {
            self
        }

        #[inline(always)]
        fn cast_unsigned(self) -> $t_uint {
            self.cast_unsigned()
        }

        #[inline(always)]
        fn mask(bits: u32) -> Self {
            debug_assert!(bits <= Self::Unsigned::SAFE_SHIFT);
            // The mask computation needs to be done as unsigned, because it can overflow
            // to the sign bit before we subtract one.
            (((1 as $t_uint) << bits) - 1) as $t
        }

        #[inline(always)]
        fn highest_mask_bit(bits: u32) -> Self {
            debug_assert!(bits <= Self::Unsigned::SAFE_SHIFT);
            // The mask computation needs to be done as unsigned, because it can overflow
            // to the sign bit before we shift right.
            (((1 as $t_uint) << bits) >> 1) as $t
        }

        #[inline(always)]
        unsafe fn unchecked_mask(bits: u32) -> Self {
            debug_assert!(bits <= Self::Unsigned::SAFE_SHIFT);
            // The mask computation needs to be done as unsigned, because it can overflow
            // to the sign bit before we subtract one.
            unsafe { ((1 as $t_uint).unchecked_shl(bits) - 1) as $t }
        }

        #[inline(always)]
        unsafe fn unchecked_highest_mask_bit(bits: u32) -> Self {
            debug_assert!(bits <= Self::Unsigned::SAFE_SHIFT);
            // The mask computation needs to be done as unsigned, because it can overflow
            // to the sign bit before we shift right.
            unsafe { (1 as $t_uint).unchecked_shl(bits).unchecked_shr(1) as $t }
        }
    };
}

macro_rules! impl_int {
    ($t_uint:ty, $t_sint:ty) => {
        impl Int for $t_uint {
            type Unsigned = $t_uint;
            type Signed = $t_sint;

            impl_uint!($t_uint, $t_sint);
        }

        impl Int for $t_sint {
            type Unsigned = $t_uint;
            type Signed = $t_sint;

            impl_sint!($t_sint, $t_uint);
        }
    };
}

impl_int!(u8, i8);
impl_int!(u16, i16);
impl_int!(u32, i32);
impl_int!(u64, i64);
impl_int!(u128, i128);
impl_int!(usize, isize);

/// Internal generics utility trait. While it is publicly exposed for the purpose
/// of specifying generic bounds it is highly discouraged to use it in any other way.
pub trait IntAtLeastAsWide<T>: Int
where
    T: UnsignedInt,
{
}

macro_rules! impl_int_at_least_as_wide {
    ($u:ty => [$($t:ty),*]) => {
        $(impl IntAtLeastAsWide<$u> for $t {})*
    };
}

impl_int_at_least_as_wide!(u8 => [i8, u8, i16, u16, i32, u32, i64, u64, i128, u128, isize, usize]);
impl_int_at_least_as_wide!(u16 => [i16, u16, i32, u32, i64, u64, i128, u128, isize, usize]);

#[cfg(target_pointer_width = "16")]
impl_int_at_least_as_wide!(u32 => [i32, u32, i64, u64, i128, u128]);

#[cfg(target_pointer_width = "32")]
impl_int_at_least_as_wide!(u32 => [i32, u32, i64, u64, i128, u128, isize, usize]);

#[cfg(target_pointer_width = "64")]
impl_int_at_least_as_wide!(u32 => [i32, u32, i64, u64, i128, u128, isize, usize]);

#[cfg(target_pointer_width = "16")]
impl_int_at_least_as_wide!(u64 => [i64, u64, i128, u128]);

#[cfg(target_pointer_width = "32")]
impl_int_at_least_as_wide!(u64 => [i64, u64, i128, u128]);

#[cfg(target_pointer_width = "64")]
impl_int_at_least_as_wide!(u64 => [i64, u64, i128, u128, isize, usize]);

impl_int_at_least_as_wide!(u128 => [i128, u128]);

macro_rules! impl_trait_signed_unsigned {
    ($trait:ty, signed_body $signed_body:tt, unsigned_body $unsigned_body:tt) => {
        impl $trait for u8  $unsigned_body
        impl $trait for u16 $unsigned_body
        impl $trait for u32 $unsigned_body
        impl $trait for u64 $unsigned_body
        impl $trait for u128 $unsigned_body
        impl $trait for usize $unsigned_body

        impl $trait for i8  $signed_body
        impl $trait for i16 $signed_body
        impl $trait for i32 $signed_body
        impl $trait for i64 $signed_body
        impl $trait for i128 $signed_body
        impl $trait for isize $signed_body
    };
}

macro_rules! impl_generic_trait_signed_unsigned {
    (<$($gen_t:ident),*> $trait:ty where { $($wc:tt)+ }, signed_body $signed_body:tt, unsigned_body $unsigned_body:tt) => {
        impl<$($gen_t),*> $trait for u8 where $($wc)+ $unsigned_body
        impl<$($gen_t),*> $trait for u16 where $($wc)+ $unsigned_body
        impl<$($gen_t),*> $trait for u32 where $($wc)+ $unsigned_body
        impl<$($gen_t),*> $trait for u64 where $($wc)+ $unsigned_body
        impl<$($gen_t),*> $trait for u128 where $($wc)+ $unsigned_body
        impl<$($gen_t),*> $trait for usize where $($wc)+ $unsigned_body

        impl<$($gen_t),*> $trait for i8 where $($wc)+ $signed_body
        impl<$($gen_t),*> $trait for i16 where $($wc)+ $signed_body
        impl<$($gen_t),*> $trait for i32 where $($wc)+ $signed_body
        impl<$($gen_t),*> $trait for i64 where $($wc)+ $signed_body
        impl<$($gen_t),*> $trait for i128 where $($wc)+ $signed_body
        impl<$($gen_t),*> $trait for isize where $($wc)+ $signed_body
    };
}

macro_rules! impl_trait_all_ints {
    ($trait:ty => $body:tt) => {
        impl_trait_signed_unsigned!($trait, signed_body $body, unsigned_body $body);
    };
}

macro_rules! impl_generic_trait_all_ints {
    (<$($gen_t:ident),*> $trait:ty where { $($wc:tt)+ } => $body:tt) => {
        impl_generic_trait_signed_unsigned!(<$($gen_t),*> $trait where { $($wc)+ }, signed_body $body, unsigned_body $body);
    };
}
