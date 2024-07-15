
## Learnt:

The 'static bound on a type doesn't control how long that object lives; it controls the allowable lifetime of references that object holds.
So a 'static bound means that a value of that type can only hold 'static references, but the value itself might live for less time than 'static

One of the key things that a lot of people trip over is thinking that lifetime annotations refer to the lifetime of the object they are applied to.  They do not; they refer to the minimum possible lifetime of any borrowed references that the object contains. This, of course, constrains the possible lifetimes of that object; an object cannot outlive its borrowed references, so its maximum possible lifetime must be shorter than the minimum possible lifetimes of the references it contains.
ref: https://users.rust-lang.org/t/why-does-thread-spawn-need-static-lifetime-for-generic-bounds/4541

---

## Atomics reference

https://marabos.nl/atomics/memory-ordering.html#seqcst
https://marabos.nl/atomics/atomics.html

---

## Higher-Rank Trait Bound

Higher-Rank Trait Bound, used in with_pool and with_reserved_pool, are used in order to specify which lifetime
the inner scope will use.
It basically specifies an "inner function" lifetime

Examples:
Here foo uses the caller lifetime to determine 'a
```rust
fn foo<'a>(b: Box<Trait<&'a usize>>) {
    let x: usize = 10;
    b.do_something(&x);
}
```

Here the HRTB defines 'a as the inner function lifetime
```rust
fn bar(b: Box<for<'a> Trait<&'a usize>>) {
    let x: usize = 10;
    b.do_something(&x);
}
```

ref: https://stackoverflow.com/questions/35592750/how-does-for-syntax-differ-from-a-regular-lifetime-bound/35595491#35595491