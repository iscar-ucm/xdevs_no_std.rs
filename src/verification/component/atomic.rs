//! Verification helpers for atomic components.

use crate::{bag::Bag, Atomic, Component};

/// Checks basic DEVS sanity of an atomic model in the given state:
///
/// - `ta()` is non-negative and not NaN,
/// - `lambda` does not panic (or overflow, access out of bounds, etc.),
/// - `delta_int` and `delta_conf` do not panic and preserve `ta() >= 0`,
/// - `delta_ext` does not panic and preserves `ta() >= 0` for any elapsed time `0 <= e < ta()`.
///
/// Pass a component with symbolic (`kani::any()`) state and a symbolic input
/// bag, both constrained to your model's invariants with `kani::assume`.
///
/// # Note
///
/// The `delta_ext` check is checked last: if `ta() == 0` for every state you pass,
/// no valid elapsed time exists (an external event cannot interrupt an immediate internal transition),
/// so that check is vacuous while the preceding ones remain meaningful.
pub fn atomic_sanity<T>(component: T, input: <T as Component>::Input)
where
    T: Atomic + Clone,
{
    let ta = component.ta();
    // `ta >= 0.0` is false for NaN, so this also rules out NaN.
    kani::assert(ta >= 0.0, "ta() is non-negative and not NaN");

    let mut output = <T as Component>::Output::build();
    component.lambda(&mut output);

    let mut after_int = component.clone();
    after_int.delta_int();
    kani::assert(after_int.ta() >= 0.0, "ta() non-negative after delta_int");

    let mut after_conf = component.clone();
    after_conf.delta_conf(&input);
    kani::assert(after_conf.ta() >= 0.0, "ta() non-negative after delta_conf");

    // Kept last: the assumption is unsatisfiable when ta() == 0 and would make
    // any check after it vacuous.
    let elapsed: f64 = kani::any();
    kani::assume(elapsed >= 0.0 && elapsed < ta);
    let mut after_ext = component;
    after_ext.delta_ext(elapsed, &input);
    kani::assert(after_ext.ta() >= 0.0, "ta() non-negative after delta_ext");
}
