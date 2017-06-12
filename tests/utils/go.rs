extern crate tempdir;

use std::process::{Command,Output};
use self::tempdir::TempDir;
use std::fs::File;
use std::io::Write;

pub fn run(code: &str) -> Output {
    let dir = TempDir::new("rust_gob_tests").unwrap();
    let path = dir.as_ref().join("test.go");
    let mut file = File::create(&path).unwrap();
    
    file.write(code.as_bytes()).unwrap();
    drop(file);

    Command::new("go")
        .arg("run")
        .arg(path)
        .output()
        .unwrap()
}
