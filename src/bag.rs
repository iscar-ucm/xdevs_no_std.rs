#[cfg(feature = "alloc")]
pub mod alloc;
pub mod heapless;
#[cfg(feature = "std")]
pub mod std;

/// Port is an alias for a heapless::Vec.
pub type Port<T, const N: usize> = ::heapless::Vec<T, N>;

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
    fn couple<B: Bag<Value = Self::Value>>(&self, _to: &mut B) -> Result<(), Self::Value> {
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
    fn array_len_sums_sub_bags() {
        let mut bags = <[Port<u32, 3>; 3] as Bag>::build();
        assert_eq!(bags.len(), 0);

        bags.add_value((0, 10)).unwrap();
        assert_eq!(bags.len(), 1);

        bags.add_value((2, 30)).unwrap();
        bags.add_value((2, 31)).unwrap();
        assert_eq!(bags.len(), 3);

        // Clearing one sub-bag reduces the total.
        bags[2].clear();
        assert_eq!(bags.len(), 1);

        bags.clear();
        assert_eq!(bags.len(), 0);
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

        let mut collected: ::heapless::Vec<(usize, u32), 4> = ::heapless::Vec::new();
        bags.propagate(|(i, v)| {
            let _ = collected.push((i, v));
        });
        assert_eq!(collected.as_slice(), &[(0, 10), (0, 11), (2, 30)]);
    }

    #[test]
    fn array_couple_to_port() {
        let mut src = <[Port<u32, 2>; 2] as Bag>::build();
        src.add_value((0, 10)).unwrap();
        src.add_value((1, 20)).unwrap();
        src.add_value((0, 11)).unwrap();
        // Target is a Port of (index, value) tuples — same Value type.
        let mut dst: Port<(usize, u32), 5> = Port::new();
        assert!(src.couple(&mut dst).is_ok());
        assert_eq!(dst.as_slice(), &[(0, 10), (0, 11), (1, 20)]);
    }

    #[test]
    fn array_couple_capacity_error() {
        let mut src = <[Port<u32, 2>; 2] as Bag>::build();
        src.add_value((0, 10)).unwrap();
        src.add_value((1, 20)).unwrap();
        src.add_value((0, 11)).unwrap();
        // Target too small: only fits 2 of the 3 events.
        let mut dst: Port<(usize, u32), 2> = Port::new();
        let result = src.couple(&mut dst);
        // The third event that didn't fit is returned.
        assert_eq!(result, Err((1, 20)));
        assert_eq!(dst.as_slice(), &[(0, 10), (0, 11)]);
    }

    #[test]
    fn array_couple_to_array() {
        let mut src = <[Port<u32, 2>; 2] as Bag>::build();
        src.add_value((0, 10)).unwrap();
        src.add_value((1, 20)).unwrap();
        let mut dst = <[Port<u32, 2>; 2] as Bag>::build();
        assert!(src.couple(&mut dst).is_ok());
        assert_eq!(dst[0].as_slice(), &[10]);
        assert_eq!(dst[1].as_slice(), &[20]);
    }

    #[test]
    fn array_adapt_and_couple_to_port() {
        let mut src = <[Port<u32, 2>; 2] as Bag>::build();
        src.add_value((0, 10)).unwrap();
        src.add_value((1, 20)).unwrap();
        // Target is a Port of (index, value) tuples.
        let mut dst: Port<(usize, u32), 5> = Port::new();
        assert!(src.adapt_and_couple(&mut dst, |v| v).is_ok());
        assert_eq!(dst.as_slice(), &[(0, 10), (1, 20)]);
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
    fn option_len_tracks_presence() {
        let mut bag = <Option<u32> as Bag>::build();
        assert_eq!(bag.len(), 0);

        bag.add_value(7).unwrap();
        assert_eq!(bag.len(), 1);

        // A second value is rejected; len stays at 1.
        assert_eq!(bag.add_value(99), Err(99));
        assert_eq!(bag.len(), 1);

        bag.clear();
        assert_eq!(bag.len(), 0);
    }

    #[test]
    fn option_bag_inject_eject_contract() {
        let mut bag = <Option<u32> as Bag>::build();
        assert!(bag.is_empty());

        assert!(bag.add_value(7).is_ok());
        assert_eq!(bag.add_value(99), Err(99));

        let mut collected: ::heapless::Vec<u32, 4> = ::heapless::Vec::new();
        bag.propagate(|v| {
            let _ = collected.push(v);
        });
        assert_eq!(collected.as_slice(), &[7]);

        bag.clear();
        assert!(bag.is_empty());

        let mut collected_after: ::heapless::Vec<u32, 4> = ::heapless::Vec::new();
        bag.propagate(|v| {
            let _ = collected_after.push(v);
        });
        assert!(collected_after.is_empty());
    }

    #[test]
    fn option_couple_to_port() {
        let mut src = <Option<u32> as Bag>::build();
        src.add_value(42).unwrap();
        let mut dst: Port<u32, 5> = Port::new();
        assert!(src.couple(&mut dst).is_ok());
        assert_eq!(dst.as_slice(), &[42]);
    }

    #[test]
    fn option_couple_capacity_error() {
        let mut src = <Option<u32> as Bag>::build();
        src.add_value(42).unwrap();
        // Target is a full Port — the single event cannot be inserted.
        let mut dst: Port<u32, 1> = Port::new();
        dst.add_value(99).unwrap();
        let result = src.couple(&mut dst);
        assert_eq!(result, Err(42));
        assert_eq!(dst.as_slice(), &[99]);
    }

    #[test]
    fn option_couple_empty_is_noop() {
        let src = <Option<u32> as Bag>::build();
        let mut dst: Port<u32, 5> = Port::new();
        assert!(src.couple(&mut dst).is_ok());
        assert!(dst.is_empty());
    }

    #[test]
    fn option_adapt_and_couple_to_port() {
        let mut src = <Option<u32> as Bag>::build();
        src.add_value(42).unwrap();
        let mut dst: Port<u64, 5> = Port::new();
        assert!(src.adapt_and_couple(&mut dst, |v| v as u64 + 100).is_ok());
        assert_eq!(dst.as_slice(), &[142]);
    }

    #[test]
    fn option_adapt_and_couple_empty_is_noop() {
        let src = <Option<u32> as Bag>::build();
        let mut dst: Port<u64, 5> = Port::new();
        assert!(src.adapt_and_couple(&mut dst, |v| v as u64).is_ok());
        assert!(dst.is_empty());
    }

    #[test]
    fn tuple_len_sums_elements() {
        let mut bag = <(Port<u32, 3>, Port<bool, 3>) as Bag>::build();
        assert_eq!(bag.len(), 0);

        bag.add_value((Some(1), None)).unwrap();
        bag.add_value((Some(2), None)).unwrap();
        bag.add_value((None, Some(true))).unwrap();
        assert_eq!(bag.len(), 3);

        // Clearing reduces the total.
        bag.0.clear();
        assert_eq!(bag.len(), 1);

        bag.clear();
        assert_eq!(bag.len(), 0);
    }

    #[test]
    fn tuple_bag_inject_eject_2_elements() {
        let mut bag = <(Port<u32, 1>, Port<bool, 1>) as Bag>::build();
        assert!(bag.is_empty());

        assert!(bag.add_value((Some(7u32), None)).is_ok());
        assert!(bag.add_value((None, Some(true))).is_ok());
        assert_eq!(bag.add_value((Some(99u32), None)), Err((Some(99), None)));

        let mut got_u32: ::heapless::Vec<u32, 4> = ::heapless::Vec::new();
        let mut got_bool: ::heapless::Vec<bool, 4> = ::heapless::Vec::new();
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

        let mut got_u32: ::heapless::Vec<u32, 4> = ::heapless::Vec::new();
        let mut got_bool: ::heapless::Vec<bool, 4> = ::heapless::Vec::new();
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
    fn tuple_couple_to_port() {
        let mut src = <(Port<u32, 2>, Port<bool, 2>) as Bag>::build();
        src.add_value((Some(7), None)).unwrap();
        src.add_value((None, Some(true))).unwrap();
        // Target is a Port of the mux value type — same Value type.
        let mut dst: Port<(Option<u32>, Option<bool>), 5> = Port::new();
        assert!(src.couple(&mut dst).is_ok());
        assert_eq!(dst.as_slice(), &[(Some(7), None), (None, Some(true))]);
    }

    #[test]
    fn tuple_couple_capacity_error() {
        let mut src = <(Port<u32, 2>, Port<bool, 2>) as Bag>::build();
        src.add_value((Some(7), None)).unwrap();
        src.add_value((None, Some(true))).unwrap();
        // Target too small: only fits 1 of the 2 events.
        let mut dst: Port<(Option<u32>, Option<bool>), 1> = Port::new();
        let result = src.couple(&mut dst);
        // The second event that didn't fit is returned.
        assert_eq!(result, Err((None, Some(true))));
        assert_eq!(dst.as_slice(), &[(Some(7), None)]);
    }

    #[test]
    fn tuple_couple_to_tuple() {
        let mut src = <(Port<u32, 2>, Port<bool, 2>) as Bag>::build();
        src.add_value((Some(7), None)).unwrap();
        src.add_value((None, Some(true))).unwrap();
        let mut dst = <(Port<u32, 2>, Port<bool, 2>) as Bag>::build();
        assert!(src.couple(&mut dst).is_ok());
        assert_eq!(dst.0.as_slice(), &[7]);
        assert_eq!(dst.1.as_slice(), &[true]);
    }

    #[test]
    fn tuple_adapt_and_couple_to_port() {
        let mut src = <(Port<u32, 2>, Port<bool, 2>) as Bag>::build();
        src.add_value((Some(7), None)).unwrap();
        src.add_value((None, Some(true))).unwrap();
        // Flatten the mux into a single Port of an enum-like tuple.
        let mut dst: Port<(Option<u32>, Option<bool>), 5> = Port::new();
        assert!(src.adapt_and_couple(&mut dst, |v| v).is_ok());
        assert_eq!(dst.as_slice(), &[(Some(7), None), (None, Some(true))]);
    }

    #[test]
    fn unit_bag_impl() {
        <() as Bag>::build();
        assert!(<() as Bag>::is_empty(&()));
        <() as Bag>::clear(&mut ());
        assert!(<() as Bag>::add_value(&mut (), ()).is_ok());
        assert!(<() as Bag>::add_values(&mut (), &[(), ()]).is_ok());
        assert_eq!(<() as Bag>::len(&()), 0);
    }

    #[test]
    fn unit_propagate_never_invokes_closure() {
        // () has no events, so the propagator closure must never be called.
        let mut called = false;
        <() as Bag>::propagate(&(), |_| called = true);
        assert!(!called, "propagate on () must not invoke the closure");
    }

    #[test]
    fn unit_couple_to_unit() {
        // () has no events, so coupling to another () is always Ok.
        let result = <() as Bag>::couple(&(), &mut ());
        assert!(result.is_ok());
    }

    #[test]
    fn unit_couple_to_port() {
        // () has no events, so coupling to a Port<()> is a no-op.
        let mut dst: Port<(), 5> = Port::new();
        assert!(<() as Bag>::couple(&(), &mut dst).is_ok());
        assert!(dst.is_empty());
    }

    #[test]
    fn unit_adapt_and_couple_to_port() {
        // () has no events, so adapt_and_couple is a no-op regardless of adapter.
        let mut dst: Port<u32, 5> = Port::new();
        assert!(<() as Bag>::adapt_and_couple(&(), &mut dst, |_| 42u32).is_ok());
        assert!(dst.is_empty());
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

        let mut got_a: ::heapless::Vec<u32, 4> = ::heapless::Vec::new();
        let mut got_b: ::heapless::Vec<bool, 4> = ::heapless::Vec::new();
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
