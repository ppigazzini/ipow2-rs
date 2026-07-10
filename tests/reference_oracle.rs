//! Reference-oracle tests for every public arithmetic operation.
//!
//! Each operation is checked against an independent mathematical model computed
//! in a wider integer type (`i128`/`u128`), for every value and every exponent.
//! `u8`/`i8`/`u16`/`i16` are covered exhaustively; wider types are sampled over
//! edge and stride values. Both the `Pow2<T>` and `UnboundedPow2` right-hand
//! sides are exercised, together with the `/`, `%`, `*` operators and their
//! assigning forms, plus every `checked_*` and `unbounded_*` variant.
//!
//! The oracle is deliberately expressed via infinite-precision arithmetic and a
//! representability predicate, so it never copies the crate's branch logic.

use ipow2::{
    Pow2, UnboundedPow2, ceil_to_multiple, checked_ceil_to_multiple, checked_div, checked_div_ceil,
    checked_div_floor, checked_div_round, checked_floor_to_multiple, checked_mul, checked_rem,
    checked_rem_floor, checked_round_to_multiple, div_ceil, div_floor, div_round,
    floor_to_multiple, is_multiple_of, rem_floor, round_to_multiple, unbounded_ceil_to_multiple,
    unbounded_div, unbounded_div_ceil, unbounded_div_floor, unbounded_div_round,
    unbounded_floor_to_multiple, unbounded_is_multiple_of, unbounded_rem,
    unbounded_round_to_multiple,
};

