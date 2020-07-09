///! Generational Index Vec with exposed Index allocator

use std::vec::Vec;
use std::collections::VecDeque;

/// An index of a `GenerationalVec`
#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy, Debug)]
pub struct Index
{
    index: usize,
    generation: usize
}

pub struct AllocatedIndex
{
    is_free: bool,
    generation: usize
}

pub struct IndexAllocator
{
    free_indices: VecDeque<usize>,
    indices: Vec<AllocatedIndex>
}

impl IndexAllocator
{
    pub fn new() -> IndexAllocator
    {
        IndexAllocator
        {
            free_indices: VecDeque::new(),
            indices: Vec::new()
        }
    }

    pub fn with_capacity(capacity: usize) -> IndexAllocator
    {
        IndexAllocator
        {
            free_indices: VecDeque::with_capacity(capacity),
            indices: Vec::with_capacity(capacity)
        }
    }

    pub fn is_active(&self, index: Index) -> bool
    {
        match self.indices.get(index.index)
        {
            Some(AllocatedIndex{ is_free, generation }) => *generation == index.generation && !*is_free,
            _ => false
        }
    }

    pub fn allocate(&mut self) -> Index
    {
        match self.free_indices.pop_front()
        {
            Some(index) =>
                {
                    match self.indices.get_mut(index)
                    {
                        Some(AllocatedIndex{ is_free, generation }) if *is_free =>
                            {
                                *is_free = false;
                                *generation += 1;
                                Index { index: index, generation: *generation+1 }
                            },
                        _ => self.allocate()
                    }
                },
            _ =>
                {
                    self.indices.push(AllocatedIndex{ is_free: false, generation: 0 });
                    Index{ index: self.indices.len()-1, generation: 0 }
                }
        }
    }

    pub fn deallocate(&mut self, index: Index)
    {
        if let Some(AllocatedIndex{ is_free, generation }) = self.indices.get_mut(index.index)
        {
            if *generation == index.generation && !*is_free
            {
                *is_free = true;
                self.free_indices.push_back(index.index);
            }
        }
    }

}

/// An item within a `GenerationalVec`
#[derive(Debug)]
struct Item<T>
{
    value: T,
    generation: usize
}

/// A vector with reusable indices
#[derive(Debug)]
pub struct GenerationalVec<T>
{
    free_indices: VecDeque<usize>,
    items: Vec<Option<Item<T>>>,
    generation: usize,
    length: usize
}