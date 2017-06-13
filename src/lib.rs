extern crate byteorder;
#[macro_use] extern crate serde;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate error_chain;
#[macro_use] extern crate log;

pub mod errors;
pub mod de;
mod types;

pub use de::Deserializer;
pub use errors::*;
