//! Float factors (a curve with a fractional exponent, a logarithm), computed only
//! through the `libm` crate so native and wasm get the same bits, and handed back
//! as [`Fx`] at once. Platform `f64` transcendentals are banned in `hh-core`
//! (`clippy.toml`); add a function here instead of calling one. Whole powers need
//! no float: [`Fx::pow_int`].

use crate::fx::Fx;

/// `base ^ exp` for a fractional exponent: `libm::pow`, then `Fx::try_from_f64`
/// (floor of the float). `None` when out of `Fx`'s range or not a number.
pub fn pow_frac(base: Fx, exp: Fx) -> Option<Fx> {
    Fx::try_from_f64(libm::pow(base.to_f64(), exp.to_f64()))
}

/// `log_base(x)` (`ln x / ln base`) through `libm::log`. `None` for a
/// non-positive `x` or a `base` of 1 or less.
pub fn log_in(x: Fx, base: Fx) -> Option<Fx> {
    if x <= Fx::ZERO || base <= Fx::ONE {
        return None;
    }
    Fx::try_from_f64(libm::log(x.to_f64()) / libm::log(base.to_f64()))
}
