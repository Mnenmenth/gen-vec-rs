use std::
{
    vec,
    vec::Vec,
    iter,
    slice
};
use crate::{Index, Item};

/// Generationally indexed vector
#[derive(Debug)]
pub struct ExposedGenVec<T>
{
    items: Vec<Option<Item<T>>>
}

impl<T> ExposedGenVec<T>
{
    /// Returns an empty `ExposedGenVec`
    ///
    /// # Examples
    ///
    /// ```
    /// use gen_vec::exposed::ExposedGenVec;
    /// let mut vec: ExposedGenVec<i32> = ExposedGenVec::new();
    /// ```
    pub fn new() -> ExposedGenVec<T>
    {
        ExposedGenVec
        {
            items: Vec::new()
        }
    }

    /// Returns a `ExposedGenVec` with initial capacity of `capacity`
    ///
    /// Allows the `ExposedGenVec` to hold `capacity` elements before
    /// allocating more space
    ///
    /// # Examples
    ///
    /// ```
    /// use gen_vec::exposed::ExposedGenVec;
    /// let mut vec: ExposedGenVec<i32> = ExposedGenVec::with_capacity(5);
    /// ```
    pub fn with_capacity(capacity: usize) -> ExposedGenVec<T>
    {
        ExposedGenVec
        {
            items: Vec::with_capacity(capacity)
        }
    }

    /// Reserved capacity within the `ExposedGenVec`
    ///
    /// # Examples
    ///
    /// ```
    /// use gen_vec::exposed::ExposedGenVec;
    /// let mut vec: ExposedGenVec<i32> = ExposedGenVec::with_capacity(5);
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
    /// use gen_vec::Index;
    /// use gen_vec::exposed::{IndexAllocator, ExposedGenVec};
    /// 
    /// let mut allocator: IndexAllocator = IndexAllocator::new();
    /// let index: Index = allocator.allocate();
    /// 
    /// let mut vec: ExposedGenVec<i32> = ExposedGenVec::new();
    /// assert_eq!(vec.capacity(), 0);
    ///
    /// vec.set(index, 0);
    ///
    /// vec.reserve(4);
    /// assert!(vec.capacity() >= 4);
    /// ```
    pub fn reserve(&mut self, additional: usize)
    {
        self.items.reserve(additional)
    }

    /// Returns `true` if the `index` points to a valid item
    ///
    /// # Examples
    ///
    /// ```
    /// use gen_vec::Index;
    /// use gen_vec::exposed::{IndexAllocator, ExposedGenVec};
    /// 
    /// let mut allocator: IndexAllocator = IndexAllocator::new();
    /// let index: Index = allocator.allocate();
    ///
    /// let mut vec: ExposedGenVec<i32> = ExposedGenVec::new();
    /// assert!(!vec.contains(index));
    /// vec.set(index, 0);
    /// assert!(vec.contains(index));
    /// ```
    pub fn contains(&self, index: Index) -> bool
    {
        self.get(index).is_some()
    }

    /// Set the value for the given `index` and returns the previous value (if any)
    ///
    /// This may overwrite past (but not future) generations
    ///
    /// # Examples
    ///
    /// ```
    /// use gen_vec::Index;
    /// use gen_vec::exposed::{IndexAllocator, ExposedGenVec};
    /// 
    /// let mut allocator: IndexAllocator = IndexAllocator::new();
    /// let index: Index = allocator.allocate();
    ///
    /// let mut vec: ExposedGenVec<i32> = ExposedGenVec::new();
    /// vec.set(index, 0);
    /// assert!(vec.contains(index));
    ///
    /// let replaced: i32 = vec.set(index, 1).expect("0");
    /// assert_eq!(replaced, 0);
    /// ```
    pub fn set(&mut self, index: Index, value: T) -> Option<T>
    {
        // If vec is smaller than the index, resize it and fill intermittent indices with None
        if self.items.len() < index.index + 1
        {
            self.items.resize_with(index.index + 1, | | None);
        }

        match self.items.get_mut(index.index)
        {
            Some(Some(item)) if item.generation <= index.generation =>
                {
                    match std::mem::replace(&mut self.items[index.index], Some(Item { value, generation: index.generation }))
                    {
                        Some(item) => Some(item.value),
                        _ => None
                    }
                },
            Some(_) =>
                {
                    self.items[index.index] = Some(Item { value, generation: index.generation });
                    None
                }
            _ => panic!(format!("Index is out of bounds despite internal vec being resized\n{:?}", index))
        }
    }

