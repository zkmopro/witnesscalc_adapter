use std::{env, fs, path::Path, process::Command};

const WITNESSCALC_BUILD_SCRIPT: &str = include_str!("../clone_witnesscalc.sh");

pub fn build_and_link() {
    println!("WE ARE HERE");

    let target = env::var("TARGET").expect("Cargo did not provide the TARGET environment variable");
    println!("The target is: {}", target);
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let lib_dir = Path::new(&out_dir)
        .join("witnesscalc")
        .join("package")
        .join("lib");

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

    // If the witnesscalc library is not built, build it
    // TODO detect circuit source changes and rebuild
    if !lib_dir.exists() {
        Command::new("sh")
            .current_dir(&witnesscalc_path)
            .arg("./build_gmp.sh")
            .arg("host")
            .spawn()
            .expect("Failed to spawn build_gmp.sh")
            .wait()
            .expect("build_gmp.sh errored");
        // TODO detect target architecture and build the correct target
        // "Switch" the target and set the build_target_architecture

        //const TARGET_ARCHS: [&str; 5] = ["host", "arm64_host", "android", "android_x86_64", "ios"];

        //Map the target to the correct build target
        let build_target_architecture = match target.as_str() {
            "aarch64-apple-ios" => "ios",
            "aarch64-apple-ios-sim" => "ios",
            "x86_64-apple-ios" => "ios",
            "x86_64-linux-android" => "android_x86_64",
            "i686-linux-android" => "android_x86_64",
            "armv7-linux-androideabi" => "android",
            "aarch64-linux-android" => "android",
            "aarch64-apple-darwin" => "arm64_host",
            _ => "host",
        };

        Command::new("make")
            .arg(build_target_architecture)
            .current_dir(&witnesscalc_path)
            .spawn()
            .expect("Failed to spawn make arm64_host")
            .wait()
            .expect("make arm64_host errored");
    }

    // The following are commands that link the witnesscalc library
    println!(
        "cargo:rustc-link-search=native={}",
        lib_dir.to_string_lossy()
    );
    // TODO generalize the circuit name
    println!("cargo:rustc-link-lib=witnesscalc_authV2");
    println!(
        "cargo:rustc-link-arg=-Wl,-rpath,{}",
        lib_dir.to_string_lossy()
    );
}
