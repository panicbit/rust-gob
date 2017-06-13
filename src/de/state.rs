use types::{TypeDef,FieldId};

#[derive(Debug,Clone)]
pub enum State {
    Start,
    DecodeFieldName(TypeDef, FieldId),
    DecodeFieldValue(TypeDef, FieldId),
    DecodeValue(TypeDef),
}