    /// Removes the value of `index` from the vec and internally sets the element to `None`
    ///
    /// # Examples
    ///
    /// ```
    /// use gen_vec::exposed::{IndexAllocator, ExposedGenVec};
    ///
    /// let mut allocater: IndexAllocator = IndexAllocator::new();
    /// let index = allocater.allocate();
    ///
    /// let mut vec: ExposedGenVec<i32> = ExposedGenVec::new();
    /// vec.set(index, 0);
    /// let replaced: Option<i32> = vec.set(index, 1);
    ///
    /// assert_eq!(replaced, Some(0));
    /// ```
    pub fn remove(&mut self, index: Index) -> Option<T>
    {
        match self.items.get(index.index)
        {
            Some(Some(item)) if index.generation == item.generation =>
                {
                    let removed = std::mem::replace(&mut self.items[index.index], None)
                                            .expect(format!("{:?} is None", index).as_str());
                    Some(removed.value)
                },
            _ => None
        }
    }

    /// Returns an immutable reference to the value of `index` if `index` is valid
    ///
    /// # Examples
    ///
    /// ```
    /// use gen_vec::Index;
    /// use gen_vec::exposed::{IndexAllocator, ExposedGenVec};
    /// 
    /// let mut allocator: IndexAllocator = IndexAllocator::new();
    /// let index: Index = allocator.allocate();
    ///
    /// let mut vec: ExposedGenVec<i32> = ExposedGenVec::new();
    /// vec.set(index, 0);
    /// let value: &i32 = vec.get(index).unwrap();
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
    /// use gen_vec::Index;
    /// use gen_vec::exposed::{IndexAllocator, ExposedGenVec};
    /// 
    /// let mut allocator: IndexAllocator = IndexAllocator::new();
    /// let index: Index = allocator.allocate();
    ///
    /// let mut vec: ExposedGenVec<i32> = ExposedGenVec::new();
    /// vec.set(index, 0);
    ///
    /// let mut value: &mut i32 = vec.get_mut(index).unwrap();
    /// assert_eq!(*value, 0);
    ///
    /// *value = 1;
    ///
    /// let value = vec.get(index).unwrap();
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

    /// Returns an iterator of immutable references to the vec elements
    ///
    /// Each iterator step returns (Index, &T)
    ///
    /// # Examples
    ///
    /// ```
    /// use gen_vec::Index;
    /// use gen_vec::exposed::{IndexAllocator, ExposedGenVec};
    ///
    /// let mut allocator: IndexAllocator = IndexAllocator::new();
    ///
    /// let mut vec: ExposedGenVec<i32> = ExposedGenVec::new();
    /// vec.set(allocator.allocate(), 0);
    /// vec.set(allocator.allocate(), 1);
    ///
    /// for (index, value) in vec.iter()
    /// {
    ///     println!("Index: {:?}, Value: {}", index, value);
    /// }
    ///
    /// // This works as well
    /// for (index, value) in vec
    /// {
    ///     println!("Index: {:?}, Value: {}", index, value);
    /// }
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
    /// use gen_vec::Index;
    /// use gen_vec::exposed::{IndexAllocator, ExposedGenVec};
    ///
    /// let mut allocator: IndexAllocator = IndexAllocator::new();
    ///
    /// let mut vec: ExposedGenVec<i32> = ExposedGenVec::new();
    /// vec.set(allocator.allocate(), 0);
    /// vec.set(allocator.allocate(), 1);
    ///
    /// for (index, value) in vec.iter_mut()
    /// {
    ///     *value = 30;
    /// }
    /// ```
    pub fn iter_mut(&mut self) -> IterMut<T>
    {
        IterMut
        {
            internal: self.items.iter_mut().enumerate()
        }
    }
}

