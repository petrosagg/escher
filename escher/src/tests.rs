#![cfg(test)]
use super::*;
use crate as escher;

// #[test]
// fn test_dtonlay() {
//     use std::cell::Cell;
//
//     #[derive(Rebindable)]
//     struct Struct<'a>(fn(&'a String));
//
//     fn main() {
//         static STRING: String = String::new();
//         thread_local!(static CELL: Cell<&'static String> = Cell::new(&STRING));
//         let escher = Escher::new(|r| async {
//             r.capture(Struct(|x| CELL.with(|cell| cell.set(x)))).await;
//         });
//         let mut string = Ok(".".repeat(3));
//         let f = escher.as_ref().0;
//         let s = string.as_ref().unwrap();
//         f(s);
//         string = Err((s.as_ptr(), 100usize, 100usize));
//         CELL.with(|cell| println!("{}", cell.get()));
//         string.unwrap_err();
//     }
// }

#[test]
#[should_panic(expected = "capture no longer live")]
fn adversarial_sync_fn() {
    #[derive(Rebindable)]
    struct MyStr<'a>(&'a str);

    Escher::new(|r| {
        let data: Vec<u8> = vec![240, 159, 146, 150];
        let sparkle_heart = std::str::from_utf8(&data).unwrap();

        let _ = r.capture(MyStr(sparkle_heart));

        // dummy future to satisfy escher
        std::future::ready(())
    });
}

#[test]
#[should_panic(expected = "captured value outside of async stack")]
fn adversarial_capture_non_stack() {
    #[derive(Rebindable)]
    struct MyStr<'a>(&'a str);

    Escher::new(|r| {
        let data: Vec<u8> = vec![240, 159, 146, 150];
        let sparkle_heart = std::str::from_utf8(&data).unwrap();

        let fut = r.capture(MyStr(sparkle_heart));
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

    let escher_heart = Escher::new(|r| async move {
        let data: Vec<u8> = vec![240, 159, 146, 150];
        let sparkle_heart = std::str::from_utf8(&data).unwrap();

        r.capture(MaybeStr::Some(sparkle_heart)).await;
    });
    assert_eq!(MaybeStr::Some("💖"), *escher_heart.as_ref());
}

#[test]
fn capture_union() {
    /// Holds a vector and a str reference to the data of the vector
    #[derive(Rebindable)]
    union MaybeStr<'a> {
        some: &'a str,
        none: (),
    }

    let escher_heart = Escher::new(|r| async move {
        let data: Vec<u8> = vec![240, 159, 146, 150];
        let sparkle_heart = std::str::from_utf8(&data).unwrap();

        r.capture(MaybeStr {
            some: sparkle_heart,
        })
        .await;
    });

    unsafe {
        assert_eq!("💖", escher_heart.as_ref().some);
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
