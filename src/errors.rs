use std::io;
use std::fmt;
use types::TypeId;

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

impl ::serde::de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        ErrorKind::Msg(msg.to_string()).into()
    }
}
