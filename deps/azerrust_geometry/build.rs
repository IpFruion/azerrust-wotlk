fn main() {
    println!("cargo:rerun-if-changed=src/lib.rs");
    cxx_build::bridge("src/lib.rs")
        .std("c++20")
        .compile("azerrust_geometry");
}
