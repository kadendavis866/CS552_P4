use std::sync::{Arc, Condvar, Mutex};
use std::sync::atomic::{AtomicBool};
use std::sync::atomic::Ordering::SeqCst;

/// A circular buffer that can be used as a queue
struct Buffer<T> {
    data: Vec<Option<T>>,
    size: usize,
    front: usize,
    back: usize,
}

impl<T> Buffer<T> {
    /// Create a new buffer with the given capacity
    fn with_capacity(size: usize) -> Self {
        let mut data = Vec::with_capacity(size);
        data.resize_with(size, || None);
        Self {
            data,
            size,
            front: 0,
            back: 0,
        }
    }

    /// Push an item to the back of the buffer
    ///
    /// # Arguments
    /// * item - the item to add to the back of the buffer
    ///
    /// # Returns
    /// a Result indicating success or failure of the operation
    fn push(&mut self, item: T) -> Result<(), &str> {
        assert!(!self.is_full());
        self.data[self.back] = Some(item);
        self.back = (self.back + 1) % self.size;
        Ok(())
    }

    /// Pop an item from the front of the buffer
    ///
    /// # Returns
    /// the item at the front of the buffer, or an error if the buffer is empty
    fn pop(&mut self) -> Result<T, &str> {
        assert!(!self.is_empty());
        let item = self.data[self.front].take().ok_or("Buffer is empty")?;
        self.front = (self.front + 1) % self.size;
        Ok(item)
    }

    /// Check if the buffer is empty
    ///
    /// # Returns
    /// true if the buffer is empty, false otherwise
    fn is_empty(&self) -> bool {
        self.front == self.back
    }

    /// Check if the buffer is full
    ///
    /// # Returns
    /// true if the buffer is full, false otherwise
    fn is_full(&self) -> bool {
        (self.back + 1) % self.size == self.front
    }
}

/// A thread-safe blocking queue implementation
pub struct QueueR<T> {
    data: Mutex<Buffer<T>>,
    shutdown: AtomicBool,
    not_empty: Condvar,
    not_full: Condvar,
}

impl<T> QueueR<T> {
    /// Create a new queue
    ///
    /// # Arguments
    /// * capacity - the maximum capacity of the queue
    /// # Returns
    /// a fully initialized queue
    pub fn new(capacity: usize) -> Arc<Self> {
        Arc::new(Self {
            data: Mutex::new(Buffer::with_capacity(capacity)),
            shutdown: AtomicBool::new(false),
            not_empty: Condvar::new(),
            not_full: Condvar::new(),
        })
    }

    /// Add an element to the back of the queue
    ///
    /// # Arguments
    /// * q - The queue
    /// * data - The data to add
    ///
    /// # Returns
    /// a Result indicating success or failure of the operation
    pub fn enqueue(&self, data: T) -> Result<(), &str> {
        if self.shutdown.load(SeqCst) {
            return Err("Queue is shutdown");
        }
        let mut buffer = self.data.lock().or(Err("Failed to lock queue"))?;
        while buffer.is_full() {
            buffer = self.not_full.wait(buffer).unwrap();
        }
        buffer.push(data).unwrap();
        self.not_empty.notify_one();
        Ok(())
    }

    /// Remove and return the first element in the queue.
    ///
    /// # Returns
    /// the first element in the queue, or None if the queue is empty
    pub fn dequeue(&self) -> Option<T> {
        let mut data = self.data.lock().expect("Failed to lock queue");
        while data.is_empty() {
            if self.shutdown.load(SeqCst) {
                return None;
            }
            data = self.not_empty.wait(data).unwrap();
        }
        let item = data.pop().ok();
        self.not_full.notify_one();
        item
    }

    /// Sets the shutdown flag of the queue to true. This indicates that producer threads should not
    /// enqueue additional items and consumer threads should not wait for additional items (though
    /// they may continue to consume existing items). This allows the threads to be shutdown
    /// properly
    ///
    /// # Returns
    /// a Result indicating success or failure of the operation
    pub fn shutdown(&self) {
        self.shutdown.fetch_or(true, SeqCst);
        self.not_empty.notify_all();
    }

    /// Return true if the queue is empty
    ///
    /// # Returns
    /// true if the queue is empty, false otherwise
    #[cfg(test)]
    pub fn is_empty(&self) -> bool {
        self.data.lock().unwrap().is_empty()
    }

    /// Return true if the queue is shutdown
    ///
    /// # Returns
    /// true if the queue is shutdown, false otherwise
    pub fn is_shutdown(&self) -> bool {
        self.shutdown.load(SeqCst)
    }
}
