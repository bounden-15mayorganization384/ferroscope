#![allow(dead_code)]
/// A fixed-capacity ring buffer that keeps the N most recent items.
#[derive(Debug, Clone)]
pub struct RingBuffer<T> {
    data: Vec<T>,
    head: usize, // index where next item will be written
    len: usize,  // number of valid items
    capacity: usize,
}

impl<T: Default + Clone> RingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        let data = if capacity == 0 {
            Vec::new()
        } else {
            vec![T::default(); capacity]
        };
        Self {
            data,
            head: 0,
            len: 0,
            capacity,
        }
    }

    pub fn push(&mut self, val: T) {
        if self.capacity == 0 {
            return;
        }
        self.data[self.head] = val;
        self.head = (self.head + 1) % self.capacity;
        if self.len < self.capacity {
            self.len += 1;
        }
    }

    /// Iterates items from oldest to newest.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        let start = if self.len < self.capacity {
            0
        } else {
            self.head
        };
        let len = self.len;
        let cap = self.capacity;
        let data = self.data.as_slice();
        (0..len).map(move |i| &data[(start + i) % cap])
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn latest(&self) -> Option<&T> {
        if self.len == 0 {
            return None;
        }
        let idx = if self.head == 0 {
            self.capacity - 1
        } else {
            self.head - 1
        };
        Some(&self.data[idx])
    }

    pub fn as_vec(&self) -> Vec<T> {
        self.iter().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_zero_capacity() {
        let rb: RingBuffer<u32> = RingBuffer::new(0);
        assert_eq!(rb.len(), 0);
        assert!(rb.is_empty());
        assert_eq!(rb.capacity(), 0);
        assert!(rb.latest().is_none());
        assert_eq!(rb.as_vec(), Vec::<u32>::new());
    }

    #[test]
    fn test_new_nonzero_capacity() {
        let rb: RingBuffer<u32> = RingBuffer::new(3);
        assert_eq!(rb.len(), 0);
        assert!(rb.is_empty());
        assert_eq!(rb.capacity(), 3);
    }

    #[test]
    fn test_push_zero_capacity_does_not_panic() {
        let mut rb: RingBuffer<u32> = RingBuffer::new(0);
        rb.push(42); // should not panic
        assert_eq!(rb.len(), 0);
    }

    #[test]
    fn test_push_under_capacity() {
        let mut rb = RingBuffer::new(5);
        rb.push(1u32);
        rb.push(2);
        assert_eq!(rb.len(), 2);
        assert!(!rb.is_empty());
        assert_eq!(rb.as_vec(), vec![1, 2]);
        assert_eq!(rb.latest(), Some(&2));
    }

    #[test]
    fn test_push_exactly_capacity() {
        let mut rb = RingBuffer::new(3);
        rb.push(1u32);
        rb.push(2);
        rb.push(3);
        assert_eq!(rb.len(), 3);
        assert_eq!(rb.as_vec(), vec![1, 2, 3]);
        assert_eq!(rb.latest(), Some(&3));
    }

    #[test]
    fn test_push_over_capacity_wraps() {
        let mut rb = RingBuffer::new(3);
        rb.push(1u32);
        rb.push(2);
        rb.push(3);
        rb.push(4);
        rb.push(5);
        // Should keep most recent 3: [3, 4, 5]
        assert_eq!(rb.len(), 3);
        assert_eq!(rb.as_vec(), vec![3, 4, 5]);
        assert_eq!(rb.latest(), Some(&5));
    }

    #[test]
    fn test_latest_when_empty() {
        let rb: RingBuffer<i32> = RingBuffer::new(3);
        assert!(rb.latest().is_none());
    }

    #[test]
    fn test_iter_order() {
        let mut rb = RingBuffer::new(4);
        rb.push(10u32);
        rb.push(20);
        rb.push(30);
        rb.push(40);
        rb.push(50); // wraps, oldest is now 20
        let v: Vec<u32> = rb.iter().copied().collect();
        assert_eq!(v, vec![20, 30, 40, 50]);
    }
}
