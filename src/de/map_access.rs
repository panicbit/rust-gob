use std::io::Read;
use serde;
use serde::de::{Visitor,IntoDeserializer};
use errors::*;
use types::{TypeDef,WireTypeEnum};
use super::ReadGob;

pub struct MapAccess<'a, R: Read + 'a> {
    de: &'a mut super::Deserializer<R>,
    type_def: TypeDef,
    current_field: isize,
}

impl<'a, R: Read + 'a> MapAccess<'a, R> {
    pub fn new(de: &'a mut super::Deserializer<R>, type_def: TypeDef) -> Self {
        MapAccess {
            de,
            type_def,
            current_field: -1,
        }
    }
}

// `MapAccess` is provided to the `Visitor` to give it the ability to iterate
// through entries of the map.
impl<'a, 'de, R: Read> serde::de::MapAccess<'de> for MapAccess<'a, R> {
    type Error = Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
        where K: serde::de::DeserializeSeed<'de>
    {
        trace!("Next key");
        let field_increment = self.de.reader().read_gob_usize()?;
        self.current_field += field_increment as isize;

        trace!("Increment {}", field_increment);

        // TODO: Check wether self.current_field exceeds type_id's field num?

        if field_increment == 0 {
            return Ok(None)
        }
        
        let field_id = self.current_field;
        let field_name = match self.type_def {
              TypeDef::Bool
            | TypeDef::Int
            | TypeDef::Uint
            | TypeDef::Float
            | TypeDef::ByteSlice
            | TypeDef::String
            | TypeDef::Interface
            | TypeDef::ArrayType
            | TypeDef::Complex
            | TypeDef::FieldTypeSlice
            | TypeDef::MapType
            | TypeDef::SliceType => bail!("Field name for {} not implemented", self.type_def.id()),
            // TypeDef::MapType => visitor.visit_map(MapMap::new(de, ))
            TypeDef::WireType => match field_id {
                0 => "ArrayT",
                1 => "SliceT",
                2 => "StructT",
                3 => "MapT",
                _ => bail!(ErrorKind::InvalidField)
            },
            TypeDef::CommonType => match field_id {
                0 => "Name",
                1 => "Id",
                _ => bail!(ErrorKind::InvalidField)
            },
            TypeDef::StructType => match field_id {
                0 => "CommonType",
                1 => "Field",
                _ => bail!(ErrorKind::InvalidField)
            },
            TypeDef::FieldType => match field_id {
                0 => "Name",
                1 => "Id",
                _ => bail!(ErrorKind::InvalidField)
            },
            TypeDef::Custom(ref wire_type) => match **wire_type {
                WireTypeEnum::Struct(ref t) => t.fields().get(field_id as usize).map(|field| field.name()).ok_or(ErrorKind::InvalidField)?,
                _ => bail!("Decoding of field name for {} not implemented")
            }
        };

        trace!("### FIELD NAME: {}", field_name);

        seed.deserialize(field_name.into_deserializer()).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
        where V: serde::de::DeserializeSeed<'de>
    {
        trace!("Next value");
        seed.deserialize(self)
    }
}

impl<'b, 'a, 'de, R: Read> serde::Deserializer<'de> for &'b mut MapAccess<'a, R> {
   type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
        where V: Visitor<'de>
    {
        let field_id = self.current_field;
        match self.type_def.clone() {
            TypeDef::WireType => match field_id {
                2 => self.de.deserialize_value(visitor, TypeDef::StructType),
                3 => self.de.deserialize_value(visitor, TypeDef::MapType),
                _ => bail!("wireType field {} unimplemented", field_id),
            },
            TypeDef::StructType => match field_id {
                0 => self.de.deserialize_value(visitor, TypeDef::CommonType),
                1 => self.de.deserialize_value(visitor, TypeDef::FieldTypeSlice),
                _ => bail!("structType field {} unimplemented", field_id),
            },
            TypeDef::FieldType => match field_id {
                0 => self.de.deserialize_value(visitor, TypeDef::String),
                1 => self.de.deserialize_value(visitor, TypeDef::Int), // TypeId
                _ => bail!("fieldType field {} unimplemented", field_id),
            },
            TypeDef::CommonType => match field_id {
                0 => self.de.deserialize_value(visitor, TypeDef::String),
                1 => self.de.deserialize_value(visitor, TypeDef::Int), // TypeId
                _ => bail!("commonType field {} unimplemented", field_id)
            },
            TypeDef::Custom(ref wire_type) => match **wire_type {
                WireTypeEnum::Struct(ref t) => {
                    let type_id = t.fields().get(field_id as usize).map(|field| field.id()).ok_or(ErrorKind::InvalidField)?;
                    let type_def = self.de.look_up_type(type_id)?; // chain_err?
                    self.de.deserialize_value(visitor, type_def)
                },
                _ => bail!("Decoding of field name for {} not implemented")
            },
            _ => bail!("Decoding of field value for id {} not implemented", self.type_def.id())
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
        where V: Visitor<'de>
    {
        visitor.visit_some(self)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string bytes
        byte_buf unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}
