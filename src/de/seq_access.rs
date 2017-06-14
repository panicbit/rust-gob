use std::io::Read;
use serde;
use errors::*;
use types::TypeDef;
use super::{Deserializer,ValueDeserializer,ReadGob};

pub struct SeqAccess<'a, R: Read + 'a> {
    de: &'a mut Deserializer<R>,
    len: usize,
    current_index: usize,
    type_def: TypeDef,
}

impl<'a, R: Read + 'a> SeqAccess<'a, R> {
    pub fn new(de: &'a mut Deserializer<R>, type_def: TypeDef) -> Result<Self> {
        let len = de.reader().read_gob_usize()?;
        Ok(SeqAccess {
            de,
            len,
            current_index: 0,
            type_def,
        })
    }
}

impl<'a, 'de, R: Read> serde::de::SeqAccess<'de> for SeqAccess<'a, R> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
        where T: serde::de::DeserializeSeed<'de>
    {
        if self.current_index >= self.len {
            return Ok(None);
        }

        self.current_index += 1;

        let ref mut de = ValueDeserializer::new(self.de, self.type_def.clone());
        seed.deserialize(de).map(Some)
    }
}
