fn main() {
    // let link_script = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("link_script.ld");
    // println!("cargo:rustc-link-arg=-T{}", link_script.display());

    println!("cargo:rerun-if-changed=link_script.ld");
    println!("cargo:rerun-if-changed=build.rs");
}
