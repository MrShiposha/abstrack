use std::ops::{Index, IndexMut};

pub type Result<T> = std::result::Result<(), Error<T>>;

#[derive(Debug)]
pub enum Error<T> {
    Overflow(T)
}

/// Track internal buffer (ring buffer)
pub struct Buffer<T: Default + Clone> {
    inner: Vec<T>,
    start_index: usize,
    len: usize,
    is_reversed: bool,
}

impl<T: Default + Clone> Buffer<T> {
    pub fn new(size: usize) -> Self {
        assert!(size > 0);

        Self {
            inner: vec![Default::default(); size],
            start_index: 0,
            len: 0,
            is_reversed: false,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn capacity(&self) -> usize {
        self.inner.len()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn grow(&mut self) {
        let capacity = self.capacity();
        let cap_diff = capacity / 2;

        let new_cap = capacity + cap_diff;
        self.inner.resize_with(new_cap, Default::default);

        let mut stop_idx = 0;

        for copy_idx in 0..self.start_index {
            if copy_idx == cap_diff {
                stop_idx = self.start_index - copy_idx;

                break;
            }

            self.inner.swap(copy_idx, copy_idx + self.len);
        }

        for copy_idx in 0..stop_idx {
            self.inner.swap(copy_idx, copy_idx + cap_diff);
        }
    }

    pub fn clear(&mut self) -> Truncated<T> {
        let old_start_index = self.start_index;
        let old_len = self.len;

        self.start_index = 0;
        self.len = 0;

        let cleared = Truncated::new(self, old_start_index, old_len);

        cleared
    }

    pub fn first(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            Some(&self.inner[self.start_index])
        }
    }

    pub fn first_mut(&mut self) -> Option<&mut T> {
        if self.is_empty() {
            None
        } else {
            Some(&mut self.inner[self.start_index])
        }
    }

    pub fn last(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            Some(&self.inner[self.wrap_index(self.len - 1)])
        }
    }

    pub fn last_mut(&mut self) -> Option<&mut T> {
        if self.is_empty() {
            None
        } else {
            let index = self.wrap_index(self.len - 1);
            Some(&mut self.inner[index])
        }
    }

    pub fn iter(&self) -> Iter<T> {
        Iter {
            buffer: self,
            index: 0,
        }
    }

    pub fn reverse(&mut self) {
        self.is_reversed = !self.is_reversed;
    }

    pub fn is_reversed(&self) -> bool {
        self.is_reversed
    }

    /// Add new item at the end
    /// Returns an error when the buffer is full
    pub fn try_push(&mut self, el: T) -> Result<T> {
        let capacity = self.capacity();

        if self.len + 1 > capacity {
            Err(Error::Overflow(el))
        } else if self.is_reversed {
            let (mut start_index, is_overflowed) = self.start_index.overflowing_sub(1);
            if is_overflowed {
                start_index = capacity - 1;
            }

            self.start_index = start_index;
            self.inner[self.start_index] = el;
            self.len += 1;

            Ok(())
        } else {
            let index = self.wrap_raw_index(self.start_index + self.len);
            self.inner[index] = el;
            self.len += 1;

            Ok(())
        }
    }

    pub fn try_append<I: IntoIterator<Item = T>>(&mut self, iter: I) -> Result<T> {
        for item in iter {
            self.try_push(item)?;
        }

        Ok(())
    }

    pub fn truncate_back(&mut self, mut index: usize) -> Truncated<T> {
        if self.is_empty() {
            return Truncated::empty(self);
        }

        if index >= self.len {
            index = self.len - 1;
        }

        if self.is_reversed {
            self.len -= index;
            Truncated::new(
                self, 
                self.wrap_raw_index(self.start_index + self.len), 
                index
            )
        } else {    
            let old_start_index = self.start_index;
            self.start_index = self.wrap_index(index);
            self.len -= index;
    
            Truncated::new(self, old_start_index, index)
        }
    }

    pub fn truncate_forward(&mut self, index: usize) -> Truncated<T> {
        if self.is_empty() {
            return Truncated::empty(self);
        }

        let old_len = self.len;

        if index < self.len {            
            if self.is_reversed {
                let old_start_index = self.start_index;
                self.start_index = self.wrap_index(index);
                self.len = index + 1;

                Truncated::new(
                    self, 
                    old_start_index, 
                    old_len - self.len
                )
            } else {
                self.len = index + 1;

                Truncated::new(
                    self, 
                    self.wrap_raw_index(self.start_index + self.len), 
                    old_len - self.len
                )
            }
        } else {
            Truncated::empty(self)
        }
    }

    pub fn get(&self, index: usize) -> &T {
        unsafe {
            self.inner.get_unchecked(self.wrap_index(index))
        }
    }

    pub fn get_mut(&mut self, index: usize) -> &mut T {
        unsafe {
            let index = self.wrap_index(index);
            self.inner.get_unchecked_mut(index)
        }
    }

    fn wrap_index(&self, mut index: usize) -> usize {
        if self.is_reversed {
            index = self.len - index - 1;
        }

        self.wrap_raw_index(self.start_index + index)
    }

    pub fn wrap_raw_index(&self, index: usize) -> usize {
        index % self.capacity()
    }
}

impl<T> Index<usize> for Buffer<T>
where
    T: Default + Clone,
{
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index)
    }
}

