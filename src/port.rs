/// Port is a generic structure that can be used to store values of any type `T`.
/// It is the main artifact to exchange data between components.
/// Note that, in `no_std` environments, the capacity of the port `N` must be known at compile time.
#[derive(Debug)]
pub struct Port<T: Clone, const N: usize>(heapless::Vec<T, N>);

impl<T: Clone, const N: usize> Default for Port<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone, const N: usize> Port<T, N> {
    /// Creates a new empty port.
    #[inline]
    pub const fn new() -> Self {
        Self(heapless::Vec::new())
    }

    /// Returns `true` if the port is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns `true` if the port is full.
    #[inline]
    pub fn is_full(&self) -> bool {
        self.0.is_full()
    }

    /// Returns the number of elements in the port.
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Clears the port, removing all values.
    #[inline]
    pub fn clear(&mut self) {
        self.0.clear()
    }

    /// Adds multiple values to the port.
    #[inline]
    pub fn add_values(&mut self, items: &[T]) -> Result<(), heapless::CapacityError> {
        self.0.extend_from_slice(items)
    }

    /// Returns a slice of the port's values.
    #[inline]
    pub fn get_values(&self) -> &[T] {
        self.0.as_slice()
    }

    /// Easy port mapping method
    #[inline]
    pub fn couple<const M: usize>(
        &self,
        to: &mut Port<T, M>,
    ) -> Result<(), heapless::CapacityError> {
        to.add_values(self.get_values())
    }
}

unsafe impl<T: Clone, const N: usize> Bag for Port<T, N> {
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
        self.0.push(event)
    }

    #[inline]
    fn eject_events(&self, mut ejector: impl FnMut(Self::Value)) {
        for value in self.get_values() {
            ejector(value.clone());
        }
    }
}

/// Trait that defines the methods that a DEVS event bag set must implement.
///
/// # Safety
///
/// This trait must be implemented via the [`Bag`] macro. Do not implement it manually.
pub unsafe trait Bag {
    /// The data type of the events stored in the event bag.
    type Value;

    /// Build a new instance of the bag.
    fn build() -> Self;

    /// Returns `true` if the event bag is empty.
    fn is_empty(&self) -> bool;

    /// Clears the event bag, removing all values.
    fn clear(&mut self);

    /// Adds a new value into the bag.
    fn add_value(&mut self, event: Self::Value) -> Result<(), Self::Value>;

    /// Ejects all events from the bag.
    ///
    /// This function is mainly used internally by the [`RtEngine`](crate::rt_engine::RtEngine) to collect all events from the model.
    fn eject_events(&self, ejector: impl FnMut(Self::Value));
}

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
    fn eject_events(&self, mut ejector: impl FnMut(Self::Value)) {
        self.iter().enumerate().for_each(|(index, elem)| {
            elem.eject_events(|v| ejector((index, v)));
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
    fn eject_events(&self, _ejector: impl FnMut(Self::Value)) {}
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
    fn eject_events(&self, mut ejector: impl FnMut(Self::Value)) {
        if let Some(value) = self {
            ejector(value.clone());
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
            fn eject_events(&self, mut ejector: impl FnMut(Self::Value)) {
                $(
                    self.$idx.eject_events(|v| {
                        let mut mux: Self::Value = Default::default();
                        mux.$idx = Some(v);
                        ejector(mux);
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
        assert_eq!(port.get_values(), &[1, 2, 3]);
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
        assert_eq!(port.get_values(), &[10, 20, 30]);
    }

    #[test]
    fn port_add_values_from_slice() {
        let mut port: Port<u32, 5> = Port::new();
        assert!(port.add_values(&[10, 20, 30]).is_ok());
        assert_eq!(port.len(), 3);
        assert_eq!(port.get_values(), &[10, 20, 30]);
    }

    #[test]
    fn port_add_values_capacity_error() {
        let mut port: Port<u32, 3> = Port::new();
        port.add_values(&[1, 2, 3]).unwrap();
        assert!(port.is_full());
        let result = port.add_values(&[4]);
        assert!(result.is_err());
        assert_eq!(port.get_values(), &[1, 2, 3]);
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
        assert_eq!(dst.get_values(), &[1, 2, 3]);
        assert_eq!(src.get_values(), &[1, 2, 3]);
    }

    #[test]
    fn port_couple_capacity_error() {
        let mut src: Port<u32, 5> = Port::new();
        src.add_values(&[1, 2, 3]).unwrap();
        let mut dst: Port<u32, 2> = Port::new();
        let result = src.couple(&mut dst);
        assert!(result.is_err());
        assert_eq!(src.get_values(), &[1, 2, 3]);
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
        bag.eject_events(|v| {
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
        bags.eject_events(|(i, v)| {
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
        bag.eject_events(|v| {
            let _ = collected.push(v);
        });
        assert_eq!(collected.as_slice(), &[7]);

        bag.clear();
        assert!(bag.is_empty());

        let mut collected_after: heapless::Vec<u32, 4> = heapless::Vec::new();
        bag.eject_events(|v| {
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
        bag.eject_events(|ev| match ev {
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
        bag.eject_events(|ev| match ev {
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

        let inner_event = _xdevs_no_std_inner_bag_bag::PortMux::A(42u32);
        let outer_inner = _xdevs_no_std_outer_bag_bag::PortMux::Inner(inner_event);
        assert!(outer.add_value(outer_inner).is_ok());
        assert!(!outer.is_empty());

        assert!(outer
            .add_value(_xdevs_no_std_outer_bag_bag::PortMux::B(true))
            .is_ok());

        let mut got_a: heapless::Vec<u32, 4> = heapless::Vec::new();
        let mut got_b: heapless::Vec<bool, 4> = heapless::Vec::new();
        outer.eject_events(|ev| match ev {
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
