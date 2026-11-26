use super::f;
use crate::fx::Fx;
use crate::math;

#[test]
fn parses_and_prints_exactly() {
    assert_eq!(f("12").to_string(), "12.000000");
    assert_eq!(f("-0.5").to_string(), "-0.500000");
    assert_eq!(f("1.000001").to_string(), "1.000001");
    assert_eq!(f("0.000001"), Fx::EPSILON);
    assert_eq!(f("1"), Fx::ONE);
    assert_eq!(f(&f("-123.456789").to_string()), f("-123.456789"));
    for bad in ["", "-", "1.", ".5", "1.0000001", "1e3", "+1", "1,5", "abc", "99999999999999999"] {
        assert!(Fx::parse(bad).is_err(), "{bad:?} parsed");
    }
}

#[test]
fn json_is_a_decimal_string() {
    assert_eq!(serde_json::to_string(&f("2.5")).unwrap(), "\"2.500000\"");
    assert_eq!(serde_json::from_str::<Fx>("\"2.5\"").unwrap(), f("2.5"));
    assert!(serde_json::from_str::<Fx>("2.5").is_err(), "a JSON number is refused");
}

#[test]
fn products_and_quotients_round_toward_negative_infinity() {
    // 0.000001 * 0.5 = 0.0000005: floor gives 0, and -0.0000005 gives -0.000001.
    assert_eq!(Fx::EPSILON * f("0.5"), Fx::ZERO);
    assert_eq!(-Fx::EPSILON * f("0.5"), -Fx::EPSILON);
    assert_eq!(f("1") / f("3"), f("0.333333"));
    assert_eq!(f("-1") / f("3"), f("-0.333334"));
    assert_eq!(f("1") / f("-3"), f("-0.333334"));
    assert_eq!(f("-1") / f("-3"), f("0.333333"));
    assert_eq!(f("2") / f("3"), f("0.666666"));
    assert_eq!(f("10").mul_ratio(1, 3), f("3.333333"));
    assert_eq!(f("-10").mul_ratio(1, 3), f("-3.333334"));
    assert_eq!(f("2.5").floor_int(), 2);
    assert_eq!(f("-2.5").floor_int(), -3);
    // i128 intermediates: a product whose raw form overflows i64 midway still works.
    assert_eq!(f("1000000") * f("1000000"), f("1000000000000"));
}

#[test]
fn from_f64_floors() {
    assert_eq!(Fx::from_f64(0.5), f("0.5"));
    assert_eq!(Fx::from_f64(-0.0000005), -Fx::EPSILON);
    assert_eq!(Fx::from_f64(1.0 / 3.0), f("0.333333"));
}

#[test]
fn whole_powers_are_exact() {
    assert_eq!(f("2").pow_int(10), Some(f("1024")));
    assert_eq!(f("1.15").pow_int(0), Some(Fx::ONE));
    assert_eq!(f("1.5").pow_int(2), Some(f("2.25")));
    // Exact, not the float's floor (1.15 * 1.15 as an f64 is just under 1.3225).
    assert_eq!(f("1.15").pow_int(2), Some(f("1.3225")));
    assert_eq!(f("1.5").pow_int(20), Some(f("3325.256730")));
    assert_eq!(f("0.97").pow_int(3), Some(f("0.912673")));
    // Negative exponents: 1 / x^n, floored once.
    assert_eq!(f("1.05").pow_int(-3), Some(f("0.863837")));
    assert_eq!(f("2").pow_int(-2), Some(f("0.25")));
    assert_eq!(f("10").pow_int(20), None, "out of range");
    assert_eq!(f("1.5").pow_int(i64::from(u32::MAX)), None, "a huge exponent ends early");
    assert_eq!(f("0.5").pow_int(i64::from(u32::MAX)), Some(Fx::ZERO));
    assert_eq!(Fx::ZERO.pow_int(-1), None);
}

#[test]
fn libm_factors() {
    // A fractional power goes through libm and floors the float.
    assert_eq!(math::pow_frac(f("4"), f("0.5")), Some(f("2")));
}

#[test]
fn checked_and_saturating() {
    assert_eq!(Fx::MAX.checked_mul(f("2")), None);
    assert_eq!(f("3").checked_mul(f("0.5")), Some(f("1.5")));
    assert_eq!(Fx::MAX.saturating_add(Fx::ONE), Fx::MAX);
    assert_eq!(Fx::try_from_f64(f64::INFINITY), None);
    assert_eq!(Fx::try_from_f64(f64::NAN), None);
}

#[test]
#[should_panic(expected = "Fx overflow")]
fn overflow_panics() {
    let big = Fx::from_int(9_000_000_000_000);
    let _ = big + big;
}

#[test]
#[should_panic(expected = "division by zero")]
fn division_by_zero_panics() {
    let _ = Fx::ONE / Fx::ZERO;
}
