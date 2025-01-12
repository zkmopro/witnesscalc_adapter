use std::{env, fs, path::Path, process::Command};

const WITNESSCALC_BUILD_SCRIPT: &str = include_str!("./clone_witnesscalc.sh");

fn main() {
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
        Command::new("make")
            .arg("arm64_host")
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
