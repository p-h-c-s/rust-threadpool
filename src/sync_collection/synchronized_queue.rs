
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Condvar, Mutex, MutexGuard};
use std::collections::VecDeque;


type SynchronizedVec<T> = Mutex<VecDeque<T>>;
type SynchronizedQueueTuple<T> = (SynchronizedVec<T>, Condvar);

pub struct SynchronizedQueue<T>{
    task_queue: SynchronizedQueueTuple<T>,
    is_closed: AtomicBool,
}

/// Thread-safe wrapper for a vec. Intended to be used as a queue
/// We delegate the job of Wrapping this in an Arc to the user
impl <T> SynchronizedQueue<T> {
    pub fn new() -> Self {
        SynchronizedQueue {
            task_queue: (Mutex::new(VecDeque::new()),  Condvar::new()),
            is_closed: AtomicBool::new(false)
        }
    }

    pub fn close(&self) {
        self.is_closed.store(true, Ordering::Release);
        self.task_queue.1.notify_all();
    }

    fn lock_unwrap(&self) -> MutexGuard<VecDeque<T>> {
        self.task_queue.0.lock().unwrap()
    }

    pub fn push(&self, item: T) {
        self.lock_unwrap().push_front(item);
        self.task_queue.1.notify_one();
    }

    pub fn pop(&self) -> Option<T> {
        let item = self.lock_unwrap().pop_back();
        item
    }

    /// Blocking pop operation. Waits until task_queue is not empty.
    pub fn pop_wait(&self) -> Option<T> {
        let (queue, cvar) = &self.task_queue;
        let mut q_ref = queue.lock().unwrap();
        q_ref = cvar.wait_while(q_ref, |q| q.is_empty() && !self.is_closed.load(Ordering::Acquire)).unwrap();
        q_ref.pop_back()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_new() {
        let queue: SynchronizedQueue<i32> = SynchronizedQueue::new();
        let locked_queue = queue.lock_unwrap();
        assert!(locked_queue.is_empty());
    }

    #[test]
    fn test_push() {
        let queue: Arc<SynchronizedQueue<i32>> = Arc::new(SynchronizedQueue::new());
        queue.push(1);
        let locked_queue = queue.lock_unwrap();
        assert_eq!(locked_queue.len(), 1);
        assert_eq!(locked_queue[0], 1);
    }

    #[test]
    fn test_pop() {
        let queue: Arc<SynchronizedQueue<i32>> = Arc::new(SynchronizedQueue::new());
        queue.push(1);
        let item = queue.pop();
        assert_eq!(item, Some(1));
        let locked_queue = queue.lock_unwrap();
        assert!(locked_queue.is_empty());
    }

    #[test]
    fn test_pop_wait() {
        let queue: Arc<SynchronizedQueue<i32>> = Arc::new(SynchronizedQueue::new());

        // Spawn a thread to push an item after a delay
        let queue_clone = Arc::clone(&queue);
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(50));
            queue_clone.push(1);
        });

        // `pop_wait` should block until the item is pushed
        let item = queue.pop_wait().unwrap();
        assert_eq!(item, 1);
        let locked_queue = queue.lock_unwrap();
        assert!(locked_queue.is_empty());
    }

    #[test]
    fn test_push_pop_multithreaded() {
        let queue: Arc<SynchronizedQueue<i32>> = Arc::new(SynchronizedQueue::new());
        let queue_clone = Arc::clone(&queue);

        let producer = thread::spawn(move || {
            for i in 0..10 {
                queue_clone.push(i);
                thread::sleep(Duration::from_millis(10));
            }
        });

        let queue_clone = Arc::clone(&queue);
        let consumer = thread::spawn(move || {
            let mut sum = 0;
            for _ in 0..10 {
                let item = queue_clone.pop_wait().unwrap();
                sum += item;
            }
            assert_eq!(sum, 45); // 0 + 1 + 2 + ... + 9 = 45
        });

        producer.join().unwrap();
        consumer.join().unwrap();
    }
}