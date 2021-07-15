use std::collections::vec_deque::{Iter, IterMut};
use std::collections::VecDeque;

#[derive(Clone)]
pub struct RingBuffer<T> {
    data: VecDeque<T>,
    capacity: usize,
}

impl<T> RingBuffer<T> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: VecDeque::new(),
            capacity,
        }
    }

    // Push onto the ring buffer, overwrite the oldest entry when at capacity
    // Return the overwritten value
    pub fn push(&mut self, val: T) -> Option<T> {
        self.data.push_back(val);
        if self.data.len() == self.capacity {
            self.data.pop_front()
        } else {
            None
        }
    }

    // Pop the newest value
    pub fn pop(&mut self) -> Option<T> {
        self.data.pop_back()
    }

    // Peak on the newest value
    pub fn peak(&self) -> Option<&T> {
        self.data.back()
    }

    pub fn peak_mut(&mut self) -> Option<&mut T> {
        self.data.back_mut()
    }

    // Iterator implementation
    // old to new
    pub fn iter(&self) -> Iter<T> {
        self.data.iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<T> {
        self.data.iter_mut()
    }

    // Getters
    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn elem(&self, index: usize) -> Option<&T> {
        if index < self.len() {
            Some(&self.data[index])
        } else {
            None
        }
    }
}
