#![cfg(test)]
use crate as escher;
use super::*;

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
