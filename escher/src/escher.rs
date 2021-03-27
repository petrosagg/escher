use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::sync::Arc;
use std::task::Context;

use futures_task::noop_waker;

/// The `RebindTo` trait defines a type level function that allows you convert a type that holds
/// references of lifetime `'a` to a type that holds references of lifetime `'b`.
///
/// The trait is unsafe because the implementer needs to make sure that the associated type
/// differs with the implementing type only on their lifetimes. In other words, it's meant to
/// prevent incantations like:
///
/// ```ignore
/// unsafe impl<'a> RebindTo<'a> for Foo<'_> {
///     type Out = Bar<'a>; // !!WRONG!!
/// }
///
/// unsafe impl<'a> RebindTo<'a> for Foo<'_> {
///     type Out = Foo<'a>; // CORRECT
/// }
/// ```
pub unsafe trait RebindTo<'a> {
    type Out: 'a;
}

/// Blanket implementation for any reference to owned data
unsafe impl<'a, T: ?Sized + 'static> RebindTo<'a> for &'_ T {
    type Out = &'a T;
}

/// Blanket implementation for any mutable reference to owned data
unsafe impl<'a, T: ?Sized + 'static> RebindTo<'a> for &'_ mut T {
    type Out = &'a mut T;
}

/// Marker trait for any type that implements [RebindTo] for any lifetime. All types can derive
/// this trait using the [Rebindable](escher_derive::Rebindable) derive macro.
pub trait Rebindable: for<'a> RebindTo<'a> {}
impl<T: for<'a> RebindTo<'a>> Rebindable for T {}

/// Type-level function that takes a lifetime `'a` and a type `T` computes a new type `U` that is
/// identical to `T` except for its lifetimes that are now bound to `'a`.
///
/// A type `T` must implement [Rebindable] in order to use this type level function.
///
/// For example:
///
/// * `Rebind<'a, &'static str> == &'a str`
/// * `Rebind<'static, &'a str> == &'static str`
/// * `Rebind<'c, T<'a, 'b>> == T<'c, 'c>`
pub type Rebind<'a, T> = <T as RebindTo<'a>>::Out;

/// A containter of a self referencial struct. The self-referencial struct is constructed with the
/// aid of the async/await machinery of rustc, see Escher::new.
pub struct Escher<T> {
    _fut: Pin<Box<dyn Future<Output = ()>>>,
    ptr: *mut T,
}

impl<T: Rebindable> Escher<T> {
    /// Construct a self referencial struct using the provided closure. The user is expected to
    /// construct the desired data and references to them in the async stack and capture the
    /// desired state when ready.
    ///
    /// ```rust
    /// use escher::Escher;
    ///
    /// let escher_heart = Escher::new(|r| async move {
    ///     let data: Vec<u8> = vec![240, 159, 146, 150];
    ///     let sparkle_heart = std::str::from_utf8(&data).unwrap();
    ///
    ///     r.capture(sparkle_heart).await;
    /// });
    ///
    /// assert_eq!("💖", *escher_heart.as_ref());
    /// ```
    pub fn new<B, F>(builder: B) -> Self
    where
        B: FnOnce(Capturer<T>) -> F,
        F: Future<Output = ()> + 'static,
    {
        let ptr = Arc::new(AtomicPtr::new(std::ptr::null_mut()));
        let r = Capturer { ptr: ptr.clone() };
        let mut fut = Box::pin(builder(r));

        let waker = noop_waker();
        let mut cx = Context::from_waker(&waker);
        let _ = fut.as_mut().poll(&mut cx);

        // Adversarial code can attempt to capture a value without awaiting on the result
        assert!(
            Arc::strong_count(&ptr) == 2,
            "capture no longer live. Did you forget to .await the result of capture()?"
        );

        let ptr = ptr.load(Ordering::Acquire);

        let low = &*fut as *const _ as usize;
        let high = low + std::mem::size_of_val(&*fut);
        // Adversarial code can attempt to capture a value that does not live on the async stack
        assert!(
            low <= ptr as usize && ptr as usize <= high,
            "captured value outside of async stack. Did you run capture() in a non async function?"
        );

        // SAFETY: At this point we know that:
        // 1. We are given a future that has no external references because it is 'static
        // 2. We have a pointer that points into the state of the future
        // 3. The state of the future will never move again because it's behind a Pin<Box<T>>
        // 4. The pointer `ptr` points to a valid instance of T because:
        //    a. The only way to set the pointer is through Capturer::capture that expects a T
        //    b. The strong count of AtomicPtr is 2, so the async stack is in Capturer::capture_ref because:
        //       α. Capturer is not Clone, so one cannot fake the increased refcount
        //       β. Capturer::capture consumes Capturer so when the function returns the Arc will be dropped
        Escher { _fut: fut, ptr }
    }

    /// Get a shared reference to the inner T with its lifetime bound to &self
    pub fn as_ref<'a>(&'a self) -> &Rebind<'a, T> {
        // SAFETY
        // Validity of reference
        //    self.ptr points to a valid instance of T in side of self._fut (see safety argument in
        //    constructor)
        // Liveness of reference
        //    The resulting reference is has all its lifetimes bound to the lifetime of self that
        //    contains _fut that contains all the data that ptr could be referring to because it's
        //    a 'static Future
        unsafe { &*(self.ptr as *mut _) }
    }

    /// Get a mut reference to the inner T with its lifetime bound to &mut self
    pub fn as_mut<'a>(&'a mut self) -> &mut Rebind<'a, T> {
        // SAFETY: see safety argument of Self::as_ref
        unsafe { &mut *(self.ptr as *mut _) }
    }
}

/// An instance of `Capturer` is given to the closure passed to `Escher::new` and is used to
/// capture a reference from the async stack.
pub struct Capturer<T> {
    ptr: Arc<AtomicPtr<T>>,
}

impl<StaticT> Capturer<StaticT> {
    async fn capture_ref<T>(self, val: &mut T)
    where
        // once rustc supports equality constraints this can become: `StaticT = Rebind<'static, T>`
        T: RebindTo<'static, Out = StaticT>,
    {
        self.ptr.store(val as *mut _ as *mut StaticT, Ordering::Release);
        std::future::pending::<()>().await;
    }

    /// Captures the passed value into a future that never resolves.
    /// Callers of this method **must** `.await` it in order for Escher to capture the value.
    pub async fn capture<T>(self, mut val: T)
    where
        // once rustc supports equality constraints this can become: `StaticT = Rebind<'static, T>`
        T: RebindTo<'static, Out = StaticT>,
    {
        self.capture_ref(&mut val).await;
    }
}
