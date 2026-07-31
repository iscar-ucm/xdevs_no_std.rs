use super::Bag;

use core::hash::Hash;
use std::collections::{HashMap, HashSet};

unsafe impl<T: Clone + Eq + Hash> Bag for HashSet<T> {
    type Value = T;

    #[inline]
    fn build() -> Self {
        Self::new()
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    #[inline]
    fn clear(&mut self) {
        self.clear()
    }

    #[inline]
    fn add_value(&mut self, event: Self::Value) -> Result<(), Self::Value> {
        if self.insert(event.clone()) {
            Ok(())
        } else {
            Err(event)
        }
    }

    #[inline]
    fn len(&self) -> usize {
        self.len()
    }

    #[inline]
    fn propagate(&self, mut propagator: impl FnMut(Self::Value)) {
        for value in self.iter() {
            propagator(value.clone());
        }
    }
}

unsafe impl<K: Clone + Eq + Hash, V: Clone> Bag for HashMap<K, V> {
    type Value = (K, V);

    #[inline]
    fn build() -> Self {
        Self::new()
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    #[inline]
    fn clear(&mut self) {
        self.clear()
    }

    #[inline]
    fn add_value(&mut self, event: Self::Value) -> Result<(), Self::Value> {
        let (key, value) = event;
        match self.entry(key) {
            std::collections::hash_map::Entry::Occupied(entry) => Err((entry.key().clone(), value)),
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(value);
                Ok(())
            }
        }
    }

    #[inline]
    fn len(&self) -> usize {
        self.len()
    }

