//! Implementations of the `Bag` trait for various types of the `alloc` crate.

use super::Bag;

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
    fn add_values(
        &mut self,
        events: impl IntoIterator<Item = Self::Value>,
    ) -> Result<(), Self::Value> {
        self.extend(events);
        Ok(())
    }

    #[inline]
    fn get_values(&self) -> impl Iterator<Item = Self::Value> + '_ {
        self.iter().cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bag::*;
    use alloc::vec::Vec;

    #[test]
    fn vec_bag_impl_contract() {
        let mut bag = <Vec<u32> as Bag>::build();
        assert!(Bag::is_empty(&bag));

        assert!(bag.add_value(7).is_ok());
        assert!(!Bag::is_empty(&bag));

        Bag::clear(&mut bag);
        assert!(Bag::is_empty(&bag));
    }

    #[test]
    fn vec_bag_is_empty_and_clear_cycle() {
        let mut bag = <Vec<u32> as Bag>::build();
        // Freshly built is empty.
        assert!(Bag::is_empty(&bag));

        // Adding values makes it non-empty.
        Bag::add_values(&mut bag, [1, 2, 3]).unwrap();
        assert!(!Bag::is_empty(&bag));

        // Clear empties it.
        Bag::clear(&mut bag);
        assert!(Bag::is_empty(&bag));

        // The bag can be reused after clear.
        bag.add_value(99).unwrap();
        assert!(!Bag::is_empty(&bag));

        // Clearing again works.
        Bag::clear(&mut bag);
        assert!(Bag::is_empty(&bag));

        // Clearing an already-empty bag is a no-op.
        Bag::clear(&mut bag);
        assert!(Bag::is_empty(&bag));
    }

    #[test]
    fn vec_bag_add_values_and_len() {
        let mut bag = <Vec<u32> as Bag>::build();
        assert!(Bag::add_values(&mut bag, [1, 2, 3]).is_ok());
        assert_eq!(bag.as_slice(), &[1, 2, 3]);
    }

    #[test]
    fn vec_bag_propagate_emits_all() {
        let mut bag = <Vec<u32> as Bag>::build();
        Bag::add_values(&mut bag, [10, 20, 30]).unwrap();

        let mut collected: Vec<u32> = Vec::new();
        propagate(&bag, |v| collected.push(v));
        // Order is preserved for Vec.
        assert_eq!(collected.as_slice(), &[10, 20, 30]);
    }

    #[test]
    fn vec_bag_get_values_yields_all_in_order() {
        let mut bag = <Vec<u32> as Bag>::build();
        assert_eq!(bag.get_values().count(), 0);

        Bag::add_values(&mut bag, [1, 2, 3]).unwrap();
        let collected: Vec<u32> = bag.get_values().collect();
        // Order is preserved for Vec.
        assert_eq!(collected.as_slice(), &[1, 2, 3]);
        // Source is unchanged.
        assert_eq!(bag.as_slice(), &[1, 2, 3]);
    }

    #[test]
    fn vec_bag_couple_to_vec() {
        let mut src = <Vec<u32> as Bag>::build();
        Bag::add_values(&mut src, [1, 2, 3]).unwrap();
        let mut dst = <Vec<u32> as Bag>::build();
        assert!(couple(&src, &mut dst).is_ok());
        assert_eq!(dst.as_slice(), &[1, 2, 3]);
        // Source is unchanged.
        assert_eq!(src.as_slice(), &[1, 2, 3]);
    }

    #[test]
    fn vec_bag_adapt_and_couple_to_vec() {
        let mut src = <Vec<u32> as Bag>::build();
        Bag::add_values(&mut src, [1, 2, 3]).unwrap();
        let mut dst = <Vec<u64> as Bag>::build();
        assert!(adapt_and_couple(&src, &mut dst, |v| v as u64 * 2).is_ok());
        assert_eq!(dst.as_slice(), &[2, 4, 6]);
        assert_eq!(src.as_slice(), &[1, 2, 3]);
    }
}
