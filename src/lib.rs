#[macro_use]
extern crate error_chain;
extern crate byteorder;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;

use std::io::{self, Read};
use std::mem::size_of;
use std::mem;
use std::fmt::Display;
use std::marker::PhantomData;
use std::collections::HashMap;
use std::rc::Rc;
use byteorder::ReadBytesExt;
use byteorder::BigEndian as BE;
use serde::Deserialize;

pub trait GobDecodable: Sized {
    fn decode<R: Read>(r: &mut R) -> Result<Self>;
}

impl GobDecodable for u64 {
    fn decode<R: Read>(r: &mut R) -> Result<Self> {
        let byte = r.read_i8()?;

        if byte >= 0 {
            return Ok(byte as Self);
        }

        let n_bytes = (-byte) as usize;

        if n_bytes == 0 {
            bail!(ErrorKind::NumZeroBytes);
        }

        if n_bytes > size_of::<u64>() {
            bail!(ErrorKind::NumOutOfRange);
        }

        let bytes = r.read_uint::<BE>(n_bytes)?;

        Ok(bytes as Self)
    }
}

impl GobDecodable for usize {
    fn decode<R: Read>(r: &mut R) -> Result<Self> {
        let byte = r.read_i8()?;

        if byte >= 0 {
            return Ok(byte as Self);
        }

        let n_bytes = (-byte) as usize;

        if n_bytes == 0 {
            bail!(ErrorKind::NumZeroBytes);
        }

        if n_bytes > size_of::<Self>() || n_bytes > size_of::<u64>() {
            bail!(ErrorKind::NumOutOfRange);
        }

        let bytes = r.read_uint::<BE>(n_bytes)?;

        Ok(bytes as usize)
    }
}

impl GobDecodable for isize {
    fn decode<R: Read>(r: &mut R) -> Result<Self> {
        let bytes = usize::decode(r)? as isize;
        let is_complement = bytes & 1 == 1;
        let bytes = bytes >> 1;

        if is_complement {
            Ok(!bytes)
        } else {
            Ok(bytes)
        }
    }
}

impl GobDecodable for Vec<u8> {
    fn decode<R: Read>(r: &mut R) -> Result<Self> {
        let len = usize::decode(r)?;
        // TODO: Allow setting maximum length to avoid malicious OOM
        let mut r = r.take(len as u64); // TODO: Fix cast
        let mut data = Vec::with_capacity(len);
        r.read_to_end(&mut data)?;

        Ok(data)
    }
}

impl GobDecodable for f64 {
    fn decode<R: Read>(r: &mut R) -> Result<Self> {
        unsafe {
            let float: u64 = u64::decode(r)?;
            let float: f64 = mem::transmute(float.swap_bytes());
            Ok(float)
        }
    }
}

#[derive(Default,Debug,Clone,Deserialize)]
pub struct WireType {
    #[serde(rename="ArrayT")] array_type: Option<ArrayType>,
    #[serde(rename="SliceT")] slice_type: Option<SliceType>,
    #[serde(rename="StructT")] struct_type: Option<StructType>,
    #[serde(rename="MapT")] map_type: Option<MapType>,
}

impl WireType {
    fn try_into_enum(self) -> Result<WireTypeEnum> {
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
    fn id(&self) -> TypeId {
        match *self {
            WireTypeEnum::Array(ref t) => t.common.id,
            WireTypeEnum::Slice(ref t) => t.common.id,
            WireTypeEnum::Struct(ref t) => t.common.id,
            WireTypeEnum::Map(ref t) => t.common.id,
        }
    }
}

#[derive(Debug,Clone)]
enum TypeDef {
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
    fn id(&self) -> TypeId {
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

    fn from_id(type_id: TypeId, types: &HashMap<TypeId, TypeDef>) -> Option<TypeDef>{
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

    fn deserialize_value<'de, R, V>(self, de: &mut Decoder<'de, R>, visitor: V) -> Result<V::Value>
        where R: Read,
              V: serde::de::Visitor<'de>,
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

    fn deserialize_field_name<'de, R, V>(self, de: &mut Decoder<'de, R>, visitor: V, field_id: FieldId) -> Result<V::Value>
        where R: Read,
              V: serde::de::Visitor<'de>,
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

