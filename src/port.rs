/// Port is a generic structure that can be used to store values of any type `T`.
/// It is the main artifact to exchange data between components.
/// Note that, in no_std environment, the capacity of the port, `N`, must be known at compile time.
#[derive(Debug, Default)]
pub struct Port<T: Clone, const N: usize>(heapless::Vec<T, N>);

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

    /// Returns a slice of the port's values.
    #[inline]
    pub fn get_values(&self) -> &[T] {
        self.0.as_slice()
    }
}

/// UnsafePort is a trait that defines the methods that a port must implement.
///
/// # Safety
///
/// This trait must be implemented via the [`atomic!`] and [`coupled!`] macros. Do not implement it manually.
pub unsafe trait UnsafePort {
    /// Returns `true` if the port(s) is empty.
    fn is_empty(&self) -> bool;

    /// Clears the port(s), removing all values.
    fn clear(&mut self);
}

unsafe impl<T: Clone, const N: usize> UnsafePort for Port<T, N> {
    #[inline]
    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    #[inline]
    fn clear(&mut self) {
        self.clear()
    }
}
