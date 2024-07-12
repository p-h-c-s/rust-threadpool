use std::{sync::{atomic::AtomicBool, Arc, Condvar, Mutex, atomic::Ordering}, thread::ScopedJoinHandle};
use super::synchronized_queue::SynchronizedQueue;
use std::thread;

// Todo: benchmark
// Revisar static lifetime
// Improve ergonomics
// Create range method to chain into new()

// Aprendizado: lifetimes definem o espaço "mínimo" que algo pode existir, não necessariamente quanto
// exatamente algo vai existir. Uma variavel 'static pode ser destruida imediatamente, mas ela deve poder existir até o final do programa
// No caso, o lifetime 'a faz com que os campos da struct tenham que viver ao menos o mesmo espaço que ThreadPool
// Além disso, os objetos captados pelas closures devem existir ao menos o mesmo lifetime da Threadpool
// Unwrapping locked Mutexes is "safe" because we should panic if a single thread panic due to mutex poisoning risks

// deal with panics -> with poisoned threads/mutexes
// do we need to keep pool?

type Job<'a> = Box<dyn FnOnce() + Send + 'a>;

// 'env pode ser maior que 'scope (variaveis escapam do scope)
pub struct ThreadPool<'scope, 'env>{
    pool: Vec<thread::ScopedJoinHandle<'scope, ()>>,
    task_queue: Arc<SynchronizedQueue<Job<'env>>>,
    t_scope: &'scope thread::Scope<'scope, 'env>,
    stop_signal: Arc<AtomicBool>
}


// https://marabos.nl/atomics/memory-ordering.html#seqcst
// https://marabos.nl/atomics/atomics.html
impl <'scope, 'env> Drop for ThreadPool<'scope, 'env> {
    fn drop(&mut self) {
        self.stop_signal.store(true, Ordering::Release);
        // drop can be called before the actual queue is empty, causing a data race. 
        // Pushing a fake job here will drain the queue
        self.task_queue.push_fake(Box::new(||{}));
        println!("drop jobs: {:?}", self.task_queue.get_remaining_jobs());
    }
}


impl <'scope, 'env> ThreadPool<'scope, 'env> {
    pub fn new(num_threads: usize, t_scope: &'scope thread::Scope<'scope, 'env>) -> Self {
        ThreadPool {
            // queue might not be needed because of scope
            pool: Vec::with_capacity(num_threads),
            task_queue: Arc::new(SynchronizedQueue::new()),
            stop_signal: Arc::new(AtomicBool::new(false)),
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

    // maybe arc isnt needed for atomics
    // maybe refactor so the synchronized queue doesn't need to implement pop_wait.
    // It doesn't make a lot of sense for it to have a "push_fake" method, as it's T can be anything (it doesn't have to be closures)
    fn spawn_persistent_worker(&self) -> ScopedJoinHandle<'scope, ()> {
        let task_q_ref  =  Arc::clone(&self.task_queue);
        let should_stop_ref = Arc::clone(&self.stop_signal);
        self.t_scope.spawn(move || {
                loop {
                    // problema: entrar no loop antes do should_stop
                    match should_stop_ref.load(Ordering::Acquire) && task_q_ref.get_remaining_jobs() <= 0{
                        true => {
                            // push fake jobs so blocked threads can exit cvar empty queue condition
                            // println!("fake: {:?}", task_q_ref.get_remaining_jobs());
                            task_q_ref.push_fake(Box::new(|| {}));
                            break;
                        },
                        false => {
                            // println!("running: {:?}", task_q_ref.get_remaining_jobs());
                            task_q_ref.pop_wait()()
                        }
                    }
                }
            }
        )
    }

}



#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

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
        let num_threads = 1;
        // understand why variables inside the scope break it
        let owned_str = &String::from("I am outside scope");

        let before = Instant::now();
        thread::scope(|s| {
            let mut t_pool = ThreadPool::new(num_threads, s);
            for _ in 1..10000000 {
                t_pool.submit(move || {
                    let _ = owned_str;
                    let _x = 2.2*2.2;
                    // println!("{:?}", owned_str);
                });
            }
            // assert!(t_pool.pool.capacity() == num_threads);
        });
        println!("Elapsed time: {:.2?}", before.elapsed());
    }
}
