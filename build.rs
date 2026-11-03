//! Records the target triple the binary is built for, so `rok update`
//! downloads the matching release archive.

fn main() {
    let target = std::env::var("TARGET").expect("cargo sets TARGET for build scripts");
    println!("cargo:rustc-env=ROK_TARGET={target}");
    println!("cargo:rerun-if-changed=build.rs");
}
