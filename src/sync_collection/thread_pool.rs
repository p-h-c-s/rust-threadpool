use std::{sync::Arc, thread::ScopedJoinHandle};
use super::synchronized_queue::SynchronizedQueue;
use std::thread;


/// Toy threadpool to run tasks with a limited number of threads. Avoids overhead of spawning a 
/// thread for each task. 
/// To avoid using non-scoped threads and thus requiring only 'static lifetime variables in the closures,
/// the pool requires a std::thread::scope value as input.
/// 
/// We provide a wrapper function `with_pool` that creates the pool and then injects it as a parameter
/// to a user provided closure. The threads are instantiated on-demand (each submit call creates a new thread until max_threads)
/// 
/// Usage:
/// ```
/// with_pool(num_threads, |t_pool| {
///     let task = || {
///         ...
///     }
///     t_pool.submit(task)
/// })
/// ```
/// 
/// To pre-emptively incur the thread spawn costs we also provide a with_reserved_pool function. It
/// pre-spawns the threads before passing the thread_pool to the user-provided closure. This can be useful as it
/// incurs the spawning costs before the actual user defined computation. 
///  
/// 
pub struct ThreadPool<'scope, 'env> {
    task_queue: Arc<SynchronizedQueue<Job<'env>>>,
    num_threads: usize,
    max_threads: usize,
    t_scope: &'scope thread::Scope<'scope, 'env>,
}

type Job<'a> = Box<dyn FnOnce() + Send + 'a>;

pub fn with_pool<'env, F>(num_threads: usize, f: F)
where F: for<'scope> FnOnce(&mut ThreadPool<'scope, 'env>)
{
    thread::scope(|s| {
        let mut t_pool = ThreadPool::new(num_threads, s);
        f(&mut t_pool);
    })
}

pub fn with_reserved_pool<'env, F>(num_threads: usize, f: F)
where F: for<'scope> FnOnce(&mut ThreadPool<'scope, 'env>)
{
    thread::scope(|s| {
        let mut t_pool = ThreadPool::new(num_threads, s);
        t_pool.reserve_threads(num_threads);
        f(&mut t_pool);
    })
}

impl <'scope, 'env> Drop for ThreadPool<'scope, 'env> {
    fn drop(&mut self) {
        self.task_queue.close();
    }
}

impl <'scope, 'env> ThreadPool<'scope, 'env> {
    pub fn new(max_threads: usize, t_scope: &'scope thread::Scope<'scope, 'env>) -> Self {
        ThreadPool {
            task_queue: Arc::new(SynchronizedQueue::new()),
            num_threads: 0,
            max_threads,
            t_scope,
        }
    }

    pub fn submit<F>(&mut self, func: F)
    where F: FnOnce() + Send + 'env
    {
        self.task_queue.push_front(Box::new(func));
        if self.num_threads < self.max_threads {
            self.spawn_persistent_worker();
        }
    }

    fn reserve_threads(&mut self, num_threads: usize) {
        for _ in 0..num_threads {
            self.spawn_persistent_worker();
        }
    }

    fn spawn_persistent_worker(&mut self) {
        let task_q_ref  =  Arc::clone(&self.task_queue);
        self.t_scope.spawn(move || {
                loop {
                    match task_q_ref.pop_back_wait() {
                        Some(f) => f(),
                        None => break
                    }
                }
            }
        );
        self.num_threads += 1;
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
            assert!(t_pool.max_threads == num_threads);
        });

    }

    #[test]
    fn test_submit() {
        let num_threads = 3;
        let executed_tasks = &AtomicI32::new(0);
        let num_tasks = 100;
        let mut used_threads = 0;

        thread::scope(|s| {
            let mut t_pool = ThreadPool::new(num_threads, s);
            for _ in 1..num_tasks+1 {
                t_pool.submit(move || {
                    executed_tasks.fetch_add(1, Ordering::Relaxed);
                });
            }
            used_threads = t_pool.num_threads;
        });

        assert_eq!(used_threads, num_threads);
        assert_eq!(num_tasks, executed_tasks.load(Ordering::Relaxed));
    }


    #[test]
    fn test_with_pool() {
        let num_threads = 3;
        let executed_tasks = &AtomicI32::new(0);
        let mut used_threads = 0;

        with_pool(num_threads, |t_pool| {
            t_pool.submit(|| {
                executed_tasks.fetch_add(1, Ordering::Relaxed);
            });
            t_pool.submit(|| {
                executed_tasks.fetch_add(1, Ordering::Relaxed);
            });
            used_threads = t_pool.num_threads;
        });

        assert_eq!(used_threads, num_threads - 1);
        assert_eq!(executed_tasks.load(Ordering::Relaxed), 2);
    }

    #[test]
    fn test_with_reserved_pool() {
        let num_threads = 3;
        let executed_tasks = &AtomicI32::new(0);
        let mut used_threads = 0;

        with_reserved_pool(num_threads, |t_pool| {
            t_pool.submit(|| {
                executed_tasks.fetch_add(1, Ordering::Relaxed);
            });
            t_pool.submit(|| {
                executed_tasks.fetch_add(1, Ordering::Relaxed);
            });
            used_threads = t_pool.num_threads;
        });

        assert_eq!(used_threads, num_threads);
        assert_eq!(executed_tasks.load(Ordering::Relaxed), 2);
    }
}
