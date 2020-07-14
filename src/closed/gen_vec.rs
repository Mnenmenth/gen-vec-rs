/// Generational Index Vec

use std::vec::Vec;
use std::collections::VecDeque;
use crate::{Index, Item};

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
    /// Returns an empty `GenerationalVec`
    ///
    /// # Examples
    ///
    /// ```
    /// use gen_vec::closed::GenerationalVec;
    /// let mut vec: GenerationalVec<i32> = GenerationalVec::new();
    /// ```
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

    /// Returns a `GenerationalVec` with initial capacity of `capacity`
    ///
    /// Allows the `GenerationalVec` to hold `capacity` elements before
    /// allocating more space
    ///
    /// # Examples
    ///
    /// ```
    /// use gen_vec::closed::GenerationalVec;
    /// let mut vec: GenerationalVec<i32> = GenerationalVec::with_capacity(5);
    /// ```
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
    ///
    /// The internal item vec may actually be larger depending on the number of freed indices
    ///
    /// # Examples
    /// ```
    /// use gen_vec::Index;
    /// use gen_vec::closed::GenerationalVec;
    ///
    /// let mut vec: GenerationalVec<i32> = GenerationalVec::new();
    ///
    /// let index: Index = vec.insert(42);
    /// assert_eq!(vec.len(), 1);
    ///
    /// vec.remove(index);
    /// assert_eq!(vec.len(), 0);
    /// ```
    pub fn len(&self) -> usize
    {
        self.length
    }

    /// Returns `true` if there are no active items
    ///
    /// # Examples
    ///
    /// ```
    /// use gen_vec::Index;
    /// use gen_vec::closed::GenerationalVec;
    ///
    /// let mut vec: GenerationalVec<i32> = GenerationalVec::new();
    /// assert!(vec.is_empty());
    ///
    /// let index: Index = vec.insert(23);
    /// assert!(!vec.is_empty());
    ///
    /// vec.remove(index);
    /// assert!(vec.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool
    {
        self.length <= 0
    }

    /// Reserved capacity within the `GenerationalVec`
    ///
    /// # Examples
    ///
    /// ```
    /// use gen_vec::closed::GenerationalVec;
    ///
    /// let mut vec: GenerationalVec<i32> = GenerationalVec::with_capacity(5);
    /// assert_eq!(vec.capacity(), 5);
    /// ```
    pub fn capacity(&self) -> usize
    {
        self.items.capacity()
    }

    /// Free all items
    ///
    /// Internal capacity will not change. Internally this
    /// is performed as all `Some(_)` being replaced with `None`
    /// 
    /// # Examples
    /// 
    /// ```
    /// use gen_vec::Index;
    /// use gen_vec::closed::GenerationalVec;
    /// 
    /// let mut vec: GenerationalVec<i32> = GenerationalVec::new();
    ///
    /// let index: Index = vec.insert(42);
    /// assert!(vec.contains(index));
    ///
    /// vec.clear();
    ///
    /// assert!(!vec.contains(index));
    /// ```
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

    /// Reserves extra space for *at least* `additional` more elements
    ///
    /// More space may be allocated to avoid frequent re-allocations
    /// (as per the specifications of std::vec::Vec)
    ///
    /// # Examples
    ///
    /// ```
    /// use gen_vec::Index;
    /// use gen_vec::closed::GenerationalVec;
    ///
    /// let mut vec: GenerationalVec<i32> = GenerationalVec::new();
    /// assert_eq!(vec.capacity(), 0);
    ///
    /// let index: Index = vec.insert(13);
    ///
    /// vec.reserve(4);
    /// assert!(vec.capacity() >= 4)
    /// ```
    pub fn reserve(&mut self, additional: usize)
    {
        if additional > 0
        {
            self.items.reserve(additional);
            self.free_indices.reserve(additional);

            if self.items.len() > 0
            {
                let last_index = self.items.len().saturating_sub(1);
                // Add all new reserved
                for i in last_index..(last_index+additional)
                {
                    self.free_indices.push_back(i);
                    self.items.push(None);
                }
            }
        }
    }

    /// Returns `true` if the `index` points to a valid item within
    ///
    /// # Examples
    ///
    /// ```
    /// use gen_vec::Index;
    /// use gen_vec::closed::GenerationalVec;
    ///
    /// let mut vec: GenerationalVec<i32> = GenerationalVec::new();
    ///
    /// let index: Index = vec.insert(124);
    /// assert!(vec.contains(index));
    ///
    /// vec.remove(index);
    /// assert!(!vec.contains(index));
    /// ```
    pub fn contains(&self, index: Index) -> bool
    {
        self.get(index).is_some()
    }

    /// Insert `value` and return an associated `Index`
    ///
    /// # Examples
    ///
    /// ```
    /// use gen_vec::Index;
    /// use gen_vec::closed::GenerationalVec;
    /// let mut vec: GenerationalVec<i32> = GenerationalVec::new();
    ///
    /// let index: Index = vec.insert(23);
    /// ```
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
                    Index{ index: self.items.len().saturating_sub(1), generation: self.generation }
                }
        }
    }

    /// Returns an immutable reference to the value of `index` if `index` is valid
    ///
    /// # Examples
    ///
    /// ```
    /// use gen_vec::Index;
    /// use gen_vec::closed::GenerationalVec;
    /// let mut vec: GenerationalVec<i32> = GenerationalVec::new();
    ///
    /// let index: Index = vec.insert(23);
    ///
    /// let value: &i32 = vec.get(index).unwrap();
    /// assert_eq!(*value, 23);
    /// ```
    pub fn get(&self, index: Index) -> Option<&T>
    {
        match self.items.get(index.index)
        {
            Some(Some(item)) if index.generation == item.generation => Some(&item.value),
            _ => None
        }
    }

    /// Returns a mutable reference to the value of `index` if `index` is valid
    ///
    /// # Examples
    ///
    /// ```
    /// use gen_vec::Index;
    /// use gen_vec::closed::GenerationalVec;
    /// let mut vec: GenerationalVec<i32> = GenerationalVec::new();
    ///
    /// let index: Index = vec.insert(23);
    ///
    /// let mut value: &mut i32 = vec.get_mut(index).unwrap();
    /// assert_eq!(*value, 23);
    ///
    /// *value = 0;
    /// let value = vec.get(index).unwrap();
    /// assert_eq!(*value, 0);
    /// ```
    pub fn get_mut(&mut self, index: Index) -> Option<&mut T>
    {
        match self.items.get_mut(index.index)
        {
            Some(Some(item)) if index.generation == item.generation => Some(&mut item.value),
            _ => None
        }
    }

    /// Returns the value of `index` if `index` is valid
    ///
    /// Afterwards, `index` is added to the pool of free indices
    /// available for reuse
    ///
    /// # Examples
    ///
    /// ```
    /// use gen_vec::Index;
    /// use gen_vec::closed::GenerationalVec;
    ///
    /// let mut vec: GenerationalVec<i32> = GenerationalVec::new();
    ///
    /// let index: Index = vec.insert(124);
    /// assert!(vec.contains(index));
    ///
    /// vec.remove(index);
    /// assert!(!vec.contains(index));
    /// ```
    pub fn remove(&mut self, index: Index) -> Option<T>
    {
        match self.items.get(index.index)
        {
            Some(Some(item)) if index.generation == item.generation =>
                {
                    let removed = std::mem::replace(&mut self.items[index.index], None).expect("replaced vec item");
                    self.generation += 1;
                    self.length = self.length.saturating_sub(1);
                    self.free_indices.push_back(index.index);
                    Some(removed.value)
                },
            _ => None
        }
    }
}



