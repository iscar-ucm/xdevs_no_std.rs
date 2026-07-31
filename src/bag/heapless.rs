use super::Bag;

unsafe impl<T: Clone, const N: usize> Bag for heapless::Vec<T, N> {
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
    fn len(&self) -> usize {
        self.as_slice().len()
    }

    #[inline]
    fn add_value(&mut self, event: Self::Value) -> Result<(), Self::Value> {
        self.push(event)
    }

    #[inline]
    fn propagate(&self, mut propagator: impl FnMut(Self::Value)) {
        for value in self.iter() {
            propagator(value.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::Bag;
    use super::super::Port;

    #[test]
    fn port_new_is_empty() {
        let port: Port<u32, 5> = Port::new();
        assert!(port.is_empty());
        assert_eq!(port.len(), 0);
    }

    #[test]
    fn port_add_value_and_get_values() {
        let mut port: Port<u32, 5> = Port::new();
        assert!(port.add_value(1).is_ok());
        assert!(port.add_value(2).is_ok());
        assert!(port.add_value(3).is_ok());
        assert_eq!(port.as_slice(), &[1, 2, 3]);
    }

    #[test]
    fn port_add_value_rejects_when_full() {
        let mut port: Port<u32, 3> = Port::new();
        assert!(port.add_value(10).is_ok());
        assert!(port.add_value(20).is_ok());
        assert!(port.add_value(30).is_ok());
        assert!(port.is_full());
        let result = port.add_value(40);
        assert_eq!(result, Err(40));
        assert_eq!(port.as_slice(), &[10, 20, 30]);
    }

    #[test]
    fn port_add_values_from_slice() {
        let mut port: Port<u32, 5> = Port::new();
        assert!(port.add_values(&[10, 20, 30]).is_ok());
        assert_eq!(port.len(), 3);
        assert_eq!(port.as_slice(), &[10, 20, 30]);
    }

    #[test]
    fn port_add_values_capacity_error() {
        let mut port: Port<u32, 3> = Port::new();
        port.add_values(&[1, 2, 3]).unwrap();
        assert!(port.is_full());
        let result = port.add_values(&[4]);
        assert!(result.is_err());
        assert_eq!(port.as_slice(), &[1, 2, 3]);
    }

    #[test]
    fn port_is_full_len_cycle() {
        let mut port: Port<u32, 3> = Port::new();
        assert_eq!(port.len(), 0);
        assert!(port.is_empty());
        assert!(!port.is_full());

        port.add_value(1).unwrap();
        assert_eq!(port.len(), 1);
        assert!(!port.is_empty());
        assert!(!port.is_full());

        port.add_value(2).unwrap();
        assert_eq!(port.len(), 2);

        port.add_value(3).unwrap();
        assert_eq!(port.len(), 3);
        assert!(port.is_full());

        port.clear();
        assert_eq!(port.len(), 0);
        assert!(port.is_empty());
        assert!(!port.is_full());
    }

    #[test]
    fn port_multiple_add_clear_cycle() {
        let mut port: Port<u32, 3> = Port::new();
        for _ in 0..3 {
            port.add_value(99).unwrap();
            assert_eq!(port.len(), 1);
            port.clear();
            assert!(port.is_empty());
        }
    }

    #[test]
    fn port_default_creates_empty() {
        let port: Port<u32, 5> = Default::default();
        assert!(port.is_empty());
        assert_eq!(port.len(), 0);
    }

    // --- Bag trait contract (calls go through the trait) ---

    #[test]
    fn port_bag_impl_contract() {
        let mut bag = <Port<u32, 5> as Bag>::build();
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
    fn port_bag_is_empty_and_clear_cycle() {
        let mut bag = <Port<u32, 5> as Bag>::build();
        assert!(Bag::is_empty(&bag));

        Bag::add_values(&mut bag, &[1, 2, 3]).unwrap();
        assert!(!Bag::is_empty(&bag));

        Bag::clear(&mut bag);
        assert!(Bag::is_empty(&bag));
        assert_eq!(Bag::len(&bag), 0);

        // Reusable after clear.
        bag.add_value(99).unwrap();
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
    fn port_bag_inject_eject_contract() {
        let mut bag = <Port<u32, 2> as Bag>::build();
        assert!(Bag::is_empty(&bag));

        assert!(bag.add_value(7).is_ok());
        assert!(bag.add_value(99).is_ok());
        assert_eq!(bag.add_value(42), Err(42));

        let mut collected: ::heapless::Vec<u32, 4> = ::heapless::Vec::new();
        bag.propagate(|v| {
            let _ = collected.push(v);
        });
        assert_eq!(collected.as_slice(), &[7, 99]);
    }

    #[test]
    fn port_bag_couple_copies_values() {
        let mut src: Port<u32, 5> = Port::new();
        src.add_values(&[1, 2, 3]).unwrap();
        let mut dst: Port<u32, 5> = Port::new();
        assert!(src.couple(&mut dst).is_ok());
        assert_eq!(dst.as_slice(), &[1, 2, 3]);
        assert_eq!(src.as_slice(), &[1, 2, 3]);
    }

    #[test]
    fn port_bag_couple_capacity_error() {
        let mut src: Port<u32, 5> = Port::new();
        src.add_values(&[1, 2, 3]).unwrap();
        let mut dst: Port<u32, 2> = Port::new();
        let result = src.couple(&mut dst);
        assert!(result.is_err());
        assert_eq!(src.as_slice(), &[1, 2, 3]);
    }

    #[test]
    fn port_bag_adapt_and_couple_transforms_values() {
        let mut src: Port<u32, 5> = Port::new();
        src.add_values(&[1, 2, 3]).unwrap();
        let mut dst: Port<u64, 5> = Port::new();
        // Adapter doubles each value and widens to u64.
        assert!(src.adapt_and_couple(&mut dst, |v| v as u64 * 2).is_ok());
        assert_eq!(dst.as_slice(), &[2, 4, 6]);
        // Source is unchanged.
        assert_eq!(src.as_slice(), &[1, 2, 3]);
    }

    #[test]
    fn port_bag_adapt_and_couple_capacity_error() {
        let mut src: Port<u32, 5> = Port::new();
        src.add_values(&[1, 2, 3]).unwrap();
        let mut dst: Port<u64, 2> = Port::new();
        let result = src.adapt_and_couple(&mut dst, |v| v as u64 * 2);
        // The third adapted event (6) cannot be inserted.
        assert_eq!(result, Err(6));
        // The first two events were inserted before the failure.
        assert_eq!(dst.as_slice(), &[2, 4]);
        assert_eq!(src.as_slice(), &[1, 2, 3]);
    }

    #[test]
    fn port_bag_adapt_and_couple_empty_source() {
        let src: Port<u32, 5> = Port::new();
        let mut dst: Port<u64, 5> = Port::new();
        assert!(src.adapt_and_couple(&mut dst, |v| v as u64 * 2).is_ok());
        assert!(dst.is_empty());
    }

    #[test]
    fn port_bag_adapt_and_couple_type_conversion() {
        let mut src: Port<u32, 5> = Port::new();
        src.add_values(&[0, 1, 2]).unwrap();
        let mut dst: Port<bool, 5> = Port::new();
        // Adapter converts non-zero to true.
        assert!(src.adapt_and_couple(&mut dst, |v| v != 0).is_ok());
        assert_eq!(dst.as_slice(), &[false, true, true]);
    }
}
