use std::io::Read;
use serde;
use errors::*;
use types::TypeDef;
use super::{Deserializer,State};

pub struct SliceDecoder<'a, R: 'a> {
    de: &'a mut Deserializer<R>,
    len: usize,
    current_index: usize,
    type_def: TypeDef,
}

impl<'a, R: Read> SliceDecoder<'a, R> {
    pub fn new(de: &'a mut Deserializer<R>, type_def: TypeDef) -> Result<Self> {
        let len = de.read_usize()?;
        Ok(SliceDecoder {
            de,
            len,
            current_index: 0,
            type_def,
        })
    }
}

// `SeqAccess` is provided to the `Visitor` to give it the ability to iterate
// through elements of the sequence.
impl<'a, 'de, R: Read> serde::de::SeqAccess<'de> for SliceDecoder<'a, R> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
        where T: serde::de::DeserializeSeed<'de>
    {
        if self.current_index >= self.len {
            self.de.state = State::Start;
            return Ok(None);
        }

        self.current_index += 1;
        self.de.state = State::DecodeValue(self.type_def.clone());

        seed.deserialize(&mut *self.de).map(Some)
    }
}
