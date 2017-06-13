mod type_def;
mod wire_type;

pub use self::type_def::TypeDef;
pub use self::wire_type::{WireType, WireTypeEnum};

pub mod ids {
    use super::TypeId;
    pub const BOOL_ID: TypeId = 1;
    pub const INT_ID: TypeId = 2;
    pub const UINT_ID: TypeId = 3;
    pub const FLOAT_ID: TypeId = 4;
    pub const BYTE_SLICE_ID: TypeId = 5;
    pub const STRING_ID: TypeId = 6;
    pub const COMPLEX_ID: TypeId = 7;
    pub const INTERFACE_ID: TypeId = 8;
    pub const WIRE_TYPE_ID: TypeId = 16;
    pub const ARRAY_TYPE_ID: TypeId = 17;
    pub const COMMON_TYPE_ID: TypeId = 18;
    pub const SLICE_TYPE_ID: TypeId = 19;
    pub const STRUCT_TYPE_ID: TypeId = 20;
    pub const FIELD_TYPE_ID: TypeId = 21;
    pub const FIELD_TYPE_SLICE_ID: TypeId = 22;
    pub const MAP_TYPE_ID: TypeId = 23;
}

pub type TypeId = isize;
pub type FieldId = isize;

#[derive(Default,Debug,Clone,Deserialize)]
pub struct ArrayType {
    #[serde(rename="CommenType")] common: CommonType,
          #[serde(rename="Elem")] elem: TypeId,
           #[serde(rename="Len")] len: isize,
}

#[derive(Default,Debug,Clone,Deserialize)]
#[serde(default)]
pub struct CommonType {
    #[serde(rename="Name")] name: String,
      #[serde(rename="Id")] id: isize,
}

#[derive(Default,Debug,Clone,Deserialize)]
#[serde(default)]
pub struct SliceType {
    #[serde(rename="CommonType")] common: CommonType,
          #[serde(rename="Elem")] elem: TypeId,
}

#[derive(Default,Debug,Clone,Deserialize)]
#[serde(default)]
pub struct StructType {
    #[serde(rename="CommonType")] common: CommonType,
         #[serde(rename="Field")] fields: Vec<FieldType>,
}

#[derive(Default,Debug,Clone,Deserialize)]
#[serde(default)]
pub struct FieldType {
    #[serde(rename="Name")] name: String,
      #[serde(rename="Id")] id: TypeId,
}

#[derive(Default,Debug,Clone,Deserialize)]
#[serde(default)]
pub struct MapType {
    common: CommonType,
    key: TypeId,
    elem: TypeId,
}
