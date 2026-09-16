//! Implementations of the `Bag` trait for various types of the `heapless` crate.

use super::Bag;
use heapless::{binary_heap, BinaryHeap, Deque, HistoryBuf, Vec};

unsafe impl<T: Clone, const N: usize> Bag for Vec<T, N> {
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
        self.push(event)
    }

    #[inline]
    fn get_values(&self) -> impl Iterator<Item = Self::Value> + '_ {
        self.iter().cloned()
    }
}

unsafe impl<T: Clone, const N: usize> Bag for Deque<T, N> {
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
        self.push_back(event)
    }

    #[inline]
    fn get_values(&self) -> impl Iterator<Item = Self::Value> + '_ {
        self.iter().cloned()
    }
}

unsafe impl<T: Clone, const N: usize> Bag for HistoryBuf<T, N> {
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
        self.write(event);
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

unsafe impl<T: Clone + Ord, K: binary_heap::Kind, const N: usize> Bag for BinaryHeap<T, K, N> {
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
        self.push(event)
    }

    /// Note: The order of values returned by `get_values` is not guaranteed to be
    /// sorted, as `BinaryHeap` does not maintain a sorted order for iteration.
    #[inline]
    fn get_values(&self) -> impl Iterator<Item = Self::Value> + '_ {
        self.iter().cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bag::propagate;
    use heapless::binary_heap::Max;

    #[test]
    fn vec_new_is_empty() {
        let vec: Vec<u32, 5> = Vec::new();
        assert!(vec.is_empty());
        assert_eq!(vec.len(), 0);
    }

    #[test]
    fn vec_add_value_and_get_values() {
        let mut vec: Vec<u32, 5> = Vec::new();
        assert!(vec.add_value(1).is_ok());
        assert!(vec.add_value(2).is_ok());
        assert!(vec.add_value(3).is_ok());
        assert_eq!(vec.as_slice(), &[1, 2, 3]);
    }

    #[test]
    fn vec_add_value_rejects_when_full() {
        let mut vec: Vec<u32, 3> = Vec::new();
        assert!(vec.add_value(10).is_ok());
        assert!(vec.add_value(20).is_ok());
        assert!(vec.add_value(30).is_ok());
        assert!(vec.is_full());
        let result = vec.add_value(40);
        assert_eq!(result, Err(40));
        assert_eq!(vec.as_slice(), &[10, 20, 30]);
    }

    #[test]
    fn vec_add_values_from_slice() {
        let mut vec: Vec<u32, 5> = Vec::new();
        assert!(vec.add_values([10, 20, 30]).is_ok());
        assert_eq!(vec.len(), 3);
        assert_eq!(vec.as_slice(), &[10, 20, 30]);
    }

    #[test]
    fn vec_add_values_capacity_error() {
        let mut vec: Vec<u32, 3> = Vec::new();
        vec.add_values([1, 2, 3]).unwrap();
        assert!(vec.is_full());
        let result = vec.add_values([4]);
        assert!(result.is_err());
        assert_eq!(vec.as_slice(), &[1, 2, 3]);
    }

    #[test]
    fn vec_clear_empties() {
        let mut vec: Vec<u32, 5> = Vec::new();
        vec.add_value(99).unwrap();
        assert!(!vec.is_empty());
        vec.clear();
        assert!(vec.is_empty());
        assert_eq!(vec.len(), 0);
    }

    #[test]
    fn vec_is_full_len_cycle() {
        let mut vec: Vec<u32, 3> = Vec::new();
        assert_eq!(vec.len(), 0);
        assert!(vec.is_empty());
        assert!(!vec.is_full());

        vec.add_value(1).unwrap();
        assert_eq!(vec.len(), 1);
        assert!(!vec.is_empty());
        assert!(!vec.is_full());

        vec.add_value(2).unwrap();
        assert_eq!(vec.len(), 2);

        vec.add_value(3).unwrap();
        assert_eq!(vec.len(), 3);
        assert!(vec.is_full());

        vec.clear();
        assert_eq!(vec.len(), 0);
        assert!(vec.is_empty());
        assert!(!vec.is_full());
    }

    #[test]
    fn vec_multiple_add_clear_cycle() {
        let mut vec: Vec<u32, 3> = Vec::new();
        for _ in 0..3 {
            vec.add_value(99).unwrap();
            assert_eq!(vec.len(), 1);
            vec.clear();
            assert!(vec.is_empty());
        }
    }

    #[test]
    fn vec_bag_impl_contract() {
        let mut bag = <Vec<u32, 5> as Bag>::build();
        assert!(bag.is_empty());

        bag.add_value(7).unwrap();
        assert!(!bag.is_empty());

        bag.clear();
        assert!(bag.is_empty());
    }

