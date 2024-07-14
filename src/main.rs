use sync_collection::thread_pool;
use std::time::Instant;

pub mod sync_collection;

fn run(num_threads: usize) {
    thread_pool::with_pool(num_threads, |t_pool| {
        for _ in 1..100000{
            t_pool.submit(|| {
                for _ in 1..10000 {
                    let _ = 2 * 2;
                }
            })
        }
    })
}

fn main() {
    let before = Instant::now();
    run(5);
    println!("Elapsed time: {:.2?}", before.elapsed());
}