fn main() {
    cc::Build::new()
        .file("src/shim.c")
        .define("FASTLED_SCALE8_FIXED", "1")
        .warnings(true)
        .compile("fastled_ref_shim");

    println!("cargo:rerun-if-changed=src/shim.c");
}
