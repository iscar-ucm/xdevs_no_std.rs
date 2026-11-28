//! Verification helpers for atomic simulators.

use crate::{
    bag::Bag,
    simulation::{simulator::Simulator, AbstractSimulator},
    Atomic, Component,
};

/// Checks quiescent `delta` behavior after initializing the given model with
/// [`Simulator::start`] at a symbolic finite start time:
///
/// - the component's post-start state is unchanged (`Clone` + `PartialEq`),
/// - the output bag remains empty,
/// - the next event time returned by `start` is returned unchanged.
///
/// Checks all finite `f64` start times with `t_start < t_next` and call times
/// satisfying `t_start <= t < t_next`, where `t_next` is computed by `start`
/// and `t_last = t_start`. Private timing-field invariants are checked
/// separately by the simulator's internal proofs.
///
/// Pass a component with symbolic (`kani::any()`) state, constrained to your
/// model's invariants with `kani::assume`. The model's `start` hook runs before
/// the state snapshot is taken; exclude NaN fields if using derived `PartialEq`.
pub fn delta_is_noop_when_quiescent<T>(component: T)
where
    T: Atomic + Clone + PartialEq,
{
    let t_start: f64 = kani::any();
    kani::assume(t_start.is_finite());
    let mut sim = Simulator::new(component);
    let t_next = sim.start(t_start);
    let t: f64 = kani::any();
    kani::assume(t_start < t_next);
    kani::assume(t_start <= t);
    kani::assume(t < t_next);

    let snapshot = (*sim).clone();
    let mut input = <T as Component>::Input::build();
    let mut output = <T as Component>::Output::build();

    let returned = sim.delta(&mut input, &mut output, t);

    kani::assert(*sim == snapshot, "component state unchanged");
    kani::assert(
        returned.to_bits() == t_next.to_bits(),
        "delta returns t_next",
    );
    kani::assert(output.is_empty(), "output bag remains empty");
}

/// Checks that [`Simulator::lambda`] produces no output before the next event
/// after initializing the model with [`Simulator::start`].
///
/// Checks all finite `f64` start times with `t_start < t_next` and call times
/// satisfying `t_start <= t < t_next`, where `t_next` is computed by `start`
/// and `t_last = t_start`.
pub fn lambda_is_noop_when_quiescent<T>(component: T)
where
    T: Atomic,
{
    let t_start: f64 = kani::any();
    kani::assume(t_start.is_finite());
    let mut sim = Simulator::new(component);
    let t_next = sim.start(t_start);
    let t: f64 = kani::any();
    kani::assume(t_start < t_next);
    kani::assume(t_start <= t);
    kani::assume(t < t_next);

    let mut output = <T as Component>::Output::build();
    sim.lambda(&mut output, t);

    kani::assert(output.is_empty(), "output bag remains empty");
}
