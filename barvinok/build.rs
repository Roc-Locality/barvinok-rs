use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let isl_include_dir = manifest_dir
        .parent()
        .unwrap()
        .join("barvinok-sys")
        .join("barvinok")
        .join("isl")
        .join("include")
        .join("isl");
    let barvinok_include_dir = manifest_dir
        .parent()
        .unwrap()
        .join("barvinok-sys")
        .join("barvinok")
        .join("barvinok");
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    println!("cargo:rerun-if-changed={}", isl_include_dir.display());
    println!("cargo:rerun-if-changed={}", barvinok_include_dir.display());
    println!(
        "cargo:rerun-if-changed={}",
        manifest_dir.join("src").display()
    );

    let header_roots = vec![isl_include_dir, barvinok_include_dir];
    if let Err(err) = barvinok_gen::generate(&manifest_dir, &header_roots, &out_dir) {
        panic!("failed to generate API bindings: {err}");
    }
}
