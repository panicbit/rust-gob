use std::io::Read;
use std::collections::HashMap;
use std::rc::Rc;
use std::mem::{self,size_of};
use byteorder::ReadBytesExt;
use byteorder::BigEndian as BE;
use serde::{self,Deserialize};
use serde::de::Visitor;
use errors::*;
use types::{TypeId,FieldId,TypeDef,WireType,WireTypeEnum};
use types::ids::*;
use super::{State,FieldMap,SliceDecoder};

pub struct Deserializer<R> {
    reader: R,
    pub(crate) state: State,
    types: HashMap<TypeId, TypeDef>,
}

impl<R: Read> Deserializer<R> {
    pub fn new(reader: R) -> Self {
        Deserializer {
            reader,
            state: State::Start,
            types: HashMap::new(),
        }
    }
}

impl<R: Read> Deserializer<R> {
    pub fn read_u64(&mut self) -> Result<u64> {
        let byte = self.reader.read_i8()?;

        if byte >= 0 {
            return Ok(byte as u64);
        }

        let n_bytes = (-byte) as usize;

        if n_bytes == 0 {
            bail!(ErrorKind::NumZeroBytes);
        }

        if n_bytes > size_of::<u64>() {
            bail!(ErrorKind::NumOutOfRange);
        }

        let bytes = self.reader.read_uint::<BE>(n_bytes)?;

        Ok(bytes as u64)
    }

    pub fn read_usize(&mut self) -> Result<usize> {
        let byte = self.reader.read_i8()?;

        if byte >= 0 {
            return Ok(byte as usize);
        }

        let n_bytes = (-byte) as usize;

        if n_bytes == 0 {
            bail!(ErrorKind::NumZeroBytes);
        }

        if n_bytes > size_of::<usize>() || n_bytes > size_of::<u64>() {
            bail!(ErrorKind::NumOutOfRange);
        }

        let bytes = self.reader.read_uint::<BE>(n_bytes)?;

        Ok(bytes as usize)
    }

    pub fn read_i64(&mut self) -> Result<i64> {
        let bytes = self.read_u64()? as i64;
        let is_complement = bytes & 1 == 1;
        let bytes = bytes >> 1;

        if is_complement {
            Ok(!bytes)
        } else {
            Ok(bytes)
        }
    }

    pub fn read_isize(&mut self) -> Result<isize> {
        let bytes = self.read_usize()? as isize;
        let is_complement = bytes & 1 == 1;
        let bytes = bytes >> 1;

        if is_complement {
            Ok(!bytes)
        } else {
            Ok(bytes)
        }
    }

    pub fn read_bytes(&mut self) -> Result<Vec<u8>> {
        let len = self.read_usize()?;
        // TODO: Allow setting maximum length to avoid malicious OOM
        let mut r = self.reader.by_ref().take(len as u64); // TODO: Fix cast
        let mut data = Vec::with_capacity(len);
        r.read_to_end(&mut data)?;

        Ok(data)
    }

    pub fn read_f64(&mut self) -> Result<f64> {
        unsafe {
            let float: u64 = self.read_u64()?;
            let float: f64 = mem::transmute(float.swap_bytes());
            Ok(float)
        }
    }

    pub fn read_bool(&mut self) -> Result<bool> {
        self.read_usize().map(|b| b != 0)
    }

    pub fn read_type_id(&mut self) -> Result<TypeId> {
        self.read_i64()
    }
}