    #[test]
    fn vec_bag_get_values_yields_all_in_order() {
        let mut bag: Vec<u32, 5> = Vec::new();
        assert_eq!(bag.get_values().count(), 0);

        bag.add_values([1, 2, 3]).unwrap();
        let mut collected: ::heapless::Vec<u32, 4> = ::heapless::Vec::new();
        for v in bag.get_values() {
            let _ = collected.push(v);
        }
        assert_eq!(collected.as_slice(), &[1, 2, 3]);
        // Source is untouched.
        assert_eq!(bag.as_slice(), &[1, 2, 3]);
    }

    #[test]
    fn vec_bag_inject_eject_contract() {
        let mut bag = <Vec<u32, 2> as Bag>::build();
        assert!(bag.is_empty());

        assert!(bag.add_value(7).is_ok());
        assert!(bag.add_value(99).is_ok());
        assert_eq!(bag.add_value(42), Err(42));

        let mut collected: heapless::Vec<u32, 4> = heapless::Vec::new();
        propagate(&bag, |v| {
            let _ = collected.push(v);
        });
        assert_eq!(collected.as_slice(), &[7, 99]);
    }

    #[test]
    fn vec_default_creates_empty() {
        let vec: Vec<u32, 5> = Default::default();
        assert!(vec.is_empty());
        assert_eq!(vec.len(), 0);
    }

    #[test]
    fn deque_bag_impl() {
        let mut bag = Deque::build();
        assert!(<Deque<usize, 5> as Bag>::is_empty(&bag)); // to force the trait method is used
        assert_eq!(bag.len(), 0);

        for i in 0..5 {
            assert!(bag.add_value(i).is_ok());
            assert!(!bag.is_empty());
            assert_eq!(bag.len(), i + 1);
        }

        assert!(bag.add_value(5).is_err_and(|e| e == 5));
        assert!(!bag.is_empty());
        assert_eq!(bag.len(), 5);

        for (i, v) in bag.get_values().enumerate() {
            assert_eq!(i, v);
        }
        assert!(!bag.is_empty());
        assert_eq!(bag.len(), 5);

        <Deque<usize, 5> as Bag>::clear(&mut bag); // to force the trait method is used
        assert!(bag.is_empty());
    }

    #[test]
    fn historybuf_bag_impl() {
        let mut bag = HistoryBuf::build();
        assert!(<HistoryBuf<usize, 5> as Bag>::is_empty(&bag)); // to force the trait method is used
        assert_eq!(bag.len(), 0);

        for i in 0..5 {
            for j in 0..5 {
                let expected_len = if i == 0 { j + 1 } else { 5 };

                assert!(bag.add_value(i * 5 + j).is_ok());
                assert!(!bag.is_empty());
                assert_eq!(bag.len(), expected_len);
            }
            for (k, v) in bag.get_values().enumerate() {
                let expected_value = if i == 0 { k } else { i * 5 + k };
                assert_eq!(v, expected_value);
            }
        }

        <HistoryBuf<usize, 5> as Bag>::clear(&mut bag); // to force the trait method is used
        assert!(bag.is_empty());
        assert_eq!(bag.len(), 0);

        for i in 0..5 {
            assert!(bag.add_values(i * 5..i * 5 + 5).is_ok());
            assert!(!bag.is_empty());
            assert_eq!(bag.len(), 5);
            for (k, v) in bag.get_values().enumerate() {
                let expected_value = i * 5 + k;
                assert_eq!(v, expected_value);
            }
        }
    }

    #[test]
    fn binaryheap_bag_impl() {
        let mut bag = BinaryHeap::<usize, Max, 5>::build();
        assert!(<BinaryHeap<usize, Max, 5> as Bag>::is_empty(&bag)); // to force the trait method is used
        assert_eq!(bag.len(), 0);

        for i in 0..5 {
            assert!(bag.add_value(i).is_ok());
            assert!(!bag.is_empty());
            assert_eq!(bag.len(), i + 1);
        }

        assert!(bag.add_value(5).is_err_and(|e| e == 5));
        assert!(!bag.is_empty());
        assert_eq!(bag.len(), 5);

        let vals = bag.get_values().collect::<heapless::Vec<usize, 5>>();
        assert_eq!(vals.len(), 5);
        for v in 0..5 {
            assert!(vals.contains(&v));
        }
        assert!(!bag.is_empty());

        <BinaryHeap<usize, Max, 5> as Bag>::clear(&mut bag); // to force the trait method is used
        assert!(bag.is_empty());
    }
}