/// Struct for consuming a `ExposedGenVec` into an iterator
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

impl<T> IntoIterator for ExposedGenVec<T>
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

/// Struct for creating an iterator over an immutable `ExposedGenVec` reference
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

impl<'a, T> IntoIterator for &'a ExposedGenVec<T>
{
    type Item = (Index, &'a T);
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter
    {
        self.iter()
    }
}

/// Struct for creating an iterator over a mutable `ExposedGenVec` reference
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

impl<'a, T> IntoIterator for &'a mut ExposedGenVec<T>
{
    type Item = (Index, &'a mut T);
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter
    {
        self.iter_mut()
    }
}

impl<T> std::ops::Index<Index> for ExposedGenVec<T>
{
    type Output = T;

    fn index(&self, index: Index) -> &Self::Output
    {
        self.get(index).expect(format!("Invalid index: {:?}", index).as_str())
    }
}

impl<T> std::ops::IndexMut<Index> for ExposedGenVec<T>
{
    fn index_mut(&mut self, index: Index) -> &mut Self::Output
    {
        self.get_mut(index).expect(format!("Invalid index: {:?}", index).as_str())
    }
}

#[cfg(test)]
mod vec_tests
{
    use crate::exposed::*;

    #[test]
    fn capacity()
    {
        let mut vec = ExposedGenVec::<i32>::new();
        assert_eq!(vec.capacity(), 0);
        vec.reserve(4);
        assert_eq!(vec.capacity(), 4);
    }

    #[test]
    fn set()
    {
        let mut allocator = IndexAllocator::new();
        let index = allocator.allocate();

        let mut vec = ExposedGenVec::new();
        let replaced = vec.set(index, 0);
        assert!(vec.contains(index));
        assert_eq!(replaced, None);

        allocator.deallocate(index);
        let index1 = allocator.allocate();

        let replaced = vec.set(index1, 1);
        assert!(!vec.contains(index));
        assert!(vec.contains(index1));
        assert_eq!(replaced, Some(0));

        for _ in 0..50
        {
            allocator.allocate();
        }

        let index = allocator.allocate();
        let replaced = vec.set(index, 20);
        assert!(vec.contains(index));
        assert_eq!(*vec.get(index).unwrap(), 20);
        assert_eq!(replaced, None);
    }

    #[test]
    fn get()
    {
        let mut allocator = IndexAllocator::new();
        let index = allocator.allocate();
        let index1 = allocator.allocate();

        let mut vec = ExposedGenVec::new();
        vec.set(index, 0);
        vec.set(index1, 1);

        assert_eq!(*vec.get(index).unwrap(), 0);
        assert_eq!(*vec.get(index1).unwrap(), 1);

        *vec.get_mut(index).unwrap() = 2;
        assert_eq!(*vec.get(index).unwrap(), 2);
    }

    #[test]
    fn iter()
    {
        let mut allocator = IndexAllocator::new();
        let index = allocator.allocate();
        let index1 = allocator.allocate();

        let mut vec = ExposedGenVec::<i32>::new();
        vec.set(index, 4);
        vec.set(index1, 5);

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
        let mut allocator = IndexAllocator::new();
        let index = allocator.allocate();
        let index1 = allocator.allocate();

        let mut vec = ExposedGenVec::<i32>::new();
        vec.set(index, 4);
        vec.set(index1, 5);

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
        let mut allocator = IndexAllocator::new();
        let index = allocator.allocate();

        let mut vec = ExposedGenVec::<i32>::new();
        vec.set(index, 4);

        assert_eq!(vec[index], 4);
    }

    #[test]
    fn index_mut()
    {
        let mut allocator = IndexAllocator::new();
        let index = allocator.allocate();

        let mut vec = ExposedGenVec::<i32>::new();
        vec.set(index, 3);

        vec[index] = 5;
        assert_eq!(*vec.get(index).unwrap(), 5);
    }
}