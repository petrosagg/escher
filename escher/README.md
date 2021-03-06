[![Crates.io](https://img.shields.io/crates/v/escher.svg)](https://crates.io/crates/escher)

# escher

> Self-referencial structs using async stacks

Escher is an extremely simple library providing a safe and sound API to build
self-referencial structs. It works by (ab)using the async await trasformation
of rustc. If you'd like to know more about the inner workings please take a
look at the [How it works](#how-it-works) section and the source code.

Compared to the state of the art escher:

* Is only around 100 lines of well-commented code
* Contains only two `unsafe` calls that are well argued for
* Uses rustc for all the analysis. If it compiles, the self references are correct

## Usage

This library provides the [Escher<T>](Escher) wrapper type that can hold self-referencial data
and expose them safely through the [as_ref()](Escher::as_ref) and [as_mut()](Escher::as_mut)
functions.

You construct a self reference by calling Escher's constructor and providing an async closure
that will initialize your self-references on its stack. Your closure will be provided with a
capturer `r` that has a single [capture()](Capturer::capture) method that consumes `r`.

> **Note:** It is important to `.await` the result `.capture()` in order for escher to correctly
initialize your struct.

Once all the data and references are created you can capture the desired ones. Simple
references to owned data can be captured directly (see first example).

To capture more than one variable or capture references to non-owned data you will have to
define your own reference struct that derives [Rebindable](escher_derive::Rebindable) (see
second example).

## Examples

### Simple `&str` view into an owned `Vec<u8>`

The simplest way to use Escher is to create a reference of some data and then capture it:

```rust
use escher::Escher;

let escher_heart = Escher::new(|r| async move {
    let data: Vec<u8> = vec![240, 159, 146, 150];
    let sparkle_heart = std::str::from_utf8(&data).unwrap();

    r.capture(sparkle_heart).await;
});

assert_eq!("????", *escher_heart.as_ref());
```

### Capturing both a `Vec<u8>` and a `&str` view into it

In order to capture more than one things you can define a struct that will be used to capture
the variables:

```rust
use escher::{Escher, Rebindable};

#[derive(Rebindable)]
struct VecStr<'this> {
    data: &'this Vec<u8>,
    s: &'this str,
}

let escher_heart = Escher::new(|r| async move {
    let data: Vec<u8> = vec![240, 159, 146, 150];

    r.capture(VecStr{
        data: &data,
        s: std::str::from_utf8(&data).unwrap(),
    }).await;
});

assert_eq!(240, escher_heart.as_ref().data[0]);
assert_eq!("????", escher_heart.as_ref().s);
```

### Capturing a mutable `&mut str` view into a `Vec<u8>`

If you capture a mutable reference to some piece of data then you cannot capture the data
itself like the previous example. This is mandatory as doing otherwise would create two mutable
references into the same piece of data which is not allowed.

```rust
use escher::Escher;

let mut name = Escher::new(|r| async move {
    let mut data: Vec<u8> = vec![101, 115, 99, 104, 101, 114];
    let name = std::str::from_utf8_mut(&mut data).unwrap();

    r.capture(name).await;
});

assert_eq!("escher", *name.as_ref());
name.as_mut().make_ascii_uppercase();
assert_eq!("ESCHER", *name.as_ref());
```

### Capturing multiple mixed references

```rust
use escher::{Escher, Rebindable};

#[derive(Rebindable)]
struct MyStruct<'this> {
    int_data: &'this Box<i32>,
    int_ref: &'this i32,
    float_ref: &'this mut f32,
}

let mut my_value = Escher::new(|r| async move {
    let int_data = Box::new(42);
    let mut float_data = Box::new(3.14);

    r.capture(MyStruct{
        int_data: &int_data,
        int_ref: &int_data,
        float_ref: &mut float_data,
    }).await;
});

assert_eq!(Box::new(42), *my_value.as_ref().int_data);
assert_eq!(3.14, *my_value.as_ref().float_ref);

*my_value.as_mut().float_ref = (*my_value.as_ref().int_ref as f32) * 2.0;

assert_eq!(84.0, *my_value.as_ref().float_ref);
```

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
<p align="center">
    <img src="https://github.com/petrosagg/escher/blob/master/assets/moves-invalidate-pointer.png?raw=true" width="600" />
</p>

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

<p align="center">
    <img src="https://github.com/petrosagg/escher/blob/master/assets/almost-foo.png?raw=true" width="600" />
</p>


This is pretty restrictive though. Ideally we'd lke to be able return some
owned value and be free to move it around, put it on the heap, etc.

### Actually returning an `AlmostFoo`

> **Note:** The description of async stacks bellow is not what actually happens
> in rustc but is enough to illustrate the point. `escher`'s API does make use
> that the desired values are held across an await point to force them to be
> included in the generated Future.

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

<p align="center">
    <img src="https://github.com/petrosagg/escher/blob/master/assets/async-stack.png?raw=true" width="600" />
</p>

### Putting it all together

`escher` builds upon the techniques described above and provides a solution for
getting the pointer from within the opaque future struct. Each `Escher<T>`
instance holds a Pinned Future and a raw pointer to T. The pointer to T is
computed by polling the Future just enough times for the desired T to be
constructed.

As its API, it provides the `as_ref()` and `as_mut()` methods that unsafely
turn the raw pointer to T into a &T with its lifetime bound to the lifetime of
`Escher<T>` itself. This ensures that the future will outlive any usage of the
self-reference!

Thank you for reading this far! If you would like to learn how escher uses the
above concepts in detail please take a look at the implementation.

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
