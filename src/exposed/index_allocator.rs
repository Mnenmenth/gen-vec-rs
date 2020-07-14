use std::collections::VecDeque;
use std::vec::Vec;
use crate::Index;

/// An allocated index of a `IndexAllocator`
pub struct AllocatedIndex
{
    is_free: bool,
    generation: usize
}

/// Allocates and deallocates indices for a `GenerationalVec`
pub struct IndexAllocator
{
    free_indices: VecDeque<usize>,
    active_indices: Vec<AllocatedIndex>
}

impl IndexAllocator
{
    /// Returns a new empty `IndexAllocator`
    pub fn new() -> IndexAllocator
    {
        IndexAllocator
        {
            free_indices: VecDeque::new(),
            active_indices: Vec::new()
        }
    }

    /// Returns an `IndexAllocator` with initial capacity of `capacity`
    pub fn with_capacity(capacity: usize) -> IndexAllocator
    {
        IndexAllocator
        {
            free_indices: VecDeque::with_capacity(capacity),
            active_indices: Vec::with_capacity(capacity)
        }
    }

    /// Allocates and returns a new `Index`
    /// Activates a freed index if there are any, otherwise adds a new index to `active_indices`
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

    /// Frees `index` if it hasn't been already
    pub fn deallocate(&mut self, index: Index)
    {
        if self.is_active(index)
        {
            self.active_indices[index.index].is_free = true;
            self.free_indices.push_back(index.index);
        }
    }

    /// Internal reserved capacity of the `IndexAllocator`
    pub fn capacity(&self) -> usize
    {
        self.active_indices.capacity()
    }

    /// `additional` space is reserved on top of the existing capacity
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

    /// Is `index` active and not deallocated
    pub fn is_active(&self, index: Index) -> bool
    {
        match self.active_indices.get(index.index)
        {
            Some(AllocatedIndex{ is_free, generation }) => *generation == index.generation && !*is_free,
            _ => false
        }
    }

    /// Number of free indices waiting to be allocated
    pub fn num_free(&self) -> usize
    {
        self.free_indices.len()
    }

    /// Number of active indices
    pub fn num_active(&self) -> usize
    {
        self.active_indices.len().saturating_sub(self.free_indices.len())
    }
}

#[cfg(test)]
mod allocator_tests
{
    use crate::exposed::*;

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
}