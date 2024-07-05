use std::thread;
use super::synchronized_queue::ArcSynchronizedQueue;

// TODO: remove static lifetime, avoid cloning arc? 
// Todo: benchmark
// Revisar static lifetime
// Improve ergonomics
// Create range method to chain into new()
pub struct ThreadPool{
    pool: Vec<thread::JoinHandle<()>>,
    task_queue: ArcSynchronizedQueue<Box<dyn FnOnce() + Send + 'static>>,
}

impl ThreadPool {
    pub fn new(num_threads: usize) -> Self {
        ThreadPool {
            pool: Vec::with_capacity(num_threads),
            task_queue: ArcSynchronizedQueue::new()
        }
    }

    // TODO maybe implement bulk-submit to then use notify-all
    pub fn submit<F>(&mut self, func: F)
    where
        F: FnOnce() + 'static + Send
    {
        self.task_queue.push(Box::new(func));
    }

    pub fn run_server(&mut self) {
        for _ in 0..self.pool.capacity() {
            let pool_ref  =  self.task_queue.shallow_clone();
            self.pool.push(
                thread::spawn(
                    move || ThreadPool::runner_server_fn(pool_ref)
                )
            )
        }
    }

    pub fn collect(&mut self) {
        for _ in 0..self.pool.capacity() {
            let pool_ref  = self.task_queue.shallow_clone();
            self.pool.push(
                thread::spawn(
                    move || ThreadPool::runner(pool_ref)
                )
            )
        }
    }

    pub fn join_all(&mut self) -> Result<(), ()> {
        for handle in self.pool.drain(..) {
            handle.join().unwrap();
        }
        Ok(())
    }

    // Runs until task queue is empty
    fn runner(task_queue_ref: ArcSynchronizedQueue<Box<dyn FnOnce() + Send + 'static>>) {
        loop {
            match task_queue_ref.pop() {
                Some(f) => f(),
                None => break
            }
        }
    }

    // &* can do deref coercion -> the deref becomes another type. Useful for Box<T> like structs: you get T
    fn runner_server_fn(task_queue_ref: ArcSynchronizedQueue<Box<dyn FnOnce() + Send + 'static>>) {
        loop {
            task_queue_ref.pop_wait()();
        }
    }

}

