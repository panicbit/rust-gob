use errors::*;
use super::{TypeId,ArrayType,SliceType,StructType,MapType};

#[derive(Default,Debug,Clone,Deserialize)]
pub struct WireType {
    #[serde(rename="ArrayT")] array_type: Option<ArrayType>,
    #[serde(rename="SliceT")] slice_type: Option<SliceType>,
    #[serde(rename="StructT")] struct_type: Option<StructType>,
    #[serde(rename="MapT")] map_type: Option<MapType>,
}

impl WireType {
    pub fn try_into_enum(self) -> Result<WireTypeEnum> {
        let WireType {
            array_type,
            slice_type,
            struct_type,
            map_type,
        } = self;

        let mut n_some = 0;
        if array_type .is_some() { n_some += 1 }
        if slice_type .is_some() { n_some += 1 }
        if struct_type.is_some() { n_some += 1 }
        if map_type   .is_some() { n_some += 1 }

        if n_some != 1 {
            bail!(ErrorKind::AmbiguousWireType)
        }

                    array_type .map(WireTypeEnum::Array)
        .or_else(|| slice_type .map(WireTypeEnum::Slice))
        .or_else(|| struct_type.map(WireTypeEnum::Struct))
        .or_else(|| map_type   .map(WireTypeEnum::Map))
        .ok_or("BUG: Unhandled WireType case".into())
    }
}

#[derive(Debug,Clone)]
pub enum WireTypeEnum {
    Array(ArrayType),
    Slice(SliceType),
    Struct(StructType),
    Map(MapType),
}

impl WireTypeEnum {
    pub fn id(&self) -> TypeId {
        match *self {
            WireTypeEnum::Array(ref t) => t.common.id,
            WireTypeEnum::Slice(ref t) => t.common.id,
            WireTypeEnum::Struct(ref t) => t.common.id,
            WireTypeEnum::Map(ref t) => t.common.id,
        }
    }
}
