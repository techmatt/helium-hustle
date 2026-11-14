//! Float factors (a cost-scaling multiplier, a curve), computed only through
//! the `libm` crate so native and wasm get the same bits, and handed back as
//! [`Fx`] at once. Platform `f64` transcendentals are banned in `hh-core`
//! (`clippy.toml`); add a function here instead of calling one.

use crate::fx::Fx;

/// `base ^ exp` for a whole exponent, e.g. a cost growth factor raised to the
/// number already owned: `libm::pow`, then `Fx::try_from_f64` (floor of the
/// float). `None` when the result is out of `Fx`'s range.
pub fn pow_int(base: Fx, exp: u32) -> Option<Fx> {
    Fx::try_from_f64(libm::pow(base.to_f64(), exp as f64))
}
