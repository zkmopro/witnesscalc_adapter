pub use paste;
pub use serde_json;
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

pub mod convert_type;
pub use convert_type::*;

#[doc(hidden)]
pub mod __macro_deps {
    pub use anyhow;
}

/// Macro to generate a witness for a given circuit
#[macro_export]
macro_rules! witness {
    ($x: ident) => {
        $crate::paste::paste! {
            #[allow(non_upper_case_globals)]
            const [<$x _CIRCUIT_DATA>]: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/witnesscalc/src/", stringify!($x), ".dat"));
            #[link(name = "witnesscalc_" [<$x>], kind = "static")]
            extern "C" {
                fn [<witnesscalc_ $x>](
                    circuit_buffer: *const std::os::raw::c_char,
                    circuit_size: std::ffi::c_ulong,
                    json_buffer: *const std::os::raw::c_char,
                    json_size: std::ffi::c_ulong,
                    wtns_buffer: *mut std::os::raw::c_char,
                    wtns_size: *mut std::ffi::c_ulong,
                    error_msg: *mut std::os::raw::c_char,
                    error_msg_maxsize: std::ffi::c_ulong,
                ) -> std::ffi::c_int;
            }
        }
        $crate::paste::item! {
            pub fn [<$x _witness>](json_input: &str) -> $crate::__macro_deps::anyhow::Result<Vec<u8>> {
                println!("Generating witness for circuit {}", stringify!($x));
                unsafe {
                    let json_input = std::ffi::CString::new(json_input).map_err(|e| $crate::__macro_deps::anyhow::anyhow!("Failed to convert JSON input to CString: {}", e))?;
                    let json_size = json_input.as_bytes().len() as std::ffi::c_ulong;

                    let circuit_buffer = [<$x _CIRCUIT_DATA>].as_ptr() as *const std::ffi::c_char;
                    let circuit_size = [<$x _CIRCUIT_DATA>].len() as std::ffi::c_ulong;

                    //TODO dynamically allocate the buffer?
                    let mut wtns_buffer = vec![0u8; 100 * 1024 * 1024]; // 8 MB buffer
                    let mut wtns_size: std::ffi::c_ulong = wtns_buffer.len() as std::ffi::c_ulong;

                    let mut error_msg = vec![0u8; 256]; // Error message buffer
                    let error_msg_ptr = error_msg.as_mut_ptr() as *mut std::ffi::c_char;

                    let result =  [<witnesscalc_ $x>](
                        circuit_buffer,
                        circuit_size,
                        json_input.as_ptr(),
                        json_size,
                        wtns_buffer.as_mut_ptr() as *mut _,
                        &mut wtns_size as *mut _,
                        error_msg.as_mut_ptr() as *mut _,
                        error_msg.len() as u64,
                    );

                    if result != 0 {
                        let error_string = std::ffi::CStr::from_ptr(error_msg_ptr)
                            .to_string_lossy()
                            .into_owned();
                        return Err($crate::__macro_deps::anyhow::anyhow!("Proof generation failed: {}", error_string));
                    }

                    let wtns_buffer = &wtns_buffer[..wtns_size as usize];
                    Ok(wtns_buffer.to_vec())
                }
            }
        }
    };
}

const WITNESSCALC_BUILD_SCRIPT: &str = include_str!("../clone_witnesscalc.sh");

