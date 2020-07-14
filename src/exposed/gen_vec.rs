use std::vec::Vec;
use crate::{Index, Item};

/// Generationally indexed vector
#[derive(Debug)]
pub struct GenerationalVec<T>
{
    items: Vec<Option<Item<T>>>
}

impl<T> GenerationalVec<T>
{
    /// Returns an empty `GenerationalVec`
    ///
    /// # Examples
    ///
    /// ```
    /// let mut vec: GenerationalVec<i32> = GenerationalVec::new();
    /// ```
    pub fn new() -> GenerationalVec<T>
    {
        GenerationalVec
        {
            items: Vec::new()
        }
    }

    /// Returns a `GenerationalVec` with initial capacity of `capacity`
    ///
    /// Allows the `GenerationalVec` to hold `capacity` elements before
    /// allocating more space
    ///
    /// # Examples
    ///
    /// ```
    /// let mut vec: GenerationalVec<i32> = GenerationalVec::with_capacity(5);
    /// ```
    pub fn with_capacity(capacity: usize) -> GenerationalVec<T>
    {
        GenerationalVec
        {
            items: Vec::with_capacity(capacity)
        }
    }

    /// Reserved capacity within the `GenerationalVec`
    ///
    /// # Examples
    ///
    /// ```
    /// let mut vec: GenerationalVec<i32> = GenerationalVec::with_capacity(5);
    /// assert_eq!(vec.capacity(), 5);
    /// ```
    pub fn capacity(&self) -> usize
    {
        self.items.capacity()
    }

    /// Reserves extra space for *at least* `additional` more elements
    ///
    /// More space may be allocated to avoid frequent re-allocations
    /// (as per the specifications of std::vec::Vec)
    ///
    /// # Examples
    ///
    /// ```
    /// let mut vec: GenerationalVec<i32> = GenerationalVec::with_capacity(2);
    /// vec.reserve(3);
    /// assert_eq!(vec.capacity(), 5);
    /// ```
    pub fn reserve(&mut self, additional: usize)
    {
        self.items.reserve(additional)
    }

    /// Returns `true` if the `index` points to a valid item within the vec
    ///
    /// # Examples
    ///
    /// ```
    /// let mut allocator: IndexAllocator = IndexAllocator::new();
    /// let index = allocator.allocate();
    ///
    /// let mut vec: GenerationalVec<i32> = GenerationalVec::new();
    /// assert!(!vec.contains(index));
    /// vec.set(index, 0);
    /// assert!(vec.contains(index));
    /// ```
    pub fn contains(&self, index: Index) -> bool
    {
        self.get(index).is_some()
    }

    /// Set the value for the given `index`
    ///
    /// This may overwrite past (but not future) generations
    ///
    /// # Examples
    ///
    /// ```
    /// let mut allocator: IndexAllocator = IndexAllocator::new();
    /// let index = allocator.allocate();
    ///
    /// let mut vec: GenerationalVec<i32> = GenerationalVec::new();
    /// vec.set(index, 0);
    /// assert!(vec.contains(index));
    /// ```
    pub fn set(&mut self, index: Index, value: T)
    {
        // If vec is smaller than the index, resize it and fill intermittent indices with None
        if self.items.len() < index.index + 1
        {
            self.items.resize_with(index.index + 1, | | None);
        }

        match self.items.get_mut(index.index)
        {
            Some(item) =>
                {
                    if item.is_none() || (item.is_some() && item.as_ref().unwrap().generation <= index.generation)
                    {
                        *item = Some(Item { value, generation: index.generation });
                    }
                },
            _ => panic!(format!("Index should point to a valid index in items vec, but it doesn't\n{:?}", index))
        }
    }

    /// Returns an immutable reference to the value of `index` if `index` is valid
    ///
    /// # Examples
    ///
    /// ```
    /// let mut allocator: IndexAllocator = IndexAllocator::new();
    /// let index = allocator.allocate();
    ///
    /// let mut vec: GenerationalVec<i32> = GenerationalVec::new();
    /// vec.set(index, 0);
    /// let value: &i32 = vec.get(index);
    /// assert_eq!(*value, 0);
    /// ```
    pub fn get(&self, index: Index) -> Option<&T>
    {
        match self.items.get(index.index)
        {
            Some(Some(item)) if item.generation == index.generation => Some(&item.value),
            _ => None
        }
    }

    /// Returns a mutable reference to the value of `index` if `index` is valid
    ///
    /// # Examples
    ///
    /// ```
    /// let mut allocator: IndexAllocator = IndexAllocator::new();
    /// let index = allocator.allocate();
    ///
    /// let mut vec: GenerationalVec<i32> = GenerationalVec::new();
    /// vec.set(index, 0);
    /// let mut value: &mut i32 = vec.get_mut(index);
    /// assert_eq!(*value, 0);
    /// *value = 1;
    /// let value = vec.get(index);
    /// assert_eq!(*value, 1);
    /// ```
    pub fn get_mut(&mut self, index: Index) -> Option<&mut T>
    {
        match self.items.get_mut(index.index)
        {
            Some(Some(item)) if item.generation == index.generation => Some(&mut item.value),
            _ => None
        }
    }
}

#[cfg(test)]
mod vec_tests
{
    use crate::exposed::*;

    #[test]
    fn capacity()
    {
        let mut vec = GenerationalVec::<i32>::new();
        assert_eq!(vec.capacity(), 0);
        vec.reserve(4);
        assert_eq!(vec.capacity(), 4);
    }

    #[test]
    fn set()
    {
        let mut allocator = IndexAllocator::new();
        let index = allocator.allocate();

        let mut vec = GenerationalVec::new();
        vec.set(index, 0);
        assert!(vec.contains(index));

        allocator.deallocate(index);
        let index1 = allocator.allocate();

        vec.set(index1, 0);
        assert!(!vec.contains(index));
        assert!(vec.contains(index1));

        for _ in 0..50
        {
            allocator.allocate();
        }

        let index = allocator.allocate();
        vec.set(index, 20);
        assert!(vec.contains(index));
        assert_eq!(*vec.get(index).unwrap(), 20);
    }

    #[test]
    fn get()
    {
        let mut allocator = IndexAllocator::new();
        let index = allocator.allocate();
        let index1 = allocator.allocate();

        let mut vec = GenerationalVec::new();
        vec.set(index, 0);
        vec.set(index1, 1);

        assert_eq!(*vec.get(index).unwrap(), 0);
        assert_eq!(*vec.get(index1).unwrap(), 1);

        *vec.get_mut(index).unwrap() = 2;
        assert_eq!(*vec.get(index).unwrap(), 2);
    }
}