
macro_rules! test {
    ($name:ident, $go_typ:ident, $go_value:expr, $typ:ty, $value:expr) => {
        de_test! {
            $name

            go_decls concat!("
                type Value struct {
                    V ", stringify!($go_typ) ,"
                }
            "),

            go_value Value format!("
                return Value {{
                    V: {},
                }}
            ", $go_value),

            decls {
                #[derive(Deserialize,Default)]
                #[serde(default)]
                struct Value {
                    V: $typ,
                }
            },

            validate v: Value {
                assert_eq!(v.V, $value);
            }
        }
    }
}

macro_rules! fail {
    ($name:ident, $go_typ:ident, $go_value:expr, $typ:ty, $value:expr) => {
        de_test! {
            $name

            go_decls concat!("
                type Value struct {
                    V ", stringify!($go_typ) ,"
                }
            "),

            go_value Value format!("
                return Value {{
                    V: {},
                }}
            ", $go_value),

            decls {
                #[derive(Deserialize,Default)]
                #[serde(default)]
                struct Value {
                    #[allow(dead_code)]
                    V: $typ,
                }
            },

            validate _v: Value {
                panic!("failed");
            }
        }
    }
}

test!(bool, bool, "true", bool , true);

mod uint {
    mod max {
        mod unsigned {
            test!(u8   , uint64, u8::max_value()   , u8   , u8::max_value());
            test!(u16  , uint64, u16::max_value()  , u16  , u16::max_value());
            test!(u32  , uint64, u32::max_value()  , u32  , u32::max_value());
            test!(u64  , uint64, u64::max_value()  , u64  , u64::max_value());
            test!(usize, uint  , usize::max_value(), usize, usize::max_value());
        }

        mod signed {
            test!(i8   , uint64, i8::max_value()   , i8   , i8::max_value());
            test!(i16  , uint64, i16::max_value()  , i16  , i16::max_value());
            test!(i32  , uint64, i32::max_value()  , i32  , i32::max_value());
            test!(i64  , uint64, i64::max_value()  , i64  , i64::max_value());
            test!(isize, uint  , isize::max_value(), isize, isize::max_value());
        }
    }
}

mod int {
    mod max {
        mod unsigned {
            test!(u8   , int64, u8::max_value()   , u8   , u8::max_value());
            test!(u16  , int64, u16::max_value()  , u16  , u16::max_value());
            test!(u32  , int64, u32::max_value()  , u32  , u32::max_value());
            test!(u64  , int64, u64::max_value()  , u64  , u64::max_value());
            test!(usize, int  , usize::max_value(), usize, usize::max_value());
        }

        mod signed {
            test!(i8   , int64, i8::max_value()   , i8   , i8::max_value());
            test!(i16  , int64, i16::max_value()  , i16  , i16::max_value());
            test!(i32  , int64, i32::max_value()  , i32  , i32::max_value());
            test!(i64  , int64, i64::max_value()  , i64  , i64::max_value());
            test!(isize, int  , isize::max_value(), isize, isize::max_value());
        }
    }
}

// test!()