#[cfg(feature = "alloc")]
pub mod alloc;
#[cfg(feature = "std")]
pub mod std;

/// Port is an alias for a heapless::Vec.
pub type Port<T, const N: usize> = heapless::Vec<T, N>;

/// Trait that defines the methods that a DEVS event bag set must implement.
///
/// # Safety
///
/// This trait must be implemented via the [`Bag`] macro. Do not implement it manually.
pub unsafe trait Bag {
    /// The data type of the events stored in the event bag.
    type Value: Clone;

    /// Build a new instance of the bag.
    fn build() -> Self;

    /// Returns `true` if the event bag is empty.
    fn is_empty(&self) -> bool;

    /// Clears the event bag, removing all values.
    fn clear(&mut self);

    /// Adds a new value into the bag.
    fn add_value(&mut self, event: Self::Value) -> Result<(), Self::Value>;

    /// Returns the number of events in the bag.
    ///
    /// Implementations may override this method for better performance.
    fn len(&self) -> usize {
        let mut len = 0;
        self.propagate(|_| {
            len += 1;
        });
        len
    }

    /// Adds multiple values to the bag.
    ///
    /// Returns the first event that cannot be inserted.
    fn add_values(&mut self, events: &[Self::Value]) -> Result<(), Self::Value> {
        for event in events {
            self.add_value(event.clone())?;
        }
        Ok(())
    }

    /// Copies all events from this bag into another bag of the same event type.
    ///
    /// Returns the first event that cannot be inserted into `to`.
    fn couple<B: Bag<Value = Self::Value>>(&self, to: &mut B) -> Result<(), Self::Value> {
        let mut failure: Option<Self::Value> = None;
        self.propagate(|event| {
            if failure.is_none() {
                if let Err(err) = to.add_value(event) {
                    failure = Some(err);
                }
            }
        });

        match failure {
            Some(err) => Err(err),
            None => Ok(()),
        }
    }

    /// Copies all events from this bag into another bag using an adapter closure.
    ///
    /// The adapter transforms each source event into the target bag event type.
    /// Returns the first adapted event that cannot be inserted into `to`.
    fn adapt_and_couple<B: Bag, F>(&self, to: &mut B, mut adapter: F) -> Result<(), B::Value>
    where
        F: FnMut(Self::Value) -> B::Value,
    {
        let mut failure: Option<B::Value> = None;
        self.propagate(|event| {
            if failure.is_none() {
                let adapted = adapter(event);
                if let Err(err) = to.add_value(adapted) {
                    failure = Some(err);
                }
            }
        });

        match failure {
            Some(err) => Err(err),
            None => Ok(()),
        }
    }

    /// Propagates all events from the bag according to the provided closure.
    fn propagate(&self, propagator: impl FnMut(Self::Value));
}

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

unsafe impl<T: Bag, const N: usize> Bag for [T; N] {
    type Value = (usize, T::Value); // Include index to identify which bag the value came from

    #[inline]
    fn build() -> Self {
        core::array::from_fn(|_| T::build())
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.as_slice().iter().all(|bag| bag.is_empty())
    }

    #[inline]
    fn clear(&mut self) {
        self.iter_mut().for_each(|bag| bag.clear());
    }

    #[inline]
    fn add_value(&mut self, (index, event): Self::Value) -> Result<(), Self::Value> {
        match self.get_mut(index) {
            Some(elem) => elem.add_value(event).map_err(|err| (index, err)),
            None => Err((index, event)),
        }
    }

    #[inline]
    fn len(&self) -> usize {
        self.as_slice().iter().map(|bag| bag.len()).sum()
    }

    #[inline]
    fn propagate(&self, mut propagator: impl FnMut(Self::Value)) {
        self.as_slice()
            .iter()
            .enumerate()
            .for_each(|(index, elem)| {
                elem.propagate(|v| propagator((index, v)));
            });
    }
}

unsafe impl Bag for () {
    type Value = ();

    #[inline]
    fn build() -> Self {}

    #[inline]
    fn is_empty(&self) -> bool {
        true
    }

    #[inline]
    fn clear(&mut self) {}

    #[inline]
    fn add_value(&mut self, _event: Self::Value) -> Result<(), Self::Value> {
        Ok(())
    }

    #[inline]
    fn len(&self) -> usize {
        0
    }

    #[inline]
    fn add_values(&mut self, _events: &[Self::Value]) -> Result<(), Self::Value> {
        Ok(())
    }

    #[inline]
    fn couple<B: Bag<Value = Self::Value>>(&self, _to: &mut B) -> Result<(), Self::Value>
    where
        Self::Value: Clone,
    {
        Ok(())
    }

