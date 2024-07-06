use sync_collection::thread_pool::ThreadPool;
use std::{thread, time::Duration};

pub mod sync_collection;

fn main() {
    let mut tpool = ThreadPool::new(5);

    for _ in 1..10 {
        tpool.submit(|| {
            let id = thread::current().id();
            thread::sleep(Duration::from_secs(1));
            println!("Inneer func thread: {:?}", id);
        })
    }
    tpool.collect();
}