    #[inline]
    fn propagate(&self, mut propagator: impl FnMut(Self::Value)) {
        for (key, value) in self.iter() {
            propagator((key.clone(), value.clone()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::vec::Vec;

    #[test]
    fn hashset_bag_impl_contract() {
        let mut bag = <HashSet<u32> as Bag>::build();
        assert!(Bag::is_empty(&bag));
        assert_eq!(Bag::len(&bag), 0);

        assert!(bag.add_value(7).is_ok());
        assert!(!Bag::is_empty(&bag));
        assert_eq!(Bag::len(&bag), 1);

        Bag::clear(&mut bag);
        assert!(Bag::is_empty(&bag));
        assert_eq!(Bag::len(&bag), 0);
    }

    #[test]
    fn hashset_bag_is_empty_and_clear_cycle() {
        let mut bag = <HashSet<u32> as Bag>::build();
        assert!(Bag::is_empty(&bag));

        bag.add_value(1).unwrap();
        bag.add_value(2).unwrap();
        bag.add_value(3).unwrap();
        assert!(!Bag::is_empty(&bag));

        Bag::clear(&mut bag);
        assert!(Bag::is_empty(&bag));
        assert_eq!(Bag::len(&bag), 0);

        // Reusable after clear; duplicate of a previously-cleared value is accepted.
        bag.add_value(1).unwrap();
        assert!(!Bag::is_empty(&bag));
        assert_eq!(Bag::len(&bag), 1);

        Bag::clear(&mut bag);
        assert!(Bag::is_empty(&bag));

        // Clearing an already-empty bag is a no-op.
        Bag::clear(&mut bag);
        assert!(Bag::is_empty(&bag));
        assert_eq!(Bag::len(&bag), 0);
    }

    #[test]
    fn hashset_bag_rejects_duplicates() {
        let mut bag = <HashSet<u32> as Bag>::build();
        assert!(bag.add_value(7).is_ok());
        assert_eq!(bag.add_value(7), Err(7));
        assert_eq!(Bag::len(&bag), 1);
    }

    #[test]
    fn hashset_bag_propagate_emits_all() {
        let mut bag = <HashSet<u32> as Bag>::build();
        bag.add_value(10).unwrap();
        bag.add_value(20).unwrap();
        bag.add_value(30).unwrap();

        let mut collected: Vec<u32> = Vec::new();
        bag.propagate(|v| collected.push(v));
        // HashSet order is unspecified, so sort before comparing.
        collected.sort();
        assert_eq!(collected.as_slice(), &[10, 20, 30]);
    }

    #[test]
    fn hashset_bag_couple_to_vec() {
        let mut src = <HashSet<u32> as Bag>::build();
        src.add_value(1).unwrap();
        src.add_value(2).unwrap();
        src.add_value(3).unwrap();
        let mut dst = <Vec<u32> as Bag>::build();
        assert!(src.couple(&mut dst).is_ok());
        let mut got = dst.as_slice().to_vec();
        got.sort();
        assert_eq!(got.as_slice(), &[1, 2, 3]);
    }

    #[test]
    fn hashset_bag_adapt_and_couple_to_vec() {
        let mut src = <HashSet<u32> as Bag>::build();
        src.add_value(1).unwrap();
        src.add_value(2).unwrap();
        src.add_value(3).unwrap();
        let mut dst = <Vec<u64> as Bag>::build();
        assert!(src.adapt_and_couple(&mut dst, |v| v as u64 * 2).is_ok());
        let mut got = dst.as_slice().to_vec();
        got.sort();
        assert_eq!(got.as_slice(), &[2, 4, 6]);
    }

    // --- std::collections::HashMap<K, V> ---

    #[test]
    fn hashmap_bag_impl_contract() {
        let mut bag = <HashMap<u8, u32> as Bag>::build();
        assert!(Bag::is_empty(&bag));
        assert_eq!(Bag::len(&bag), 0);

        assert!(bag.add_value((1, 10)).is_ok());
        assert!(!Bag::is_empty(&bag));
        assert_eq!(Bag::len(&bag), 1);

        Bag::clear(&mut bag);
        assert!(Bag::is_empty(&bag));
        assert_eq!(Bag::len(&bag), 0);
    }

    #[test]
    fn hashmap_bag_is_empty_and_clear_cycle() {
        let mut bag = <HashMap<u8, u32> as Bag>::build();
        assert!(Bag::is_empty(&bag));

        bag.add_value((1, 10)).unwrap();
        bag.add_value((2, 20)).unwrap();
        bag.add_value((3, 30)).unwrap();
        assert!(!Bag::is_empty(&bag));

        Bag::clear(&mut bag);
        assert!(Bag::is_empty(&bag));
        assert_eq!(Bag::len(&bag), 0);

        // Reusable after clear; a previously-cleared key is accepted again.
        bag.add_value((1, 99)).unwrap();
        assert!(!Bag::is_empty(&bag));
        assert_eq!(Bag::len(&bag), 1);

        Bag::clear(&mut bag);
        assert!(Bag::is_empty(&bag));

        // Clearing an already-empty bag is a no-op.
        Bag::clear(&mut bag);
        assert!(Bag::is_empty(&bag));
        assert_eq!(Bag::len(&bag), 0);
    }

    #[test]
    fn hashmap_bag_rejects_duplicate_keys() {
        let mut bag = <HashMap<u8, u32> as Bag>::build();
        assert!(bag.add_value((1, 10)).is_ok());
        assert_eq!(bag.add_value((1, 99)), Err((1, 99)));
        assert_eq!(Bag::len(&bag), 1);
    }

    #[test]
    fn hashmap_bag_propagate_emits_all() {
        let mut bag = <HashMap<u8, u32> as Bag>::build();
        bag.add_value((1, 10)).unwrap();
        bag.add_value((2, 20)).unwrap();
        bag.add_value((3, 30)).unwrap();

        let mut collected: Vec<(u8, u32)> = Vec::new();
        bag.propagate(|v| collected.push(v));
        // HashMap order is unspecified, so sort by key before comparing.
        collected.sort();
        assert_eq!(collected.as_slice(), &[(1, 10), (2, 20), (3, 30)]);
    }

    #[test]
    fn hashmap_bag_couple_to_vec() {
        let mut src = <HashMap<u8, u32> as Bag>::build();
        src.add_value((1, 10)).unwrap();
        src.add_value((2, 20)).unwrap();
        src.add_value((3, 30)).unwrap();
        let mut dst = <Vec<(u8, u32)> as Bag>::build();
        assert!(src.couple(&mut dst).is_ok());
        let mut got = dst.as_slice().to_vec();
        got.sort();
        assert_eq!(got.as_slice(), &[(1, 10), (2, 20), (3, 30)]);
    }

    #[test]
    fn hashmap_bag_adapt_and_couple_to_vec() {
        let mut src = <HashMap<u8, u32> as Bag>::build();
        src.add_value((1, 10)).unwrap();
        src.add_value((2, 20)).unwrap();
        let mut dst = <Vec<(u8, u64)> as Bag>::build();
        assert!(src
            .adapt_and_couple(&mut dst, |(k, v)| (k, v as u64 * 100))
            .is_ok());
        let mut got = dst.as_slice().to_vec();
        got.sort();
        assert_eq!(got.as_slice(), &[(1, 1000), (2, 2000)]);
    }
}