macro_rules! unsigned_suite {
    ($t:ty, $u:ty) => {
        type T = $t;
        const BITS: u32 = <$t>::BITS;

        /// Highest exponent probed, a few past `BITS` to cover the unbounded /
        /// out-of-range paths of `UnboundedPow2`.
        const EMAX: u32 = BITS + 3;

        fn check(x: T, e: u32) {
            let xw = x as u128;
            let dw = 1u128 << e;
            let safe = e < BITS;
            let max = <$t>::MAX as u128;
            let unb = UnboundedPow2::from_exponent(e as u8);

            let floor = xw / dw;
            let rem = xw % dw;
            let ceil = floor + (rem != 0) as u128;
            let round = {
                let q = xw / dw;
                let r = xw % dw;
                if r * 2 >= dw { q + 1 } else { q }
            };
            let is_mult = rem == 0;
            let floor_m = floor * dw;
            let ceil_m = ceil * dw;
            let round_m = round * dw;

            // ---- checked_* : valid at every exponent, `None` when the
            // exponent is unsafe for the LHS width or the result overflows.
            assert_eq!(
                checked_div(x, unb),
                safe.then_some(floor as T),
                "checked_div x={x} e={e}"
            );
            assert_eq!(
                checked_rem(x, unb),
                safe.then_some(rem as T),
                "checked_rem x={x} e={e}"
            );
            assert_eq!(checked_div_floor(x, unb), safe.then_some(floor as T));
            assert_eq!(checked_rem_floor(x, unb), safe.then_some(rem as T));
            assert_eq!(checked_div_ceil(x, unb), safe.then_some(ceil as T));
            assert_eq!(checked_div_round(x, unb), safe.then_some(round as T));
            assert_eq!(
                checked_floor_to_multiple(x, unb),
                safe.then_some(floor_m as T)
            );
            assert_eq!(
                checked_ceil_to_multiple(x, unb),
                (safe && ceil_m <= max).then_some(ceil_m as T),
                "checked_ceil_to_multiple x={x} e={e}"
            );
            assert_eq!(
                checked_round_to_multiple(x, unb),
                (safe && round_m <= max).then_some(round_m as T),
                "checked_round_to_multiple x={x} e={e}"
            );

            // ---- unbounded_* : defined at every exponent.
            assert_eq!(unbounded_div(x, unb), floor as T);
            assert_eq!(unbounded_rem(x, unb), rem as T);
            assert_eq!(unbounded_div_floor(x, unb), floor as T);
            assert_eq!(unbounded_div_ceil(x, unb), ceil as T);
            assert_eq!(
                unbounded_div_round(x, unb),
                round as T,
                "unbounded_div_round x={x} e={e}"
            );
            assert_eq!(unbounded_is_multiple_of(x, unb), is_mult);
            // Rounding an unsigned value down can never leave `[0, MAX]`, so the
            // unsigned variant returns `T` directly rather than `Option<T>`.
            assert_eq!(unbounded_floor_to_multiple(x, unb), floor_m as T);
            assert_eq!(
                unbounded_ceil_to_multiple(x, unb),
                (ceil_m <= max).then_some(ceil_m as T)
            );
            assert_eq!(
                unbounded_round_to_multiple(x, unb),
                (round_m <= max).then_some(round_m as T),
                "unbounded_round_to_multiple x={x} e={e}"
            );

            if !safe {
                // A power of two too large for the LHS width never multiplies
                // back into range, so `checked_mul` is always `None`.
                assert_eq!(checked_mul(x, unb), None, "checked_mul oob x={x} e={e}");
                return;
            }

            let p = Pow2::<$u>::from_exponent(e as u8).unwrap();
            let mul_m = xw * dw;

            // Both right-hand-side representations must agree with the oracle.
            macro_rules! both {
                ($rhs:expr) => {{
                    let r = $rhs;
                    assert_eq!(div_floor(x, r), floor as T, "div_floor x={x} e={e}");
                    assert_eq!(rem_floor(x, r), rem as T, "rem_floor x={x} e={e}");
                    assert_eq!(div_ceil(x, r), ceil as T, "div_ceil x={x} e={e}");
                    assert_eq!(div_round(x, r), round as T, "div_round x={x} e={e}");
                    assert_eq!(is_multiple_of(x, r), is_mult, "is_multiple_of x={x} e={e}");
                    assert_eq!(
                        floor_to_multiple(x, r),
                        floor_m as T,
                        "floor_to_multiple x={x} e={e}"
                    );
                    assert_eq!(x / r, floor as T, "div op x={x} e={e}");
                    assert_eq!(x % r, rem as T, "rem op x={x} e={e}");
                    {
                        let mut a = x;
                        a /= r;
                        assert_eq!(a, floor as T, "div_assign x={x} e={e}");
                    }
                    {
                        let mut a = x;
                        a %= r;
                        assert_eq!(a, rem as T, "rem_assign x={x} e={e}");
                    }
                    // Overflow-capable operations: assert the value only when it
                    // is representable (the overflow contract is covered by the
                    // `checked_*`/`unbounded_*` asserts above and the dedicated
                    // release-semantics test).
                    if ceil_m <= max {
                        assert_eq!(
                            ceil_to_multiple(x, r),
                            ceil_m as T,
                            "ceil_to_multiple x={x} e={e}"
                        );
                    }
                    if round_m <= max {
                        assert_eq!(
                            round_to_multiple(x, r),
                            round_m as T,
                            "round_to_multiple x={x} e={e}"
                        );
                    }
                    if mul_m <= max {
                        assert_eq!(x * r, mul_m as T, "mul op x={x} e={e}");
                        let mut a = x;
                        a *= r;
                        assert_eq!(a, mul_m as T, "mul_assign x={x} e={e}");
                        assert_eq!(
                            checked_mul(x, r),
                            Some(mul_m as T),
                            "checked_mul x={x} e={e}"
                        );
                    } else {
                        assert_eq!(checked_mul(x, r), None, "checked_mul overflow x={x} e={e}");
                    }
                }};
            }
            both!(p);
            both!(unb);
        }
    };
}

