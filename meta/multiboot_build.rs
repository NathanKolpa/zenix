use std::{
    path::{Path, PathBuf},
    process::Command,
};

fn main() {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    let pre_kernel = build_pre_kernel(&out_dir);
    let kernel = PathBuf::from(std::env::var_os("CARGO_BIN_FILE_KERNEL_kernel").unwrap());

    println!("cargo:rustc-env=KERNEL_PATH={}", kernel.display());
    println!("cargo:rustc-env=PRE_KERNEL_PATH={}", pre_kernel.display());
}

fn build_pre_kernel(out_dir: &Path) -> PathBuf {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let mut cmd = Command::new(cargo);
    cmd.arg("install").arg("pre-kernel");
    let local_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("pre-kernel");
    let target_json_path = local_path.join("target.json");
    if local_path.exists() {
        cmd.arg("--path").arg(&local_path);
        println!("cargo:rerun-if-changed={}", local_path.display());
    }
    cmd.arg("--locked");
    cmd.arg("-v");
    cmd.arg("--target").arg(target_json_path);
    cmd.arg("-Zbuild-std=core")
        .arg("-Zbuild-std-features=compiler-builtins-mem");
    cmd.arg("--root").arg(out_dir);
    cmd.env_remove("RUSTFLAGS");
    cmd.env_remove("CARGO_ENCODED_RUSTFLAGS");
    cmd.env_remove("RUSTC_WORKSPACE_WRAPPER"); // used by clippy

    let profile = std::env::var("PROFILE").unwrap();
    if profile == "debug" {
        cmd.arg("--profile");
        cmd.arg("pre-kernel-debug");
    }

    let status = cmd
        .status()
        .expect("failed to run cargo install for pre-kernel stage");

    if status.success() {
        let elf_path = out_dir.join("bin").join("pre-kernel");
        assert!(
            elf_path.exists(),
            "pre-kernel executable does not exist after building"
        );
        elf_path
    } else {
        panic!("failed to build pre-kernel");
    }
}
