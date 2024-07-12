use std::{sync::Arc, thread::ScopedJoinHandle};
use super::synchronized_queue::SynchronizedQueue;
use std::thread;

// Create range method to chain into new()

// Aprendizado: lifetimes definem o espaço "mínimo" que algo pode existir, não necessariamente quanto
// exatamente algo vai existir. Uma variavel 'static pode ser destruida imediatamente, mas ela deve poder existir até o final do programa
// No caso, o lifetime 'a faz com que os campos da struct tenham que viver ao menos o mesmo espaço que ThreadPool
// Além disso, os objetos captados pelas closures devem existir ao menos o mesmo lifetime da Threadpool
// Unwrapping locked Mutexes is "safe" because we should panic if a single thread panic due to mutex poisoning risks

// https://marabos.nl/atomics/memory-ordering.html#seqcst
// https://marabos.nl/atomics/atomics.html

// deal with panics -> with poisoned threads/mutexes
// do we need to keep pool?

type Job<'a> = Box<dyn FnOnce() + Send + 'a>;

// 'env pode ser maior que 'scope (variaveis escapam do scope)
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
        self.task_queue.push(Box::new(func));
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
                    match task_q_ref.pop_wait() {
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
        // understand why variables created inside the scope break it
        let num_threads = 3;
        let owned_str = &String::from("I am outside scope");

        let executed_tasks = &AtomicI32::new(0);
        let num_tasks = 100;

        thread::scope(|s| {
            let mut t_pool = ThreadPool::new(num_threads, s);
            for _ in 1..num_tasks+1 {
                t_pool.submit(move || {
                    let _ = owned_str;
                    let _x = 2.2*2.2;
                    executed_tasks.fetch_add(1, Ordering::Relaxed);
                });
            }
        });
        assert_eq!(num_tasks, executed_tasks.load(Ordering::Relaxed));
    }
}
