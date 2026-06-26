use sealed::Sealed;

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

    /// Adds a value to the port.
    #[inline]
    pub fn add_value(&mut self, item: T) -> Result<(), T> {
        self.0.push(item)
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
    fn build() -> Self {
        Self::new()
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn clear(&mut self) {
        self.clear()
    }
}

impl<T: Clone, const N: usize> AsPort for Port<T, N> {
    type Item = T;
}

impl<T: Clone, const N: usize> Sealed for Port<T, N> {}

/// Trait that defines the methods that a DEVS event bag set must implement.
///
/// # Safety
///
/// This trait must be implemented via the [`Bag`](crate::Bag) macro. Do not implement it manually.
pub unsafe trait Bag {
    /// Build a new instance of the bag.
    fn build() -> Self;

    /// Returns `true` if the ports are empty.
    fn is_empty(&self) -> bool;

    /// Clears the ports, removing all values.
    fn clear(&mut self);
}

/// Trait that defines the type inside of a Bag for rt_engine enums.
///
/// # Note
///
/// This trait is sealed and cannot be implemented by the user
pub trait AsPort: Bag + Sealed {
    /// The type of the values contained in the bag.
    type Item;
}

/// Trait that defines a type that maps to event bag ports.
/// Its main purpose is its usage by the `RtEngine` to inject and eject events from the model's ports.
///
/// # Safety
///
/// This trait must be implemented via the [`Bag`](crate::Bag) macro. Do not implement it manually.
pub unsafe trait BagMux: Bag {
    /// The type that represents the ports of the model. Each variant corresponds to a port.
    type Mux;

    /// Maps the type to the corresponding port, allowing to inject events to the bag.
    fn inject_event(&mut self, event: Self::Mux) -> Result<(), Self::Mux>;

    /// Maps the type to the corresponding port, allowing to receive events from the bag.
    fn eject_events(&self, ejector: impl FnMut(Self::Mux));
}

unsafe impl<T: Bag, const N: usize> Bag for [T; N] {
    fn build() -> Self {
        core::array::from_fn(|_| T::build())
    }

    fn is_empty(&self) -> bool {
        self.iter().all(|bag| bag.is_empty())
    }

    fn clear(&mut self) {
        self.iter_mut().for_each(|bag| bag.clear());
    }
}

impl<T: AsPort, const N: usize> AsPort for [T; N] {
    type Item = (usize, T::Item); // Include index to identify which bag the value came from
}
impl<T: AsPort, const N: usize> Sealed for [T; N] {}

unsafe impl Bag for () {
    fn build() -> Self {}

    fn is_empty(&self) -> bool {
        true
    }

    fn clear(&mut self) {}
}

macro_rules! impl_bag_for_tuple {
    ($($idx:tt => $T:ident),+) => {
        unsafe impl<$($T: Bag),+> Bag for ($($T,)+) {
            fn build() -> Self {
                ($($T::build(),)+)
            }

            fn is_empty(&self) -> bool {
                let mut empty = true;
                $(empty = empty && self.$idx.is_empty();)+
                empty
            }

            fn clear(&mut self) {
                $(self.$idx.clear();)+
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

mod sealed {
    /// Trait used to prevent users from implementing certain traits manually.
    pub trait Sealed {}
}

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
        port.add_value(42).unwrap();
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

        bag.0.add_value(42).unwrap();
        assert!(!bag.is_empty());

        bag.1.add_value(42).unwrap();
        assert!(!bag.is_empty());

        bag.2.add_value(42).unwrap();
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
}