pub fn build_and_link(circuits_dir: &str) {
    let target = env::var("TARGET").expect("Cargo did not provide the TARGET environment variable");
    if target.contains("android") {
        let android_ndk = env::var("ANDROID_NDK").expect("ANDROID_NDK not set");
        if android_ndk.is_empty() {
            panic!("ANDROID_NDK must be non-empty");
        }
    }

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let lib_dir = Path::new(&out_dir)
        .join("witnesscalc")
        .join("package")
        .join("lib");

    if !Path::is_dir(Path::new(circuits_dir)) {
        panic!("circuits_dir must be a directory");
    }
    println!("cargo:rerun-if-changed={}", circuits_dir);

    let witnesscalc_path = Path::new(&out_dir).join(Path::new("witnesscalc"));
    // If the witnesscalc repo is not cloned, clone it
    if !witnesscalc_path.exists() {
        let witnesscalc_script_path = Path::new(&out_dir).join(Path::new("clone_witnesscalc.sh"));
        fs::write(&witnesscalc_script_path, WITNESSCALC_BUILD_SCRIPT)
            .expect("Failed to write build script");
        Command::new("sh")
            .arg(witnesscalc_script_path.to_str().unwrap())
            .spawn()
            .expect("Failed to spawn witnesscalc build")
            .wait()
            .expect("witnesscalc build errored");
    }

    println!("Detected target: {}", target);
    //For possible options see witnesscalc/build_gmp.sh
    let gmp_build_target = match target.as_str() {
        "aarch64-apple-ios" => "ios",
        "aarch64-apple-ios-sim" => "ios_simulator",
        "x86_64-apple-ios" => "ios_simulator",
        "x86_64-linux-android" => "android_x86_64",
        "i686-linux-android" => "android_x86_64",
        "armv7-linux-androideabi" => "android",
        "aarch64-linux-android" => "android",
        "aarch64-apple-darwin" => "host", //Use "host" for M Macs, macos_arm64 would fail the subsequent build
        _ => "host",
    };

    let gmp_lib_folder = match target.as_str() {
        "aarch64-apple-ios" => "package_ios_arm64",
        "aarch64-apple-ios-sim" => "package_iphone_simulator_arm64",
        "x86_64-apple-ios" => "package_iphone_simulator_x86_64",
        "x86_64-linux-android" => "package_android_x86_64",
        "i686-linux-android" => "package_android_x86_64",
        "armv7-linux-androideabi" => "package_android_arm64",
        "aarch64-linux-android" => "package_android_arm64",
        _ => "package",
    };
    //For possible options see witnesscalc/Makefile
    let witnesscalc_build_target = match target.as_str() {
        "aarch64-apple-ios" => "ios",
        "aarch64-apple-ios-sim" => "ios_simulator_arm64",
        "x86_64-apple-ios" => "ios_simulator_x86_64",
        "x86_64-linux-android" => "android_x86_64",
        "i686-linux-android" => "android_x86_64",
        "armv7-linux-androideabi" => "android",
        "aarch64-linux-android" => "android",
        "aarch64-apple-darwin" => "arm64_host",
        _ => "host",
    };

    // If the witnesscalc library is not built, build it
    let gmp_dir = witnesscalc_path.join("depends").join("gmp");
    let target_dir = gmp_dir.join(gmp_lib_folder);
    if !target_dir.exists() {
        Command::new("bash")
            .current_dir(&witnesscalc_path)
            .arg("./build_gmp.sh")
            .arg(gmp_build_target)
            .spawn()
            .expect("Failed to spawn build_gmp.sh")
            .wait()
            .expect("build_gmp.sh errored");
    }

    //find all the .cpp files in the circuits_dir
    let circuit_files = fs::read_dir(circuits_dir)
        .expect("Failed to read circuits directory")
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some() && path.extension().unwrap() == "cpp")
        .collect::<Vec<_>>();

    // Copy each circuit .cpp and .dat into witnesscalc/src, replacing any existing files
    circuit_files.iter().for_each(|path| {
        let circuit_name = path.file_stem().unwrap().to_str().unwrap();
        let circuit_dat = path.with_extension("dat");
        let circuit_dat_name = circuit_dat.file_name().unwrap().to_str().unwrap();
        let circuit_dat_dest = witnesscalc_path.join("src").join(circuit_dat_name);
        fs::copy(&circuit_dat, &circuit_dat_dest).expect("Failed to copy circuit .dat file");
        //For each .cpp file, do the following: find the last include statement (should be #include "calcwit.hpp") and insert the following on the next line: namespace CIRCUIT_NAME {. Then, insert the closing } at the end of the file:
        let circuit_cpp = fs::read_to_string(path).expect("Failed to read circuit .cpp file");
        let circuit_cpp = circuit_cpp.replace(
            "#include \"calcwit.hpp\"",
            "#include \"calcwit.hpp\"\nnamespace CIRCUIT_NAME {",
        );
        let circuit_cpp = circuit_cpp + "\n}";
        let circuit_cpp_name = witnesscalc_path.join("src").join(circuit_name);
        let circuit_cpp_dest = circuit_cpp_name.with_extension("cpp");
        fs::write(&circuit_cpp_dest, circuit_cpp).expect("Failed to write circuit .cpp file");

        //Find a witnesscalc_template.cpp template file in the src. Replace all the @CIRCUIT_NAME@ inside it with the circuit name and write it to the src directory, replacing "template" in the name with the circuit name
        let template_path = witnesscalc_path
            .join("src")
            .join("witnesscalc_template.cpp");
        let template = fs::read_to_string(&template_path).expect("Failed to read template file");
        let template = template.replace("@CIRCUIT_NAME@", circuit_name);
        let template_dest = witnesscalc_path
            .join("src")
            .join(format!("witnesscalc_{}.cpp", circuit_name));
        fs::write(&template_dest, template).expect("Failed to write the templated .cpp file");
        //Find a witnesscalc_template.h template file in the src. Replace all the @CIRCUIT_NAME@ inside it with the circuit name, @CIRCUIT_NAME_CAPS@ with the capitalized name, and write it to the src directory, replacing "template" in the name with the circuit name
        let template_path = witnesscalc_path.join("src").join("witnesscalc_template.h");
        let template = fs::read_to_string(&template_path).expect("Failed to read template file");
        let template = template
            .replace("@CIRCUIT_NAME@", circuit_name)
            .replace("@CIRCUIT_NAME_CAPS@", &circuit_name.to_uppercase());
        let template_dest = witnesscalc_path
            .join("src")
            .join(format!("witnesscalc_{}.h", circuit_name));
        fs::write(&template_dest, template).expect("Failed to write the templated .h file");
    });

    //the circuit name list would look like "circuit1;circuit2;circuit3"
    let circuit_names = circuit_files
        .iter()
        .map(|path| path.file_stem().unwrap().to_str().unwrap())
        .collect::<Vec<_>>();

    let circuit_names_semicolon = circuit_names.join(";");

    Command::new("make")
        .env("CIRCUIT_NAMES", circuit_names_semicolon)
        .arg(witnesscalc_build_target)
        .current_dir(&witnesscalc_path)
        .spawn()
        .expect("Failed to spawn make arm64_host")
        .wait()
        .expect("make arm64_host errored");

    // Link the witnesscalc library for the circuit
    circuit_names.iter().for_each(|circuit_name| {
        println!("cargo:rustc-link-lib=static=witnesscalc_{}", circuit_name);
    });

    // Link the C++ standard library. This is necessary for Rust tests to run on the host,
    // non-host targets may require a specific way of linking (e.g., through linking flags in xcode)
    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-lib=c++"); // macOS default
    }
    #[cfg(not(target_os = "macos"))]
    {
        println!("cargo:rustc-link-lib=stdc++"); // Linux or other platforms
    }
    // Link the gmp and fr libraries
    println!("cargo:rustc-link-lib=static=gmp");
    println!("cargo:rustc-link-lib=static=fr");
    // Specify the path to the witnesscalc library for the linker
    println!(
        "cargo:rustc-link-search=native={}",
        lib_dir.to_string_lossy()
    );
    if !(env::var("CARGO_CFG_TARGET_OS").unwrap().contains("ios")
        || env::var("CARGO_CFG_TARGET_OS").unwrap().contains("android"))
    {
        circuit_names.iter().for_each(|circuit_name| {
            println!("cargo:rustc-link-lib=dylib=witnesscalc_{}", circuit_name);
        });
        println!("cargo:rustc-link-lib=dylib=fr");
        println!("cargo:rustc-link-lib=dylib=gmp");
    }

    // refer to https://github.com/bbqsrc/cargo-ndk to see how to link the libc++_shared.so file in Android
    if env::var("CARGO_CFG_TARGET_OS").unwrap() == "android" {
        android();
    }
}

fn android() {
    println!("cargo:rustc-link-lib=c++_shared");

    if let Ok(output_path) = env::var("CARGO_NDK_OUTPUT_PATH") {
        let sysroot_libs_path = PathBuf::from(env::var_os("CARGO_NDK_SYSROOT_LIBS_PATH").unwrap());
        let lib_path = sysroot_libs_path.join("libc++_shared.so");
        assert!(
            lib_path.exists(),
            "Error: Source file {:?} does not exist",
            lib_path
        );
        let dest_dir = Path::new(&output_path).join(env::var("CARGO_NDK_ANDROID_TARGET").unwrap());
        println!("cargo:rerun-if-changed={}", dest_dir.display());
        if !dest_dir.exists() {
            fs::create_dir_all(&dest_dir).unwrap();
        }
        fs::copy(lib_path, Path::new(&dest_dir).join("libc++_shared.so")).unwrap();
    }
}
