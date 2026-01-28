fn main() {
    println!("cargo:rerun-if-changed=link_script.ld");
    println!("cargo:rerun-if-changed=build.rs");
}
