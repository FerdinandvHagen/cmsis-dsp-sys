use bindgen::builder;
use reqwest::blocking;
use std::env;
use std::fs::File;
use std::path::{Path, PathBuf};

#[derive(Debug)]
enum Endian {
    Big,
    Little,
}

#[derive(Debug, PartialEq)]
enum Core {
    NotM7,
    M7,
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=wrapper.h");

    let manifest = env::var("CARGO_MANIFEST_DIR").unwrap();
    let outdir = PathBuf::from(env::var("OUT_DIR").unwrap());

    let cmsis_filepath = outdir.join("CMSIS.zip");
    {
        let mut file = File::create(cmsis_filepath.clone()).unwrap();
        blocking::get(
            "https://github.com/ARM-software/CMSIS_5/releases/download/5.9.0/ARM.CMSIS.5.9.0.pack",
        )
            .unwrap()
            .copy_to(&mut file)
            .unwrap();
    }

    let dsp_filepath = outdir.join("CMSIS-DSP.zip");
    {
        let mut file = File::create(dsp_filepath.clone()).unwrap();
        blocking::get(
            "https://github.com/ARM-software/CMSIS-DSP/releases/download/v1.10.1/ARM.CMSIS-DSP.1.10.1.pack",
        )
            .unwrap()
            .copy_to(&mut file)
            .unwrap();
    }

    let file = File::open(cmsis_filepath).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();
    archive.extract(outdir.join("CMSIS")).unwrap();

    let file = File::open(dsp_filepath).unwrap();
    let mut archive = zip::ZipArchive::new(file).unwrap();
    archive.extract(outdir.join("CMSIS_DSP")).unwrap();

    let manifest_dir = Path::new(&manifest);

    // Copy over the CMakeLists.txt file
    let input_path = manifest_dir.join("CMakeLists.txt");
    let output_path = outdir.join("CMSIS_DSP").join("CMakeLists.txt");
    let _ = std::fs::copy(input_path, output_path).unwrap();

    let dst = cmake::Config::new(outdir.join("CMSIS_DSP"))
        .env("CMSIS_PATH", outdir.join("CMSIS").to_str().unwrap())
        .always_configure(true)
        .no_build_target(true)
        .build();

    println!(
        "cargo:rustc-link-search=native={}",
        dst.join("build").join("bin_dsp").display()
    );
    println!("cargo:rustc-link-lib=static=CMSISDSP");

    let bb = builder()
        .header("wrapper.h")
        .derive_default(false)
        .ctypes_prefix("cty")
        .use_core()
        .generate_comments(true)
        .rustfmt_bindings(true)
        .clang_arg(format!("-I{}", manifest_dir.join("include").display()))
        .clang_arg(format!(
            "-I{}",
            outdir.join("CMSIS/CMSIS/Core/Include").display()
        ))
        .clang_arg(format!(
            "-I{}",
            outdir.join("CMSIS/CMSIS/DSP/Include").display()
        ))
        .clang_arg("-nostdinc");

    let cmd = bb.command_line_flags().join(" ");
    eprintln!("{:?}", cmd);

    let bindings = bb.generate().expect("Unable to generate bindings");
    bindings
        .write_to_file(outdir.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
