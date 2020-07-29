use std::
{
    vec,
    vec::Vec,
    collections::VecDeque,
    iter,
    slice
};
use crate::Index;

#[cfg(feature = "serde")]
use serde::{Serialize, Deserialize};

/// An allocated index of a `IndexAllocator`
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
struct AllocatedIndex
{
    is_free: bool,
    generation: usize
}

/// Allocates and deallocates indices for a `ExposedGenVec`
#[derive(Default, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct IndexAllocator
{
    free_indices: VecDeque<usize>,
    active_indices: Vec<AllocatedIndex>
}

impl IndexAllocator
{
    /// Returns a new empty `IndexAllocator`
    ///
    /// # Examples
    ///
    /// ```
    /// use gen_vec::exposed::IndexAllocator;
    /// let mut allocator: IndexAllocator = IndexAllocator::new();
    /// ```
    pub fn new() -> IndexAllocator
    {
        IndexAllocator
        {
            free_indices: VecDeque::new(),
            active_indices: Vec::new()
        }
    }

    /// Returns a `IndexAllocator` with initial capacity of `capacity`
    ///
    /// Allows the `IndexAllocator` to hold `capacity` elements before
    /// allocating more space
    ///
    /// # Examples
    ///
    /// ```
    /// use gen_vec::exposed::IndexAllocator;
    /// let mut allocator: IndexAllocator = IndexAllocator::with_capacity(5);
    /// ```
    pub fn with_capacity(capacity: usize) -> IndexAllocator
    {
        IndexAllocator
        {
            free_indices: VecDeque::with_capacity(capacity),
            active_indices: Vec::with_capacity(capacity)
        }
    }

    /// Allocates and returns a new `Index`
    ///
    /// Activates a freed index if there are any, otherwise creates
    /// and adds a new index to `active_indices`
    ///
    /// # Examples
    ///
    /// ```
    /// use gen_vec::Index;
    /// use gen_vec::exposed::IndexAllocator;
    ///
    /// let mut allocator: IndexAllocator = IndexAllocator::new();
    /// let index: Index = allocator.allocate();
    /// ```
    pub fn allocate(&mut self) -> Index
    {
        match self.free_indices.pop_front()
        {
            Some(index) =>
                {
                    match self.active_indices.get_mut(index)
                    {
                        Some(AllocatedIndex{ is_free, generation }) if *is_free =>
                            {
                                *is_free = false;
                                *generation += 1;
                                Index { index: index, generation: *generation }
                            },
                        // Try again if the free index was invalid
                        _ => self.allocate()
                    }
                },
            _ =>
                {
                    self.active_indices.push(AllocatedIndex{ is_free: false, generation: 0 });
                    Index{ index: self.active_indices.len().saturating_sub(1), generation: 0 }
                }
        }
    }

    /// Frees `index` if it hasn't been already.
    ///
    /// Afterwards, `index` is added to the pool of free indices
    /// available for reuse
    ///
    /// # Examples
    ///
    /// ```
    /// use gen_vec::Index;
    /// use gen_vec::exposed::IndexAllocator;
    ///
    /// let mut allocator: IndexAllocator = IndexAllocator::new();
    /// let index: Index = allocator.allocate();
    /// allocator.deallocate(index);
    /// ```
    pub fn deallocate(&mut self, index: Index)
    {
        if self.is_active(index)
        {
            self.active_indices[index.index].is_free = true;
            self.free_indices.push_back(index.index);
        }
    }

    /// Frees all active indices and adds them to the pool of free indices
    ///
    /// # Examples
    ///
    /// ```
    /// use gen_vec::exposed::IndexAllocator;
    /// use gen_vec::Index;
    ///
    /// let mut allocator: IndexAllocator = IndexAllocator::new();
    /// for _ in 0..10
    /// {
    ///     allocator.allocate();
    /// }
    /// assert_eq!(allocator.num_active(), 10);
    /// assert_eq!(allocator.num_free(), 0);
    ///
    /// allocator.deallocate_all();
    /// assert_eq!(allocator.num_active(), 0);
    /// assert_eq!(allocator.num_free(), 10);
    /// ```
    pub fn deallocate_all(&mut self)
    {
        for (index, alloc_index) in self.active_indices.iter_mut().enumerate()
        {
            alloc_index.is_free = true;
            self.free_indices.push_back(index);
        }
    }

