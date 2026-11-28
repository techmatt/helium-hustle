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

/// The `i64` fast path of the rounding rule (137) gives the `i128` floor: every sign,
/// remainders and none, and products on both sides of `i64`'s range.
#[test]
fn products_and_quotients_floor_on_both_paths() {
    let raw = |k: i64| Fx::EPSILON.mul_int(k);
    let floor = |a: i128, b: i128| -> i128 {
        let q = a / b;
        if a % b != 0 && ((a < 0) != (b < 0)) {
            q - 1
        } else {
            q
        }
    };
    let mut vals: Vec<i64> = vec![0, 1, -1, 2, -2, 7, -7, 999_999, -999_999, 1_000_000, -1_000_000, 1_000_001, -1_000_001];
    vals.extend([3_037_000_499, -3_037_000_499, 3_037_000_500, -3_037_000_500, 730_000_000, -730_000_000, 998_102]);
    vals.extend([9_223_372_036_854, -9_223_372_036_854, 9_223_372_036_854_775, i64::MAX / 7, i64::MIN / 7, i64::MAX, i64::MIN + 1]);
    let mut x: u64 = 0x9E37_79B9_7F4A_7C15;
    for _ in 0..400 {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        vals.push((x as i64) >> (x % 50));
    }
    for &a in &vals {
        for &b in &vals {
            let p = floor(a as i128 * b as i128, 1_000_000);
            if let Ok(p) = i64::try_from(p) {
                assert_eq!((raw(a) * raw(b)).steps(), p, "{a} * {b}");
            }
            if b != 0 {
                let q = floor(a as i128 * 1_000_000, b as i128);
                if let Ok(q) = i64::try_from(q) {
                    assert_eq!((raw(a) / raw(b)).steps(), q, "{a} / {b}");
                    assert_eq!(raw(a).mul_ratio(1_000_000, b).steps(), q, "{a} ratio {b}");
                }
            }
        }
        assert_eq!(raw(a).floor_int() as i128, floor(a as i128, 1_000_000), "floor {a}");
    }
}

#[test]
#[should_panic(expected = "division by zero")]
fn division_by_zero_panics() {
    let _ = Fx::ONE / Fx::ZERO;
}