impl<'de, R: Read> Deserializer<R> {
    pub fn deserialize_value<V>(&mut self, visitor: V, type_def: TypeDef) -> Result<V::Value>
        where V: Visitor<'de>
    {
        match type_def.clone() {
            TypeDef::String
            | TypeDef::Interface
            | TypeDef::ArrayType
            | TypeDef::Complex
            | TypeDef::SliceType
            | TypeDef::FieldTypeSlice
            | TypeDef::MapType
            | TypeDef::StructType => bail!("Decoding for {:?} not implemented", type_def),
            TypeDef::ByteSlice => visitor.visit_seq(SliceDecoder::new(self, TypeDef::Uint)?),
            TypeDef::Bool => visitor.visit_bool(self.read_bool()?),
            TypeDef::Uint => visitor.visit_u64(self.read_u64()?),
            TypeDef::Float => visitor.visit_f64(self.read_f64()?),
            TypeDef::Int => visitor.visit_i64(self.read_i64()?),
            TypeDef::CommonType => visitor.visit_map(FieldMap::new(self, TypeDef::CommonType)),
            TypeDef::WireType => visitor.visit_map(FieldMap::new(self, TypeDef::WireType)),
            TypeDef::FieldType => visitor.visit_map(FieldMap::new(self, TypeDef::FieldType)),
            TypeDef::Custom(wire_type) => match *wire_type {
                WireTypeEnum::Struct(_) => visitor.visit_map(FieldMap::new(self, type_def)),
                _ => bail!("Decoding for {} not implemented")
            }
        }
    }

    pub fn deserialize_field_name<V>(&mut self, visitor: V, type_def: TypeDef, field_id: FieldId) -> Result<V::Value>
        where V: Visitor<'de>
    {
        let field_name = match type_def {
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
            | TypeDef::SliceType => bail!("Field name for {} not implemented", type_def.id()),
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

        trace!("{}", field_name);
        visitor.visit_str(&field_name)
    }

    pub fn deserialize_field_value<V>(&mut self, visitor: V, type_def: TypeDef, field_id: FieldId) -> Result<V::Value>
        where V: Visitor<'de>
    {
        match type_def {
            TypeDef::WireType => match field_id {
                2 => visitor.visit_map(FieldMap::new(self, TypeDef::StructType)),
                3 => self.deserialize_value(visitor, TypeDef::MapType),
                _ => bail!("wireType field {} unimplemented", field_id),
            },
            TypeDef::StructType => match field_id {
                0 => self.deserialize_value(visitor, TypeDef::CommonType),
                1 => visitor.visit_seq(SliceDecoder::new(self, TypeDef::FieldType)?),
                _ => bail!("structType field {} unimplemented", field_id),
            },
            TypeDef::FieldType => match field_id {
                0 => visitor.visit_byte_buf(self.read_bytes()?),
                1 => visitor.visit_i64(self.read_type_id()?),
                _ => bail!("fieldType field {} unimplemented", field_id),
            },
            TypeDef::CommonType => match field_id {
                0 => visitor.visit_byte_buf(self.read_bytes()?),
                1 => visitor.visit_i64(self.read_type_id()?),
                _ => bail!("commonType field {} unimplemented", field_id)
            },
            TypeDef::Custom(ref wire_type) => match **wire_type {
                WireTypeEnum::Struct(ref t) => {
                    let type_id = t.fields().get(field_id as usize).map(|field| field.id()).ok_or(ErrorKind::InvalidField)?;
                    let type_def = TypeDef::from_id(type_id, &self.types).ok_or(ErrorKind::UndefinedType(type_id))?;
                    self.deserialize_value(visitor, type_def)
                },
                _ => bail!("Decoding of field name for {} not implemented")
            },
            _ => bail!("Decoding of field value for id {} not implemented", type_def.id())
        }
    }
}

impl<'a, 'de, R: Read> serde::Deserializer<'de> for &'a mut Deserializer<R> {
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
                    len = self.read_usize()?;
                    type_id = self.read_type_id()?;

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
            State::DecodeValue(type_def) => self.deserialize_value(visitor, type_def),
            State::DecodeFieldName(type_def, field_id) => self.deserialize_field_name(visitor, type_def, field_id),
            State::DecodeFieldValue(type_def, field_id) => self.deserialize_field_value(visitor, type_def, field_id),
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