macro_rules! signed_suite {
    ($t:ty, $u:ty) => {
        type T = $t;
        const BITS: u32 = <$t>::BITS;
        const EMAX: u32 = BITS + 3;

        fn check(x: T, e: u32) {
            let xw = x as i128;
            let dw = 1i128 << e;
            let safe = e < BITS;
            let lo = <$t>::MIN as i128;
            let hi = <$t>::MAX as i128;
            let repr = |v: i128| lo <= v && v <= hi;
            let unb = UnboundedPow2::from_exponent(e as u8);

            let floor = xw.div_euclid(dw);
            let rem = xw.rem_euclid(dw); // floor remainder, always non-negative
            let ceil = floor + (rem != 0) as i128;
            let round = {
                let q = xw / dw;
                let r = xw % dw;
                if r.unsigned_abs() * 2 >= dw as u128 {
                    q + xw.signum()
                } else {
                    q
                }
            };
            let trunc = xw / dw; // toward zero, matches the `/` operator
            let trunc_rem = xw % dw; // matches the `%` operator
            let is_mult = trunc_rem == 0;
            let floor_m = floor * dw; // provably representable
            let ceil_m = ceil * dw;
            let round_m = round * dw;

            // ---- checked_*
            assert_eq!(
                checked_div(x, unb),
                safe.then_some(trunc as T),
                "checked_div x={x} e={e}"
            );
            assert_eq!(checked_rem(x, unb), safe.then_some(trunc_rem as T));
            assert_eq!(checked_div_floor(x, unb), safe.then_some(floor as T));
            assert_eq!(checked_rem_floor(x, unb), safe.then_some(rem as T));
            assert_eq!(checked_div_ceil(x, unb), safe.then_some(ceil as T));
            assert_eq!(
                checked_div_round(x, unb),
                safe.then_some(round as T),
                "checked_div_round x={x} e={e}"
            );
            assert_eq!(
                checked_floor_to_multiple(x, unb),
                safe.then_some(floor_m as T)
            );
            assert_eq!(
                checked_ceil_to_multiple(x, unb),
                (safe && repr(ceil_m)).then_some(ceil_m as T),
                "checked_ceil_to_multiple x={x} e={e}"
            );
            assert_eq!(
                checked_round_to_multiple(x, unb),
                (safe && repr(round_m)).then_some(round_m as T),
                "checked_round_to_multiple x={x} e={e}"
            );

            // ---- unbounded_*
            assert_eq!(
                unbounded_div(x, unb),
                trunc as T,
                "unbounded_div x={x} e={e}"
            );
            assert_eq!(unbounded_rem(x, unb), trunc_rem as T);
            assert_eq!(
                unbounded_div_floor(x, unb),
                floor as T,
                "unbounded_div_floor x={x} e={e}"
            );
            assert_eq!(
                unbounded_div_ceil(x, unb),
                ceil as T,
                "unbounded_div_ceil x={x} e={e}"
            );
            assert_eq!(
                unbounded_div_round(x, unb),
                round as T,
                "unbounded_div_round x={x} e={e}"
            );
            assert_eq!(unbounded_is_multiple_of(x, unb), is_mult);
            assert_eq!(
                unbounded_floor_to_multiple(x, unb),
                repr(floor_m).then_some(floor_m as T),
                "unbounded_floor_to_multiple x={x} e={e}"
            );
            assert_eq!(
                unbounded_ceil_to_multiple(x, unb),
                repr(ceil_m).then_some(ceil_m as T)
            );
            assert_eq!(
                unbounded_round_to_multiple(x, unb),
                repr(round_m).then_some(round_m as T),
                "unbounded_round_to_multiple x={x} e={e}"
            );

            if !safe {
                assert_eq!(checked_mul(x, unb), None, "checked_mul oob x={x} e={e}");
                return;
            }

            let p = Pow2::<$u>::from_exponent(e as u8).unwrap();
            let mul_m = xw * dw;

            macro_rules! both {
                ($rhs:expr) => {{
                    let r = $rhs;
                    assert_eq!(div_floor(x, r), floor as T, "div_floor x={x} e={e}");
                    assert_eq!(rem_floor(x, r), rem as T, "rem_floor x={x} e={e}");
                    assert_eq!(div_ceil(x, r), ceil as T, "div_ceil x={x} e={e}");
                    assert_eq!(div_round(x, r), round as T, "div_round x={x} e={e}");
                    assert_eq!(is_multiple_of(x, r), is_mult, "is_multiple_of x={x} e={e}");
                    assert_eq!(
                        floor_to_multiple(x, r),
                        floor_m as T,
                        "floor_to_multiple x={x} e={e}"
                    );
                    assert_eq!(x / r, trunc as T, "div op x={x} e={e}");
                    assert_eq!(x % r, trunc_rem as T, "rem op x={x} e={e}");
                    {
                        let mut a = x;
                        a /= r;
                        assert_eq!(a, trunc as T, "div_assign x={x} e={e}");
                    }
                    {
                        let mut a = x;
                        a %= r;
                        assert_eq!(a, trunc_rem as T, "rem_assign x={x} e={e}");
                    }
                    if repr(ceil_m) {
                        assert_eq!(
                            ceil_to_multiple(x, r),
                            ceil_m as T,
                            "ceil_to_multiple x={x} e={e}"
                        );
                    }
                    if repr(round_m) {
                        assert_eq!(
                            round_to_multiple(x, r),
                            round_m as T,
                            "round_to_multiple x={x} e={e}"
                        );
                    }
                    if repr(mul_m) {
                        assert_eq!(x * r, mul_m as T, "mul op x={x} e={e}");
                        let mut a = x;
                        a *= r;
                        assert_eq!(a, mul_m as T, "mul_assign x={x} e={e}");
                        assert_eq!(
                            checked_mul(x, r),
                            Some(mul_m as T),
                            "checked_mul x={x} e={e}"
                        );
                    } else {
                        assert_eq!(checked_mul(x, r), None, "checked_mul overflow x={x} e={e}");
                    }
                }};
            }
            both!(p);
            both!(unb);
        }
    };
}

