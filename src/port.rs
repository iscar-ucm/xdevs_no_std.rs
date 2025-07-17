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
    #[allow(clippy::result_unit_err)]
    pub fn add_values(&mut self, items: &[T]) -> Result<(), ()> {
        self.0.extend_from_slice(items)
    }

    /// Returns a slice of the port's values.
    #[inline]
    pub fn get_values(&self) -> &[T] {
        self.0.as_slice()
    }
}

unsafe impl<T: Clone, const N: usize> crate::traits::Bag for Port<T, N> {
    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn clear(&mut self) {
        self.clear()
    }
}
