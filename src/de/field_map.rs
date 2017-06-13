use std::io::Read;
use serde;
use de::Deserializer;
use errors::*;
use types::TypeDef;
use super::State;

pub struct FieldMap<'a, 'de: 'a, R: 'a> {
    current_field: isize,
    de: &'a mut Deserializer<'de, R>,
    type_def: TypeDef,
}

impl<'a, 'de: 'a, R: 'a> FieldMap<'a, 'de, R> {
    pub fn new(de: &'a mut Deserializer<'de, R>, type_def: TypeDef) -> Self {
        FieldMap {
            current_field: -1,
            de,
            type_def,
        }
    }
}

// `MapAccess` is provided to the `Visitor` to give it the ability to iterate
// through entries of the map.
impl<'a, 'de, R: Read> serde::de::MapAccess<'de> for FieldMap<'a, 'de, R> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
        where K: serde::de::DeserializeSeed<'de>
    {
        trace!("Next key");
        let field_increment = self.de.read_usize()?;
        self.current_field += field_increment as isize;

        trace!("Increment {}", field_increment);

        // TODO: Check wether self.current_field exceeds type_id's field num?

        if field_increment == 0 {
            self.de.state = State::Start;
            return Ok(None)
        }
            
        self.de.state = State::DecodeFieldName(self.type_def.clone(), self.current_field);
        seed.deserialize(&mut *self.de).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
        where V: serde::de::DeserializeSeed<'de>
    {
        trace!("Next value");
        self.de.state = State::DecodeFieldValue(self.type_def.clone(), self.current_field);
        seed.deserialize(&mut *self.de)
    }
}
