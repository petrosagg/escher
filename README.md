# Escher
> Self-referencial structs using async stacks

## How it works

Escher is based on the idea that Rust allows you to construct
quasi-self-referencial structs on the stack. This is something everyone is
familiar with:

```rust
struct QuasiSelfRef<'a> {
    s: &'a String,
    p: &'a str,
}

fn foo() {
    // create the String
    let s = "some owned string".to_string();
    // ..and the reference
    let p = &s[0..4];

    // capture both in a struct
    let q = QuasiSelfRef {
        s: &s,
        p: p,
    }

    // After this point **both** q.s and q.p can be used by this function or
    // passed as an argument to other functions
    do_something(q);
}
```

This pattern however is restrictive as the self reference exists only on the
stack and you need to make sure to run the setup code as early in your stack as
possible if you'd like to pass it around. There is no way to return the
quasi-selfreference as that would immediately invalidate it. What if we could
capture all the relevant state of the stack and put it on the heap?

That's exactly what escher does using the `async`/`await` transformation! The
user writes the initialization code inside an async block which rustc converts
into a Future. Escher then puts that Future into a Pinned Box and steps the
execution of the future until the internal stack frame reaches the point where
the quasi-self-reference is initialized.

At that point the Future is never touched again. It's as if you had a thread
running the initialization of a self-referencial struct on its stack and then
slept forever. This of would be extremly wasteful in terms of memory but with
`async`/`await` we only need to store enough space for the variables we
capture.
