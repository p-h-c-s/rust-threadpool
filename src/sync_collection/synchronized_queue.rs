
use std::sync::atomic::{AtomicI16, Ordering};
use std::sync::{Condvar, Mutex, MutexGuard};

use std::sync::Arc;

type SynchronizedVec<T> = Mutex<Vec<T>>;
type SynchronizedQueueTuple<T> = (SynchronizedVec<T>, Condvar);

pub struct SynchronizedQueue<T>{
    task_queue: SynchronizedQueueTuple<T>,
    remaining_jobs: AtomicI16
}

impl <T> SynchronizedQueue<T> {
    pub fn new() -> Self {
        SynchronizedQueue {
            task_queue: (Mutex::new(Vec::new()),  Condvar::new()),
            remaining_jobs: AtomicI16::new(0)
        }
    }

    fn lock_unwrap(&self) -> MutexGuard<Vec<T>> {
        self.task_queue.0.lock().unwrap()
    }

    pub fn get_remaining_jobs(&self) -> i16 {
        self.remaining_jobs.load(Ordering::Relaxed)
    }

    /// Even though we are mutating the conceptual "queue", 
    /// we need a shared ref (&self) in order to have concurrent access.
    /// The underlying mutex allows interior mutability
    pub fn push(&self, item: T) {
        self.lock_unwrap().insert(0, item);
        self.remaining_jobs.fetch_add(1, Ordering::Relaxed);
        self.task_queue.1.notify_one();
    }

    /// Pushes a job meant to allow the cvar to be unlocked
    pub fn push_fake(&self, item: T) {
        self.lock_unwrap().insert(0, item);
        self.task_queue.1.notify_one();
    }

    pub fn pop(&self) -> Option<T> {
        let item = self.lock_unwrap().pop();
        let prev = self.remaining_jobs.fetch_sub(1, Ordering::Relaxed);
        item
    }

    /// Blocking pop operation. Waits until task_queue is not empty.
    /// Doesn't return Option<T> as pop will always access a non-empty list (the cvar is only released if q is not empty)
    /// This makes it hard to manually stop blocked threads. One way to do this is by pushing fake data to it.
    pub fn pop_wait(&self) -> T {
        let (queue, cvar) = &self.task_queue;
        let mut q_ref = queue.lock().unwrap();
        // println!("qlen: {:?}", q_ref.len());
        q_ref = cvar.wait_while(q_ref, |q| q.is_empty()).unwrap();
        let item = q_ref.pop().unwrap();
        self.remaining_jobs.fetch_sub(1, Ordering::Relaxed);
        drop(q_ref);
        item
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
        let item = queue.pop_wait();
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
                let item = queue_clone.pop_wait();
                sum += item;
            }
            assert_eq!(sum, 45); // 0 + 1 + 2 + ... + 9 = 45
        });

        producer.join().unwrap();
        consumer.join().unwrap();
    }
}