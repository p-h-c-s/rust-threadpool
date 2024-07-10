use std::thread;
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use super::synchronized_queue::SynchronizedQueue;
use super::executor::Executor;

// Todo: benchmark
// Revisar static lifetime -> fixed
// Improve ergonomics
// Create range method to chain into new()

// Aprendizado: lifetimes definem o espaço "mínimo" que algo pode existir, não necessariamente quanto
// exatamente algo vai existir. Uma variavel 'static pode ser destruida imediatamente, mas ela deve poder existir até o final do programa
// No caso, o lifetime 'a faz com que os campos da struct tenham que viver ao menos o mesmo espaço que ThreadPool
// Além disso, os objetos captados pelas closures devem existir ao menos o mesmo lifetime da Threadpool
// Unwrapping locked Mutexes is "safe" because we should panic if a single thread panic due to mutex poisoning risks
pub struct ThreadPool<'a>{
    pool: Vec<thread::ScopedJoinHandle<'a, ()>>,
    task_queue: Arc<SynchronizedQueue<Box<dyn FnOnce() + Send + 'a>>>,
    server_break_sign: (Mutex<bool>, Condvar)
}

impl <'a> ThreadPool<'a> {
    pub fn new(num_threads: usize) -> Self {
        ThreadPool {
            pool: Vec::with_capacity(num_threads),
            task_queue: Arc::new(SynchronizedQueue::new()),
            server_break_sign: (Mutex::new(false), Condvar::new())
        }
    }

    pub fn collect_server(&mut self) {
        let (should_break, cvar) = &self.server_break_sign;
        thread::scope(|s| {
            for _ in 0..self.pool.capacity() {
                let task_q_ref  =  &self.task_queue;
                s.spawn(
                    move || {
                        loop {
                            let mut should_break_ref = should_break.lock().unwrap();
                            match &*should_break_ref {
                                false => {
                                    should_break_ref = cvar.wait_while(should_break_ref, |sb| *sb).unwrap();
                                    drop(should_break_ref);
                                    task_q_ref.pop_wait()();
                                },
                                true => break
                            }
                        }
                    }
                );
            }
        })
    }

    pub fn stop_server(&mut self) {
        let (should_break, cvar) = &self.server_break_sign;
        *should_break.lock().unwrap() = false;
        cvar.notify_all();
    }


}

impl <'a> Executor<'a> for ThreadPool<'a> {

    // TODO maybe implement bulk-submit to then use notify-all
    fn submit<F>(&mut self, func: F)
    where F: FnOnce() + Send + 'a
    {
        self.task_queue.push(Box::new(func));
    }

    fn collect(&mut self) {
        thread::scope(|s| {
            for _ in 0..self.pool.capacity() {
                let task_q_ref  =  &self.task_queue;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let num_threads = 5;

        let t_pool = ThreadPool::new(num_threads);

        assert!(t_pool.pool.len() == num_threads);
    }


    #[test]
    fn test_collet() {
        let num_threads = 5;
        let sum = Arc::new(Mutex::new(0));
        let sum_ref = &sum;
        let mut t_pool = ThreadPool::new(num_threads);
        let task = || {
            *sum_ref.lock().unwrap() += 1;
        };
    
        for _ in 1..t_pool.pool.capacity()+1 {
            t_pool.submit(task);
        }

        t_pool.collect();
        assert!(*sum.lock().unwrap() == 5);
    }
}
