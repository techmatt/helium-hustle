//! `Fx`, the fixed-point number every quantity in game state is held in.
//!
//! An `Fx` is a signed count of millionths (10^-6): 1.0 is `1_000_000`. Its
//! representation is private to this file, so its width or scale can change
//! here alone.
//!
//! **The rounding rule** (the only one in `hh-core`): a result that falls
//! between two representable values is rounded **toward negative infinity**
//! (floor). It applies to `*`, `/`, [`Fx::mul_ratio`] and [`Fx::from_f64`].
//! Products and quotients are computed in `i128`, so only the final result is
//! rounded. Addition and subtraction are exact.
//!
//! **Overflow panics** (checked arithmetic) in every build: a wrapped number
//! would be a silent rules bug.
//!
//! **Serialized** as an exact decimal string with six places (`"12.500000"`),
//! so JSON never passes it through a float. Parsing accepts up to six places
//! and refuses more (never silently rounds input).

use serde::{de::Error as _, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

type Raw = i64;
type Wide = i128;

/// Number of decimal places.
const PLACES: u32 = 6;
/// Raw units per 1.0.
const SCALE: Raw = 1_000_000;

#[derive(Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Fx(Raw);

/// Floor division (the rounding rule), for any sign of either operand.
fn div_floor(a: Wide, b: Wide) -> Wide {
    assert!(b != 0, "Fx: division by zero");
    let q = a / b;
    if a % b != 0 && ((a < 0) != (b < 0)) {
        q - 1
    } else {
        q
    }
}

fn narrow(v: Wide, what: &str) -> Fx {
    Fx(Raw::try_from(v).unwrap_or_else(|_| panic!("Fx overflow in {what}")))
}

impl Fx {
    pub const ZERO: Fx = Fx(0);
    pub const ONE: Fx = Fx(SCALE);
    /// The smallest positive step (10^-6).
    pub const EPSILON: Fx = Fx(1);
    /// The largest value (about 9.2 × 10^12).
    pub const MAX: Fx = Fx(Raw::MAX);
    /// The smallest value.
    pub const MIN: Fx = Fx(Raw::MIN);

    /// An exact whole number.
    pub fn from_int(n: i64) -> Fx {
        narrow(n as Wide * SCALE as Wide, "from_int")
    }

    /// The whole part, rounded toward negative infinity.
    pub fn floor_int(self) -> i64 {
        div_floor(self.0 as Wide, SCALE as Wide) as i64
    }

    /// `self * num / den` with one rounding at the end, for exact ratios.
    pub fn mul_ratio(self, num: i64, den: i64) -> Fx {
        narrow(div_floor(self.0 as Wide * num as Wide, den as Wide), "mul_ratio")
    }

    /// The nearest multiple of `unit` (a positive `Fx`), halves away from zero:
    /// a deliberate display-grade rounding a rule asks for (a price to whole
    /// credits, a threshold to two figures), not arithmetic's rounding rule.
    /// Clamped to [`Fx::MAX`]/[`Fx::MIN`] at the range's edges.
    pub fn round_to(self, unit: Fx) -> Fx {
        assert!(unit.0 > 0, "Fx::round_to: unit must be positive");
        let (a, u) = (self.0.unsigned_abs() as Wide, unit.0 as Wide);
        let r = (a + u / 2) / u * u;
        let v = if self.0 < 0 { -r } else { r };
        Fx(v.clamp(Raw::MIN as Wide, Raw::MAX as Wide) as Raw)
    }

    /// [`Fx::round_to`] whole units.
    pub fn round_whole(self) -> Fx {
        self.round_to(Fx::ONE)
    }

    /// [`Fx::round_to`] `figures` significant figures (725.3 at 2 is 730; 0 is 0).
    pub fn round_sig(self, figures: u32) -> Fx {
        let a = self.0.unsigned_abs() as Wide;
        if a == 0 || figures == 0 {
            return self;
        }
        let digits = a.ilog10() + 1; // of the raw value
        if digits <= figures {
            return self;
        }
        let unit = (10 as Wide).pow(digits - figures);
        self.round_to(narrow(unit, "round_sig"))
    }

    /// `self * n` for a count `n`: exact.
    pub fn mul_int(self, n: i64) -> Fx {
        narrow(self.0 as Wide * n as Wide, "mul_int")
    }

    /// The `k`-th step of `[lo, lo + EPSILON * k]`: `lo + k * EPSILON`. For
    /// turning a uniform integer draw into a value.
    pub fn plus_steps(self, k: i64) -> Fx {
        narrow(self.0 as Wide + k as Wide, "plus_steps")
    }

    /// The number of `EPSILON` steps in `self` (its size in raw units). For
    /// sizing a uniform draw over a range; not a back door to the raw value.
    pub fn steps(self) -> i64 {
        self.0
    }

    /// **The one float boundary:** a float computed through `crate::math`,
    /// rounded by the rounding rule (floor) — the floor of the *float's*
    /// value, so a float a hair below an exact decimal (1.15² as an `f64` is
    /// 1.3224999999999998) lands one step low (1.322499). `None` when it is
    /// not finite or out of range.
    pub fn try_from_f64(x: f64) -> Option<Fx> {
        if !x.is_finite() {
            return None;
        }
        // `* SCALE` is one IEEE multiplication (correctly rounded on every
        // platform); `libm::floor` is exact.
        let scaled = libm::floor(x * SCALE as f64);
        (scaled >= Raw::MIN as f64 && scaled < Raw::MAX as f64).then_some(Fx(scaled as Raw))
    }

    /// [`Fx::try_from_f64`], panicking on a non-finite or out-of-range value.
    pub fn from_f64(x: f64) -> Fx {
        Fx::try_from_f64(x).unwrap_or_else(|| panic!("Fx::from_f64({x}): not finite or out of range"))
    }

    /// `self ^ n` for a whole `n` (a negative `n` is `1 / self ^ |n|`), for cost
    /// curves and per-rank bonuses. No float: the power is built by repeated
    /// multiplication at 10^-18 in `i128` (each step floored there), then floored
    /// once to 10^-6, so an exact decimal power comes out exact (1.15² is
    /// 1.3225, not the float's 1.322499). `None` past `Fx`'s range or for
    /// `0 ^ negative`.
    pub fn pow_int(self, n: i64) -> Option<Fx> {
        const FINE: Wide = 1_000_000_000_000_000_000; // 1.0 at 10^-18
        const UP: Wide = FINE / SCALE as Wide;
        let base = self.0 as Wide;
        if base == SCALE as Wide || n == 0 {
            return Some(Fx::ONE);
        }
        // Past this the value is beyond Fx::MAX whatever comes next (for |base| >= 1).
        let limit = (Raw::MAX as Wide + 1) * UP;
        let mut acc = FINE;
        for _ in 0..n.unsigned_abs() {
            acc = div_floor(acc.checked_mul(base)?, SCALE as Wide);
            if acc == 0 {
                break;
            }
            if acc.abs() >= limit && base.abs() >= SCALE as Wide {
                return if n > 0 { None } else { Some(Fx::ZERO) };
            }
        }
        if n > 0 {
            Raw::try_from(div_floor(acc, UP)).ok().map(Fx)
        } else if acc == 0 {
            None
        } else {
            Raw::try_from(div_floor(FINE * SCALE as Wide, acc)).ok().map(Fx)
        }
    }

    /// `*` (same rounding), or `None` on overflow.
    pub fn checked_mul(self, o: Fx) -> Option<Fx> {
        Raw::try_from(div_floor(self.0 as Wide * o.0 as Wide, SCALE as Wide))
            .ok()
            .map(Fx)
    }

    /// `+`, clamped to [`Fx::MAX`]/[`Fx::MIN`] instead of panicking: for an
    /// uncapped running total that a long game could push past the range.
    pub fn saturating_add(self, o: Fx) -> Fx {
        Fx(self.0.saturating_add(o.0))
    }

    /// The value as a float, for computing a factor in `crate::math` only.
    /// Exact for magnitudes under 2^53 millionths.
    pub fn to_f64(self) -> f64 {
        self.0 as f64 / SCALE as f64
    }

    pub fn is_negative(self) -> bool {
        self.0 < 0
    }

    pub fn min(self, other: Fx) -> Fx {
        Fx(self.0.min(other.0))
    }

    pub fn max(self, other: Fx) -> Fx {
        Fx(self.0.max(other.0))
    }

    /// Parses a decimal (`"12"`, `"-0.5"`, `"1.000001"`): at most six places,
    /// no exponent, no float on the way.
    pub fn parse(s: &str) -> Result<Fx, String> {
        let bad = || format!("{s:?} is not a decimal with at most {PLACES} places");
        let t = s.trim();
        let (neg, body) = match t.strip_prefix('-') {
            Some(rest) => (true, rest),
            None => (false, t),
        };
        let (whole, frac) = match body.split_once('.') {
            Some((w, f)) => (w, f),
            None => (body, ""),
        };
        if whole.is_empty()
            || !whole.bytes().all(|b| b.is_ascii_digit())
            || !frac.bytes().all(|b| b.is_ascii_digit())
            || frac.len() > PLACES as usize
            || (body.contains('.') && frac.is_empty())
        {
            return Err(bad());
        }
        let w: Wide = whole.parse().map_err(|_| bad())?;
        let mut f: Wide = if frac.is_empty() { 0 } else { frac.parse().map_err(|_| bad())? };
        for _ in frac.len()..PLACES as usize {
            f *= 10;
        }
        let v = w
            .checked_mul(SCALE as Wide)
            .and_then(|v| v.checked_add(f))
            .ok_or_else(|| format!("{s:?} is out of range"))?;
        let v = if neg { -v } else { v };
        Raw::try_from(v)
            .map(Fx)
            .map_err(|_| format!("{s:?} is out of range"))
    }
}

impl fmt::Display for Fx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let v = self.0 as Wide;
        let sign = if v < 0 { "-" } else { "" };
        let a = v.abs();
        write!(
            f,
            "{sign}{}.{:0width$}",
            a / SCALE as Wide,
            a % SCALE as Wide,
            width = PLACES as usize
        )
    }
}