impl<T> IndexMut<usize> for Buffer<T>
where
    T: Default + Clone,
{
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index)
    }
}

pub struct Iter<'rb, T: Default + Clone> {
    buffer: &'rb Buffer<T>,
    index: usize,
}

impl<'rb, T: Default + Clone> Iterator for Iter<'rb, T> {
    type Item = &'rb T;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.buffer.len() {
            None
        } else {
            let index = self.index;
            self.index += 1;

            Some(self.buffer.get(index))
        }
    }
}

/// Iterator that contains truncated elements of `Buffer`
pub struct Truncated<'rb, T: Default + Clone> {
    buffer: &'rb mut Buffer<T>,
    index_base: usize,
    index: usize,
    len: usize
}

impl<'rb, T: Default + Clone> Truncated<'rb, T> {
    pub fn new(
        buffer: &'rb mut Buffer<T>, 
        index_base: usize, 
        len: usize
    ) -> Self {
        Self {
            buffer,
            index_base,
            index: 0,
            len
        }
    }

    pub fn empty(buffer: &'rb mut Buffer<T>) -> Self {
        Self {
            buffer,
            index_base: 0,
            index: 0,
            len: 0
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn peek_first(&mut self) -> Option<<Self as Iterator>::Item> {
        <Self as Iterator>::nth(self, 0)
    }

    pub fn peek_last(&mut self) -> Option<<Self as Iterator>::Item> {
        <Self as Iterator>::nth(self, self.len - 1)
    }

    unsafe fn get_option_mut(&mut self, index: usize) -> Option<<Self as Iterator>::Item> {
        let item = self.buffer.inner.get_unchecked_mut(index);

        Some(&mut *(item as *mut _))
    }

    fn wrap_index(&self, index: usize) -> usize {
        let index = if self.buffer.is_reversed {
            self.index_base + (self.len - index - 1)
        } else {
            self.index_base + index
        };

        self.buffer.wrap_raw_index(index)
    }
}

impl<'rb, T: Default + Clone> Iterator for Truncated<'rb, T> {
    type Item = &'rb mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index != self.len {
            unsafe {
                let old_index = self.index;
                self.index += 1;

                self.get_option_mut(
                    self.wrap_index(old_index)
                )
            }
        } else {
            None
        }
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        if n < self.len {
            unsafe {
                self.get_option_mut(self.wrap_index(self.index + n))
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        buf::{
            Buffer,
            Result,
        }
    };

    #[test]
    fn test_try_push() -> Result<u32> {
        let mut buffer = Buffer::<u32>::new(3);
        test_try_push_helper(&mut buffer)?;

        buffer.clear();
        buffer.start_index = 1;
        test_try_push_helper(&mut buffer)?;


        buffer.clear();
        buffer.start_index = 2;
        test_try_push_helper(&mut buffer)?;
        
        Ok(())
    }

    fn test_try_push_helper(buffer: &mut Buffer<u32>) -> Result<u32> {
        assert!(buffer.is_empty());
        assert!(buffer.len() == 0);

        buffer.try_push(1)?;
        assert!(!buffer.is_empty());
        assert!(buffer.len() == 1);
        assert_eq!(buffer[0], 1);

        buffer.try_push(2)?;
        assert!(!buffer.is_empty());
        assert!(buffer.len() == 2);
        assert_eq!(buffer[0], 1);
        assert_eq!(buffer[1], 2);

        buffer.try_push(3)?;
        assert!(!buffer.is_empty());
        assert!(buffer.len() == 3);
        assert_eq!(buffer[0], 1);
        assert_eq!(buffer[1], 2);
        assert_eq!(buffer[2], 3);

        assert!(buffer.try_push(4).is_err());

        Ok(())
    }

    #[test]
    fn test_append() -> Result<u32> {
        let mut buffer = Buffer::<u32>::new(3);
        assert!(buffer.is_empty());
        assert!(buffer.len() == 0);

        buffer.try_append(vec![1, 2, 3])?;

        assert!(!buffer.is_empty());
        assert!(buffer.len() == 3);
        assert_eq!(buffer[0], 1);
        assert_eq!(buffer[1], 2);
        assert_eq!(buffer[2], 3);

        Ok(())
    }

    #[test]
    fn test_grow() -> Result<u32> {
        let mut buffer = Buffer::<u32>::new(3);
        test_grow_helper(&mut buffer)?;

        buffer = Buffer::<u32>::new(3);
        buffer.start_index = 1;
        test_grow_helper(&mut buffer)?;

        buffer = Buffer::<u32>::new(3);
        buffer.start_index = 2;
        test_grow_helper(&mut buffer)?;

        let mut buffer = Buffer::<u32>::new(4);
        test_grow_helper(&mut buffer)?;

        Ok(())
    }

    #[test]
    fn test_ring() -> Result<u32> {
        let mut buffer = Buffer::<u32>::new(3);
        buffer.try_append(vec![1, 2, 3])?;

        assert_eq!(buffer[0], 1);
        assert_eq!(buffer[1], 2);
        assert_eq!(buffer[2], 3);

        assert_eq!(buffer[3], 1);
        assert_eq!(buffer[4], 2);
        assert_eq!(buffer[5], 3);

        Ok(())
    }

    #[test]
    fn test_reverse() -> Result<u32> {
        let mut buffer = Buffer::<u32>::new(3);
        test_reverse_helper(&mut buffer)?;

        buffer.clear();
        buffer.start_index = 1;
        test_reverse_helper(&mut buffer)?;

        buffer.clear();
        buffer.start_index = 2;
        test_reverse_helper(&mut buffer)?;

        Ok(())
    }

    fn test_reverse_helper(buffer: &mut Buffer<u32>) -> Result<u32> {
        assert!(buffer.is_empty());
        assert!(buffer.len() == 0);

        buffer.try_append(vec![1, 2, 3])?;
        buffer.reverse();
        assert_eq!(buffer[0], 3);
        assert_eq!(buffer[1], 2);
        assert_eq!(buffer[2], 1);

        buffer.reverse();
        assert_eq!(buffer[0], 1);
        assert_eq!(buffer[1], 2);
        assert_eq!(buffer[2], 3);

        Ok(())
    }

    #[test]
    fn test_reverse_push() -> Result<u32> {
        let mut buffer = Buffer::<u32>::new(3);
        test_reverse_push_helper(&mut buffer)?;

        buffer.clear();
        buffer.start_index = 1;
        test_reverse_push_helper(&mut buffer)?;

        buffer.clear();
        buffer.start_index = 2;
        test_reverse_push_helper(&mut buffer)?;

        Ok(())
    }

    fn test_reverse_push_helper(buffer: &mut Buffer<u32>) -> Result<u32> {
        assert!(buffer.is_empty());
        assert!(buffer.len() == 0);
        let start_index = buffer.start_index;

        buffer.try_append(vec![1, 2])?;
        buffer.reverse();
        buffer.try_push(3)?;
        assert_eq!(buffer[0], 2);
        assert_eq!(buffer[1], 1);
        assert_eq!(buffer[2], 3);

        buffer.clear();
        buffer.start_index = start_index;

        buffer.try_append(vec![1])?;
        buffer.reverse();
        buffer.try_push(2)?;
        buffer.try_push(3)?;
        assert_eq!(buffer[0], 1);
        assert_eq!(buffer[1], 2);
        assert_eq!(buffer[2], 3);

        let start_index = buffer.start_index;
        buffer.clear();
        buffer.start_index = start_index;
        
        buffer.reverse();
        buffer.try_push(1)?;
        buffer.try_push(2)?;
        buffer.try_push(3)?;
        assert_eq!(buffer[0], 1);
        assert_eq!(buffer[1], 2);
        assert_eq!(buffer[2], 3);

        Ok(())
    }

    fn test_grow_helper(buffer: &mut Buffer<u32>) -> Result<u32> {
        assert!(buffer.is_empty());
        assert!(buffer.len() == 0);

        buffer.try_append(vec![1, 2, 3])?;

        if buffer.capacity() < 4 {
            assert!(buffer.try_push(4).is_err());
        }

        buffer.grow();
        buffer.try_push(4)?;

        assert_eq!(buffer[0], 1);
        assert_eq!(buffer[1], 2);
        assert_eq!(buffer[2], 3);
        assert_eq!(buffer[3], 4);

        if buffer.capacity() < 5 {
            assert!(buffer.try_push(5).is_err());
        }

        buffer.grow();

        buffer.try_push(5)?;
        buffer.try_push(6)?;

        if buffer.capacity() < 7 {
            assert!(buffer.try_push(7).is_err());
        }

        assert_eq!(buffer[0], 1);
        assert_eq!(buffer[1], 2);
        assert_eq!(buffer[2], 3);
        assert_eq!(buffer[3], 4);
        assert_eq!(buffer[4], 5);
        assert_eq!(buffer[5], 6);

        Ok(())
    }

    #[test]
    fn test_truncate_back() -> Result<u32> {
        let mut buffer = Buffer::<u32>::new(3);
        test_truncate_back_helper(&mut buffer)?;

        buffer.clear();
        buffer.start_index = 1;
        test_truncate_back_helper(&mut buffer)?;

        buffer.clear();
        buffer.start_index = 2;
        test_truncate_back_helper(&mut buffer)?;

        buffer.clear();
        buffer.reverse();
        test_truncate_back_helper(&mut buffer)?;

        buffer.clear();
        buffer.reverse();
        buffer.start_index = 1;
        test_truncate_back_helper(&mut buffer)?;

        buffer.clear();
        buffer.reverse();
        buffer.start_index = 2;
        test_truncate_back_helper(&mut buffer)?;

        buffer.clear();
        let t = buffer.truncate_back(0);
        assert_eq!(t.map(|i| *i).collect::<Vec<_>>(), vec![]);

        Ok(())
    }

    fn test_truncate_back_helper(buffer: &mut Buffer<u32>) -> Result<u32> {
        assert!(buffer.is_empty());
        assert!(buffer.len() == 0);
        let start_index = buffer.start_index;

        buffer.try_append(vec![1, 2, 3])?;

        let t = buffer.truncate_back(0);
        assert_eq!(t.map(|i| *i).collect::<Vec<_>>(), vec![]);
        assert!(buffer.len() == 3);
        assert!(buffer[0] == 1);
        assert!(buffer[1] == 2);
        assert!(buffer[2] == 3);

        let t = buffer.truncate_back(3);
        assert_eq!(t.map(|i| *i).collect::<Vec<_>>(), vec![1, 2]);
        assert!(buffer.len() == 1);
        assert!(buffer[0] == 3);

        buffer.clear();
        buffer.start_index = start_index;
        buffer.try_append(vec![1, 2, 3])?;

        let t = buffer.truncate_back(1);
        assert_eq!(t.map(|i| *i).collect::<Vec<_>>(), vec![1]);
        assert!(buffer.len() == 2);
        assert!(buffer[0] == 2);
        assert!(buffer[1] == 3);

        buffer.clear();
        buffer.start_index = start_index;
        buffer.try_append(vec![1, 2, 3])?;
        let t = buffer.truncate_back(2);
        assert_eq!(t.map(|i| *i).collect::<Vec<_>>(), vec![1, 2]);
        assert!(buffer.len() == 1);
        assert!(buffer[0] == 3);

        Ok(())
    }

    #[test]
    fn test_truncate_foward() -> Result<u32> {
        let mut buffer = Buffer::<u32>::new(3);
        test_truncate_forward_helper(&mut buffer)?;

        buffer.clear();
        buffer.start_index = 1;
        test_truncate_forward_helper(&mut buffer)?;

        buffer.clear();
        buffer.start_index = 2;
        test_truncate_forward_helper(&mut buffer)?;

        buffer.clear();
        buffer.reverse();
        test_truncate_forward_helper(&mut buffer)?;

        buffer.clear();
        buffer.reverse();
        buffer.start_index = 1;
        test_truncate_forward_helper(&mut buffer)?;

        buffer.clear();
        buffer.reverse();
        buffer.start_index = 2;
        test_truncate_forward_helper(&mut buffer)?;

        buffer.clear();
        let t = buffer.truncate_back(0);
        assert_eq!(t.map(|i| *i).collect::<Vec<_>>(), vec![]);

        Ok(())
    }

    fn test_truncate_forward_helper(buffer: &mut Buffer<u32>) -> Result<u32> {
        assert!(buffer.is_empty());
        assert!(buffer.len() == 0);
        let start_index = buffer.start_index;

        buffer.try_append(vec![1, 2, 3])?;

        let t = buffer.truncate_forward(0);
        assert_eq!(t.map(|i| *i).collect::<Vec<_>>(), vec![2, 3]);
        assert!(buffer.len() == 1);
        assert!(buffer[0] == 1);

        buffer.clear();
        buffer.start_index = start_index;
        buffer.try_append(vec![1, 2, 3])?;

        let t = buffer.truncate_forward(3);
        assert_eq!(t.map(|i| *i).collect::<Vec<_>>(), vec![]);
        assert!(buffer.len() == 3);
        assert!(buffer[0] == 1);
        assert!(buffer[1] == 2);
        assert!(buffer[2] == 3);

        let t = buffer.truncate_forward(1);
        assert_eq!(t.map(|i| *i).collect::<Vec<_>>(), vec![3]);
        assert!(buffer.len() == 2);
        assert!(buffer[0] == 1);
        assert!(buffer[1] == 2);

        buffer.clear();
        buffer.start_index = start_index;
        buffer.try_append(vec![1, 2, 3])?;
        let t = buffer.truncate_forward(2);
        assert_eq!(t.map(|i| *i).collect::<Vec<_>>(), vec![]);
        assert!(buffer.len() == 3);
        assert!(buffer[0] == 1);
        assert!(buffer[1] == 2);
        assert!(buffer[2] == 3);

        Ok(())
    }

    #[test]
    fn test_iter() -> Result<u32> {
        let mut buffer = Buffer::<u32>::new(3);
        let src_vec = vec![1, 2, 3];

        buffer.try_append(src_vec.clone())?;

        let vec = buffer.iter().map(|item| *item).collect::<Vec<_>>();
        assert_eq!(vec, src_vec);

        Ok(())
    }

    #[test]
    fn test_clear() -> Result<u32> {
        let mut buffer = Buffer::<u32>::new(3);
        let src_vec = vec![1, 2, 3];
        buffer.try_append(src_vec.clone())?;

        let cleared = buffer.clear().map(|item| *item).collect::<Vec<_>>();
        assert_eq!(cleared, src_vec);

        Ok(())
    }
}