#[cfg(test)]
mod tests {
    use crate::closed::GenerationalVec;

    #[test]
    fn insert()
    {
        let mut vec = GenerationalVec::new();
        let index = vec.insert(3);
        assert_eq!(index.index, 0);
        assert_eq!(index.generation, 0);
        assert_eq!(vec.len(), 1);

        let index = vec.insert(4);
        assert_eq!(index.index, 1);
        assert_eq!(index.generation, 0);
        assert_eq!(vec.len(), 2);
    }

    #[test]
    fn get()
    {
        let mut vec = GenerationalVec::new();
        let index = vec.insert(3);
        let item = vec.get(index).unwrap();
        assert_eq!(*item, 3);

        let index = vec.insert(4);
        let item = vec.get(index).unwrap();
        assert_eq!(*item, 4);
    }

    #[test]
    fn get_mut()
    {
        let mut vec = GenerationalVec::new();
        let index = vec.insert(3);
        let item = vec.get_mut(index).unwrap();
        *item = 1;
        assert_eq!(*vec.get(index).unwrap(), 1);
    }

    #[test]
    fn remove()
    {
        let mut vec = GenerationalVec::new();
        let index = vec.insert(3);
        let item = vec.remove(index).unwrap();
        assert_eq!(item, 3);
        assert_eq!(vec.len(), 0);
        assert_eq!(vec.get(index), None);

        let index = vec.insert(4);
        assert_eq!(index.index, 0);
        assert_eq!(index.generation, 1);
        assert_eq!(vec.len(), 1);
    }

    #[test]
    fn clear()
    {
        let mut vec = GenerationalVec::new();
        vec.insert(4);
        let index = vec.insert(5);

        vec.clear();
        assert_eq!(vec.free_indices.len(), 2);
        assert!(!vec.contains(index));

        assert_eq!(vec.len(), 0);
        let index1 = vec.insert(1);
        assert_eq!(index1.index, 0);
        assert_eq!(index1.generation, index.generation+1);
        assert!(vec.contains(index1));
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
        let index = vec.insert(4);
        vec.remove(index);
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

    #[test]
    fn contains()
    {
        let mut vec = GenerationalVec::<i32>::new();
        let index = vec.insert(3);
        assert!(vec.contains(index));
        vec.remove(index);
        assert!(!vec.contains(index));
        let index = vec.insert(5);

        vec = GenerationalVec::new();
        assert!(!vec.contains(index));
    }
}