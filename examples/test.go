package main

import (
    "fmt"
    "os"
    "encoding/gob"
)

func main() {
    file, err := os.Create("/tmp/out.bin")
    if err != nil {
        fmt.Println(err)
        return
    }

    enc := gob.NewEncoder(file)
    err = enc.Encode(Example {
        Bool: true,
        Int: 42,
        Uint: 777,
        Float: 3.14159265,
        Bytes: []byte {1,2,3},
        String: "Hello gophers!",
        // Complex: complex(42.0, 777.0),
        Nested: Point {
            X: 42,
            Y: 7777,
        },
    })

    if err != nil {
        fmt.Println(err)
        return
    }
}

type Example struct {
    Bool bool
    Int int
    Uint uint
    Float float64
    Bytes []byte
    String string
    // Complex complex64
    // Interface interface{}
    // Map map[int]string
    Nested Point
}

type Point struct {
    X int
    Y int
}
