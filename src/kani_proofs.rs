//! Bounded-model-checking proofs, run under `cargo kani` (see the `kani` CI
//! job). Compiled only when `cfg(kani)` is set, so they add no normal-build
//! dependency. Kani explores every `u8`/`i8` value symbolically, turning these
//! into exhaustive proofs that the formulas equal their mathematical
//! definitions.

use crate::{Pow2, div_ceil, div_floor, div_round, floor_to_multiple, is_multiple_of, rem_floor};

#[kani::proof]
fn unsigned_u8_matches_definitions() {
    let x: u8 = kani::any();
    let e: u8 = kani::any();
    kani::assume(e < u8::BITS as u8);
    let p = Pow2::<u8>::from_exponent(e).unwrap();
    let mask = (1u8 << e) - 1;

    assert_eq!(div_floor(x, p), x >> e);
    assert_eq!(rem_floor(x, p), x & mask);
    assert_eq!(floor_to_multiple(x, p), x & !mask);
    assert_eq!(is_multiple_of(x, p), (x & mask) == 0);
    assert_eq!(floor_to_multiple(x, p) + rem_floor(x, p), x);
}

#[kani::proof]
fn signed_i8_laws_hold() {
    let x: i8 = kani::any();
    let e: u8 = kani::any();
    kani::assume(e < u8::BITS as u8);
    let p = Pow2::<u8>::from_exponent(e).unwrap();

    // floor + rem reconstructs the input, and the remainder is in [0, 2^e).
    let r = rem_floor(x, p);
    assert_eq!(floor_to_multiple(x, p) + r, x);
    assert!(r >= 0 && (r as u16) < (1u16 << e));
    assert_eq!(is_multiple_of(x, p), r == 0);

    // floor <= round <= ceil, and ceil - floor is 0 or 1.
    let f = div_floor(x, p);
    let c = div_ceil(x, p);
    let rd = div_round(x, p);
    assert!(f <= rd && rd <= c);
    assert!(c - f == 0 || c - f == 1);
}

#[kani::proof]
fn div_round_matches_half_away_i8() {
    let x: i8 = kani::any();
    let e: u8 = kani::any();
    kani::assume(e < u8::BITS as u8);
    let p = Pow2::<u8>::from_exponent(e).unwrap();

    // Independent reference: round half away from zero, computed in i16.
    let d = 1i16 << e;
    let xw = x as i16;
    let q = xw / d;
    let rem = xw % d;
    let expected = if rem.unsigned_abs() * 2 >= d as u16 {
        q + xw.signum()
    } else {
        q
    };
    assert_eq!(div_round(x, p) as i16, expected);
}
