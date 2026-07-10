//! Property (law) tests for the arithmetic operations.
//!
//! Where `reference_oracle.rs` checks each result against an absolute
//! wide-integer model (exhaustive for the small types), this file checks the
//! algebraic *laws* that must hold for every value and exponent regardless of
//! width. Because the laws are relational they need no wider type, so they
//! extend meaningful coverage to `u128`/`i128`, which the oracle can only
//! sample.

use ipow2::{
    Pow2, UnboundedPow2, ceil_to_multiple, checked_ceil_to_multiple, div_ceil, div_floor,
    div_round, floor_to_multiple, is_multiple_of, rem_floor,
};
use proptest::prelude::*;

macro_rules! law_suite {
    ($modname:ident, $t:ty, $u:ty, $strategy:expr) => {
        mod $modname {
            use super::*;

            proptest! {
                #![proptest_config(ProptestConfig::with_cases(4096))]

                /// floor_to_multiple + rem_floor reconstructs the input, and the
                /// remainder lies in [0, 2^e).
                #[test]
                fn floor_reconstruction(x in $strategy, e in 0u8..<$t>::BITS as u8) {
                    let p = Pow2::<$u>::from_exponent(e).unwrap();
                    let lo = floor_to_multiple(x, p);
                    let r = rem_floor(x, p);
                    prop_assert_eq!(lo + r, x);
                    // 0 <= r < 2^e: reinterpreting as u128 turns any (buggy)
                    // negative remainder into a huge value that fails the bound.
                    prop_assert!((r as u128) < (1u128 << e));
                }

                /// is_multiple_of iff the floor remainder is zero.
                #[test]
                fn multiple_iff_zero_remainder(x in $strategy, e in 0u8..<$t>::BITS as u8) {
                    let p = Pow2::<$u>::from_exponent(e).unwrap();
                    prop_assert_eq!(is_multiple_of(x, p), rem_floor(x, p) == 0 as $t);
                }

                /// The floored quotient, scaled back up, is the floored multiple.
                #[test]
                fn quotient_scales_to_multiple(x in $strategy, e in 0u8..<$t>::BITS as u8) {
                    let p = Pow2::<$u>::from_exponent(e).unwrap();
                    prop_assert_eq!(floor_to_multiple(x, p), div_floor(x, p) << e);
                }

                /// floor <= round <= ceil, and ceil - floor is 0 or 1.
                #[test]
                fn rounding_is_ordered(x in $strategy, e in 0u8..<$t>::BITS as u8) {
                    let p = Pow2::<$u>::from_exponent(e).unwrap();
                    let f = div_floor(x, p);
                    let c = div_ceil(x, p);
                    let r = div_round(x, p);
                    prop_assert!(f <= r && r <= c);
                    prop_assert!(c - f == 0 as $t || c - f == 1 as $t);
                }

                /// A `Pow2<T>` and a same-exponent (safe) `UnboundedPow2` agree.
                #[test]
                fn pow2_agrees_with_unbounded(x in $strategy, e in 0u8..<$t>::BITS as u8) {
                    let p = Pow2::<$u>::from_exponent(e).unwrap();
                    let u = UnboundedPow2::from_exponent(e);
                    prop_assert_eq!(div_floor(x, p), div_floor(x, u));
                    prop_assert_eq!(rem_floor(x, p), rem_floor(x, u));
                    prop_assert_eq!(div_ceil(x, p), div_ceil(x, u));
                    prop_assert_eq!(div_round(x, p), div_round(x, u));
                    prop_assert_eq!(floor_to_multiple(x, p), floor_to_multiple(x, u));
                    prop_assert_eq!(is_multiple_of(x, p), is_multiple_of(x, u));
                }

                /// checked_ceil_to_multiple agrees with the panicking version
                /// exactly when it returns Some.
                #[test]
                fn checked_matches_normal_when_some(x in $strategy, e in 0u8..<$t>::BITS as u8) {
                    let p = Pow2::<$u>::from_exponent(e).unwrap();
                    if let Some(v) = checked_ceil_to_multiple(x, p) {
                        prop_assert_eq!(v, ceil_to_multiple(x, p));
                    }
                }
            }
        }
    };
}

law_suite!(u64_laws, u64, u64, any::<u64>());
law_suite!(u128_laws, u128, u128, any::<u128>());
law_suite!(i64_laws, i64, u64, any::<i64>());
law_suite!(i128_laws, i128, u128, any::<i128>());
