use std::rc::Rc;
use std::collections::HashMap;
use super::{WireTypeEnum, TypeId};
use super::ids::*;

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
}
