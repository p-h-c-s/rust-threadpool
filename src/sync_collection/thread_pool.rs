use std::{sync::{Arc, Condvar, Mutex}, thread::ScopedJoinHandle};
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
    task_queue: Arc<SynchronizedQueue<Job<'scope>>>,
    server_break_sign: Arc<(Mutex<bool>, Condvar)>,
    t_scope: &'scope thread::Scope<'scope, 'env>
}

impl <'scope, 'env> ThreadPool<'scope, 'env> {
    pub fn new(num_threads: usize, t_scope: &'scope thread::Scope<'scope, 'env>) -> Self {
        ThreadPool {
            pool: Vec::with_capacity(num_threads),
            task_queue: Arc::new(SynchronizedQueue::new()),
            server_break_sign: Arc::new(
                (Mutex::new(false), Condvar::new())
            ),
            t_scope
        }
    }

    pub fn collect_server(&mut self) {
        for _ in 0..self.pool.capacity() {
            let server_break_sign = Arc::clone(&self.server_break_sign);
            let task_q_ref  =  Arc::clone(&self.task_queue);
            self.t_scope.spawn(move || {
                let (should_break, cvar) = &*server_break_sign;
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
            });
        }
    }

    pub fn stop_server(&mut self) {
        let (should_break, cvar) = &*self.server_break_sign;
        *should_break.lock().unwrap() = false;
        cvar.notify_all();
    }


    // TODO maybe implement bulk-submit to then use notify-all
    pub fn submit<F>(&mut self, func: F)
    where F: FnOnce() + Send + 'env
    {
        self.task_queue.push(Box::new(func));
        if self.pool.len() < self.pool.capacity() {
            self.pool.push(
                self.spawn_worker()
            )
        }
    }

    fn spawn_worker(&self) -> ScopedJoinHandle<'scope, ()> {
        let task_q_ref  =  Arc::clone(&self.task_queue);
        self.t_scope.spawn(
            move || {
                loop {
                    match task_q_ref.pop() {
                        Some(task) => task(),
                        None => break
                    }
                }
            }
        )
    }

    pub fn collect(& mut self)
    {
        for _ in 0..self.pool.capacity() {
            self.spawn_worker();
        }
    }


}



#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn test_new() {
        let num_threads = 10;
        let owned_str = String::from("123");

        thread::scope(|s| {
            let mut t_pool = ThreadPool::new(num_threads, s);
            for _ in 1..10 {
                t_pool.submit(|| {
                    println!("123");
                    thread::sleep(Duration::from_millis(1000));
                    println!("after");
                });
            }
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
