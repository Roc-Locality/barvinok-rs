use std::path::PathBuf;

fn main() {
    use autotools::Config;

    let dst = Config::new("barvinok").reconf("-ivf").build();

    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-lib=static=barvinok");
    println!("cargo:rustc-link-lib=static=isl");
    println!("cargo:rustc-link-lib=dylib=gmp");
    println!("cargo:rustc-link-lib=dylib=ntl");
    println!("cargo:rerun-if-changed=build.rs");
    let include_dir = dst.join("include");
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header(format!("{}/include/barvinok/barvinok.h", dst.display()))
        .header(format!("{}/include/isl/val.h", dst.display()))
        .clang_arg(format!("-I{}", include_dir.display()))
        // allow only those functions starts with barvinok and isl and recursively
        .allowlist_function("isl.*")
        .allowlist_function("barvinok.*")
        .allowlist_recursively(true)
        // use core
        .use_core()
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
