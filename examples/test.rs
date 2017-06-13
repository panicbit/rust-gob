#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate gob;

use std::fs::File;
use std::collections::HashMap;
use serde::Deserialize;

fn main() {
    let path = "/tmp/out.bin";
    let input = File::open(path).expect(path);
    let mut decoder = gob::Deserializer::new(input);
    let n: gob::Result<Example> = Example::deserialize(&mut decoder);
    println!("{:#?}", n);
}

#[derive(Debug,Default,Clone,Deserialize)]
#[serde(default)]
struct Example {
    #[serde(rename="Bool")] bool: bool,
    #[serde(rename="Int")] int: isize,
    #[serde(rename="Uint")] uint: usize,
    #[serde(rename="Float")] float: f64,
    #[serde(rename="Bytes")] bytes: Vec<u8>,
    #[serde(rename="String")] string: String,
    #[serde(rename="Complex")] complex: Complex64,
    // #[serde(rename="Interface")] interface: interface{},
    #[serde(rename="Map")] map: HashMap<i32, String>,
    #[serde(rename="Nested")] nested: Point,
}

#[derive(Debug,Default,Clone,Deserialize)]
#[serde(default)]
struct Point {
    #[serde(rename="X")] x: i32,
    #[serde(rename="Y")] y: i32,
}

#[derive(Debug,Default,Clone,Deserialize)]
struct Complex64(f64, f64);
