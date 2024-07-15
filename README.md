## A custom thread pool implementation in rust

This repo contains a custom thread pool implementation in Rust. It was created in order to learn the quirks of the language. It allows the submission of closures that capture non 'static lifetime references.

## Usage

``` rust
let num_threads = 5
let unscoped = String::from("123");
sync_collection::with_pool(num_threads, |t_pool| {
    t_pool.submit(|| {
        let x = 2 * 2;
        let z = &unscoped;
    })
})
```

### Small quirk
Standard library's thread::scope can be a bit finicky to use. If you create a variable inside the scope you might not borrow it to the spawned threads. You must move it.

```rust
thread::scope(|s| {
    let val = &String::from("123");
    s.spawn(move || {
        let z = val;
    });
});
```

This same behaviour exists for the pool, as it uses a thread::Scope:

```rust
    thread_pool::with_pool(num_threads, |t_pool| {
        let val = String::from("1234");
        t_pool.submit(move || {
            let z = &val;
        });
    }
```

## Self-imposed challenges

I challenged myself to avoid using the simple thread::spawn as it only allowed the closures to capture 'static lifetime references. This made everything a little bit harder as we then had to worry about the thread::Scope lifetimes carefully.

I also avoided channels are they're a very useful abstraction. I wanted to learn what it would take to implement a synchronized queue in rust. Channels basically abstract that away.