/// Exhaustive: every value of the type against every exponent.
macro_rules! exhaustive_test {
    ($modname:ident, $macro:ident, $t:ty, $u:ty) => {
        mod $modname {
            use super::*;
            $macro!($t, $u);

            #[test]
            fn oracle() {
                for e in 0..EMAX {
                    let mut x = <$t>::MIN;
                    loop {
                        check(x, e);
                        match x.checked_add(1) {
                            Some(n) => x = n,
                            None => break,
                        }
                    }
                }
            }
        }
    };
}

/// Sampled: edge values plus an evenly strided sweep, against every exponent.
macro_rules! sampled_test {
    ($modname:ident, $macro:ident, $t:ty, $u:ty) => {
        mod $modname {
            use super::*;
            $macro!($t, $u);

            fn samples() -> Vec<T> {
                let mut v = vec![
                    <$t>::MIN,
                    <$t>::MAX,
                    0,
                    1,
                    <$t>::MIN.wrapping_add(1),
                    <$t>::MAX - 1,
                ];
                // Even stride across the whole range (~200 points).
                let span = (<$t>::MAX as i128) - (<$t>::MIN as i128);
                let step = (span / 200).max(1);
                let mut w = <$t>::MIN as i128;
                while w <= <$t>::MAX as i128 {
                    v.push(w as $t);
                    w += step;
                }
                // A scatter of small magnitudes and exact powers of two.
                for k in 0..BITS {
                    let p = 1i128 << k;
                    for d in [-1i128, 0, 1] {
                        let c = p + d;
                        if (c >= <$t>::MIN as i128) && (c <= <$t>::MAX as i128) {
                            v.push(c as $t);
                        }
                        let c = -p + d;
                        if (c >= <$t>::MIN as i128) && (c <= <$t>::MAX as i128) {
                            v.push(c as $t);
                        }
                    }
                }
                v.sort_unstable();
                v.dedup();
                v
            }

            #[test]
            fn oracle() {
                let xs = samples();
                for e in 0..EMAX {
                    for &x in &xs {
                        check(x, e);
                    }
                }
            }
        }
    };
}

exhaustive_test!(u8_all, unsigned_suite, u8, u8);
exhaustive_test!(u16_all, unsigned_suite, u16, u16);
exhaustive_test!(i8_all, signed_suite, i8, u8);
exhaustive_test!(i16_all, signed_suite, i16, u16);

sampled_test!(u32_sampled, unsigned_suite, u32, u32);
sampled_test!(u64_sampled, unsigned_suite, u64, u64);
sampled_test!(i32_sampled, signed_suite, i32, u32);
sampled_test!(i64_sampled, signed_suite, i64, u64);
