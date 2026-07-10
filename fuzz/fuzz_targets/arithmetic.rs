#![no_main]
//! Fuzz the arithmetic laws for `u64` / `UnboundedPow2` against random inputs.

use ipow2::{
    Pow2, UnboundedPow2, ceil_to_multiple, checked_ceil_to_multiple, div_ceil, div_floor,
    div_round, floor_to_multiple, is_multiple_of, rem_floor,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.len() < 9 {
        return;
    }
    let x = u64::from_le_bytes(data[0..8].try_into().unwrap());
    let e = data[8] % u64::BITS as u8; // 0..64 is always a valid exponent
    let p = Pow2::<u64>::from_exponent(e).unwrap();

    let lo = floor_to_multiple(x, p);
    let r = rem_floor(x, p);
    assert_eq!(lo.wrapping_add(r), x);
    assert!((r as u128) < (1u128 << e));
    assert_eq!(is_multiple_of(x, p), r == 0);

    let f = div_floor(x, p);
    let c = div_ceil(x, p);
    let rd = div_round(x, p);
    assert!(f <= rd && rd <= c);
    assert!(c - f <= 1);

    if let Some(v) = checked_ceil_to_multiple(x, p) {
        assert_eq!(v, ceil_to_multiple(x, p));
    }

    // The same laws through a same-exponent (safe) UnboundedPow2.
    let u = UnboundedPow2::from_exponent(e);
    assert_eq!(div_floor(x, u), f);
    assert_eq!(rem_floor(x, u), r);
    assert_eq!(is_multiple_of(x, u), is_multiple_of(x, p));
});
