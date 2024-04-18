#[derive(Debug)]
pub struct Queue<T> {
    items: Vec<T>,
}

impl<T> Default for Queue<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Queue<T> {
    /// Creates a new Queue
    pub fn new() -> Self {
        Queue { items: Vec::new() }
    }

    /// Enqueues an item at the end of the queue
    pub fn enqueue(&mut self, item: T) {
        self.items.push(item);
    }

    /// Inserts an item at the beginning of the queue
    pub fn prepend(&mut self, item: T) {
        self.items.insert(0, item);
    }

    /// Dequeues an item from the beginning of the queue
    pub fn dequeue(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            Some(self.items.remove(0))
        }
    }

    /// Returns true if the queue is empty. False otherwise
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Returns the number of items in the queue
    pub fn size(&self) -> usize {
        self.items.len()
    }
}

#[cfg(test)]

mod queue_tests {
    use super::*;

    #[test]
    fn test_queue() {
        let mut queue = Queue::new();
        assert_eq!(queue.size(), 0);
        assert!(queue.is_empty());

        queue.enqueue(1);
        queue.enqueue(2);
        queue.enqueue(3);
        assert_eq!(queue.size(), 3);
        assert!(!queue.is_empty());

        assert_eq!(queue.dequeue(), Some(1));
        assert_eq!(queue.dequeue(), Some(2));
        assert_eq!(queue.dequeue(), Some(3));
        assert_eq!(queue.dequeue(), None);
        assert_eq!(queue.size(), 0);
        assert!(queue.is_empty());
    }

    #[test]
    fn test_preprend() {
        let mut queue = Queue::new();
        queue.enqueue(1);
        queue.enqueue(2);
        queue.enqueue(3);
        queue.prepend(4);
        assert_eq!(queue.dequeue(), Some(4));
        assert_eq!(queue.dequeue(), Some(1));
        assert_eq!(queue.dequeue(), Some(2));
        assert_eq!(queue.dequeue(), Some(3));
        assert!(queue.dequeue().is_none());
    }
}
