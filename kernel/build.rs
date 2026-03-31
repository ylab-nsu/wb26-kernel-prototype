use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let workspace_root = manifest_dir.parent().unwrap();
    let target = env::var("TARGET").unwrap();

    let link_script = manifest_dir.join("link_script.ld");
    println!("cargo:rustc-link-arg=-T{}", link_script.display());
    println!("cargo:rustc-link-arg=--allow-multiple-definition");
    println!("cargo:rerun-if-changed=link_script.ld");

    let user_target_dir = &workspace_root.join("target/user-programs");
    let mut user_cargo_args = vec![
        "build",
        "--package", "user-processes",
        "--target", &target,
    ];
    match env::var("PROFILE").as_deref() {
        Ok("debug") => {}
        Ok("release") => {user_cargo_args.push("--release");}
        Ok(p) => {panic!("Unknown profile: {p}")}
        Err(e) => {panic!("{e}")}
    }

    let status = Command::new("cargo")
        .args(user_cargo_args)
        .env("CARGO_TARGET_DIR", &user_target_dir)
        .status();

    if status.is_err() || !status.unwrap().success() {
        panic!("Failed to build user-processes module");
    }

    let workspace_root = manifest_dir.parent().unwrap();
    let app_elf = workspace_root
        .join(user_target_dir)
        .join(&target)
        .join("debug/libuser.a");

    println!("cargo:rustc-link-arg={}", app_elf.display());
    println!("cargo:rerun-if-changed=../user-processes", );
    println!("cargo:rerun-if-changed=build.rs");
}
