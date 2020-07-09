//! Generational Index Vec

use std::vec::Vec;
use std::collections::VecDeque;

/// An index of a `GenerationalVec`
#[derive(Eq, PartialEq, Ord, PartialOrd, Hash, Clone, Copy, Debug)]
pub struct Index
{
    index: usize,
    generation: usize
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

impl<T> GenerationalVec<T>
{
    /// Create an empty `GenerationalVec`
    pub fn new() -> GenerationalVec<T>
    {
        GenerationalVec
        {
            free_indices: VecDeque::new(),
            items: Vec::new(),
            generation: 0,
            length: 0
        }
    }

    /// Create an empty `GenerationVec` and reserve `capacity`
    pub fn with_capacity(capacity: usize) -> GenerationalVec<T>
    {
        GenerationalVec
        {
            free_indices: VecDeque::with_capacity(capacity),
            items: Vec::with_capacity(capacity),
            generation: 0,
            length: 0
        }
    }

    /// Number of active `Item`s within the `GenerationalVec`
    /// The internal vec may actually be larger depending on the number of freed indices
    pub fn len(&self) -> usize
    {
        self.length
    }

    /// Are all items within `GenerationalVec` freed
    pub fn is_empty(&self) -> bool
    {
        self.length <= 0
    }

    /// Internal reserved capacity of the `GenerationalVec`
    /// This may be more depending on the number of freed indices
    pub fn capacity(&self) -> usize
    {
        self.items.capacity()
    }

    /// Free all `Item`s within the `GenerationalVec`
    pub fn clear(&mut self)
    {
        self.free_indices.clear();
        for i in 0..self.items.len()
        {
            self.free_indices.push_back(i);
        }
        self.items.clear();

        self.length = 0;
        self.generation += 1;
    }

    /// `additional` space is reserved on top of the existing capacity
    pub fn reserve(&mut self, additional: usize)
    {
        if additional > 0
        {
            self.items.reserve(additional);
            self.free_indices.reserve(additional);

            if self.items.len() > 0
            {
                let last_index = self.items.len()-1;
                // Add all new reserved
                for i in last_index..(last_index+additional)
                {
                    self.free_indices.push_back(i);
                    self.items.push(None);
                }
            }
        }
    }

    pub fn contains(&self, index: Index) -> bool
    {
        self.get(index).is_some()
    }

    /// Insert `value` in the `GenerationalVec` and return an associated `Index`
    pub fn insert(&mut self, value: T) -> Index
    {
        // Get the next free index
        match self.free_indices.pop_front()
        {
            // If an index was returned, modify the item at that position
            Some(index) =>
                {
                    match self.items.get_mut(index) {
                        Some(None) =>
                            {
                                self.items[index] = Some(Item{ value, generation: self.generation });
                                self.length += 1;
                                Index{ index, generation: self.generation }
                            }
                        // If index contained an Item, the index was invalid so try again
                        _ =>
                            {
                                self.insert(value)
                            }

                    }
                },
            // If there are no free indices, add it to the end of the item vec
            None =>
                {
                    self.items.push(Some(Item{ value, generation: self.generation }));
                    self.length += 1;
                    Index{ index: self.items.len()-1, generation: self.generation }
                }
        }
    }

    /// Get the immutable value at `index` if exists
    pub fn get(&self, index: Index) -> Option<&T>
    {
        match self.items.get(index.index)
        {
            Some(Some(item)) if index.generation == item.generation => Some(&item.value),
            _ => None
        }
    }

    /// Get the mutable value at `index` if exists
    pub fn get_mut(&mut self, index: Index) -> Option<&mut T>
    {
        match self.items.get_mut(index.index)
        {
            Some(Some(item)) if index.generation == item.generation => Some(&mut item.value),
            _ => None
        }
    }

    /// Free the value stored at `index` if exists
    /// Returns stored value if exists
    pub fn remove(&mut self, index: Index) -> Option<T>
    {
        match self.items.get(index.index)
        {
            Some(Some(item)) if index.generation == item.generation =>
                {
                    let removed = std::mem::replace(&mut self.items[index.index], None).expect("replaced vec item");
                    self.generation += 1;
                    self.length -= 1;
                    self.free_indices.push_back(index.index);
                    Some(removed.value)
                },
            _ => None
        }
    }
}



#[cfg(test)]
mod tests {
    use crate::gen_vec::GenerationalVec;

    #[test]
    fn insert()
    {
        let mut givec = GenerationalVec::new();
        let i = givec.insert(3);
        assert_eq!(i.index, 0);
        assert_eq!(i.generation, 0);
        assert_eq!(givec.len(), 1);

        let i = givec.insert(4);
        assert_eq!(i.index, 1);
        assert_eq!(i.generation, 0);
        assert_eq!(givec.len(), 2);
    }

    #[test]
    fn get()
    {
        let mut givec = GenerationalVec::new();
        let i = givec.insert(3);
        let item = givec.get(i).unwrap();
        assert_eq!(*item, 3);

        let i = givec.insert(4);
        let item = givec.get(i).unwrap();
        assert_eq!(*item, 4);
    }

    #[test]
    fn get_mut()
    {
        let mut givec = GenerationalVec::new();
        let i = givec.insert(3);
        let item = givec.get_mut(i).unwrap();
        *item = 1;
        assert_eq!(*givec.get(i).unwrap(), 1);
    }

    #[test]
    fn remove()
    {
        let mut givec = GenerationalVec::new();
        let i = givec.insert(3);
        let item = givec.remove(i).unwrap();
        assert_eq!(item, 3);
        assert_eq!(givec.len(), 0);
        assert_eq!(givec.get(i), None);

        let i2 = givec.insert(4);
        assert_eq!(i2.index, 0);
        assert_eq!(i2.generation, 1);
        assert_eq!(givec.len(), 1);
    }

    #[test]
    fn clear()
    {
        let mut vec = GenerationalVec::new();
        vec.insert(4);
        let gen = vec.insert(5).generation;

        vec.clear();
        assert_eq!(vec.free_indices.len(), 2);

        assert_eq!(vec.len(), 0);
        let item = vec.insert(1);
        assert_eq!(item.index, 0);
        assert_eq!(item.generation, gen+1);
    }

    #[test]
    fn len()
    {
        let mut vec = GenerationalVec::new();
        assert_eq!(vec.len(), 0);
        vec.insert(3);
        assert_eq!(vec.len(), 1);
    }

    #[test]
    fn is_empty()
    {
        let mut vec = GenerationalVec::new();
        assert!(vec.is_empty());
        vec.insert(3);
        assert!(!vec.is_empty());
        vec.clear();
        assert!(vec.is_empty());
        let i = vec.insert(4);
        vec.remove(i);
        assert!(vec.is_empty());
    }

    #[test]
    fn capacity()
    {
        let mut vec = GenerationalVec::<i32>::new();
        assert_eq!(vec.capacity(), 0);

        vec.reserve(24);
        assert!(vec.capacity() >= 24);

        vec = GenerationalVec::with_capacity(5);
        assert_eq!(vec.capacity(), 5);
    }
}