use std::
{
    vec,
    vec::Vec,
    collections::VecDeque,
    iter,
    slice
};
use crate::{Index, Item};

/// A vector with reusable indices
#[derive(Debug)]
pub struct ClosedGenVec<T>
{
    free_indices: VecDeque<usize>,
    items: Vec<Option<Item<T>>>,
    generation: usize,
    length: usize
}

impl<T> ClosedGenVec<T>
{
    /// Returns an empty `ClosedGenVec`
    ///
    /// # Examples
    ///
    /// ```
    /// use gen_vec::closed::ClosedGenVec;
    /// let mut vec: ClosedGenVec<i32> = ClosedGenVec::new();
    /// ```
    pub fn new() -> ClosedGenVec<T>
    {
        ClosedGenVec
        {
            free_indices: VecDeque::new(),
            items: Vec::new(),
            generation: 0,
            length: 0
        }
    }

    /// Returns a `ClosedGenVec` with initial capacity of `capacity`
    ///
    /// Allows the `ClosedGenVec` to hold `capacity` elements before
    /// allocating more space
    ///
    /// # Examples
    ///
    /// ```
    /// use gen_vec::closed::ClosedGenVec;
    /// let mut vec: ClosedGenVec<i32> = ClosedGenVec::with_capacity(5);
    /// ```
    pub fn with_capacity(capacity: usize) -> ClosedGenVec<T>
    {
        ClosedGenVec
        {
            free_indices: VecDeque::with_capacity(capacity),
            items: Vec::with_capacity(capacity),
            generation: 0,
            length: 0
        }
    }

    /// Number of active `Item`s within the `ClosedGenVec`
    ///
    /// The internal item vec may actually be larger depending on the number of freed indices
    ///
    /// # Examples
    /// ```
    /// use gen_vec::Index;
    /// use gen_vec::closed::ClosedGenVec;
    ///
    /// let mut vec: ClosedGenVec<i32> = ClosedGenVec::new();
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
    /// use gen_vec::closed::ClosedGenVec;
    ///
    /// let mut vec: ClosedGenVec<i32> = ClosedGenVec::new();
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

    /// Reserved capacity within the `ClosedGenVec`
    ///
    /// # Examples
    ///
    /// ```
    /// use gen_vec::closed::ClosedGenVec;
    ///
    /// let mut vec: ClosedGenVec<i32> = ClosedGenVec::with_capacity(5);
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
    /// use gen_vec::closed::ClosedGenVec;
    /// 
    /// let mut vec: ClosedGenVec<i32> = ClosedGenVec::new();
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
    /// use gen_vec::closed::ClosedGenVec;
    ///
    /// let mut vec: ClosedGenVec<i32> = ClosedGenVec::new();
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
    /// use gen_vec::closed::ClosedGenVec;
    ///
    /// let mut vec: ClosedGenVec<i32> = ClosedGenVec::new();
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
    /// use gen_vec::closed::ClosedGenVec;
    /// let mut vec: ClosedGenVec<i32> = ClosedGenVec::new();
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
    /// use gen_vec::closed::ClosedGenVec;
    /// let mut vec: ClosedGenVec<i32> = ClosedGenVec::new();
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
    /// use gen_vec::closed::ClosedGenVec;
    /// let mut vec: ClosedGenVec<i32> = ClosedGenVec::new();
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
    /// use gen_vec::closed::ClosedGenVec;
    ///
    /// let mut vec: ClosedGenVec<i32> = ClosedGenVec::new();
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

    /// Returns an iterator of immutable references to the vec elements
    ///
    /// Each iterator step returns (Index, &T)
    ///
    /// # Examples
    ///
    /// ```
    /// use gen_vec::closed::ClosedGenVec;
    ///
    /// let mut vec: ClosedGenVec<i32> = ClosedGenVec::new();
    /// vec.insert(0);
    /// vec.insert(1);
    ///
    /// for (index, value) in vec // vec.iter() is valid too
    /// {
    ///     println!("Index: {:?}, Value: {}", index, value);
    /// }
    ///
    /// ```
    pub fn iter(&self) -> Iter<T>
    {
        Iter
        {
            internal: self.items.iter().enumerate()
        }
    }

    /// Returns an iterator of mutable references to the vec elements
    ///
    /// Each iterator step returns (Index, &mut T)
    ///
    /// # Examples
    ///
    /// ```
    /// use gen_vec::closed::ClosedGenVec;
    ///
    /// let mut vec: ClosedGenVec<i32> = ClosedGenVec::new();
    /// vec.insert(0);
    /// vec.insert(1);
    ///
    /// for (index, value) in vec.iter_mut()
    /// {
    ///     *value = 0;
    /// }
    ///
    /// ```
    pub fn iter_mut(&mut self) -> IterMut<T>
    {
        IterMut
        {
            internal: self.items.iter_mut().enumerate()
        }
    }

}

/// Struct for consuming a `ClosedGenVec` into an iterator
pub struct IntoIter<T>
{
    internal: iter::Enumerate<vec::IntoIter<Option<Item<T>>>>
}

impl<T> Iterator for IntoIter<T>
{
    type Item = (Index, T);

    fn next(&mut self) -> Option<Self::Item>
    {
        loop
        {
            match self.internal.next()
            {
                Some((_, None)) => { continue; },
                Some((index, Some(item))) => return Some((Index { index, generation: item.generation}, item.value)),
                _ => return None
            };
        }
    }
}

impl<T> IntoIterator for ClosedGenVec<T>
{
    type Item = (Index, T);
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter
    {
        IntoIter
        {
            internal: self.items.into_iter().enumerate()
        }
    }
}

