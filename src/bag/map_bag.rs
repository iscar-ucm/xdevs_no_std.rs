/// Treats a map of bags as a single bag of `(key, event)` pairs.
///
/// The wrapper distinguishes this behavior from the existing [`Bag`](super::Bag)
/// implementation for maps, which treats each map value as one event.
/// The wrapped map is accessible through the public field.
/// Both `std::collections::HashMap` (with the `std` feature) and
/// `hashbrown::HashMap` (with `alloc`) are supported. Empty inner bags contain
/// no events, and iteration order between keys is unspecified.
///
/// Insertion uses the inner bag's capacity and returns the original key and
/// rejected event on failure. Clearing removes all keys.
///
/// # Note
///
/// For better user experience, do not use this struct directly.
/// Use the provided type aliases [`HashbrownMapBag`] or [`StdMapBag`] instead.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MapBag<M>(pub M);

/// A bag backed by a `hashbrown` hash map of bags.
///
/// ```
/// use xdevs::bag::{Bag, map_bag::HashbrownMapBag};
/// let mut bag = HashbrownMapBag::<u8, Vec<u64>>::build();
/// bag.add_value((1, 42)).unwrap();
/// ```
pub type HashbrownMapBag<K, B> = MapBag<hashbrown::HashMap<K, B>>;

/// A bag backed by a `std` hash map of bags.
///
/// ```
/// use xdevs::bag::{Bag, map_bag::StdMapBag};
/// let mut bag = StdMapBag::<u8, Vec<u64>>::build();
/// bag.add_value((1, 42)).unwrap();
/// ```
#[cfg(feature = "std")]
pub type StdMapBag<K, B> = MapBag<std::collections::HashMap<K, B>>;
