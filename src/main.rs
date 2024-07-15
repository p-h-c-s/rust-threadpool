use scoped_tpool::thread_pool;
use std::{thread, time::Instant};

fn run(num_threads: usize) {
    let val2 = String::from("123");

    thread_pool::with_pool(num_threads, |t_pool| {
        let val = String::from("1234");
        t_pool.submit(  move || {
            let z = &val;
        });
        t_pool.submit(  || {
            for _ in 1..10000 {
                let z = &val2;
                let _ = 2 * 2;
            }
        });
    })
}

fn main() {

    thread::scope(|s| {
        let val = String::from("123");
        s.spawn(move || {
            let z = &val;
        });
    });

    let before = Instant::now();
    run(5);
    println!("Elapsed time: {:.2?}", before.elapsed());
}