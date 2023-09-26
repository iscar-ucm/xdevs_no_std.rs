#[derive(Debug, Default)]
pub struct Port<T: Clone, const N: usize>(heapless::Vec<T, N>);

impl<T: Clone, const N: usize> Port<T, N> {
    #[inline]
    pub const fn new() -> Self {
        Self(heapless::Vec::new())
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[inline]
    pub fn is_full(&self) -> bool {
        self.0.is_full()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    pub fn clear(&mut self) {
        self.0.clear()
    }

    #[inline]
    pub fn add_value(&mut self, item: T) -> Result<(), T> {
        self.0.push(item).map_err(|e| e)
    }
}
