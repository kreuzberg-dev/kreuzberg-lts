// Copyright 2021, pdfium-sys Developers
// Copyright 2022 - 2024, pdfium-render Developers
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be

#[cfg(feature = "bindings")]
extern crate bindgen;

#[cfg(feature = "bindings")]
use {
    bindgen::BindgenError,
    std::env::var,
    std::ffi::OsStr,
    std::fs::{read_dir, write},
    std::path::PathBuf,
};

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
enum BuildError {
    #[cfg(feature = "bindings")]
    #[allow(dead_code)]
    IoError(std::io::Error),
    #[cfg(feature = "bindings")]
    #[allow(dead_code)]
    BindgenError(BindgenError),
    #[cfg(feature = "bindings")]
    #[allow(dead_code)]
    PathConversionError(PathBuf),
}

#[cfg(feature = "bindings")]
impl From<std::io::Error> for BuildError {
    fn from(item: std::io::Error) -> Self {
        BuildError::IoError(item)
    }
}

#[cfg(feature = "bindings")]
impl From<BindgenError> for BuildError {
    fn from(item: BindgenError) -> Self {
        BuildError::BindgenError(item)
    }
}

fn main() -> Result<(), BuildError> {
    #[cfg(feature = "bindings")]
    build_bindings()?;

    if std::env::var("PDFIUM_STATIC_LIB_PATH").is_ok() {
        println!("cargo:rustc-cfg=pdfium_use_static");
        statically_link_pdfium();
    } else if let Ok(path) = std::env::var("PDFIUM_DYNAMIC_LIB_PATH") {
        println!("cargo:rustc-link-lib=dylib=pdfium");
        println!("cargo:rustc-link-search=native={}", path);
    }

    println!("cargo:rerun-if-env-changed=PDFIUM_STATIC_LIB_PATH");
    println!("cargo:rerun-if-env-changed=PDFIUM_DYNAMIC_LIB_PATH");

    Ok(())
}

#[cfg(feature = "bindings")]
fn build_bindings() -> Result<(), BuildError> {
    for release in read_dir("include/")? {
        let release = release?.path();

        if release.is_dir() {
            build_bindings_for_one_pdfium_release(
                release
                    .file_name()
                    .ok_or(BuildError::PathConversionError(release.clone()))?
                    .to_str()
                    .ok_or(BuildError::PathConversionError(release.clone()))?,
            )?;
        }
    }

    Ok(())
}

#[cfg(feature = "bindings")]
fn build_bindings_for_one_pdfium_release(release: &str) -> Result<(), BuildError> {
    if var("DOCS_RS").is_err() {
        let header_file_extension = OsStr::new("h");
        let wrapper_file_name = OsStr::new("rust-import-wrapper.h");
        let mut included_header_files = Vec::new();

        for header in read_dir(format!("include/{}/", release))? {
            let header = header?.path();

            if header.is_file()
                && header.file_name().is_some()
                && header.extension() == Some(header_file_extension)
                && header.file_name() != Some(wrapper_file_name)
            {
                let header_file_name = header
                    .file_name()
                    .ok_or(BuildError::PathConversionError(header.clone()))?
                    .to_str()
                    .ok_or(BuildError::PathConversionError(header.clone()))?
                    .to_owned();

                included_header_files.push(header_file_name);
            }
        }

        if included_header_files.is_empty() {
            eprintln!(
                "No header files found in include/{}/; skipping bindgen, using pre-generated bindings",
                release
            );
            return Ok(());
        }

        let wrapper = included_header_files
            .iter()
            .map(|file_name| format!("#include \"{}\"", file_name))
            .collect::<Vec<_>>()
            .join("\n");

        write(format!("include/{}/rust-import-wrapper.h", release), wrapper)?;

        let bindings = bindgen::Builder::default()
            .header(format!("include/{}/rust-import-wrapper.h", release))
            .clang_arg("-DPDF_USE_SKIA")
            .clang_arg("-D_SKIA_SUPPORT_")
            .clang_arg("-DPDF_ENABLE_XFA")
            .clang_arg("-DPDF_ENABLE_V8")
            .generate_cstr(true)
            .enable_function_attribute_detection()
            .size_t_is_usize(true)
            .layout_tests(false)
            .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
            .clang_args(["-fretain-comments-from-system-headers", "-fparse-all-comments"].iter())
            .generate_comments(true);

        #[cfg(all(feature = "pdfium_use_win32", target_os = "windows"))]
        let bindings = bindings.clang_arg("-D_WIN32");

        let bindings = bindings.generate()?;
        let out_path = PathBuf::from("src");

        bindings.write_to_file(out_path.join(format!("bindgen/{}.rs", release)))?;
    }

    Ok(())
}

fn statically_link_pdfium() {
    if let Ok(path) = std::env::var("PDFIUM_STATIC_LIB_PATH") {
        println!("cargo:rustc-link-lib=static=pdfium");
        println!("cargo:rustc-link-search=native={}", path);

        let target = std::env::var("TARGET").unwrap_or_default();
        if target.contains("apple") {
            println!("cargo:rustc-link-lib=dylib=c++");
            println!("cargo:rustc-link-lib=framework=CoreGraphics");
        } else if target.contains("linux") || target.contains("gnu") || target.contains("musl") {
            if target.contains("musl") {
                println!("cargo:rustc-link-lib=static=stdc++");
            } else {
                println!("cargo:rustc-link-lib=dylib=stdc++");
            }
        }
    }
}
