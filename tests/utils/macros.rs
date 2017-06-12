#[macro_export]
macro_rules! de_test {
    ($test_name:ident
        go_decls $go_decls:expr,
        go_value $go_typ:ident $go_value:expr,
        decls { $($decls:item)* },
        validate $val_name:ident : $typ:ident { $($val_exprs:tt)* }
    ) => {
        #[test]
        #[allow(non_snake_case)]
        fn $test_name() {
            #[warn(non_snake_case)]
            {
                let code = format!(r#"
                    package main

                    import (
                        "fmt"
                        "os"
                        "encoding/gob"
                    )

                    {decls}

                    func main() {{
                        enc := gob.NewEncoder(os.Stdout)
                        err := enc.Encode(value())

                        if err != nil {{
                            fmt.Fprintf(os.Stderr, "%v", err)
                            return
                        }}
                    }}

                    func value() {typ} {{
                        {value}
                    }}
                "#,
                    typ = stringify!($go_typ),
                    value = $go_value,
                    decls = $go_decls,
                );

                let output = ::utils::go::run(&code);
                let stderr = ::std::string::String::from_utf8(output.stderr).unwrap();
                let mut stdout = output.stdout.as_slice();

                // panic!("code: {}", code);
                if !stderr.is_empty() {
                    panic!("{}", stderr);
                }

                let mut deserializer = ::gob::Decoder::new(&mut stdout);

                $(#[allow(non_snake_case)] $decls)*

                let $val_name: $typ = ::serde::Deserialize::deserialize(&mut deserializer).unwrap();

                $($val_exprs)*;
            }
        }
    }
}
