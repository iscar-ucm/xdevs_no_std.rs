use super::Bag;
use core::hash::Hash;
use hashbrown::{HashMap, HashSet};

unsafe impl<T: Clone> Bag for alloc::vec::Vec<T> {
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
        self.push(event);
        Ok(())
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
            hashbrown::hash_map::Entry::Occupied(entry) => Err((entry.key().clone(), value)),
            hashbrown::hash_map::Entry::Vacant(entry) => {
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
    use alloc::vec::Vec;

    #[test]
    fn vec_bag_impl_contract() {
        let mut bag = <Vec<u32> as Bag>::build();
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
    fn vec_bag_is_empty_and_clear_cycle() {
        let mut bag = <Vec<u32> as Bag>::build();
        // Freshly built is empty.
        assert!(Bag::is_empty(&bag));

        // Adding values makes it non-empty.
        Bag::add_values(&mut bag, &[1, 2, 3]).unwrap();
        assert!(!Bag::is_empty(&bag));

        // Clear empties it.
        Bag::clear(&mut bag);
        assert!(Bag::is_empty(&bag));
        assert_eq!(Bag::len(&bag), 0);

        // The bag can be reused after clear.
        bag.add_value(99).unwrap();
        assert!(!Bag::is_empty(&bag));
        assert_eq!(Bag::len(&bag), 1);

        // Clearing again works.
        Bag::clear(&mut bag);
        assert!(Bag::is_empty(&bag));

        // Clearing an already-empty bag is a no-op.
        Bag::clear(&mut bag);
        assert!(Bag::is_empty(&bag));
        assert_eq!(Bag::len(&bag), 0);
    }

    #[test]
    fn vec_bag_add_values_and_len() {
        let mut bag = <Vec<u32> as Bag>::build();
        assert!(Bag::add_values(&mut bag, &[1, 2, 3]).is_ok());
        assert_eq!(Bag::len(&bag), 3);
        assert_eq!(bag.as_slice(), &[1, 2, 3]);
    }

    #[test]
    fn vec_bag_propagate_emits_all() {
        let mut bag = <Vec<u32> as Bag>::build();
        Bag::add_values(&mut bag, &[10, 20, 30]).unwrap();

        let mut collected: Vec<u32> = Vec::new();
        bag.propagate(|v| collected.push(v));
        // Order is preserved for Vec.
        assert_eq!(collected.as_slice(), &[10, 20, 30]);
    }

    #[test]
    fn vec_bag_couple_to_vec() {
        let mut src = <Vec<u32> as Bag>::build();
        Bag::add_values(&mut src, &[1, 2, 3]).unwrap();
        let mut dst = <Vec<u32> as Bag>::build();
        assert!(src.couple(&mut dst).is_ok());
        assert_eq!(dst.as_slice(), &[1, 2, 3]);
        // Source is unchanged.
        assert_eq!(src.as_slice(), &[1, 2, 3]);
    }

    #[test]
    fn vec_bag_adapt_and_couple_to_vec() {
        let mut src = <Vec<u32> as Bag>::build();
        Bag::add_values(&mut src, &[1, 2, 3]).unwrap();
        let mut dst = <Vec<u64> as Bag>::build();
        assert!(src.adapt_and_couple(&mut dst, |v| v as u64 * 2).is_ok());
        assert_eq!(dst.as_slice(), &[2, 4, 6]);
        assert_eq!(src.as_slice(), &[1, 2, 3]);
    }

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
        // HashSet order is unspecified, so sort before comparing.
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
