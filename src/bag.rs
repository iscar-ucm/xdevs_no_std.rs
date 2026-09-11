#[cfg(feature = "alloc")]
mod alloc_impl;
mod heapless_impl;

/// Trait that defines the methods that a DEVS event bag set must implement.
///
/// # Safety
///
/// This trait must be implemented via the [`macro@crate::Bag`] macro. Do not implement it manually.
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

    /// Adds multiple values to the bag.
    ///
    /// Returns the first event that cannot be inserted; any events not yet
    /// consumed from the iterator are dropped.
    #[inline(always)]
    fn add_values(
        &mut self,
        events: impl IntoIterator<Item = Self::Value>,
    ) -> Result<(), Self::Value> {
        for event in events {
            self.add_value(event)?;
        }
        Ok(())
    }

    /// Returns an iterator over the events currently stored in the bag.
    ///
    /// Events are yielded lazily, one at a time.
    ///
    /// # Note
    ///
    /// Each value is cloned out of the bag, so the bag is left untouched.
    /// Collections that implement [`Bag`] (e.g., [`crate::Port`], which is a
    /// [`heapless::Vec`]) usually provide inherent methods that iterate over
    /// values without cloning them (e.g., `iter`, `as_slice`). Prefer those in
    /// your models when performance matters.
    fn get_values(&self) -> impl Iterator<Item = Self::Value> + '_;
}

/// Copies all events from one bag into another bag of the same event type.
///
/// Returns the first event that cannot be inserted into `to`.
pub fn couple<S: Bag, B: Bag<Value = S::Value>>(from: &S, to: &mut B) -> Result<(), S::Value> {
    to.add_values(from.get_values())
}

/// Copies all events from one bag into another bag using an adapter closure.
///
/// The adapter transforms each source event into the target bag event type.
/// Returns the first adapted event that cannot be inserted into `to`.
pub fn adapt_and_couple<S: Bag, B: Bag, F>(from: &S, to: &mut B, adapter: F) -> Result<(), B::Value>
where
    F: FnMut(S::Value) -> B::Value,
{
    to.add_values(from.get_values().map(adapter))
}

/// Propagates all events from a bag according to the provided closure.
pub fn propagate<S: Bag>(from: &S, propagator: impl FnMut(S::Value)) {
    from.get_values().for_each(propagator)
}

// Implement `Bag` for the unit type `()`, which represents an empty bag.
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
    fn get_values(&self) -> impl Iterator<Item = Self::Value> + '_ {
        core::iter::empty()
    }

    #[inline]
    fn add_values(
        &mut self,
        _events: impl IntoIterator<Item = Self::Value>,
    ) -> Result<(), Self::Value> {
        Ok(())
    }
}

// Implement `Bag` for `Option<T>`, which can hold at most one value of type `T`.
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
    fn get_values(&self) -> impl Iterator<Item = Self::Value> + '_ {
        self.iter().cloned()
    }
}

// Implement `Bag` for arrays of bags, where each element is a bag of the same type.
unsafe impl<T: Bag, const N: usize> Bag for [T; N] {
    type Value = (usize, T::Value); // Include index to identify which bag the value came from

    #[inline]
    fn build() -> Self {
        core::array::from_fn(|_| T::build())
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.iter().all(|bag| bag.is_empty())
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
    fn get_values(&self) -> impl Iterator<Item = Self::Value> + '_ {
        self.iter()
            .enumerate()
            .flat_map(|(index, elem)| elem.get_values().map(move |v| (index, v)))
    }
}

