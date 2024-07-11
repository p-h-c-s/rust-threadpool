use std::sync::{Arc, Condvar, Mutex};
use super::synchronized_queue::SynchronizedQueue;
use crossbeam::thread::{self, Scope};

// Todo: benchmark
// Revisar static lifetime
// Improve ergonomics
// Create range method to chain into new()

// Aprendizado: lifetimes definem o espaço "mínimo" que algo pode existir, não necessariamente quanto
// exatamente algo vai existir. Uma variavel 'static pode ser destruida imediatamente, mas ela deve poder existir até o final do programa
// No caso, o lifetime 'a faz com que os campos da struct tenham que viver ao menos o mesmo espaço que ThreadPool
// Além disso, os objetos captados pelas closures devem existir ao menos o mesmo lifetime da Threadpool
// Unwrapping locked Mutexes is "safe" because we should panic if a single thread panic due to mutex poisoning risks

type Job<'a> = Box<dyn FnOnce() + Send + 'a>;

pub struct ThreadPool<'a, 'scope>{
    pool: Vec<thread::ScopedJoinHandle<'a, ()>>,
    task_queue: Arc<SynchronizedQueue<Job<'a>>>,
    server_break_sign: (Mutex<bool>, Condvar),
    t_scope: &'a thread::Scope<'scope>
}

impl <'a, 'scope> ThreadPool<'a, 'scope> {
    pub fn new(num_threads: usize, t_scope: &'a thread::Scope<'scope>) -> Self {
        ThreadPool {
            pool: Vec::with_capacity(num_threads),
            task_queue: Arc::new(SynchronizedQueue::new()),
            server_break_sign: (Mutex::new(false), Condvar::new()),
            t_scope
        }
    }

    pub fn collect_server(&mut self) {
        let (should_break, cvar) = &self.server_break_sign;
        for _ in 0..self.pool.capacity() {
            let task_q_ref  =  &self.task_queue;
            self.t_scope.spawn(
                move |scope| {
                    let x = 2;
                }
            );
        }
    }

    pub fn stop_server(&mut self) {
        let (should_break, cvar) = &self.server_break_sign;
        *should_break.lock().unwrap() = false;
        cvar.notify_all();
    }


    // TODO maybe implement bulk-submit to then use notify-all
    pub fn submit<F>(&mut self, func: F)
    where F: FnOnce() + Send + 'a
    {
        self.task_queue.push(Box::new(func));
    }

    pub fn collect(&mut self)
    where 'a: 'scope
    {
        for _ in 0..self.pool.capacity() {
            let task_q_ref  =  Arc::clone(&self.task_queue);
            self.t_scope.spawn(
                move |_| {
                    loop {
                        task_q_ref.pop();
                    }
                }
            );
        }
    }


}


struct Test<'a, 'b> {
    sc: &'a Scope<'b>
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let num_threads = 5;

        thread::scope(|s| {
            let t_pool = ThreadPool::new(num_threads, s);
            
            assert!(t_pool.pool.capacity() == num_threads);
        });

    }


    // // o lifetime 'static das tasks impede que as tasks capturem dados que vivam menos que 'static. 
    // // mas elas podem mover dados menores que 'static para dentro e retorna-los via copia
    // #[test]
    // fn test_collect() {
    //     let num_threads = 5;
    //     let sum = Arc::new(Mutex::new(0));
    //     let sum_ref = &sum;
    //     let mut t_pool = ThreadPool::new(num_threads);
    //     let task = || {
    //         *sum_ref.lock().unwrap() += 1;
    //     };
    
    //     for _ in 1..t_pool.pool.capacity()+1 {
    //         t_pool.submit(task);
    //     }

    //     t_pool.collect();
    //     assert!(*sum.lock().unwrap() == 5);
    // }
}
