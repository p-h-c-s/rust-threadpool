use sync_collection::thread_pool::ThreadPool;
use sync_collection::executor::Executor;
use std::{thread, time::Duration};
use std::time::Instant;

pub mod sync_collection;


fn run() {
    let mut tpool = ThreadPool::new(10);
    for _ in 1..100000 {
        tpool.submit(|| {
            let id = thread::current().id();
            for _ in 1..10000 {
                let _ = 2 * 2;
            }
        })
    }
    tpool.collect();
}

fn main() {
    let before = Instant::now();
    run();
    println!("Elapsed time: {:.2?}", before.elapsed());

    let q:sync_collection::synchronized_queue::SynchronizedQueue<u32> = sync_collection::synchronized_queue::SynchronizedQueue::new();
    let qref = &q;
    let qref2 = &q;
    qref.push(2);
    qref2.push(3);
}