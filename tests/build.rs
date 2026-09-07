use std::fs;
use std::path::PathBuf;

fn main() {
    let cases_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("cases");
    let source = fs::read_to_string(cases_dir.join("button.slint")).unwrap();
    dbg!(source);
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    dbg!(out_dir);
    panic!();
    println!("Hello, world!");
}
