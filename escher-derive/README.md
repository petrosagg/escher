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

You are looking at the escher-derive crate. You can find the full documentation
in the main escher crate.