    /// Reserved capacity within the `IndexAllocator`
    ///
    /// # Examples
    ///
    /// ```
    /// use gen_vec::exposed::IndexAllocator;
    ///
    /// let mut allocator: IndexAllocator = IndexAllocator::with_capacity(5);
    /// assert_eq!(allocator.capacity(), 5);
    /// ```
    pub fn capacity(&self) -> usize
    {
        self.active_indices.capacity()
    }

    /// Reserves extra space for *at least* `additional` more elements
    ///
    /// More space may be allocated to avoid frequent re-allocations
    /// (as per the specifications of std::vec::Vec)
    ///
    /// # Examples
    ///
    /// ```
    /// use gen_vec::Index;
    /// use gen_vec::exposed::IndexAllocator;
    ///
    /// let mut allocator: IndexAllocator = IndexAllocator::new();
    /// let index: Index = allocator.allocate();
    /// allocator.reserve(4);
    /// assert!(allocator.capacity() >= 4);
    /// ```
    pub fn reserve(&mut self, additional: usize)
    {
        if additional > 0
        {
            self.active_indices.reserve(additional);
            self.free_indices.reserve(additional);

            if self.active_indices.len() > 0
            {
                let last_index = self.active_indices.len().saturating_sub(1);
                // Add all new reserved
                for i in last_index..(last_index+additional)
                {
                    self.free_indices.push_back(i);
                    self.active_indices.push(AllocatedIndex{ is_free: true, generation: 0 });
                }
            }
        }
    }

    /// Returns if `index` is still active and hasn't been deallocated
    ///
    /// # Examples
    ///
    /// ```
    /// use gen_vec::Index;
    /// use gen_vec::exposed::IndexAllocator;
    ///
    /// let mut allocator: IndexAllocator = IndexAllocator::new();
    /// let index: Index = allocator.allocate();
    /// assert!(allocator.is_active(index));
    /// allocator.deallocate(index);
    /// assert!(!allocator.is_active(index));
    /// ```
    pub fn is_active(&self, index: Index) -> bool
    {
        match self.active_indices.get(index.index)
        {
            Some(AllocatedIndex{ is_free, generation }) => *generation == index.generation && !*is_free,
            _ => false
        }
    }

    /// Returns the number of free indices waiting to be allocated and reused
    ///
    /// # Examples
    ///
    /// ```
    /// use gen_vec::Index;
    /// use gen_vec::exposed::IndexAllocator;
    ///
    /// let mut allocator: IndexAllocator = IndexAllocator::new();
    /// assert_eq!(allocator.num_free(), 0);
    ///
    /// let index: Index = allocator.allocate();
    ///
    /// allocator.deallocate(index);
    /// assert_eq!(allocator.num_free(), 1);
    ///
    /// let index: Index = allocator.allocate();
    /// assert_eq!(allocator.num_free(), 0);
    /// ```
    pub fn num_free(&self) -> usize
    {
        self.free_indices.len()
    }

    /// Returns the number of active indices
    ///
    /// # Examples
    ///
    /// ```
    /// use gen_vec::Index;
    /// use gen_vec::exposed::IndexAllocator;
    ///
    /// let mut allocator: IndexAllocator = IndexAllocator::new();
    /// assert_eq!(allocator.num_active(), 0);
    ///
    /// let index: Index = allocator.allocate();
    /// assert_eq!(allocator.num_active(), 1);
    ///
    /// allocator.deallocate(index);
    /// assert_eq!(allocator.num_active(), 0);
    /// ```
    pub fn num_active(&self) -> usize
    {
        self.active_indices.len().saturating_sub(self.free_indices.len())
    }

    /// Returns an iterator over an immutable `IndexAllocator`
    /// Each step returns an `Index`
    ///
    /// # Examples
    ///
    /// ```
    /// use gen_vec::Index;
    /// use gen_vec::exposed::IndexAllocator;
    ///
    /// let mut allocator: IndexAllocator = IndexAllocator::new();
    /// allocator.allocate();
    /// allocator.allocate();
    ///
    /// for index in allocator
    /// {
    ///     println!("{:?}", index);
    /// }
    /// ```
    pub fn iter(&self) -> Iter
    {
        Iter
        {
            internal: self.active_indices.iter().enumerate()
        }
    }
}

