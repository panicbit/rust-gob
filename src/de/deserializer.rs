use std::io::Read;
use std::rc::Rc;
use serde::{self,Deserialize};
use serde::de::Visitor;
use errors::*;
use types::{TypeId,TypeDef,WireType};
use types::ids::*;
use super::{ReadGob,ValueDeserializer};
use TypeMap;

pub struct Deserializer<R> {
    pub(crate) reader: R,
    types: TypeMap,
}

impl<R: Read> Deserializer<R> {
    pub fn new(reader: R) -> Self {
        Deserializer {
            reader,
            types: TypeMap::new(),
        }
    }

    pub fn deserialize<'de, T: Deserialize<'de>>(&mut self) -> Result<T> {
        T::deserialize(self)
    }

    pub(super) fn look_up_type(&self, type_id: TypeId) -> Result<TypeDef> {
        TypeDef::from_id(type_id, &self.types).ok_or(ErrorKind::UndefinedType(type_id).into())
    }

    pub(super) fn reader(&mut self) -> &mut R {
        &mut self.reader
    }

    pub(super) fn deserialize_value<'de, V>(&mut self, visitor: V, type_def: TypeDef) -> Result<V::Value>
        where V: Visitor<'de>
    {
        let ref mut de = ValueDeserializer::new(self, type_def);
        serde::Deserializer::deserialize_any(de, visitor)
    }
}

impl<'a, 'de, R: Read> serde::Deserializer<'de> for &'a mut Deserializer<R> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
        where V: Visitor<'de>
    {
        let mut len;
        let mut type_id;

        // Read as many type definitions as possible
        loop {
            len = self.reader.read_gob_usize()?;
            type_id = self.reader.read_gob_type_id()?;

            trace!("Len: {}", len);

            if type_id >= 0 { break } // The following data is a value, not a definition

            type_id = -type_id;

            trace!("Defining type {}", type_id);

            let type_def = WireType::deserialize(&mut ValueDeserializer::new(self, TypeDef::WireType))?
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

        self.deserialize_value(visitor, type_def)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
    where
        V: Visitor<'de>
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
