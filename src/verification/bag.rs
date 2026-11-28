//! Reusable checks for [`Bag`] implementations.
//!
//! Supply valid symbolic bag states in `#[kani::proof]` harnesses.
//! Verification covers only the concrete types, capacities, states, and insertion histories exercised by those harnesses.
//! Loops may require a `#[kani::unwind(...)]` bound. Keep Kani's unwinding assertions enabled.
//!
//! ```
//! # #[cfg(kani)]
//! # mod proofs {
//! #[kani::proof]
//! fn option_bag_laws() {
//!     xdevs::verification::bag::build_is_empty::<Option<u8>>();
//!     let bag: Option<u8> = kani::any();
//!     xdevs::verification::bag::is_empty_matches_iteration(&bag);
//!     xdevs::verification::bag::clear_is_empty(bag);
//! }
//! # }
//! ```

use crate::Bag;

/// Checks the common bag laws after up to `max_insertions` symbolic insertions into a
/// newly built bag. Every iteration independently inserts or skips, and inserted events
/// range over all values supported by `B::Value`'s [`kani::Arbitrary`] implementation.
///
/// Checks empty construction, agreement between emptiness and iteration after the insertion
/// history, and emptiness after clearing twice. Insertion errors are permitted; no ordering,
/// duplicate, or lossless insertion policy is assumed.
///
/// # Note
///
/// Set the calling harness's `#[kani::unwind(...)]` bound high enough for the
/// insertion loop (typically `max_insertions + 1`) and loops inside the bag.
/// This checks bounded insertion histories, not arbitrary operation sequences.
///
/// ```
/// # #[cfg(kani)]
/// # mod proofs {
/// #[kani::proof]
/// #[kani::unwind(8)]
/// fn port_bag_laws() {
///     xdevs::verification::bag::check_bag_laws::<xdevs::Port<u16, 2>>(4);
/// }
/// # }
/// ```
pub fn check_bag_laws<B: Bag>(max_insertions: usize)
where
    B::Value: kani::Arbitrary,
{
    build_is_empty::<B>();
    let mut bag = B::build();
    for _ in 0..max_insertions {
        if kani::any() {
            let _ = bag.add_value(kani::any());
        }
    }
    is_empty_matches_iteration(&bag);
    clear_is_empty(bag);
}

/// Checks that a newly built bag reports empty.
pub fn build_is_empty<B: Bag>() {
    let bag = B::build();
    kani::assert(bag.is_empty(), "newly built bag is empty");
}

/// Checks that a valid bag reports empty after clearing, including when cleared again.
///
/// # Note
///
/// Pass symbolic states or bags populated with symbolic events to cover nonempty states.
/// Passing only a newly built bag does not check that case.
pub fn clear_is_empty<B: Bag>(mut bag: B) {
    bag.clear();
    kani::assert(bag.is_empty(), "cleared bag is empty");

    bag.clear();
    kani::assert(bag.is_empty(), "clearing twice leaves bag empty");
}

/// Checks that `is_empty()` agrees with whether iteration yields any events.
/// Does not impose ordering, uniqueness, or lossless insertion semantics.
pub fn is_empty_matches_iteration<B: Bag>(bag: &B) {
    let empty = bag.is_empty();
    kani::assert(
        empty == bag.get_values().next().is_none(),
        "is_empty agrees with iteration",
    );
}
