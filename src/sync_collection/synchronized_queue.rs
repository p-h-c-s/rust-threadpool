
use std::sync::{Condvar, Mutex, MutexGuard};

type SynchronizedVec<T> = Mutex<Vec<T>>;
type SynchronizedQueueTuple<T> = (SynchronizedVec<T>, Condvar);

pub struct SynchronizedQueue<T>{
    task_queue: SynchronizedQueueTuple<T>,
}

impl <T> SynchronizedQueue<T> {
    pub fn new() -> Self {
        SynchronizedQueue {
            task_queue: (Mutex::new(Vec::new()),  Condvar::new()),
        }
    }

    fn lock_unwrap(&self) -> MutexGuard<Vec<T>> {
        self.task_queue.0.lock().unwrap()
    }

    // Even though we are mutating the conceptual "queue", 
    // we need a shared ref (&self) in order to have concurrent access.
    // The underlying mutex allows interior mutability
    pub fn push(&self, item: T) {
        self.lock_unwrap().push(item);
        self.task_queue.1.notify_one();
    }

    // Non blocking pop, will return as soon as lock is acquired
    pub fn pop(&self) -> Option<T> {
        self.lock_unwrap().pop()
    }

    // Blocking pop operation. Waits until task_queue is not empty.
    // Doesn't return Option<T> as pop will always access a non-empty list
    pub fn pop_wait(&self) -> T {
        let (queue, cvar) = &self.task_queue;
        let mut q_ref = queue.lock().unwrap();
        q_ref = cvar.wait_while(q_ref, |q| q.is_empty()).unwrap();
        let item = q_ref.pop().unwrap();
        drop(q_ref);
        item
    }
}