    #[inline]
    fn adapt_and_couple<B: Bag, F>(&self, _to: &mut B, _adapter: F) -> Result<(), B::Value>
    where
        F: FnMut(Self::Value) -> B::Value,
    {
        Ok(())
    }

    #[inline]
    fn propagate(&self, _propagator: impl FnMut(Self::Value)) {}
}

unsafe impl<T: Clone> Bag for Option<T> {
    type Value = T;

    #[inline]
    fn build() -> Self {
        None
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.is_none()
    }

    #[inline]
    fn clear(&mut self) {
        *self = None;
    }

    #[inline]
    fn add_value(&mut self, event: Self::Value) -> Result<(), Self::Value> {
        match self {
            Some(_) => Err(event),
            None => {
                *self = Some(event);
                Ok(())
            }
        }
    }

    #[inline]
    fn len(&self) -> usize {
        if self.is_some() {
            1
        } else {
            0
        }
    }

    #[inline]
    fn propagate(&self, mut propagator: impl FnMut(Self::Value)) {
        if let Some(value) = self {
            propagator(value.clone());
        }
    }
}

macro_rules! impl_bag_for_tuple {
    ($($idx:tt => $T:ident),+) => {
        unsafe impl<$($T: Bag),+> Bag for ($($T,)+) {
            type Value = ($(Option<$T::Value>,)+);

            #[inline]
            fn build() -> Self {
                ($($T::build(),)+)
            }

            #[inline]
            fn is_empty(&self) -> bool {
                let mut empty = true;
                $(empty = empty && self.$idx.is_empty();)+
                empty
            }

            #[inline]
            fn clear(&mut self) {
                $(self.$idx.clear();)+
            }

            #[inline]
            fn add_value(&mut self, event: Self::Value) -> Result<(), Self::Value> {
                let mut event = event;
                let mut had_error = false;
                $(
                    if let Some(v) = event.$idx.take() {
                        if let Err(e) = self.$idx.add_value(v) {
                            event.$idx = Some(e);
                            had_error = true;
                        }
                    }
                )+
                if had_error { Err(event) } else { Ok(()) }
            }

            #[inline]
            fn len(&self) -> usize {
                let mut len = 0;
                $(len += self.$idx.len();)+
                len
            }

            #[inline]
            fn propagate(&self, mut propagator: impl FnMut(Self::Value)) {
                $(
                    self.$idx.propagate(|v| {
                        let mut mux: Self::Value = Default::default();
                        mux.$idx = Some(v);
                        propagator(mux);
                    });
                )+
            }
        }
    }
}

impl_bag_for_tuple!(0 => T0);
impl_bag_for_tuple!(0 => T0, 1 => T1);
impl_bag_for_tuple!(0 => T0, 1 => T1, 2 => T2);
impl_bag_for_tuple!(0 => T0, 1 => T1, 2 => T2, 3 => T3);
impl_bag_for_tuple!(0 => T0, 1 => T1, 2 => T2, 3 => T3, 4 => T4);
impl_bag_for_tuple!(0 => T0, 1 => T1, 2 => T2, 3 => T3, 4 => T4, 5 => T5);
impl_bag_for_tuple!(0 => T0, 1 => T1, 2 => T2, 3 => T3, 4 => T4, 5 => T5, 6 => T6);
impl_bag_for_tuple!(0 => T0, 1 => T1, 2 => T2, 3 => T3, 4 => T4, 5 => T5, 6 => T6, 7 => T7);
impl_bag_for_tuple!(0 => T0, 1 => T1, 2 => T2, 3 => T3, 4 => T4, 5 => T5, 6 => T6, 7 => T7, 8 => T8);
impl_bag_for_tuple!(0 => T0, 1 => T1, 2 => T2, 3 => T3, 4 => T4, 5 => T5, 6 => T6, 7 => T7, 8 => T8, 9 => T9);
impl_bag_for_tuple!(0 => T0, 1 => T1, 2 => T2, 3 => T3, 4 => T4, 5 => T5, 6 => T6, 7 => T7, 8 => T8, 9 => T9, 10 => T10);
impl_bag_for_tuple!(0 => T0, 1 => T1, 2 => T2, 3 => T3, 4 => T4, 5 => T5, 6 => T6, 7 => T7, 8 => T8, 9 => T9, 10 => T10, 11 => T11);

#[cfg(test)]
mod tests {
    use super::*;

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
    fn port_clear_empties() {
        let mut port: Port<u32, 5> = Port::new();
        port.add_value(99).unwrap();
        assert!(!port.is_empty());
        port.clear();
        assert!(port.is_empty());
        assert_eq!(port.len(), 0);
    }

