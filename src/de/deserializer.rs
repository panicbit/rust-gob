use super::State;
use std::io::Read;
use std::marker::PhantomData;
use std::collections::HashMap;
use std::rc::Rc;
use serde::Deserialize;
use serde;
use errors::*;
use types::{TypeId,TypeDef,WireType};
use types::ids::*;
use GobDecodable;

pub struct Deserializer<'a, R> {
    pub(crate) reader: R,
    pub(crate) state: State,
    pub(crate) types: HashMap<TypeId, TypeDef>,
    _m: PhantomData<&'a ()>,
}

impl<'a, R: Read> Deserializer<'a, R> {
    pub fn new(reader: R) -> Self {
        Deserializer {
            reader,
            state: State::Start,
            types: HashMap::new(),
            _m: PhantomData,
        }
    }
}

impl<'de, 'a, R: Read> serde::Deserializer<'de> for &'a mut Deserializer<'de, R> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
        where V: serde::de::Visitor<'de>
    {
        trace!("State: {:?}", self.state);
        match self.state.clone() {
            State::Start => {
                let mut len;
                let mut type_id;

                // Read as many type definitions as possible
                loop {
                    len = usize::decode(&mut self.reader)?;
                    type_id = TypeId::decode(&mut self.reader)?;

                    trace!("Len: {}", len);

                    if type_id >= 0 { break } // The following data is a value, not a definition

                    type_id = -type_id;

                    trace!("Defining type {}", type_id);

                    self.state = State::DecodeValue(TypeDef::WireType);
                    let type_def = WireType::deserialize(&mut *self)?
                        .try_into_enum()
                        .map(Rc::new)
                        .map(TypeDef::Custom)?;

                    trace!("Type def: {:#?}", type_def);

                    if type_id != type_def.id() {
                        bail!(ErrorKind::DefiningIdMismatch(type_id, type_def.id()))
                    }

                    if is_type_builtin(type_id) {
                        bail!(ErrorKind::DefiningBuiltin(type_id))
                    }

                    if self.types.insert(type_def.id(), type_def).is_some() {
                        bail!(ErrorKind::TypeAlreadyDefined(type_id))
                    };
                }

                trace!("Decoding type {}", type_id);

                let type_def = TypeDef::from_id(type_id, &self.types)
                    .ok_or(ErrorKind::UndefinedType(type_id))?;

                self.state = State::DecodeValue(type_def);
                self.deserialize_any(visitor)
            }
            State::DecodeValue(type_def) => type_def.deserialize_value(self, visitor),
            State::DecodeFieldName(type_def, field_id) => type_def.deserialize_field_name(self, visitor, field_id),
            State::DecodeFieldValue(type_def, field_id) => type_def.deserialize_field_value(self, visitor, field_id),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: serde::de::Visitor<'de>
    {
        visitor.visit_some(self)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

fn is_type_builtin(id: TypeId) -> bool {
    [
        BOOL_ID,
        INT_ID,
        UINT_ID,
        FLOAT_ID,
        BYTE_SLICE_ID,
        STRING_ID,
        COMPLEX_ID,
        INTERFACE_ID,
        WIRE_TYPE_ID,
        ARRAY_TYPE_ID,
        COMMON_TYPE_ID,
        SLICE_TYPE_ID,
        STRUCT_TYPE_ID,
        FIELD_TYPE_ID,
        MAP_TYPE_ID,
    ].contains(&id)
}
