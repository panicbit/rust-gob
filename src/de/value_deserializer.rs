struct ValueDeserializer<'a, R> {
    reader: R
    types: &'a HashMap<TypeId, TypeDef>,
}

impl ValueDeserializer<R> {
    fn new(reader: R, types: &HashMap<TypeId, TypeDef>) -> Self {
        ValueDeserializer { reader, types }
    }
}

impl<'a, 'de, R: Read> serde::Deserializer<'de> for &'a mut ValueDeserializer<R> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
        where V: serde::de::Visitor<'de>
    {
        unimplemented!()
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