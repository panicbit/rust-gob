use super::State;
use std::io::Read;
use std::marker::PhantomData;
use std::collections::HashMap;
use std::rc::Rc;
use std::mem::{self,size_of};
use serde::Deserialize;
use byteorder::ReadBytesExt;
use byteorder::BigEndian as BE;
use serde;
use errors::*;
use types::{TypeId,TypeDef,WireType};
use types::ids::*;

pub struct Deserializer<'a, R> {
    reader: R,
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

impl<'a, R: Read> Deserializer<'a, R> {
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
