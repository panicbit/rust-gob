use std::io::Read;
use serde;
use serde::de::Visitor;
use types::{TypeDef,WireTypeEnum};
use errors::*;
use super::{FieldMap,SliceDecoder,ReadGob};

pub struct ValueDeserializer<'a, R: 'a> {
    de: &'a mut super::Deserializer<R>,
    type_def: TypeDef,
}

impl<'a, R: Read + 'a> ValueDeserializer<'a, R> {
    pub fn new(de: &'a mut super::Deserializer<R>, type_def: TypeDef) -> Self {
        ValueDeserializer { de, type_def }
    }

    fn deserialize_slice<'de, V>(&mut self, visitor: V, type_def: TypeDef) -> Result<V::Value>
        where V: Visitor<'de>
    {
        visitor.visit_seq(SliceDecoder::new(self.de, type_def)?)
    }

    fn deserialize_struct<'de, V>(&mut self, visitor: V, type_def: TypeDef) -> Result<V::Value>
        where V: Visitor<'de>
    {
        visitor.visit_map(FieldMap::new(self.de, type_def))
    }
}

impl<'a, 'b, 'de, R: Read> serde::Deserializer<'de> for &'b mut ValueDeserializer<'a, R> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
        where V: Visitor<'de>
    {
        trace!("### DE VALUE TYPE: {:?}", self.type_def);
        match self.type_def.clone() {
            TypeDef::Interface
            | TypeDef::ArrayType
            | TypeDef::Complex
            | TypeDef::SliceType
            | TypeDef::MapType => bail!("Decoding for {:?} not implemented", self.type_def),
            TypeDef::StructType => self.deserialize_struct(visitor, TypeDef::StructType),
            TypeDef::ByteSlice => self.deserialize_slice(visitor, TypeDef::Uint),
            TypeDef::Bool => visitor.visit_bool(self.de.reader().read_gob_bool()?),
            TypeDef::Uint => visitor.visit_u64(self.de.reader().read_gob_u64()?),
            TypeDef::Float => visitor.visit_f64(self.de.reader().read_gob_f64()?),
            TypeDef::Int => visitor.visit_i64(self.de.reader().read_gob_i64()?),
            TypeDef::String => visitor.visit_bytes(&self.de.reader().read_gob_bytes()?),
            TypeDef::CommonType => self.deserialize_struct(visitor, TypeDef::CommonType),
            TypeDef::WireType => self.deserialize_struct(visitor, TypeDef::WireType),
            TypeDef::FieldType => self.deserialize_struct(visitor, TypeDef::FieldType),
            TypeDef::FieldTypeSlice => self.deserialize_slice(visitor, TypeDef::FieldType),
            TypeDef::Custom(wire_type) => match *wire_type {
                WireTypeEnum::Struct(_) => {
                    let type_def = self.type_def.clone();
                    self.deserialize_struct(visitor, type_def)
                }
                _ => bail!("Decoding for {} not implemented")
            }
        }
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
