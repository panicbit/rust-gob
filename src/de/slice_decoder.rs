use std::io::Read;
use serde::de::{SeqAccess,DeserializeSeed};
use errors::*;
use types::TypeDef;
use super::{Deserializer,ValueDeserializer,ReadGob};

pub struct SliceDecoder<'a, R: Read + 'a> {
    de: &'a mut Deserializer<R>,
    len: usize,
    current_index: usize,
    type_def: TypeDef,
}

impl<'a, R: Read + 'a> SliceDecoder<'a, R> {
    pub fn new(de: &'a mut Deserializer<R>, type_def: TypeDef) -> Result<Self> {
        let len = de.reader().read_gob_usize()?;
        Ok(SliceDecoder {
            de,
            len,
            current_index: 0,
            type_def,
        })
    }
}

impl<'a, 'de, R: Read> SeqAccess<'de> for SliceDecoder<'a, R> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
        where T: DeserializeSeed<'de>
    {
        if self.current_index >= self.len {
            return Ok(None);
        }

        self.current_index += 1;

        let ref mut de = ValueDeserializer::new(self.de, self.type_def.clone());
        seed.deserialize(de).map(Some)
    }
}
