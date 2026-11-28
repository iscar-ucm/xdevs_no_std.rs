//! Reusable proof functions for verifying user-defined DEVS models with the
//! [Kani](https://model-checking.github.io/kani/) model checker.
//!
//! This module is only compiled under `cargo kani` (`cfg(kani)`). Kani verifies
//! *monomorphized* code, so the proofs shipped with this crate only certify the
//! concrete model types they are instantiated with. To obtain the same guarantees
//! for **your** models, instantiate these functions with your own types in a
//! `#[kani::proof]` harness:
//!
//! ```ignore
//! #[cfg(kani)]
//! mod proofs {
//!     use super::MyModel;
//!
//!     #[kani::proof]
//!     fn my_model_quiescence() {
//!         let sigma: f64 = kani::any();
//!         // Constrain the symbolic state to your model's invariants.
//!         // NOTE: f64 state must exclude NaN, as NaN != NaN breaks the
//!         // `PartialEq` snapshot comparison.
//!         kani::assume(sigma >= 0.0);
//!         xdevs::verification::simulation::simulator::delta_is_noop_when_quiescent(
//!             MyModel::new(sigma),
//!         );
//!     }
//!
//!     #[kani::proof]
//!     fn my_model_sanity() {
//!         let sigma: f64 = kani::any();
//!         kani::assume(sigma >= 0.0);
//!         let mut input = xdevs::Port::new();
//!         if kani::any() {
//!             let _ = input.push(kani::any());
//!         }
//!         xdevs::verification::component::atomic::atomic_sanity(MyModel::new(sigma), input);
//!     }
//! }
//! ```
//!
//! Then run `cargo kani` in your crate.

pub mod bag;
pub mod component;
pub mod simulation;
