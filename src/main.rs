use sync_collection::thread_pool::ThreadPool;
use std::{thread, time::Duration};
use std::time::Instant;

pub mod sync_collection;


// fn run() {
//     let x = 4;
//     let mut tpool = ThreadPool::new(10);
//     // let x = &4; -> Errors, it is dropped before tpool
//     for _ in 1..100000 {
//         tpool.submit(|| {
//             let id = thread::current().id();
//             for _ in 1..10000 {
//                 let _ = 2 * 2;
//                 let y = x;
//             }
//         })
//     }
//     tpool.collect();
// }

fn main() {
    // let before = Instant::now();
    // run();
    // println!("Elapsed time: {:.2?}", before.elapsed());
}