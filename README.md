# Escher
> Self-referencial structs using async stacks

## How it works

### The problem with self-references

The main problem with self-referencial structs is that if such a struct was
somehow constructed the compiler would then have to statically prove that it
would not move again. This analysis is necessary because any move would
invalidate self-pointers since all pointers in rust are absolute memory
addresses.

To illustrate why this is necessary, imagine we define a self-referencial
struct that holds a Vec and a pointer to it at the same time:

```rust
struct Foo {
    s: Vec<u8>,
    p: &Vec<u8>,
}
```

Then, let's assume we had a way of getting an instance of this struct. We could
then write the following code that creates a dangling pointer in safe rust!

```rust
let foo = Foo::magic_construct();

let bar = foo; // move foo to a new location
println!("{:?}", bar.p); // access the self-reference, memory error!
```

![Moves invalidate pointer](https://github.com/petrosagg/escher/blob/master/assets/moves-invalidate-pointer.png?raw=true)

### Almost-self-references on the stack

While rust doesn't allow you to explicitly write out self referencial struct
members and initialize them it is perfectly valid to write out the values of
the members individually as separate stack bindings. This is because the borrow
checker *can* do a move analysis when the values are on the stack.

Practically, we could convert the struct `Foo` from above to individual
bindings like so:

```rust
fn foo() {
    let s = vec![1, 2, 3];
    let p = &s;
}
```

Then, we could wrap both of them in a struct that only has references and use that instead:

```rust
struct AlmostFoo<'a> {
    s: &'a Vec<u8>,
    p: &'a Vec<u8>,
}

fn make_foo() {
    let s = vec![1, 2, 3];
    let p = &s;

    let foo = AlmostFoo { s, p };

    do_stuff(foo); // call a function that expects an AlmostFoo
}
```

Of course `make_foo()` cannot return an `AlmostFoo` instance since it would be
referencing values from its stack, but what it can do is call other functions
and pass an `AlmostFoo` to them. In other words, as long as the code that wants
to use `AlmostFoo` is above `make_foo()` we can use this technique and work
with almost-self-references.

This is pretty restrictive though. Ideally we'd lke to be able return some
owned value and be free to move it around, put it on the heap, etc.

### Actually returning an `AlmostFoo`

As we saw, it is impossible to return an `AlmostFoo` instance since it
references values from the stack. But what if we could freeze the stack after
an `AlmostFoo` instance got constructed and then returned the whole stack?

Well, there is no way for a regular function to capture its own stack and
return it but that is exactly what the async/await transformation does! Let's
make `make_foo` from above async and also make it never terminate:

```rust
async fn make_foo() {
    let s = vec![1, 2, 3];
    let p = &s;
    let foo = AlmostFoo { s, p };
    std::future::pending().await
}
```

Now when someone calls `make_foo()` what they get back is some struct that
implements Future. This struct is in fact a representation of the stack of
`make_foo` at its initial state, i.e in the state that the function has not be
called yet.

What we need to do now is to step the execution of the returned Future until
the instance of `AlmostFoo` is constructed. In this case we know that there is
a single await point so we only need to poll the Future once. Before we do that
though we need to put it in a Pinned Box to ensure that as we poll the future
no moves will occur. This is the same restriction as with normal function but
with async it is enforced using the `Pin<P>` type.

```rust
let foo = make_foo(); // construct a stack that will eventually make an AlmostFoo in it
let mut foo = Box::pin(foo_fut); // pin it so that it never moves again
foo.poll(); // poll it once

// now we know that somewhere inside `foo` there is a valid AlmostFoo instance!
```

We're almost there! We now have an owned value, the future, that somewhere
inside it has an AlmostFoo instance. However we have no way of retrieving the
exact memory location of it or accessing it in any way. The Future is opaque.

This is where `escher` comes in! `escher` essentially offers a way of peeking
into the opaque Future and retrieving the raw pointer of our `AlmostFoo`
instance.
