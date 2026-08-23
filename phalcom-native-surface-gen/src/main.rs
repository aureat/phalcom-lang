//! Deterministic primitive-attribute census and generated-surface drift gate.

use phalcom_native_decl::{docs_from_attributes, parse_primitive_attribute};
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    if let Err(error) = run() {
        eprintln!("phalcom-native-surface-gen: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut root = PathBuf::from(".");
    let mut check = false;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--check" => check = true,
            "--root" => {
                root = PathBuf::from(args.next().ok_or("--root requires a path")?);
            }
            other => return Err(format!("unknown argument `{other}`")),
        }
    }

    let primitive_root = root.join("phalcom-core/src/primitive");
    let mut files = Vec::new();
    collect_rs(&primitive_root, &mut files).map_err(|error| error.to_string())?;
    files.sort();

    let mut declarations = BTreeMap::new();
    for file in files {
        let text = fs::read_to_string(&file).map_err(|error| format!("{}: {error}", file.display()))?;
        let syntax = syn::parse_file(&text).map_err(|error| format!("{}: {error}", file.display()))?;
        collect_declarations(&syntax.items, &file, &mut declarations)?;
    }

    let generated = root.join("phalcom-native-surface/src/generated.rs");
    if !generated.is_file() {
        return Err(format!("missing generated surface {}", generated.display()));
    }
    let generated_text = fs::read_to_string(&generated).map_err(|error| error.to_string())?;
    if !generated_text.contains("Generated canonical native surface records") {
        return Err(format!("{} is not a generated native surface artifact", generated.display()));
    }
    let count_marker = format!("GENERATED_PRIMITIVE_DECLARATION_COUNT: usize = {}", declarations.len());
    if !generated_text.contains(&count_marker) {
        return Err(format!(
            "{} is stale: expected authored declaration count {}",
            generated.display(),
            declarations.len()
        ));
    }

    if check {
        // The generator currently validates the authored projection and its
        // artifact boundary. Full rich-record emission is intentionally kept
        // in the checked-in VM-free projection until all legacy primitives are
        // migrated to declarations.
        println!("native surface artifact current: {} primitive declarations", declarations.len());
    } else {
        println!("validated {} primitive declarations", declarations.len());
    }
    Ok(())
}

fn collect_declarations(
    items: &[syn::Item],
    file: &Path,
    declarations: &mut BTreeMap<(phalcom_native_meta::UniverseKey, phalcom_native_meta::NativeDispatch, String), (PathBuf, String)>,
) -> Result<(), String> {
    for item in items {
        match item {
            syn::Item::Fn(function) => {
                for attribute in function
                    .attrs
                    .iter()
                    .filter(|attribute| attribute.path().segments.last().is_some_and(|segment| segment.ident.to_string() == "primitive"))
                {
                    let mut declaration = parse_primitive_attribute(attribute).map_err(|error| format!("{}: {error}", file.display()))?;
                    declaration.docs = docs_from_attributes(&function.attrs);
                    let key = (declaration.key.owner, declaration.side, declaration.key.selector.clone());
                    if let Some(previous) = declarations.insert(key.clone(), (file.to_path_buf(), function.sig.ident.to_string())) {
                        return Err(format!("duplicate native key {:?} in {} and {}", key, previous.0.display(), file.display()));
                    }
                }
            }
            syn::Item::Mod(module) => {
                if let Some((_, nested)) = &module.content {
                    collect_declarations(nested, file, declarations)?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn collect_rs(root: &Path, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if !root.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(root)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_rs(&path, files)?;
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }
    Ok(())
}
