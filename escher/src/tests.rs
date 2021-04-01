#![cfg(test)]
use super::*;
use crate as escher;

#[test]
fn simple_ref() {
    let escher_heart = Escher::new(|r| async move {
        let data: Vec<u8> = vec![240, 159, 146, 150];
        let sparkle_heart = std::str::from_utf8(&data).unwrap();

        r.capture(sparkle_heart).await;
    });

    assert_eq!("💖", *escher_heart.as_ref());
}

#[test]
fn simple_mut_ref() {
    let escher_heart = Escher::new(|r| async move {
        let data: Vec<u8> = vec![240, 159, 146, 150];
        let sparkle_heart = std::str::from_utf8(&data).unwrap();

        r.capture(sparkle_heart).await;
    });

    assert_eq!("💖", *escher_heart.as_ref());
}

#[test]
#[should_panic(expected = "capture no longer live")]
fn adversarial_sync_fn() {
    Escher::new(|r| {
        let data: Vec<u8> = vec![240, 159, 146, 150];
        let sparkle_heart = std::str::from_utf8(&data).unwrap();

        let _ = r.capture(sparkle_heart);

        // dummy future to satisfy escher
        std::future::ready(())
    });
}

#[test]
#[should_panic(expected = "captured value outside of async stack")]
fn adversarial_capture_non_stack() {
    Escher::new(|r| {
        let data: Vec<u8> = vec![240, 159, 146, 150];
        let sparkle_heart = std::str::from_utf8(&data).unwrap();

        let fut = r.capture(sparkle_heart);
        // make it appear as if capture is still alive
        std::mem::forget(fut);

        // dummy future to satisfy escher
        std::future::ready(())
    });
}

#[test]
fn capture_enum() {
    /// Holds a vector and a str reference to the data of the vector
    #[derive(Rebindable, PartialEq, Debug)]
    enum MaybeStr<'a> {
        Some(&'a str),
        None,
    }

    let mut escher_heart = Escher::new(|r| async move {
        let data: Vec<u8> = vec![240, 159, 146, 150];
        let sparkle_heart = std::str::from_utf8(&data).unwrap();

        r.capture(MaybeStr::Some(sparkle_heart)).await;
    });
    assert_eq!(MaybeStr::Some("💖"), *escher_heart.as_ref());
    *escher_heart.as_mut() = MaybeStr::None;
    assert_eq!(MaybeStr::None, *escher_heart.as_ref());
}

#[test]
fn capture_union() {
    /// Holds a vector and a str reference to the data of the vector
    #[derive(Rebindable)]
    union MaybeStr<'a> {
        some: &'a str,
        none: (),
    }

    let mut escher_heart = Escher::new(|r| async move {
        let data: Vec<u8> = vec![240, 159, 146, 150];
        let sparkle_heart = std::str::from_utf8(&data).unwrap();

        r.capture(MaybeStr {
            some: sparkle_heart,
        })
        .await;
    });

    unsafe {
        assert_eq!("💖", escher_heart.as_ref().some);
        escher_heart.as_mut().none = ();
        assert_eq!((), escher_heart.as_ref().none);
    }
}

#[test]
fn it_works() {
    /// Holds a vector and a str reference to the data of the vector
    #[derive(Rebindable)]
    struct VecStr<'a> {
        data: &'a Vec<u8>,
        s: &'a str,
    }

    let escher_heart = Escher::new(|r| async move {
        let data: Vec<u8> = vec![240, 159, 146, 150];
        let sparkle_heart = std::str::from_utf8(&data).unwrap();

        r.capture(VecStr {
            data: &data,
            s: sparkle_heart,
        })
        .await;
    });

    assert_eq!(240, escher_heart.as_ref().data[0]);
    assert_eq!("💖", escher_heart.as_ref().s);
}
