use std::thread;
use std::sync::Arc;
use super::synchronized_queue::SynchronizedQueue;
use super::executor::Executor;

// Todo: benchmark
// Revisar static lifetime -> fixed
// Improve ergonomics
// Create range method to chain into new()

// Aprendizado: lifetimes definem o espaço "mínimo" que algo pode existir, não necessariamente quanto
// exatamente algo vai existir. Uma variavel 'static pode ser destruida imediatamente, mas ela deve poder existir até o final do programa
// No caso, o lifetime 'a faz com que os campos da struct tenham que viver ao menos o mesmo espaço que ThreadPool
// Além disso, os objetos captados pelas closures devem existir ao menos o mesmo lifetime que Threadpool
pub struct ThreadPool<'a>{
    pool: Vec<thread::ScopedJoinHandle<'a, ()>>,
    task_queue: Arc<SynchronizedQueue<fn()>>,
}

impl <'a> ThreadPool<'a> {
    pub fn new(num_threads: usize) -> Self {
        ThreadPool {
            pool: Vec::with_capacity(num_threads),
            task_queue: Arc::new(SynchronizedQueue::new())
        }
    }

    // Scoped threads allows us to capture non static variables (like closures)
    // Maybe allow 'static here
    pub fn run_server(&mut self) {
        thread::scope(|s| {
            for _ in 0..self.pool.capacity() {
                let task_q_ref  =  &self.task_queue;
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

}

impl <'a> Executor<'a> for ThreadPool<'a> {

    // TODO maybe implement bulk-submit to then use notify-all
    fn submit(&mut self, func: fn())
    {
        self.task_queue.push(func);
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

