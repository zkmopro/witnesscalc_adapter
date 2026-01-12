pub use paste;
pub use serde_json;
use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

pub mod convert_type;
pub use convert_type::*;

/// Sanitized env for shell commands, filtering out Xcode's CC/CXX/CPP that break GMP autoconf.
fn sanitized_env() -> HashMap<String, String> {
    const SAFE_VARS: &[&str] = &[
        "PATH",
        "HOME",
        "USER",
        "TMPDIR",
        "TERM",
        "LANG",
        "LC_ALL",
        "SDKROOT",
        "DEVELOPER_DIR",
        "IPHONEOS_DEPLOYMENT_TARGET",
    ];
    let mut env_map: HashMap<String, String> = env::vars()
        .filter(|(k, _)| SAFE_VARS.contains(&k.as_str()))
        .collect();
    let path = env_map.get("PATH").map(|p| p.as_str()).unwrap_or("");
    env_map.insert(
        "PATH".into(),
        format!("{}:/usr/bin:/bin:/usr/local/bin:/opt/homebrew/bin", path),
    );
    env_map
}

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
                // FFI return codes
                const WITNESSCALC_OK: std::ffi::c_int = 0x0;
                const WITNESSCALC_ERROR_SHORT_BUFFER: std::ffi::c_int = 0x2;

                println!("Generating witness for circuit {}", stringify!($x));
                unsafe {
                    let json_input = std::ffi::CString::new(json_input).map_err(|e| $crate::__macro_deps::anyhow::anyhow!("Failed to convert JSON input to CString: {}", e))?;
                    let json_size = json_input.as_bytes().len() as std::ffi::c_ulong;

                    let circuit_buffer = [<$x _CIRCUIT_DATA>].as_ptr() as *const std::ffi::c_char;
                    let circuit_size = [<$x _CIRCUIT_DATA>].len() as std::ffi::c_ulong;

                    let mut error_msg = vec![0u8; 256]; // Error message buffer
                    let error_msg_ptr = error_msg.as_mut_ptr() as *mut std::ffi::c_char;

                    // Two-pass dynamic allocation:
                    // Pass 1: Probe with small buffer to query required size
                    let mut probe_buffer = vec![0u8; 1024]; // 1 KB probe buffer
                    let mut wtns_size: std::ffi::c_ulong = probe_buffer.len() as std::ffi::c_ulong;

                    let result = [<witnesscalc_ $x>](
                        circuit_buffer,
                        circuit_size,
                        json_input.as_ptr(),
                        json_size,
                        probe_buffer.as_mut_ptr() as *mut _,
                        &mut wtns_size as *mut _,
                        error_msg_ptr,
                        error_msg.len() as u64,
                    );

                    // Pass 2: If buffer too small, allocate exact size and retry
                    let final_buffer = if result == WITNESSCALC_ERROR_SHORT_BUFFER {
                        // wtns_size now contains the required minimum size
                        let required_size = wtns_size as usize;
                        println!("Witness requires {} bytes, allocating and retrying...", required_size);

                        let mut wtns_buffer = vec![0u8; required_size];
                        let mut wtns_size: std::ffi::c_ulong = required_size as std::ffi::c_ulong;

                        let result = [<witnesscalc_ $x>](
                            circuit_buffer,
                            circuit_size,
                            json_input.as_ptr(),
                            json_size,
                            wtns_buffer.as_mut_ptr() as *mut _,
                            &mut wtns_size as *mut _,
                            error_msg_ptr,
                            error_msg.len() as u64,
                        );

                        if result != WITNESSCALC_OK {
                            let error_string = std::ffi::CStr::from_ptr(error_msg_ptr)
                                .to_string_lossy()
                                .into_owned();
                            return Err($crate::__macro_deps::anyhow::anyhow!("Witness generation failed: {}", error_string));
                        }

                        wtns_buffer[..wtns_size as usize].to_vec()
                    } else if result == WITNESSCALC_OK {
                        // Success on first try with probe buffer (small witness)
                        probe_buffer[..wtns_size as usize].to_vec()
                    } else {
                        // Other error
                        let error_string = std::ffi::CStr::from_ptr(error_msg_ptr)
                            .to_string_lossy()
                            .into_owned();
                        return Err($crate::__macro_deps::anyhow::anyhow!("Witness generation failed: {}", error_string));
                    };

                    Ok(final_buffer)
                }
            }
        }
    };
}

