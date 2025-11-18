use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=cpp/rapidgzip_c_wrapper.cpp");
    println!("cargo:rerun-if-changed=cpp/rapidgzip_c_wrapper.hpp");
    println!("cargo:rerun-if-changed=vendor/");

    let mut build = cc::Build::new();

    // Set C++17 standard (required by rapidgzip)
    build
        .cpp(true)
        .std("c++17")
        .flag_if_supported("-O3")
        .flag_if_supported("-Wall")
        .flag_if_supported("-Wextra");

    // Add our C wrapper
    build.file("cpp/rapidgzip_c_wrapper.cpp");

    // Add include paths
    let vendor_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("vendor");

    // Once we have the rapidgzip source, add its include path:
    // build.include(vendor_dir.join("indexed_bzip2/src/rapidgzip"));
    // build.include(vendor_dir.join("indexed_bzip2/src"));

    // For now, just add the vendor directory
    build.include(&vendor_dir);

    // Compile the library
    build.compile("rapidgzip_wrapper");

    // Link with C++ standard library
    let target = env::var("TARGET").unwrap();
    if target.contains("apple") {
        println!("cargo:rustc-link-lib=c++");
    } else if target.contains("linux") {
        println!("cargo:rustc-link-lib=stdc++");
    }

    // Once we integrate rapidgzip, we may need to link additional libraries:
    // - zlib (for gzip decompression)
    // - isal (Intel Storage Acceleration Library, optional but recommended)
    // - rpmalloc (optional, for better performance)
    //
    // Example:
    // println!("cargo:rustc-link-lib=z");
    // println!("cargo:rustc-link-lib=isal");
}
