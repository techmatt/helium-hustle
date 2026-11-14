use crate::fx::Fx;
use crate::math;

fn f(s: &str) -> Fx {
    Fx::parse(s).unwrap()
}

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
fn libm_pow() {
    assert_eq!(math::pow_int(f("2"), 10), Some(f("1024")));
    assert_eq!(math::pow_int(f("1.15"), 0), Some(Fx::ONE));
    assert_eq!(math::pow_int(f("1.5"), 2), Some(f("2.25")));
    // The floor of the float: 1.15 * 1.15 as an f64 is just under 1.3225.
    assert_eq!(math::pow_int(f("1.15"), 2), Some(f("1.322499")));
    assert_eq!(math::pow_int(f("10"), 20), None, "out of range");
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
