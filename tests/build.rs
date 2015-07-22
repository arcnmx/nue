extern crate syntex;
extern crate nue_codegen;

use std::env;
use std::path::Path;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();

    for &(src, dst) in &[
        ("../macros/tests/code.rs", "code.rs"),
    ] {
        let src = Path::new(src);
        let dst = Path::new(&out_dir).join(dst);

        let mut registry = syntex::Registry::new();

        nue_codegen::register(&mut registry);
        registry.expand("", &src, &dst).unwrap();
    }
}