/// Struct for consuming a `IndexAllocator` into an iterator
#[derive(Debug)]
pub struct IntoIter
{
    internal: iter::Enumerate<vec::IntoIter<AllocatedIndex>>
}

impl Iterator for IntoIter
{
    type Item = Index;

    fn next(&mut self) -> Option<Self::Item>
    {
        loop
        {
            match self.internal.next()
            {
                Some((index, allocated_index)) if !allocated_index.is_free => return Some(Index { index, generation: allocated_index.generation }),
                Some((_, _)) => continue,
                _ => return None
            }
        }
    }
}

impl IntoIterator for IndexAllocator
{
    type Item = Index;
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter
    {
        IntoIter
        {
            internal: self.active_indices.into_iter().enumerate()
        }
    }
}

/// Struct for creating an iterator over an immutable `IndexAllocator` reference
#[derive(Debug)]
pub struct Iter<'a>
{
    internal: iter::Enumerate<slice::Iter<'a, AllocatedIndex>>
}

impl<'a> Iterator for Iter<'a>
{
    type Item = Index;

    fn next(&mut self) -> Option<Self::Item>
    {
        loop
        {
            loop
            {
                match self.internal.next()
                {
                    Some((index, allocated_index)) if !allocated_index.is_free => return Some(Index { index, generation: allocated_index.generation }),
                    Some((_, _)) => continue,
                    _ => return None
                }
            }
        }
    }
}

impl<'a> IntoIterator for &'a IndexAllocator
{
    type Item = Index;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter
    {
        self.iter()
    }
}

#[cfg(test)]
mod allocator_tests
{
    use crate::exposed::*;
    use crate::Index;

    #[test]
    fn allocate()
    {
        let mut allocator = IndexAllocator::new();
        let index = allocator.allocate();
        assert_eq!(index.index, 0);
        assert_eq!(index.generation, 0);

        let index = allocator.allocate();
        assert_eq!(index.index, 1);
        assert_eq!(index.generation, 0);
    }

    #[test]
    fn deallocate()
    {
        let mut allocator = IndexAllocator::new();
        let index = allocator.allocate();
        allocator.allocate();

        allocator.deallocate(index);

        let index = allocator.allocate();
        assert_eq!(index.index, 0);
        assert_eq!(index.generation, 1);
    }

    #[test]
    fn deallocate_all()
    {
        let mut allocator: IndexAllocator = IndexAllocator::new();
        for _ in 0..10
        {
            allocator.allocate();
        }
        assert_eq!(allocator.num_active(), 10);
        assert_eq!(allocator.num_free(), 0);

        allocator.deallocate_all();
        assert_eq!(allocator.num_active(), 0);
        assert_eq!(allocator.num_free(), 10);
    }

    #[test]
    fn capacity()
    {
        let mut allocator = IndexAllocator::new();
        assert_eq!(allocator.capacity(), 0);
        allocator.allocate();
        assert!(allocator.capacity() >= 1);

        allocator = IndexAllocator::with_capacity(3);
        assert!(allocator.capacity() >= 3);
        allocator.allocate();
        allocator.allocate();

        allocator.reserve(4);
        assert!(allocator.capacity() >= 5);
    }

    #[test]
    fn active()
    {
        let mut allocator = IndexAllocator::new();
        let index = allocator.allocate();
        allocator.allocate();
        assert_eq!(allocator.num_active(), 2);
        assert_eq!(allocator.num_free(), 0);
        assert!(allocator.is_active(index));
        allocator.deallocate(index);
        assert!(!allocator.is_active(index));
        assert_eq!(allocator.num_active(), 1);
        assert_eq!(allocator.num_free(), 1);
    }

    #[test]
    fn iter()
    {
        let mut allocator = IndexAllocator::new();
        allocator.allocate();
        allocator.allocate();
        let i = allocator.allocate();
        allocator.deallocate(i);

        let mut iter = allocator.iter();
        assert_eq!(iter.next(), Some(Index { index: 0, generation: 0 }));
        assert_eq!(iter.next(), Some(Index { index: 1, generation: 0}));
        assert_eq!(iter.next(), None);
    }
}