//! > Self-referencial structs using async stacks
//!
//! Escher is an extremely simple library providing a safe and sound API to build self-referencial
//! structs. It works by (ab)using the async await trasformation of rustc. If you'd like to know
//! more about the inner workings please take a look at the [How it
//! works](https://github.com/petrosagg/escher#how-it-works) section and the source code.
//!
//! Compared to the state of the art escher:
//!
//! * Is only around 100 lines of well-commented code
//! * Contains only two `unsafe` calls that are well argued for
//! * Uses rustc for all the analysis. If it compiles, the self references are correct
//!
//! # Usage
//!
//! This library provides the [Escher<T>](Escher) wrapper type that can hold self-referencial data
//! and expose them safely through the [as_ref()](Escher::as_ref) and [as_mut()](Escher::as_mut)
//! functions.
//!
//! You construct a self reference by calling Escher's constructor and providing an async closure
//! that will initialize your self-references on its stack. Your closure will be provided with a
//! capturer `r` that has a single [capture()](Capturer::capture) method that consumes `r`.
//!
//! > **Note:** It is important to `.await` the result `.capture()` in order for escher to correctly
//! initialize your struct.
//!
//! Once all the data and references are created you can capture the desired ones. Simple
//! references to owned data can be captured directly (see first example).
//!
//! To capture more than one variable or capture references to non-owned data you will have to
//! define your own reference struct that derives [Rebindable](escher_derive::Rebindable) (see
//! second example).
//!
//! # Examples
//!
//! ## Simple `&str` view into an owned `Vec<u8>`
//!
//! The simplest way to use Escher is to create a reference of some data and then capture it:
//!
//! ```rust
//! use escher::Escher;
//!
//! let escher_heart = Escher::new(|r| async move {
//!     let data: Vec<u8> = vec![240, 159, 146, 150];
//!     let sparkle_heart = std::str::from_utf8(&data).unwrap();
//!
//!     r.capture(sparkle_heart).await;
//! });
//!
//! assert_eq!("💖", *escher_heart.as_ref());
//! ```
//!
//! ## Capturing both a `Vec<u8>` and a `&str` view into it
//!
//! In order to capture more than one things you can define a struct that will be used to capture
//! the variables:
//!
//! ```rust
//! use escher::{Escher, Rebindable};
//!
//! #[derive(Rebindable)]
//! struct VecStr<'this> {
//!     data: &'this Vec<u8>,
//!     s: &'this str,
//! }
//!
//! let escher_heart = Escher::new(|r| async move {
//!     let data: Vec<u8> = vec![240, 159, 146, 150];
//!
//!     r.capture(VecStr{
//!         data: &data,
//!         s: std::str::from_utf8(&data).unwrap(),
//!     }).await;
//! });
//!
//! assert_eq!(240, escher_heart.as_ref().data[0]);
//! assert_eq!("💖", escher_heart.as_ref().s);
//! ```
//!
//! ## Capturing a mutable `&mut str` view into a `Vec<u8>`
//!
//! If you capture a mutable reference to some piece of data then you cannot capture the data
//! itself like the previous example. This is mandatory as doing otherwise would create two mutable
//! references into the same piece of data which is not allowed.
//!
//! ```rust
//! use escher::Escher;
//!
//! let mut name = Escher::new(|r| async move {
//!     let mut data: Vec<u8> = vec![101, 115, 99, 104, 101, 114];
//!     let name = std::str::from_utf8_mut(&mut data).unwrap();
//!
//!     r.capture(name).await;
//! });
//!
//! assert_eq!("escher", *name.as_ref());
//! name.as_mut().make_ascii_uppercase();
//! assert_eq!("ESCHER", *name.as_ref());
//! ```
//!
//! ## Capturing multiple mixed references
//!
//! ```rust
//! use escher::{Escher, Rebindable};
//!
//! #[derive(Rebindable)]
//! struct MyStruct<'this> {
//!     int_data: &'this Box<i32>,
//!     int_ref: &'this i32,
//!     float_ref: &'this mut f32,
//! }
//!
//! let mut my_value = Escher::new(|r| async move {
//!     let int_data = Box::new(42);
//!     let mut float_data = Box::new(3.14);
//!
//!     r.capture(MyStruct{
//!         int_data: &int_data,
//!         int_ref: &int_data,
//!         float_ref: &mut float_data,
//!     }).await;
//! });
//!
//! assert_eq!(Box::new(42), *my_value.as_ref().int_data);
//! assert_eq!(3.14, *my_value.as_ref().float_ref);
//!
//! *my_value.as_mut().float_ref = (*my_value.as_ref().int_ref as f32) * 2.0;
//!
//! assert_eq!(84.0, *my_value.as_ref().float_ref);
//! ```

mod escher;
mod tests;

/// This trait can be derived for any struct, enum, or union to make its lifetimes rebindable and
/// thus compatible with the [Rebind] type level function.
///
/// ```
/// use escher::Rebindable;
///
/// #[derive(Rebindable)]
/// struct VecStr<'this> {
///     data: &'this Vec<u8>,
///     s: &'this str,
/// }
/// ```
pub use escher_derive::Rebindable;
pub use crate::escher::*;
