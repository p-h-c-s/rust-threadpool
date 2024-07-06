use core::task;
use std::thread;
use std::sync::Arc;
use super::synchronized_queue::SynchronizedQueue;

// TODO: remove static lifetime, avoid cloning arc? 
// Todo: benchmark
// Revisar static lifetime -> fixed
// Improve ergonomics
// Create range method to chain into new()
pub struct ThreadPool<'a>{
    pool: Vec<thread::ScopedJoinHandle<'a, ()>>,
    task_queue: Arc<SynchronizedQueue<'a, Box<dyn FnOnce() + Send + 'a>>>,
}

impl <'a> ThreadPool<'a> {
    pub fn new(num_threads: usize) -> Self {
        ThreadPool {
            pool: Vec::with_capacity(num_threads),
            task_queue: Arc::new(SynchronizedQueue::new())
        }
    }

    // TODO maybe implement bulk-submit to then use notify-all
    pub fn submit<F>(&mut self, func: F)
    where
        F: FnOnce() + Send + 'a
    {
        self.task_queue.push(Box::new(func));
    }

    // Scoped threads allows us to capture non static variables (like closures)
    pub fn run_server(&mut self) {
        thread::scope(|s| {
            for _ in 0..self.pool.capacity() {
                let task_q_ref  =  Arc::clone(&self.task_queue);
                s.spawn(
                    move || {
                        loop {
                            task_q_ref.pop_wait()();
                        }
                    }
                );
            }
        })
    }

    pub fn collect(&mut self) {
        thread::scope(|s| {
            for _ in 0..self.pool.capacity() {
                let task_q_ref  =  Arc::clone(&self.task_queue);
                s.spawn(
                    move || {
                        loop {
                            match task_q_ref.pop() {
                                Some(task) => task(),
                                None => break
                            }
                        }
                    }
                );
            }
        })
    }

}

