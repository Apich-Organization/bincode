#![cfg(all(feature = "derive", feature = "std"))]

extern crate bincode_next as bincode;

use bincode::{Decode, Encode};

#[derive(Encode, Decode)]
struct Foo<Bar = ()> {
    x: Bar,
}
