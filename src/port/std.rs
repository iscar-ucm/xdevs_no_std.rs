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

    #[test]
    fn hashset_bag_rejects_duplicates() {
        let mut bag = <HashSet<u32> as Bag>::build();
        assert!(bag.add_value(7).is_ok());
        assert_eq!(bag.add_value(7), Err(7));
        assert_eq!(Bag::len(&bag), 1);
    }

    #[test]
    fn hashmap_bag_rejects_duplicate_keys() {
        let mut bag = <HashMap<u8, u32> as Bag>::build();
        assert!(bag.add_value((1, 10)).is_ok());
        assert_eq!(bag.add_value((1, 99)), Err((1, 99)));
        assert_eq!(Bag::len(&bag), 1);
    }
}
