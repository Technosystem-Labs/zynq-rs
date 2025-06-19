fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    compile_memcpy();
}

fn compile_memcpy() {
    use std::path::Path;
    extern crate cc;
    let cfg = &mut cc::Build::new();
    cfg.compiler("clang");
    cfg.no_default_flags(true);
    cfg.warnings(false);
    cfg.flag("--target=armv7-none-eabihf");
    let sources = vec!["memcpy.S"];
    let root = Path::new("src/asm");
    for src in sources {
        println!("cargo:rerun-if-changed={}", src);
        cfg.file(root.join(src));
    }
    cfg.compile("memcpy");
}
