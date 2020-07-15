use crate::
{
    Index,
    exposed::{IndexAllocator, ExposedGenVec, IntoIter, Iter, IterMut}
};

#[derive(Default, Debug)]
pub struct ClosedGenVec<T>
{
    allocator: IndexAllocator,
    vec: ExposedGenVec<T>
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
            allocator: IndexAllocator::new(),
            vec: ExposedGenVec::new()
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
            allocator: IndexAllocator::with_capacity(capacity),
            vec: ExposedGenVec::with_capacity(capacity)
        }
    }

    /// Number of active `Item`s within the vec
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
        self.allocator.num_active()
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
        self.allocator.num_active() == 0
    }

    /// Reserved capacity within the vec
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
        self.allocator.capacity()
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
        self.allocator.reserve(additional);
        self.vec.reserve(additional);
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
        let index = self.allocator.allocate();
        self.vec.set(index, value);
        index
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
        self.allocator.is_active(index)
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
        let removed = self.vec.remove(index);
        self.allocator.deallocate(index);
        removed
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
        self.allocator.deallocate_all();
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
    /// let value: Option<&i32> = vec.get(index);
    /// assert_eq!(value, Some(&23));
    /// ```
    pub fn get(&self, index: Index) -> Option<&T>
    {
        self.vec.get(index)
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
    /// let mut value: Option<&mut i32> = vec.get_mut(index);
    /// assert_eq!(value, Some(&mut 23));
    ///
    /// if let Some(value) = value
    /// {
    ///     *value = 0;
    /// }
    ///
    /// let value: Option<&i32> = vec.get(index);
    /// assert_eq!(value, Some(&0));
    /// ```
    pub fn get_mut(&mut self, index: Index) -> Option<&mut T>
    {
        self.vec.get_mut(index)
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
    /// ```
    pub fn iter(&self) -> Iter<T>
    {
        self.vec.iter()
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
        self.vec.iter_mut()
    }
}

impl<T> IntoIterator for ClosedGenVec<T>
{
    type Item = (Index, T);
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter
    {
        self.vec.into_iter()
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
        self.get(index).expect(format!("Index should be valid: {:?}", index).as_str())
    }
}

impl<T> std::ops::IndexMut<Index> for ClosedGenVec<T>
{
    fn index_mut(&mut self, index: Index) -> &mut Self::Output
    {
        self.get_mut(index).expect(format!("Index should be valid: {:?}", index).as_str())
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
        let value = vec.get(index);
        assert_eq!(value, Some(&3));

        let index = vec.insert(4);
        let value = vec.get(index);
        assert_eq!(value, Some(&4));
    }

    #[test]
    fn get_mut()
    {
        let mut vec = ClosedGenVec::new();
        let index = vec.insert(3);
        let value = vec.get_mut(index);
        if let Some(value) = value
        {
            *value = 1;
        }

        let value = vec.get(index);
        assert_eq!(value, Some(&1));
    }

    #[test]
    fn remove()
    {
        let mut vec = ClosedGenVec::new();
        let index = vec.insert(3);
        let value = vec.remove(index);
        assert_eq!(value, Some(3));
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
        let index = vec.insert(4);
        let index1 = vec.insert(5);

        vec.clear();
        assert!(!vec.contains(index));
        assert!(!vec.contains(index1));

        assert_eq!(vec.len(), 0);
        let index1 = vec.insert(1);
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

        let (i, value) = iter.next().expect("Iterator should have next");
        assert_eq!(i, index);
        assert_eq!(*value, 4);

        let (i, value) = iter.next().expect("Iterator should have next");
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

        let (i, value) = iter.next().expect("Iterator should have next");
        assert_eq!(i, index);
        assert_eq!(*value, 4);
        *value = 0;

        let (i, value) = iter.next().expect("Iterator should have next");
        assert_eq!(i, index1);
        assert_eq!(*value, 5);
        *value = 1;

        let value = vec.get(index);
        assert_eq!(value, Some(&0));

        let value = vec.get(index1);
        assert_eq!(value, Some(&1));
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