const WITNESSCALC_BUILD_SCRIPT: &str = include_str!("../clone_witnesscalc.sh");

pub fn build_and_link(circuits_dir: &str) {
    let target = env::var("TARGET").expect("Cargo did not provide the TARGET environment variable");

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

    let gmp_dir = witnesscalc_path.join("depends").join("gmp");
    let target_dir = gmp_dir.join(gmp_lib_folder);
    if !target_dir.exists() {
        Command::new("bash")
            .current_dir(&witnesscalc_path)
            .env_clear()
            .envs(sanitized_env())
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

    let mut v2_1_0_circuit_files: Vec<PathBuf> = Vec::new();
    let mut v2_2_0_circuit_files: Vec<PathBuf> = Vec::new();

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
        fs::write(&circuit_cpp_dest, &circuit_cpp).expect("Failed to write circuit .cpp file");

        let circuit_cpp_str = &circuit_cpp;
        if circuit_cpp_str.contains("uint get_size_of_bus_field_map() {return 0;}") {
            v2_2_0_circuit_files.push(path.clone());
        } else {
            v2_1_0_circuit_files.push(path.clone());
        }
    });

    build_for_circuits_with_different_versions(
        &v2_1_0_circuit_files,
        &witnesscalc_path,
        &witnesscalc_build_target,
    );
    if v2_2_0_circuit_files.len() > 0 {
        Command::new("git")
            .arg("checkout")
            .arg("secq256r1-support-v2.2.0")
            .current_dir(&witnesscalc_path)
            .spawn()
            .expect("Failed to spawn git checkout secq256r1-support-v2.2.0")
            .wait()
            .expect("git checkout secq256r1-support-v2.2.0 errored");
        build_for_circuits_with_different_versions(
            &v2_2_0_circuit_files,
            &witnesscalc_path,
            &witnesscalc_build_target,
        );
    }

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
        println!("cargo:rustc-link-lib=dylib=fr");
        println!("cargo:rustc-link-lib=dylib=gmp");
    }
}

fn build_for_circuits_with_different_versions(
    circuit_files: &Vec<PathBuf>,
    witnesscalc_path: &Path,
    witnesscalc_build_target: &str,
) {
    circuit_files.iter().for_each(|path| {
        let circuit_name = path.file_stem().unwrap().to_str().unwrap();
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

    let mut make_env = sanitized_env();
    make_env.insert("CIRCUIT_NAMES".into(), circuit_names.join(";"));

    let make_process = Command::new("make")
        .env_clear()
        .envs(make_env)
        .arg(witnesscalc_build_target)
        .current_dir(&witnesscalc_path)
        .output()
        .expect("Failed to execute make arm64_host");

    if !make_process.status.success() {
        eprintln!(
            "Make command failed with exit code: {}",
            make_process.status
        );
        eprintln!("stdout: {}", String::from_utf8_lossy(&make_process.stdout));
        eprintln!("stderr: {}", String::from_utf8_lossy(&make_process.stderr));

        // Check if any of the required libraries were actually built despite the error
        let lib_dir = witnesscalc_path.join("package").join("lib");
        let mut all_libs_exist = true;

        for circuit_name in &circuit_names {
            let lib_path = lib_dir.join(format!("libwitnesscalc_{}.a", circuit_name));
            if !lib_path.exists() {
                eprintln!("Warning: Library {} was not built", lib_path.display());
                all_libs_exist = false;
            }
        }

        if !all_libs_exist {
            panic!("Make command failed and required libraries are missing");
        } else {
            eprintln!("Warning: Make command failed but required libraries exist. Continuing...");
        }
    }

    // Link the witnesscalc library for the circuit
    circuit_names.iter().for_each(|circuit_name| {
        println!("cargo:rustc-link-lib=static=witnesscalc_{}", circuit_name);
    });

    if !(env::var("CARGO_CFG_TARGET_OS").unwrap().contains("ios")
        || env::var("CARGO_CFG_TARGET_OS").unwrap().contains("android"))
    {
        circuit_names.iter().for_each(|circuit_name| {
            println!("cargo:rustc-link-lib=dylib=witnesscalc_{}", circuit_name);
        });
    }
}
