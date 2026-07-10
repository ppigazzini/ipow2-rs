//! Coverage for the publicly re-exported internal `Int` trait.
//!
//! `Int`'s methods forward to the primitives' inherent methods of the same
//! name. At ordinary call sites (`x.saturating_sub(y)`) the inherent method
//! shadows the trait one, so the trait bodies are only reachable through
//! fully-qualified syntax (`<T as Int>::saturating_sub(x, y)`) — which is what
//! this test uses to exercise every method on every supported width.

use ipow2::Int;

macro_rules! check_common {
    ($t:ty, $signed:ty, $unsigned:ty) => {{
        assert_eq!(<$t as Int>::ilog2(8 as $t), 3);
        assert_eq!(<$t as Int>::from_bool(true), 1 as $t);
        assert_eq!(<$t as Int>::from_bool(false), 0 as $t);

        assert!(<$t as Int>::is_zero(0 as $t));
        assert!(!<$t as Int>::is_zero(1 as $t));
        assert!(<$t as Int>::is_not_zero(1 as $t));
        assert!(!<$t as Int>::is_not_zero(0 as $t));

        assert_eq!(<$t as Int>::checked_shl(1 as $t, 1), Some(2 as $t));
        assert_eq!(<$t as Int>::checked_shl(1 as $t, <$t as Int>::BITS), None);
        assert_eq!(<$t as Int>::checked_add(1 as $t, 1 as $t), Some(2 as $t));
        assert_eq!(<$t as Int>::checked_add(<$t as Int>::MAX, 1 as $t), None);

        assert_eq!(<$t as Int>::trailing_zeros(8 as $t), 3);
        assert_eq!(
            <$t as Int>::saturating_sub(<$t as Int>::MIN, 1 as $t),
            <$t as Int>::MIN
        );

        // SAFETY: shift amounts are well below the bit width.
        assert_eq!(unsafe { <$t as Int>::unchecked_shl(1 as $t, 2) }, 4 as $t);
        // SAFETY: shift amounts are well below the bit width.
        assert_eq!(unsafe { <$t as Int>::unchecked_shr(8 as $t, 2) }, 2 as $t);

        assert_eq!(<$t as Int>::self_from_signed(1 as $signed), 1 as $t);
        assert_eq!(<$t as Int>::self_from_unsigned(1 as $unsigned), 1 as $t);
        assert_eq!(<$t as Int>::cast_signed(1 as $t), 1 as $signed);
        assert_eq!(<$t as Int>::cast_unsigned(1 as $t), 1 as $unsigned);

        assert_eq!(<$t as Int>::BITS, <$t>::BITS);
        assert_eq!(<$t as Int>::ZERO, 0 as $t);
        assert_eq!(<$t as Int>::ONE, 1 as $t);
        assert_eq!(<$t as Int>::MIN, <$t>::MIN);
        assert_eq!(<$t as Int>::MAX, <$t>::MAX);
    }};
}

macro_rules! check_unsigned {
    ($t:ty, $s:ty) => {{
        check_common!($t, $s, $t);

        assert!(<$t as Int>::is_power_of_two(4 as $t));
        assert!(!<$t as Int>::is_power_of_two(3 as $t));

        assert_eq!(<$t as Int>::mask(3), 7 as $t);
        assert_eq!(<$t as Int>::highest_mask_bit(3), 4 as $t);
        // SAFETY: 3 <= SAFE_SHIFT for every supported width.
        assert_eq!(unsafe { <$t as Int>::unchecked_mask(3) }, 7 as $t);
        // SAFETY: 3 <= SAFE_SHIFT for every supported width.
        assert_eq!(
            unsafe { <$t as Int>::unchecked_highest_mask_bit(3) },
            4 as $t
        );

        assert!(!<$t as Int>::IS_SIGNED);
        assert!(<$t as Int>::IS_UNSIGNED);
        assert_eq!(<$t as Int>::MINUS_ONE, <$t>::MAX);
        assert_eq!(<$t as Int>::SAFE_SHIFT, <$t>::BITS - 1);

        // Reinterpretation: the all-ones signed value maps to unsigned MAX.
        assert_eq!(<$t as Int>::self_from_signed(-1 as $s), <$t>::MAX);
        assert_eq!(<$t as Int>::cast_signed(<$t>::MAX), -1 as $s);
    }};
}

macro_rules! check_signed {
    ($t:ty, $u:ty) => {{
        check_common!($t, $t, $u);

        assert!(<$t as Int>::is_power_of_two(4 as $t));
        assert!(!<$t as Int>::is_power_of_two(-4 as $t));
        assert!(!<$t as Int>::is_power_of_two(0 as $t));

        assert_eq!(<$t as Int>::mask(3), 7 as $t);
        assert_eq!(<$t as Int>::highest_mask_bit(3), 4 as $t);
        // SAFETY: 3 <= SAFE_SHIFT for every supported width.
        assert_eq!(unsafe { <$t as Int>::unchecked_mask(3) }, 7 as $t);
        // SAFETY: 3 <= SAFE_SHIFT for every supported width.
        assert_eq!(
            unsafe { <$t as Int>::unchecked_highest_mask_bit(3) },
            4 as $t
        );

        assert!(<$t as Int>::IS_SIGNED);
        assert!(!<$t as Int>::IS_UNSIGNED);
        assert_eq!(<$t as Int>::MINUS_ONE, -1 as $t);
        assert_eq!(<$t as Int>::SAFE_SHIFT, <$t>::BITS - 2);

        // Reinterpretation: unsigned MAX maps to the all-ones signed value (-1).
        assert_eq!(<$t as Int>::self_from_unsigned(<$u>::MAX), -1 as $t);
        assert_eq!(<$t as Int>::cast_unsigned(-1 as $t), <$u>::MAX);
    }};
}

#[test]
fn unsigned_trait_methods() {
    check_unsigned!(u8, i8);
    check_unsigned!(u16, i16);
    check_unsigned!(u32, i32);
    check_unsigned!(u64, i64);
    check_unsigned!(u128, i128);
    check_unsigned!(usize, isize);
}

#[test]
fn signed_trait_methods() {
    check_signed!(i8, u8);
    check_signed!(i16, u16);
    check_signed!(i32, u32);
    check_signed!(i64, u64);
    check_signed!(i128, u128);
    check_signed!(isize, usize);
}
