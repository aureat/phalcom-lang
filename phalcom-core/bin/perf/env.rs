use std::collections::BTreeMap;
use std::fs;
use std::mem::{align_of, size_of};
use std::path::Path;
use std::process::Command;

use phalcom_core::error::{PhError, RuntimeError};
use phalcom_core::heap::{ObjRef, Object};
use phalcom_core::value::Value;

use crate::model::{BuildMetadata, GitMetadata, HostMetadata, TypeLayout};

pub fn capture_git() -> GitMetadata {
    let sha = run_cmd("git", &["rev-parse", "HEAD"]).unwrap_or_else(|| "unknown".into());
    let short_sha = run_cmd("git", &["rev-parse", "--short", "HEAD"]).unwrap_or_else(|| "unknown".into());
    let branch = run_cmd("git", &["branch", "--show-current"]).unwrap_or_else(|| "HEAD".into());
    let dirty = run_cmd("git", &["status", "--porcelain"]).map(|s| !s.trim().is_empty()).unwrap_or(false);

    GitMetadata { sha, short_sha, branch, dirty }
}

pub fn capture_build(binary: &Path) -> BuildMetadata {
    let binary_path = binary.to_string_lossy().to_string();
    let binary_size_bytes = fs::metadata(binary).map(|m| m.len()).unwrap_or(0);
    let rustc_version = run_cmd("rustc", &["-Vv"]).unwrap_or_else(|| "unknown".into());
    let cargo_version = run_cmd("cargo", &["-V"]).unwrap_or_else(|| "unknown".into());

    let profile = if binary_path.contains("/debug/") { "debug".into() } else { "release".into() };

    let target_triple = rustc_version
        .lines()
        .find(|line| line.starts_with("host: "))
        .map(|line| line.trim_start_matches("host: ").to_string())
        .unwrap_or_else(|| std::env::consts::ARCH.to_string());

    BuildMetadata {
        binary_path,
        binary_size_bytes,
        profile,
        target_triple,
        rustc_version,
        cargo_version,
    }
}

pub fn capture_host() -> HostMetadata {
    let os = std::env::consts::OS.to_string();
    let arch = std::env::consts::ARCH.to_string();
    let logical_cpus = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1);

    let os_version = run_cmd("uname", &["-r"]).unwrap_or_else(|| "unknown".into());

    let cpu_model = if cfg!(target_os = "macos") {
        run_cmd("sysctl", &["-n", "machdep.cpu.brand_string"]).unwrap_or_else(|| "Apple Silicon / Unknown".into())
    } else {
        run_cmd("sh", &["-c", "grep -m1 'model name' /proc/cpuinfo | cut -d: -f2"])
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "Unknown CPU".into())
    };

    let host_key = format!("{os}-{arch}-{cpu_model}");

    HostMetadata {
        os,
        os_version,
        arch,
        cpu_model,
        logical_cpus,
        host_key,
    }
}

pub fn capture_layouts() -> BTreeMap<String, TypeLayout> {
    let mut map = BTreeMap::new();
    map.insert(
        "Value".into(),
        TypeLayout {
            size_bytes: size_of::<Value>(),
            align_bytes: align_of::<Value>(),
        },
    );
    map.insert(
        "ObjRef".into(),
        TypeLayout {
            size_bytes: size_of::<ObjRef>(),
            align_bytes: align_of::<ObjRef>(),
        },
    );
    map.insert(
        "Object".into(),
        TypeLayout {
            size_bytes: size_of::<Object>(),
            align_bytes: align_of::<Object>(),
        },
    );
    map.insert(
        "RuntimeError".into(),
        TypeLayout {
            size_bytes: size_of::<RuntimeError>(),
            align_bytes: align_of::<RuntimeError>(),
        },
    );
    map.insert(
        "PhError".into(),
        TypeLayout {
            size_bytes: size_of::<PhError>(),
            align_bytes: align_of::<PhError>(),
        },
    );
    map.insert(
        "Result<Value, PhError>".into(),
        TypeLayout {
            size_bytes: size_of::<Result<Value, PhError>>(),
            align_bytes: align_of::<Result<Value, PhError>>(),
        },
    );
    map
}

fn run_cmd(cmd: &str, args: &[&str]) -> Option<String> {
    Command::new(cmd)
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}
