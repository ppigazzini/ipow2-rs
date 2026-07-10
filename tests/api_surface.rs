//! Coverage for the non-arithmetic public surface: constructors, accessors,
//! constants, conversions, error `Display`, and the `checked_*`/`saturating_*`
//! combinators on `Pow2` and `UnboundedPow2` themselves.

use ipow2::{NotPow2, Pow2, Pow2OutOfRange, Pow2TryFromIntError, UnboundedPow2};

#[repr(align(256))]
struct Align256(#[allow(dead_code)] u8);

#[test]
fn unbounded_accessors_and_align() {
    let p = UnboundedPow2::from_exponent(6);
    assert_eq!(p.exponent(), 6);
    assert!(p.is_safe::<u8>()); // 6 <= u8::SAFE_SHIFT (7)
    assert!(!UnboundedPow2::from_exponent(8).is_safe::<u8>());

    assert_eq!(
        UnboundedPow2::align_of::<u64>().exponent(),
        align_of::<u64>().ilog2() as u8
    );
    let v = Align256(0);
    assert_eq!(UnboundedPow2::align_of_val(&v).exponent(), 8);
}

#[test]
fn unbounded_as_int_safe() {
    assert_eq!(UnboundedPow2::from_exponent(0).as_u8(), 1);
    assert_eq!(UnboundedPow2::from_exponent(6).as_u8(), 64);
    assert_eq!(UnboundedPow2::from_exponent(6).as_i8(), 64);
    assert_eq!(UnboundedPow2::from_exponent(15).as_u16(), 1 << 15);
    assert_eq!(UnboundedPow2::from_exponent(14).as_i16(), 1 << 14);
    assert_eq!(UnboundedPow2::from_exponent(31).as_u32(), 1 << 31);
    assert_eq!(UnboundedPow2::from_exponent(63).as_u64(), 1 << 63);
    assert_eq!(UnboundedPow2::from_exponent(127).as_u128(), 1 << 127);
    assert_eq!(UnboundedPow2::from_exponent(10).as_usize(), 1024);
    assert_eq!(UnboundedPow2::from_exponent(10).as_isize(), 1024);
}

#[test]
fn unbounded_checked_saturating() {
    let a = UnboundedPow2::VAL_16; // 2^4
    let b = UnboundedPow2::VAL_4; // 2^2
    assert_eq!(a.checked_mul(b).unwrap().exponent(), 6);
    assert_eq!(a.checked_div(b).unwrap().exponent(), 2);
    assert_eq!(b.checked_div(a), None); // exponent underflow
    assert_eq!(a.saturating_div(b).exponent(), 2);
    assert_eq!(b.saturating_div(a).exponent(), 0); // saturates at 2^0

    // Exponent is a u8, so multiplication saturates its sum at 255.
    let hi = UnboundedPow2::from_exponent(200);
    assert_eq!(hi.checked_mul(hi), None);
    assert_eq!(hi.saturating_mul(hi).exponent(), 255);
}

#[test]
fn pow2_accessors_value_mask_align() {
    let p = Pow2::<u32>::VAL_16;
    assert_eq!(p.exponent(), 4);
    assert_eq!(p.value(), 16);
    assert_eq!(p.mask(), 15);

    assert_eq!(Pow2::<u64>::from_exponent(0).unwrap().value(), 1);
    assert_eq!(Pow2::<u64>::from_exponent(0).unwrap().mask(), 0);

    assert_eq!(Pow2::<u64>::align_of::<u32>().unwrap().exponent(), 2);
    let v = Align256(0);
    // A u8-backed Pow2 cannot represent alignment 256 (2^8 needs exponent < 8).
    assert_eq!(Pow2::<u8>::align_of_val(&v), Err(Pow2OutOfRange));
    assert_eq!(Pow2::<u16>::align_of_val(&v).unwrap().exponent(), 8);
}

#[test]
fn pow2_from_exponent_bounds() {
    assert!(Pow2::<u8>::from_exponent(7).is_ok());
    assert_eq!(Pow2::<u8>::from_exponent(8), Err(Pow2OutOfRange));
    assert!(Pow2::<u16>::from_exponent(15).is_ok());
    assert_eq!(Pow2::<u16>::from_exponent(16), Err(Pow2OutOfRange));
}

#[test]
fn pow2_checked_saturating() {
    let a = Pow2::<u8>::VAL_4; // 2^2
    let b = Pow2::<u8>::VAL_8; // 2^3
    assert_eq!(a.checked_mul(b).unwrap().exponent(), 5);
    // 2^4 * 2^4 = 2^8 does not fit a u8-backed Pow2.
    assert_eq!(Pow2::<u8>::VAL_16.checked_mul(Pow2::<u8>::VAL_16), None);
    assert_eq!(
        Pow2::<u8>::VAL_64
            .saturating_mul(Pow2::<u8>::VAL_64)
            .exponent(),
        7
    );

    assert_eq!(b.checked_div(a).unwrap().exponent(), 1);
    assert_eq!(a.checked_div(b), None); // exponent underflow
    assert_eq!(a.saturating_div(b).exponent(), 0);
}

#[test]
fn conversions_int_to_pow2() {
    // UnboundedPow2 from integers.
    assert_eq!(UnboundedPow2::try_from(256u16).unwrap().exponent(), 8);
    assert_eq!(UnboundedPow2::try_from(3u16), Err(NotPow2));
    assert_eq!(UnboundedPow2::try_from(0u32), Err(NotPow2));

    // Pow2<T> from integers, incl. both error arms.
    assert_eq!(Pow2::<u32>::try_from(1024i32).unwrap().exponent(), 10);
    assert_eq!(
        Pow2::<u32>::try_from(6i32),
        Err(Pow2TryFromIntError::NotPow2)
    );
    assert_eq!(
        Pow2::<u32>::try_from(-4i32),
        Err(Pow2TryFromIntError::NotPow2)
    );
    // 256 = 2^8 is a power of two but too wide for a u8-backed Pow2.
    assert_eq!(
        Pow2::<u8>::try_from(256u16),
        Err(Pow2TryFromIntError::Pow2OutOfRange)
    );
}

#[test]
fn conversions_pow2_to_int_and_between() {
    assert_eq!(u8::try_from(UnboundedPow2::from_exponent(7)).unwrap(), 128);
    assert_eq!(
        u8::try_from(UnboundedPow2::from_exponent(8)),
        Err(Pow2OutOfRange)
    );

    assert_eq!(
        Pow2::<u8>::try_from(UnboundedPow2::from_exponent(8)),
        Err(Pow2OutOfRange)
    );
    assert_eq!(
        Pow2::<u16>::try_from(UnboundedPow2::from_exponent(8))
            .unwrap()
            .exponent(),
        8
    );

    let unb: UnboundedPow2 = Pow2::<u8>::VAL_16.into();
    assert_eq!(unb.exponent(), 4);
}

#[test]
fn error_display_and_from() {
    assert_eq!(NotPow2.to_string(), "Not a power of two");
    assert_eq!(Pow2OutOfRange.to_string(), "Out of range");
    assert_eq!(
        Pow2TryFromIntError::NotPow2.to_string(),
        "Not a power of two"
    );
    assert_eq!(
        Pow2TryFromIntError::Pow2OutOfRange.to_string(),
        "Out of range"
    );

    assert_eq!(
        Pow2TryFromIntError::from(NotPow2),
        Pow2TryFromIntError::NotPow2
    );
    assert_eq!(
        Pow2TryFromIntError::from(Pow2OutOfRange),
        Pow2TryFromIntError::Pow2OutOfRange
    );
}