    #[test]
    fn port_couple_copies_values() {
        let mut src: Port<u32, 5> = Port::new();
        src.add_values(&[1, 2, 3]).unwrap();
        let mut dst: Port<u32, 5> = Port::new();
        assert!(src.couple(&mut dst).is_ok());
        assert_eq!(dst.as_slice(), &[1, 2, 3]);
        assert_eq!(src.as_slice(), &[1, 2, 3]);
    }

    #[test]
    fn port_couple_capacity_error() {
        let mut src: Port<u32, 5> = Port::new();
        src.add_values(&[1, 2, 3]).unwrap();
        let mut dst: Port<u32, 2> = Port::new();
        let result = src.couple(&mut dst);
        assert!(result.is_err());
        assert_eq!(src.as_slice(), &[1, 2, 3]);
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
    fn port_bag_impl_contract() {
        let mut bag = <Port<u32, 5> as Bag>::build();
        assert!(bag.is_empty());

        bag.add_value(7).unwrap();
        assert!(!bag.is_empty());

        bag.clear();
        assert!(bag.is_empty());
    }

    #[test]
    fn array_bag_impl_contract() {
        let mut bags = <[Port<u32, 1>; 3] as Bag>::build();
        assert!(bags.is_empty());

        bags[0].add_value(1).unwrap();
        assert!(!bags.is_empty());

        bags[1].add_value(2).unwrap();
        assert!(!bags.is_empty());

        bags.clear();
        assert!(bags.is_empty());
    }

    #[test]
    fn port_bag_inject_eject_contract() {
        let mut bag = <Port<u32, 2> as Bag>::build();
        assert!(bag.is_empty());

        assert!(bag.add_value(7).is_ok());
        assert!(bag.add_value(99).is_ok());
        assert_eq!(bag.add_value(42), Err(42));

        let mut collected: heapless::Vec<u32, 4> = heapless::Vec::new();
        bag.propagate(|v| {
            let _ = collected.push(v);
        });
        assert_eq!(collected.as_slice(), &[7, 99]);
    }

    #[test]
    fn array_bag_inject_eject_contract() {
        let mut bags = <[Port<u32, 2>; 3] as Bag>::build();
        assert!(bags.is_empty());

        assert!(bags.add_value((0, 10)).is_ok());
        assert!(bags.add_value((2, 30)).is_ok());

        assert_eq!(bags.add_value((5, 77)), Err((5, 77)));

        assert!(bags.add_value((0, 11)).is_ok());
        assert_eq!(bags.add_value((0, 12)), Err((0, 12)));

        let mut collected: heapless::Vec<(usize, u32), 4> = heapless::Vec::new();
        bags.propagate(|(i, v)| {
            let _ = collected.push((i, v));
        });
        assert_eq!(collected.as_slice(), &[(0, 10), (0, 11), (2, 30)]);
    }

    #[test]
    fn option_bag_impl_contract() {
        let mut bag = <Option<u32> as Bag>::build();
        assert!(bag.is_empty());

        assert!(bag.add_value(7).is_ok());
        assert!(!bag.is_empty());

        bag.clear();
        assert!(bag.is_empty());
    }

    #[test]
    fn option_bag_inject_eject_contract() {
        let mut bag = <Option<u32> as Bag>::build();
        assert!(bag.is_empty());

        assert!(bag.add_value(7).is_ok());
        assert_eq!(bag.add_value(99), Err(99));

        let mut collected: heapless::Vec<u32, 4> = heapless::Vec::new();
        bag.propagate(|v| {
            let _ = collected.push(v);
        });
        assert_eq!(collected.as_slice(), &[7]);

        bag.clear();
        assert!(bag.is_empty());

        let mut collected_after: heapless::Vec<u32, 4> = heapless::Vec::new();
        bag.propagate(|v| {
            let _ = collected_after.push(v);
        });
        assert!(collected_after.is_empty());
    }

    #[test]
    fn tuple_bag_inject_eject_2_elements() {
        let mut bag = <(Port<u32, 1>, Port<bool, 1>) as Bag>::build();
        assert!(bag.is_empty());

        assert!(bag.add_value((Some(7u32), None)).is_ok());
        assert!(bag.add_value((None, Some(true))).is_ok());
        assert_eq!(bag.add_value((Some(99u32), None)), Err((Some(99), None)));

        let mut got_u32: heapless::Vec<u32, 4> = heapless::Vec::new();
        let mut got_bool: heapless::Vec<bool, 4> = heapless::Vec::new();
        bag.propagate(|ev| match ev {
            (Some(v), None) => {
                let _ = got_u32.push(v);
            }
            (None, Some(v)) => {
                let _ = got_bool.push(v);
            }
            _ => {}
        });
        assert_eq!(got_u32.as_slice(), &[7]);
        assert_eq!(got_bool.as_slice(), &[true]);
    }

    #[test]
    fn tuple_bag_inject_eject_full_preserves_failed_positions() {
        let mut bag = <(Port<u32, 1>, Port<bool, 1>) as Bag>::build();
        bag.add_value((Some(1u32), None)).unwrap();
        bag.add_value((None, Some(true))).unwrap();

        assert_eq!(bag.add_value((Some(7u32), None)), Err((Some(7), None)));
        assert_eq!(bag.add_value((None, Some(false))), Err((None, Some(false))));
        assert_eq!(
            bag.add_value((Some(7u32), Some(false))),
            Err((Some(7), Some(false)))
        );

        let mut got_u32: heapless::Vec<u32, 4> = heapless::Vec::new();
        let mut got_bool: heapless::Vec<bool, 4> = heapless::Vec::new();
        bag.propagate(|ev| match ev {
            (Some(v), None) => {
                let _ = got_u32.push(v);
            }
            (None, Some(v)) => {
                let _ = got_bool.push(v);
            }
            _ => {}
        });
        assert_eq!(got_u32.as_slice(), &[1]);
        assert_eq!(got_bool.as_slice(), &[true]);
    }

    #[test]
    fn unit_bag_impl() {
        <() as Bag>::build();
        assert!(<() as Bag>::is_empty(&()));
        <() as Bag>::clear(&mut ());
    }

    #[test]
    fn tuple_bag_impl_2_elements() {
        let mut bag = <(Port<u32, 1>, Port<bool, 1>) as Bag>::build();
        assert!(bag.is_empty());

        bag.0.add_value(99).unwrap();
        assert!(!bag.is_empty());

        bag.1.add_value(true).unwrap();
        assert!(!bag.is_empty());

        bag.clear();
        assert!(bag.is_empty());
        assert!(bag.0.is_empty() && bag.1.is_empty());
    }

    #[test]
    fn tuple_bag_impl_3_elements() {
        let mut bag = <(Port<u32, 1>, Port<u32, 1>, Port<u32, 1>) as Bag>::build();
        assert!(bag.is_empty());

        bag.0.add_value(99).unwrap();
        assert!(!bag.is_empty());

        bag.1.add_value(99).unwrap();
        assert!(!bag.is_empty());

        bag.2.add_value(99).unwrap();
        assert!(!bag.is_empty());

        bag.clear();
        assert!(bag.is_empty());
        assert!(bag.0.is_empty() && bag.1.is_empty() && bag.2.is_empty());
    }

    #[test]
    fn port_default_creates_empty() {
        let port: Port<u32, 5> = Default::default();
        assert!(port.is_empty());
        assert_eq!(port.len(), 0);
    }

    #[derive(crate::Bag)]
    struct InnerBag {
        a: Port<u32, 2>,
    }

    #[derive(crate::Bag)]
    struct OuterBag {
        inner: InnerBag,
        b: Port<bool, 1>,
    }

    #[test]
    fn nested_bag_impl() {
        let mut outer = <OuterBag as Bag>::build();
        assert!(outer.is_empty());
        assert_eq!(outer.len(), 0);

        let inner_event = _xdevs_no_std_inner_bag_bag::PortMux::A(42u32);
        let outer_inner = _xdevs_no_std_outer_bag_bag::PortMux::Inner(inner_event);
        assert!(outer.add_value(outer_inner).is_ok());
        assert!(!outer.is_empty());
        assert_eq!(outer.len(), 1);

        assert!(outer
            .add_value(_xdevs_no_std_outer_bag_bag::PortMux::B(true))
            .is_ok());
        assert_eq!(outer.len(), 2);

        let mut got_a: heapless::Vec<u32, 4> = heapless::Vec::new();
        let mut got_b: heapless::Vec<bool, 4> = heapless::Vec::new();
        outer.propagate(|ev| match ev {
            _xdevs_no_std_outer_bag_bag::PortMux::Inner(inner) => match inner {
                _xdevs_no_std_inner_bag_bag::PortMux::A(v) => {
                    let _ = got_a.push(v);
                }
            },
            _xdevs_no_std_outer_bag_bag::PortMux::B(v) => {
                let _ = got_b.push(v);
            }
        });

        assert_eq!(got_a.as_slice(), &[42]);
        assert_eq!(got_b.as_slice(), &[true]);
    }
}
