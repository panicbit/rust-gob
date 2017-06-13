use std::rc::Rc;
use std::collections::HashMap;
use std::io::Read;
use serde::de::Visitor;
use errors::*;
use de::{Deserializer,SliceDecoder,FieldMap};
use super::{WireTypeEnum, TypeId, FieldId};
use super::ids::*;
use GobDecodable;

#[derive(Debug,Clone)]
pub enum TypeDef {
    Bool,
    Int,
    Uint,
    Float,
    ByteSlice,
    String,
    Complex,
    Interface,
    WireType,
    ArrayType,
    CommonType,
    SliceType,
    StructType,
    FieldType,
    FieldTypeSlice,
    MapType,
    Custom(Rc<WireTypeEnum>),
}

impl TypeDef {
    pub fn id(&self) -> TypeId {
        match *self {
            TypeDef::Bool => BOOL_ID,
            TypeDef::Int => INT_ID,
            TypeDef::Uint => UINT_ID,
            TypeDef::Float => FLOAT_ID,
            TypeDef::ByteSlice => BYTE_SLICE_ID,
            TypeDef::String => STRING_ID,
            TypeDef::Complex => COMPLEX_ID,
            TypeDef::Interface => INTERFACE_ID,
            TypeDef::WireType => WIRE_TYPE_ID,
            TypeDef::ArrayType => ARRAY_TYPE_ID,
            TypeDef::CommonType => COMMON_TYPE_ID,
            TypeDef::SliceType => SLICE_TYPE_ID,
            TypeDef::StructType => STRUCT_TYPE_ID,
            TypeDef::FieldType => FIELD_TYPE_ID,
            TypeDef::FieldTypeSlice => FIELD_TYPE_SLICE_ID,
            TypeDef::MapType => MAP_TYPE_ID,
            TypeDef::Custom(ref t) => t.id(),
        }
    }

    pub fn from_id(type_id: TypeId, types: &HashMap<TypeId, TypeDef>) -> Option<TypeDef>{
        Some(match type_id {
            BOOL_ID => TypeDef::Bool,
            INT_ID => TypeDef::Int,
            UINT_ID => TypeDef::Uint,
            FLOAT_ID => TypeDef::Float,
            BYTE_SLICE_ID => TypeDef::ByteSlice,
            STRING_ID => TypeDef::String,
            COMPLEX_ID => TypeDef::Complex,
            INTERFACE_ID => TypeDef::Interface,
            WIRE_TYPE_ID => TypeDef::WireType,
            ARRAY_TYPE_ID => TypeDef::ArrayType,
            COMMON_TYPE_ID => TypeDef::CommonType,
            SLICE_TYPE_ID => TypeDef::SliceType,
            STRUCT_TYPE_ID => TypeDef::StructType,
            FIELD_TYPE_ID => TypeDef::FieldType,
            FIELD_TYPE_SLICE_ID => TypeDef::FieldTypeSlice,
            MAP_TYPE_ID => TypeDef::MapType,
            _ => return types.get(&type_id).cloned(),
        })
    }

    pub fn deserialize_value<'de, R, V>(self, de: &mut Deserializer<'de, R>, visitor: V) -> Result<V::Value>
        where R: Read,
              V: Visitor<'de>,
    {
        match self.clone() {
            TypeDef::String
            | TypeDef::Interface
            | TypeDef::ArrayType
            | TypeDef::Complex
            | TypeDef::SliceType
            | TypeDef::FieldTypeSlice
            | TypeDef::MapType
            | TypeDef::StructType => bail!("Decoding for {:?} not implemented", self),
            TypeDef::ByteSlice => visitor.visit_seq(SliceDecoder::new(de, TypeDef::Uint)?),
            TypeDef::Bool => visitor.visit_bool(usize::decode(&mut de.reader)? != 0),
            TypeDef::Uint => visitor.visit_u64(u64::decode(&mut de.reader)?),
            TypeDef::Float => visitor.visit_f64(f64::decode(&mut de.reader)?),
            TypeDef::Int => visitor.visit_i64(isize::decode(&mut de.reader)? as i64),
            TypeDef::CommonType => visitor.visit_map(FieldMap::new(de, TypeDef::CommonType)),
            TypeDef::WireType => visitor.visit_map(FieldMap::new(de, TypeDef::WireType)),
            TypeDef::FieldType => visitor.visit_map(FieldMap::new(de, TypeDef::FieldType)),
            TypeDef::Custom(wire_type) => match *wire_type {
                WireTypeEnum::Struct(_) => visitor.visit_map(FieldMap::new(de, self)),
                _ => bail!("Decoding for {} not implemented")
            }
        }
    }

    pub fn deserialize_field_name<'de, R, V>(self, de: &mut Deserializer<'de, R>, visitor: V, field_id: FieldId) -> Result<V::Value>
        where R: Read,
              V: Visitor<'de>,
    {
        let field_name = match self {
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
            | TypeDef::SliceType => bail!("Field name for {} not implemented", self.id()),
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
                WireTypeEnum::Struct(ref t) => t.fields.get(field_id as usize).map(|field| field.name.as_str()).ok_or(ErrorKind::InvalidField)?,
                _ => bail!("Decoding of field name for {} not implemented")
            }
        };

        trace!("{}", field_name);
        visitor.visit_str(&field_name)
    }

    pub fn deserialize_field_value<'de, R, V>(self, de: &mut Deserializer<'de, R>, visitor: V, field_id: FieldId) -> Result<V::Value>
        where R: Read,
          V: Visitor<'de>,
    {
        match self {
            TypeDef::WireType => match field_id {
                2 => visitor.visit_map(FieldMap::new(de, TypeDef::StructType)),
                3 => TypeDef::MapType.deserialize_value(de, visitor),
                _ => bail!("wireType field {} unimplemented", field_id),
            },
            TypeDef::StructType => match field_id {
                0 => TypeDef::CommonType.deserialize_value(de, visitor),
                1 => visitor.visit_seq(SliceDecoder::new(de, TypeDef::FieldType)?),
                _ => bail!("structType field {} unimplemented", field_id),
            },
            TypeDef::FieldType => match field_id {
                0 => visitor.visit_byte_buf(Vec::decode(&mut de.reader)?),
                1 => visitor.visit_i64(TypeId::decode(&mut de.reader)? as i64), // TODO: Fix cast
                _ => bail!("fieldType field {} unimplemented", field_id),
            },
            TypeDef::CommonType => match field_id {
                0 => visitor.visit_byte_buf(Vec::decode(&mut de.reader)?),
                1 => visitor.visit_i64(TypeId::decode(&mut de.reader)? as i64), // TODO: Fix cast
                _ => bail!("commonType field {} unimplemented", field_id)
            },
            TypeDef::Custom(ref wire_type) => match **wire_type {
                WireTypeEnum::Struct(ref t) => {
                    let type_id = t.fields.get(field_id as usize).map(|field| field.id).ok_or(ErrorKind::InvalidField)?;
                    let type_def = TypeDef::from_id(type_id, &de.types).ok_or(ErrorKind::UndefinedType(type_id))?;
                    type_def.deserialize_value(de, visitor)
                },
                _ => bail!("Decoding of field name for {} not implemented")
            },
            _ => bail!("Decoding of field value for id {} not implemented", self.id())
        }
    }
}
