fn main() {
    let target = std::env::var("TARGET").unwrap();

    if target.contains("darwin") {
        println!("cargo:rustc-link-arg=-Wl,-undefined,dynamic_lookup");
        println!("cargo:rustc-link-arg=-Wl,-rpath,@loader_path");
    } else if target.contains("linux") {
        println!("cargo:rustc-link-arg=-Wl,-rpath,$ORIGIN");
    } else if target.contains("windows") {
    }

    println!("cargo:rerun-if-changed=build.rs");
}