/// Struct for creating an iterator over an immutable `ClosedGenVec` reference
pub struct Iter<'a, T: 'a>
{
    internal: iter::Enumerate<slice::Iter<'a, Option<Item<T>>>>
}

impl<'a, T> Iterator for Iter<'a, T>
{
    type Item = (Index, &'a T);

    fn next(&mut self) -> Option<Self::Item>
    {
        loop
        {
            match self.internal.next()
            {
                Some((_, None)) => { continue; },
                Some((index, Some(item))) => return Some((Index { index, generation: item.generation}, &item.value)),
                _ => return None
            };
        }
    }
}

impl<'a, T> IntoIterator for &'a ClosedGenVec<T>
{
    type Item = (Index, &'a T);
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter
    {
        self.iter()
    }
}

/// Struct for creating an iterator over a mutable `ClosedGenVec` reference
pub struct IterMut<'a, T: 'a>
{
    internal: iter::Enumerate<slice::IterMut<'a, Option<Item<T>>>>
}

impl<'a, T: 'a> Iterator for IterMut<'a, T>
{
    type Item = (Index, &'a mut T);

    fn next(&mut self) -> Option<Self::Item>
    {
        loop
        {
            match self.internal.next()
            {
                Some((_, None)) => { continue; },
                Some((index, Some(item))) => return Some((Index { index, generation: item.generation}, &mut item.value)),
                _ => return None
            };
        }
    }
}

impl<'a, T> IntoIterator for &'a mut ClosedGenVec<T>
{
    type Item = (Index, &'a mut T);
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter
    {
        self.iter_mut()
    }
}

impl<T> std::ops::Index<Index> for ClosedGenVec<T>
{
    type Output = T;

    fn index(&self, index: Index) -> &Self::Output
    {
        self.get(index).expect(format!("Invalid index: {:?}", index).as_str())
    }
}

impl<T> std::ops::IndexMut<Index> for ClosedGenVec<T>
{
    fn index_mut(&mut self, index: Index) -> &mut Self::Output
    {
        self.get_mut(index).expect(format!("Invalid index: {:?}", index).as_str())
    }
}

#[cfg(test)]
mod tests {
    use crate::closed::ClosedGenVec;

    #[test]
    fn insert()
    {
        let mut vec = ClosedGenVec::new();
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
        let mut vec = ClosedGenVec::new();
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
        let mut vec = ClosedGenVec::new();
        let index = vec.insert(3);
        let item = vec.get_mut(index).unwrap();
        *item = 1;
        assert_eq!(*vec.get(index).unwrap(), 1);
    }

    #[test]
    fn remove()
    {
        let mut vec = ClosedGenVec::new();
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
        let mut vec = ClosedGenVec::new();
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
        let mut vec = ClosedGenVec::new();
        assert_eq!(vec.len(), 0);
        vec.insert(3);
        assert_eq!(vec.len(), 1);
    }

    #[test]
    fn is_empty()
    {
        let mut vec = ClosedGenVec::new();
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
        let mut vec = ClosedGenVec::<i32>::new();
        assert_eq!(vec.capacity(), 0);

        vec.reserve(24);
        assert!(vec.capacity() >= 24);

        vec = ClosedGenVec::with_capacity(5);
        assert_eq!(vec.capacity(), 5);
    }

    #[test]
    fn contains()
    {
        let mut vec = ClosedGenVec::<i32>::new();
        let index = vec.insert(3);
        assert!(vec.contains(index));
        vec.remove(index);
        assert!(!vec.contains(index));
        let index = vec.insert(5);

        vec = ClosedGenVec::new();
        assert!(!vec.contains(index));
    }

    #[test]
    fn into_iter()
    {
        let mut vec = ClosedGenVec::<i32>::new();
        let index = vec.insert(4);
        let index1 = vec.insert(5);

        for (i, value) in vec
        {
            if i == index
            {
                assert_eq!(value, 4)
            }
            else if i == index1
            {
                assert_eq!(value, 5);
            }
        }
    }

    #[test]
    fn iter()
    {
        let mut vec = ClosedGenVec::<i32>::new();
        let index = vec.insert(4);
        let index1 = vec.insert(5);

        let mut iter = vec.iter();

        let (i, value) = iter.next().unwrap();
        assert_eq!(i, index);
        assert_eq!(*value, 4);

        let (i, value) = iter.next().unwrap();
        assert_eq!(i, index1);
        assert_eq!(*value, 5);
    }

    #[test]
    fn iter_mut()
    {
        let mut vec = ClosedGenVec::<i32>::new();
        let index = vec.insert(4);
        let index1 = vec.insert(5);

        let mut iter = vec.iter_mut();

        let (i, value) = iter.next().unwrap();
        assert_eq!(i, index);
        assert_eq!(*value, 4);
        *value = 0;

        let (i, value) = iter.next().unwrap();
        assert_eq!(i, index1);
        assert_eq!(*value, 5);
        *value = 1;

        assert_eq!(*vec.get(index).unwrap(), 0);
        assert_eq!(*vec.get(index1).unwrap(), 1);
    }

    #[test]
    fn index()
    {
        let mut vec = ClosedGenVec::<i32>::new();
        let index = vec.insert(4);

        assert_eq!(vec[index], 4);
    }

    #[test]
    fn index_mut()
    {
        let mut vec = ClosedGenVec::<i32>::new();
        let index = vec.insert(4);
        vec[index] = 5;

        assert_eq!(vec[index], 5);
    }
}