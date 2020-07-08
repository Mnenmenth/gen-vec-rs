use std::vec::Vec;
use std::collections::VecDeque;

#[derive(Clone, Copy, Debug)]
pub struct Index
{
    index: usize,
    generation: usize
}

#[derive(Clone, Debug)]
pub struct Item<T>
{
    value: T,
    generation: usize
}

#[derive(Debug)]
pub struct GiVec<T>
{
    free_indices: VecDeque<usize>,
    items: Vec<Option<Item<T>>>,
    generation: usize,
    length: usize
}

impl<T> GiVec<T>
{
    pub fn new() -> GiVec<T>
    {
        GiVec
        {
            free_indices: VecDeque::new(),
            items: Vec::new(),
            generation: 0,
            length: 0
        }
    }

    pub fn len(&self) -> usize
    {
        self.length
    }

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

    pub fn get(&self, index: Index) -> Option<&T>
    {
        match self.items.get(index.index)
        {
            Some(Some(item)) if index.generation == item.generation => Some(&item.value),
            _ => None
        }
    }

    pub fn get_mut(&mut self, index: Index) -> Option<&mut T>
    {
        match self.items.get_mut(index.index)
        {
            Some(Some(item)) if index.generation == item.generation => Some(&mut item.value),
            _ => None
        }
    }

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
    use crate::GiVec;

    #[test]
    fn insert()
    {
        let mut givec = GiVec::new();
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
        let mut givec = GiVec::new();
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
        let mut givec = GiVec::new();
        let i = givec.insert(3);
        let item = givec.get_mut(i).unwrap();
        *item = 1;
        assert_eq!(*givec.get(i).unwrap(), 1);
    }

    #[test]
    fn remove()
    {
        let mut givec = GiVec::new();
        let i = givec.insert(3);
        let item = givec.remove(i).unwrap();
        assert_eq!(item, 3);

        let i2 = givec.insert(4);
        assert_eq!(i2.index, 0);
        assert_eq!(i2.generation, 1);
    }
}