// Implement `Bag` for tuples of bags, where each element is a bag of potentially different types.

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

            /// Tuple positions are inserted independently: on partial failure,
            /// the returned error contains only the values not inserted, while
            /// successfully inserted values remain in the bag.
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
            fn get_values(&self) -> impl Iterator<Item = Self::Value> + '_ {
                core::iter::empty::<Self::Value>()
                $(
                    .chain(self.$idx.get_values().map(|v| {
                        let mut mux: Self::Value = Default::default();
                        mux.$idx = Some(v);
                        mux
                    }))
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
// We could go further, but 12-element tuples are already quite large and rare in practice.

#[cfg(test)]
mod tests {
    use crate::{bag::propagate, *};

    #[test]
    fn port_couple_copies_values() {
        let mut src: Port<u32, 5> = Port::new();
        src.add_values([1, 2, 3]).unwrap();
        let mut dst: Port<u32, 5> = Port::new();
        assert!(couple(&src, &mut dst).is_ok());
        assert_eq!(dst.as_slice(), &[1, 2, 3]);
        assert_eq!(src.as_slice(), &[1, 2, 3]);
    }

    #[test]
    fn port_couple_capacity_error() {
        let mut src: Port<u32, 5> = Port::new();
        src.add_values([1, 2, 3]).unwrap();
        let mut dst: Port<u32, 2> = Port::new();
        let result = couple(&src, &mut dst);
        assert!(result.is_err());
        assert_eq!(src.as_slice(), &[1, 2, 3]);
    }

    #[test]
    fn port_adapt_and_couple_transforms_values() {
        let mut src: Port<u32, 5> = Port::new();
        src.add_values([1, 2, 3]).unwrap();
        let mut dst: Port<u64, 5> = Port::new();
        // Adapter doubles each value and widens to u64.
        assert!(adapt_and_couple(&src, &mut dst, |v| v as u64 * 2).is_ok());
        assert_eq!(dst.as_slice(), &[2, 4, 6]);
        // Source is unchanged.
        assert_eq!(src.as_slice(), &[1, 2, 3]);
    }

    #[test]
    fn port_adapt_and_couple_capacity_error() {
        let mut src: Port<u32, 5> = Port::new();
        src.add_values([1, 2, 3]).unwrap();
        let mut dst: Port<u64, 2> = Port::new();
        let result = adapt_and_couple(&src, &mut dst, |v| v as u64 * 2);
        // The third adapted event (6) cannot be inserted.
        assert_eq!(result, Err(6));
        // The first two events were inserted before the failure.
        assert_eq!(dst.as_slice(), &[2, 4]);
        assert_eq!(src.as_slice(), &[1, 2, 3]);
    }

    #[test]
    fn port_adapt_and_couple_empty_source() {
        let src: Port<u32, 5> = Port::new();
        let mut dst: Port<u64, 5> = Port::new();
        assert!(adapt_and_couple(&src, &mut dst, |v| v as u64 * 2).is_ok());
        assert!(dst.is_empty());
    }

    #[test]
    fn port_adapt_and_couple_type_conversion() {
        let mut src: Port<u32, 5> = Port::new();
        src.add_values([0, 1, 2]).unwrap();
        let mut dst: Port<bool, 5> = Port::new();
        // Adapter converts non-zero to true.
        assert!(adapt_and_couple(&src, &mut dst, |v| v != 0).is_ok());
        assert_eq!(dst.as_slice(), &[false, true, true]);
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
    fn array_bag_inject_eject_contract() {
        let mut bags = <[Port<u32, 2>; 3] as Bag>::build();
        assert!(bags.is_empty());

        assert!(bags.add_value((0, 10)).is_ok());
        assert!(bags.add_value((2, 30)).is_ok());

        assert_eq!(bags.add_value((5, 77)), Err((5, 77)));

        assert!(bags.add_value((0, 11)).is_ok());
        assert_eq!(bags.add_value((0, 12)), Err((0, 12)));

        let mut collected: heapless::Vec<(usize, u32), 4> = heapless::Vec::new();
        propagate(&bags, |(i, v)| {
            let _ = collected.push((i, v));
        });
        assert_eq!(collected.as_slice(), &[(0, 10), (0, 11), (2, 30)]);
    }

    #[test]
    fn array_get_values_yields_indexed_events() {
        let mut bags = <[Port<u32, 2>; 2] as Bag>::build();
        bags.add_value((0, 10)).unwrap();
        bags.add_value((1, 20)).unwrap();
        bags.add_value((0, 11)).unwrap();
        let mut collected: ::heapless::Vec<(usize, u32), 4> = ::heapless::Vec::new();
        for ev in bags.get_values() {
            let _ = collected.push(ev);
        }
        assert_eq!(collected.as_slice(), &[(0, 10), (0, 11), (1, 20)]);
    }

    #[test]
    fn array_couple_to_port() {
        let mut src = <[Port<u32, 2>; 2] as Bag>::build();
        src.add_value((0, 10)).unwrap();
        src.add_value((1, 20)).unwrap();
        src.add_value((0, 11)).unwrap();
        // Target is a Port of (index, value) tuples — same Value type.
        let mut dst: Port<(usize, u32), 5> = Port::new();
        assert!(couple(&src, &mut dst).is_ok());
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
        let result = couple(&src, &mut dst);
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
        assert!(couple(&src, &mut dst).is_ok());
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
        assert!(adapt_and_couple(&src, &mut dst, |v| v).is_ok());
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
    fn option_bag_inject_eject_contract() {
        let mut bag = <Option<u32> as Bag>::build();
        assert!(bag.is_empty());

        assert!(bag.add_value(7).is_ok());
        assert_eq!(bag.add_value(99), Err(99));

        let mut collected: heapless::Vec<u32, 4> = heapless::Vec::new();
        propagate(&bag, |v| {
            let _ = collected.push(v);
        });
        assert_eq!(collected.as_slice(), &[7]);

        bag.clear();
        assert!(bag.is_empty());

        let mut collected_after: heapless::Vec<u32, 4> = heapless::Vec::new();
        propagate(&bag, |v| {
            let _ = collected_after.push(v);
        });
        assert!(collected_after.is_empty());
    }

    #[test]
    fn option_get_values_yields_single_event() {
        let mut bag = <Option<u32> as Bag>::build();
        assert_eq!(bag.get_values().count(), 0);

        bag.add_value(7).unwrap();
        assert_eq!(
            bag.get_values()
                .collect::<::heapless::Vec<u32, 2>>()
                .as_slice(),
            &[7]
        );

        bag.clear();
        assert_eq!(bag.get_values().count(), 0);
    }

    #[test]
    fn option_couple_to_port() {
        let mut src = <Option<u32> as Bag>::build();
        src.add_value(42).unwrap();
        let mut dst: Port<u32, 5> = Port::new();
        assert!(couple(&src, &mut dst).is_ok());
        assert_eq!(dst.as_slice(), &[42]);
    }

    #[test]
    fn option_couple_capacity_error() {
        let mut src = <Option<u32> as Bag>::build();
        src.add_value(42).unwrap();
        // Target is a full Port — the single event cannot be inserted.
        let mut dst: Port<u32, 1> = Port::new();
        dst.add_value(99).unwrap();
        let result = couple(&src, &mut dst);
        assert_eq!(result, Err(42));
        assert_eq!(dst.as_slice(), &[99]);
    }

    #[test]
    fn option_couple_empty_is_noop() {
        let src = <Option<u32> as Bag>::build();
        let mut dst: Port<u32, 5> = Port::new();
        assert!(couple(&src, &mut dst).is_ok());
        assert!(dst.is_empty());
    }

    #[test]
    fn option_adapt_and_couple_to_port() {
        let mut src = <Option<u32> as Bag>::build();
        src.add_value(42).unwrap();
        let mut dst: Port<u64, 5> = Port::new();
        assert!(adapt_and_couple(&src, &mut dst, |v| v as u64 + 100).is_ok());
        assert_eq!(dst.as_slice(), &[142]);
    }

    #[test]
    fn option_adapt_and_couple_empty_is_noop() {
        let src = <Option<u32> as Bag>::build();
        let mut dst: Port<u64, 5> = Port::new();
        assert!(adapt_and_couple(&src, &mut dst, |v| v as u64).is_ok());
        assert!(dst.is_empty());
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
        propagate(&bag, |ev| match ev {
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
        propagate(&bag, |ev| match ev {
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
    fn tuple_get_values_yields_muxed_events() {
        let mut bag = <(Port<u32, 2>, Port<bool, 2>) as Bag>::build();
        bag.add_value((Some(7), None)).unwrap();
        bag.add_value((None, Some(true))).unwrap();
        let mut got: ::heapless::Vec<(Option<u32>, Option<bool>), 4> = ::heapless::Vec::new();
        for ev in bag.get_values() {
            let _ = got.push(ev);
        }
        assert_eq!(got.as_slice(), &[(Some(7), None), (None, Some(true))]);
    }

    #[test]
    fn tuple_couple_to_port() {
        let mut src = <(Port<u32, 2>, Port<bool, 2>) as Bag>::build();
        src.add_value((Some(7), None)).unwrap();
        src.add_value((None, Some(true))).unwrap();
        // Target is a Port of the mux value type — same Value type.
        let mut dst: Port<(Option<u32>, Option<bool>), 5> = Port::new();
        assert!(couple(&src, &mut dst).is_ok());
        assert_eq!(dst.as_slice(), &[(Some(7), None), (None, Some(true))]);
    }

    #[test]
    fn tuple_couple_capacity_error() {
        let mut src = <(Port<u32, 2>, Port<bool, 2>) as Bag>::build();
        src.add_value((Some(7), None)).unwrap();
        src.add_value((None, Some(true))).unwrap();
        // Target too small: only fits 1 of the 2 events.
        let mut dst: Port<(Option<u32>, Option<bool>), 1> = Port::new();
        let result = couple(&src, &mut dst);
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
        assert!(couple(&src, &mut dst).is_ok());
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
        assert!(adapt_and_couple(&src, &mut dst, |v| v).is_ok());
        assert_eq!(dst.as_slice(), &[(Some(7), None), (None, Some(true))]);
    }

    #[test]
    fn unit_bag_impl() {
        <() as Bag>::build();
        assert!(<() as Bag>::is_empty(&()));
        assert_eq!(<() as Bag>::get_values(&()).count(), 0);
        <() as Bag>::clear(&mut ());
        assert!(<() as Bag>::add_value(&mut (), ()).is_ok());
        assert!(<() as Bag>::add_values(&mut (), [(), ()]).is_ok());
    }

    #[test]
    fn unit_propagate_never_invokes_closure() {
        // () has no events, so the propagator closure must never be called.
        let mut called = false;
        propagate(&(), |_| called = true);
        assert!(!called, "propagate on () must not invoke the closure");
    }

    #[test]
    fn unit_couple_to_unit() {
        // () has no events, so coupling to another () is always Ok.
        let result = couple(&(), &mut ());
        assert!(result.is_ok());
    }

    #[test]
    fn unit_couple_to_port() {
        // () has no events, so coupling to a Port<()> is a no-op.
        let mut dst: Port<(), 5> = Port::new();
        assert!(couple(&(), &mut dst).is_ok());
        assert!(dst.is_empty());
    }

    #[test]
    fn unit_adapt_and_couple_to_port() {
        // () has no events, so adapt_and_couple is a no-op regardless of adapter.
        let mut dst: Port<u32, 5> = Port::new();
        assert!(adapt_and_couple(&(), &mut dst, |_| 42u32).is_ok());
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

        let inner_event = _xdevs_no_std_inner_bag_bag::PortMux::A(42u32);
        let outer_inner = _xdevs_no_std_outer_bag_bag::PortMux::Inner(inner_event);
        assert!(outer.add_value(outer_inner).is_ok());
        assert!(!outer.is_empty());

        assert!(outer
            .add_value(_xdevs_no_std_outer_bag_bag::PortMux::B(true))
            .is_ok());

        let mut got_a: heapless::Vec<u32, 4> = heapless::Vec::new();
        let mut got_b: heapless::Vec<bool, 4> = heapless::Vec::new();
        propagate(&outer, |ev| match ev {
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
