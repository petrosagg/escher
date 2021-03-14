# Escher
> Self-referencial structs using async stacks

## How it works


### The problem with self-references

The main problem with self-referencial structs is that if such a struct was
somehow constructed the compiler would then have to statically prove that it
would not move again. This analysis is necessary because any move would
invalidate self-pointers. The reason for that is that all pointers in rust are
absolute memory addresses and moves just move data around in memory.

To illustrate this with an example, imagine we defined a self-referencial
struct that held a Vec and a pointer to it at the same time:

```rust
struct Foo {
    s: Vec<u8>,
    p: &Vec<u8>,
}
```

Then, let's assume we had a way of getting an instance of this. Then we could
write the following code that would lead to a dangling pointer in safe rust!

```rust
let foo = Foo::magic_construct();

let bar = foo; // move foo to a new location
```

![Moves invalidate pointer](https://github.com/petrosagg/escher/blob/master/assets/moves-invalidate-pointer.png?raw=true)


Escher is based on the idea that Rust allows you to construct
quasi-self-referencial structs on the stack. This is something everyone is
familiar with:

```rust
struct QuasiSelfRef<'a> {
    s: &'a String,
    p: &'a str,
}

fn foo() {
    // create a String
    let s = "some owned string".to_string();
    // ..and a reference to it
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