    fn deserialize_field_value<'de, R, V>(self, de: &mut Decoder<'de, R>, visitor: V, field_id: FieldId) -> Result<V::Value>
        where R: Read,
          V: serde::de::Visitor<'de>,
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

const BOOL_ID: TypeId = 1;
const INT_ID: TypeId = 2;
const UINT_ID: TypeId = 3;
const FLOAT_ID: TypeId = 4;
const BYTE_SLICE_ID: TypeId = 5;
const STRING_ID: TypeId = 6;
const COMPLEX_ID: TypeId = 7;
const INTERFACE_ID: TypeId = 8;
const WIRE_TYPE_ID: TypeId = 16;
const ARRAY_TYPE_ID: TypeId = 17;
const COMMON_TYPE_ID: TypeId = 18;
const SLICE_TYPE_ID: TypeId = 19;
const STRUCT_TYPE_ID: TypeId = 20;
const FIELD_TYPE_ID: TypeId = 21;
const FIELD_TYPE_SLICE_ID: TypeId = 22;
const MAP_TYPE_ID: TypeId = 23;

pub type TypeId = isize;
pub type FieldId = isize;

#[derive(Debug,Clone)]
enum State {
    Start,
    DecodeFieldName(TypeDef, FieldId),
    DecodeFieldValue(TypeDef, FieldId),
    DecodeValue(TypeDef),
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

pub struct Decoder<'a, R> {
    reader: R,
    state: State,
    types: HashMap<TypeId, TypeDef>,
    _m: PhantomData<&'a ()>,
}

impl<'a, R: Read> Decoder<'a, R> {
    pub fn new(reader: R) -> Self {
        Decoder {
            reader,
            state: State::Start,
            types: HashMap::new(),
            _m: PhantomData,
        }
    }
}

impl<'de, 'a, R: Read> serde::Deserializer<'de> for &'a mut Decoder<'de, R> {
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

pub struct FieldMap<'a, 'de: 'a, R: 'a> {
    current_field: isize,
    de: &'a mut Decoder<'de, R>,
    type_def: TypeDef,
}

impl<'a, 'de: 'a, R: 'a> FieldMap<'a, 'de, R> {
    fn new(de: &'a mut Decoder<'de, R>, type_def: TypeDef) -> Self {
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
        let field_increment = usize::decode(&mut self.de.reader)?;
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

pub struct SliceDecoder<'a, 'de: 'a, R: 'a> {
    de: &'a mut Decoder<'de, R>,
    len: usize,
    current_index: usize,
    type_def: TypeDef,
}

impl<'a, 'de: 'a, R: Read + 'a> SliceDecoder<'a, 'de, R> {
    fn new(de: &'a mut Decoder<'de, R>, type_def: TypeDef) -> Result<Self> {
        let len = usize::decode(&mut de.reader)?;
        Ok(SliceDecoder {
            de,
            len,
            current_index: 0,
            type_def,
        })
    }
}

// `SeqAccess` is provided to the `Visitor` to give it the ability to iterate
// through elements of the sequence.
impl<'de, 'a, R: Read + 'a> serde::de::SeqAccess<'de> for SliceDecoder<'a, 'de, R> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
        where T: serde::de::DeserializeSeed<'de>
    {
        if self.current_index >= self.len {
            self.de.state = State::Start;
            return Ok(None);
        }

        self.current_index += 1;
        self.de.state = State::DecodeValue(self.type_def.clone());

        seed.deserialize(&mut *self.de).map(Some)
    }
}

error_chain! {
    foreign_links {
        Io(io::Error);
    }

    errors {
        NumZeroBytes {}
        NumOutOfRange {}
        InvalidField {}
        AmbiguousWireType {}
        UndefinedType(type_id: TypeId) {}
        TypeAlreadyDefined(type_id: TypeId) {}
        DefiningIdMismatch(type_id: TypeId, type_def_id: TypeId) {}
        DefiningBuiltin(type_id: TypeId) {}
    }
}

impl serde::de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        ErrorKind::Msg(msg.to_string()).into()
    }
}
