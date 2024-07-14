use std::{sync::Arc, thread::ScopedJoinHandle};
use super::synchronized_queue::SynchronizedQueue;
use std::thread;


/// Toy threadpool to run tasks with a limited number of threads. Avoids overhead of spawning a 
/// thread for each task. 
/// To avoid using non-scoped threads and thus requiring only 'static variables in the closures,
/// the pool requires a std::thread::scope value as input
/// 
/// Usage:
/// std::thread::scope(|s| {
///     let t_pool = ThreadPool::new(num_threads, s);
///     t_pool.submit(|| {
///         work()
///     })
/// })
/// 
/// The scope allows the t_pool closures to capture variables with lifetimes other than 'stati
/// 
type Job<'a> = Box<dyn FnOnce() + Send + 'a>;
pub struct ThreadPool<'scope, 'env>{
    pool: Vec<thread::ScopedJoinHandle<'scope, ()>>,
    task_queue: Arc<SynchronizedQueue<Job<'env>>>,
    t_scope: &'scope thread::Scope<'scope, 'env>,
}

impl <'scope, 'env> Drop for ThreadPool<'scope, 'env> {
    fn drop(&mut self) {
        self.task_queue.close();
    }
}

impl <'scope, 'env> ThreadPool<'scope, 'env> {
    pub fn new(num_threads: usize, t_scope: &'scope thread::Scope<'scope, 'env>) -> Self {
        ThreadPool {
            // queue might not be needed because of scope
            pool: Vec::with_capacity(num_threads),
            task_queue: Arc::new(SynchronizedQueue::new()),
            t_scope,
        }
    }

    pub fn with_pool<'a>(num_threads: usize, jobs: Vec<Job<'a>>)
    {
        thread::scope(|s| {
            let mut t_pool: ThreadPool = ThreadPool::new(num_threads, s);
            for j in jobs{
                t_pool.submit(j);
            }
        })
    }

    // TODO maybe implement bulk-submit to then use notify-all
    pub fn submit<F>(&mut self, func: F)
    where F: FnOnce() + Send + 'env
    {
        self.task_queue.push_front(Box::new(func));
        if self.pool.len() < self.pool.capacity() {
            self.pool.push(
                self.spawn_persistent_worker()
            );
        }
    }

    fn spawn_persistent_worker(&self) -> ScopedJoinHandle<'scope, ()> {
        let task_q_ref  =  Arc::clone(&self.task_queue);
        self.t_scope.spawn(move || {
                loop {
                    match task_q_ref.pop_back_wait() {
                        Some(f) => f(),
                        None => break
                    }
                }
            }
        )
    }

}

#[cfg(test)]
mod tests {
    use std::{sync::atomic::AtomicI32, sync::atomic::Ordering};

    use super::*;

    #[test]
    fn test_new() {
        let num_threads = 1;

        thread::scope(|s| {
            let t_pool = ThreadPool::new(num_threads, s);
            assert!(t_pool.pool.capacity() == num_threads);
        });

    }

    #[test]
    fn test_submit() {
        let num_threads = 3;
        
        let owned_str = &String::from("I am outside scope");
        let executed_tasks = &AtomicI32::new(0);
        let num_tasks = 100;
        
        thread::scope(|s| {
            let mut t_pool = ThreadPool::new(num_threads, s);
            for _ in 1..num_tasks+1 {
                t_pool.submit(move || {
                    let _owner_str_ref = owned_str;
                    let _x = 2.2*2.2;
                    executed_tasks.fetch_add(1, Ordering::Relaxed);
                });
            }
        });
        assert_eq!(num_tasks, executed_tasks.load(Ordering::Relaxed));
    }
}
