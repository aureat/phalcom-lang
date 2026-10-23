//! Generates the builtin Universe catalog and embedded-source lookup from the
//! `packages/universe/src` directory tree, so the tree is the only source of truth.
//!
//! Mapping: a directory with `package.ph` is a package node; any other `*.ph`
//! file is a module node named by its stem. Hyphens in file or directory names
//! become underscores in module paths.

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

struct Node {
    path: Vec<String>,
    package: bool,
    file: PathBuf,
    children: Vec<String>,
}

fn component(name: &str) -> String {
    name.replace('-', "_")
}

/// Child names of a package, ordered by `expose .child` lines first, then alphabetically.
fn ordered(mut found: Vec<String>, package_source: &str) -> Vec<String> {
    found.sort();
    let mut out = Vec::new();
    for line in package_source.lines() {
        if let Some(rest) = line.trim().strip_prefix("expose .") {
            let name = rest.split_whitespace().next().unwrap_or("").to_string();
            if let Some(index) = found.iter().position(|candidate| *candidate == name) {
                out.push(found.remove(index));
            }
        }
    }
    out.extend(found);
    out
}

fn walk(dir: &Path, path: Vec<String>, nodes: &mut Vec<Node>) {
    let package_file = dir.join("package.ph");
    let source = fs::read_to_string(&package_file).unwrap_or_else(|error| panic!("{} must exist: {error}", package_file.display()));
    let mut subdirs = Vec::new();
    let mut names = Vec::new();
    for entry in fs::read_dir(dir).expect("read universe directory") {
        let entry = entry.expect("universe directory entry").path();
        let name = entry.file_name().unwrap().to_string_lossy().into_owned();
        if entry.is_dir() {
            names.push(component(&name));
            subdirs.push((component(&name), entry));
        } else if name.ends_with(".ph") && name != "package.ph" {
            let stem = component(name.trim_end_matches(".ph"));
            names.push(stem.clone());
            nodes.push(Node { path: [path.clone(), vec![stem]].concat(), package: false, file: entry, children: Vec::new() });
        }
    }
    let children = ordered(names, &source);
    nodes.push(Node { path: path.clone(), package: true, file: package_file, children });
    for (name, sub) in subdirs {
        walk(&sub, [path.clone(), vec![name]].concat(), nodes);
    }
}

fn quoted(path: &[String]) -> String {
    format!("&[{}]", path.iter().map(|part| format!("{part:?}")).collect::<Vec<_>>().join(", "))
}

fn main() {
    let root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("../packages/universe/src");
    let root = root.canonicalize().expect("packages/universe/src exists");
    println!("cargo:rerun-if-changed={}", root.display());
    let mut nodes = Vec::new();
    walk(&root, Vec::new(), &mut nodes);
    nodes.sort_by(|a, b| a.path.cmp(&b.path));

    let mut catalog = String::from("pub const UNIVERSE_NODES: &[UniverseNodeSpec] = &[\n");
    let mut lookup = String::from("fn embedded_source(components: &[&str]) -> Option<&'static str> {\n    match components {\n");
    for node in &nodes {
        let kind = if node.package { "ModuleKind::Package" } else { "ModuleKind::Module" };
        let _ = writeln!(catalog, "    UniverseNodeSpec {{ path: {}, kind: {kind}, children: {} }},", quoted(&node.path), quoted(&node.children));
        let pattern = format!("[{}]", node.path.iter().map(|part| format!("{part:?}")).collect::<Vec<_>>().join(", "));
        let _ = writeln!(lookup, "        {pattern} => Some(include_str!({:?})),", node.file.display().to_string());
        println!("cargo:rerun-if-changed={}", node.file.display());
    }
    catalog.push_str("];\n");
    lookup.push_str("        _ => None,\n    }\n}\n");
    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("universe_catalog.rs");
    fs::write(out, format!("{catalog}\n{lookup}")).expect("write universe catalog");
}