impl fmt::Debug for Fx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Fx({self})")
    }
}

impl Serialize for Fx {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Fx {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Fx, D::Error> {
        let s = String::deserialize(d)?;
        Fx::parse(&s).map_err(D::Error::custom)
    }
}

impl Add for Fx {
    type Output = Fx;
    fn add(self, o: Fx) -> Fx {
        Fx(self.0.checked_add(o.0).expect("Fx overflow in +"))
    }
}

impl Sub for Fx {
    type Output = Fx;
    fn sub(self, o: Fx) -> Fx {
        Fx(self.0.checked_sub(o.0).expect("Fx overflow in -"))
    }
}

impl AddAssign for Fx {
    fn add_assign(&mut self, o: Fx) {
        *self = *self + o;
    }
}

impl SubAssign for Fx {
    fn sub_assign(&mut self, o: Fx) {
        *self = *self - o;
    }
}

impl Neg for Fx {
    type Output = Fx;
    fn neg(self) -> Fx {
        Fx(self.0.checked_neg().expect("Fx overflow in negation"))
    }
}

impl Mul for Fx {
    type Output = Fx;
    /// Rounded by the rounding rule (floor).
    fn mul(self, o: Fx) -> Fx {
        narrow(div_floor(self.0 as Wide * o.0 as Wide, SCALE as Wide), "*")
    }
}

impl Div for Fx {
    type Output = Fx;
    /// Rounded by the rounding rule (floor). Panics on division by zero.
    fn div(self, o: Fx) -> Fx {
        // Scale the dividend up first; `mul_ratio` keeps it in i128.
        Fx(self.0).mul_ratio(SCALE, o.0)
    }
}

impl std::iter::Sum for Fx {
    fn sum<I: Iterator<Item = Fx>>(iter: I) -> Fx {
        iter.fold(Fx::ZERO, |a, b| a + b)
    }
}
