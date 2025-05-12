use std::path::PathBuf;

fn main() {
    use autotools::Config;

    let mut build = Config::new("barvinok");
    let mut additional_include_dir = Vec::new();
    if cfg!(target_os = "macos") {
        // GMP
        let gmp_prefix =
            std::env::var("GMP_PREFIX").unwrap_or_else(|_| "/usr/local/opt/gmp".to_string());
        println!("cargo:rustc-link-search=native={gmp_prefix}/lib");
        additional_include_dir.push(format!("-I{gmp_prefix}/include"));
        build.config_option("with-gmp-prefix", Some(&gmp_prefix));

        // NTL
        let ntl_prefix = std::env::var("NTL_PREFIX").unwrap_or_else(|_| "/usr/local".to_string());
        println!("cargo:rustc-link-search=native={ntl_prefix}/lib");
        additional_include_dir.push(format!("-I{ntl_prefix}/include"));
        build.config_option("with-ntl-prefix", Some(&ntl_prefix));

        // use <src dir>/misc/cc-wrapper for clang to workaround bug in barvinok's autotools
        let src_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let src_dir = PathBuf::from(src_dir);
        let cc_wrapper = src_dir.join("misc").join("cc-wrapper");
        let cxx_wrapper = src_dir.join("misc").join("cxx-wrapper");
        build.env("CC", cc_wrapper);
        build.env("CXX", cxx_wrapper);
    }

    let dst = build.reconf("-ivf").build();

    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-lib=static=barvinok");
    println!("cargo:rustc-link-lib=static=isl");
    println!("cargo:rustc-link-lib=static=polylibgmp");
    println!("cargo:rustc-link-lib=dylib=gmp");
    println!("cargo:rustc-link-lib=dylib=ntl");
    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-lib=dylib=c++");
    } else {
        println!("cargo:rustc-link-lib=dylib=stdc++");
    }
    println!("cargo:rerun-if-changed=build.rs");
    let include_dir = dst.join("include");

    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header(format!("{}/include/barvinok/barvinok.h", dst.display()))
        .header(format!("{}/include/isl/val.h", dst.display()))
        .header(format!("{}/include/isl/space.h", dst.display()))
        .header(format!("{}/include/isl/id.h", dst.display()))
        .header(format!("{}/include/isl/set.h", dst.display()))
        .header(format!("{}/include/isl/map.h", dst.display()))
        .header(format!("{}/include/isl/options.h", dst.display()))
        .header(format!("{}/include/isl/vec.h", dst.display()))
        .header(format!("{}/include/isl/mat.h", dst.display()))
        .header(format!("{}/include/isl/aff.h", dst.display()))
        .header(format!("{}/include/isl/local_space.h", dst.display()))
        .clang_arg(format!("-I{}", include_dir.display()))
        .clang_args(additional_include_dir)
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
