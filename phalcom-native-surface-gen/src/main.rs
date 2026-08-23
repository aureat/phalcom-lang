//! Deterministic primitive-attribute census and generated-surface drift gate.

use phalcom_common::selector::{Selector, SelectorKind};
use phalcom_native_decl::{docs_from_attributes, parse_primitive_attribute, NormalizedPrimitiveDecl};
use phalcom_native_meta::*;
use phalcom_type_syntax::{parse_callable_type, parse_type_expr, CallableType, ParameterTuple, TypeExpr};
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

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

    let ordered_decls: Vec<NormalizedPrimitiveDecl> = declarations.into_values().collect();
    let generated_code = generate_surface_source(&ordered_decls)?;
    let formatted_code = format_source(&generated_code)?;

    let target_path = root.join("phalcom-native-surface/src/generated.rs");
    if check {
        if !target_path.is_file() {
            return Err(format!("missing generated surface file {}", target_path.display()));
        }
        let existing_text = fs::read_to_string(&target_path).map_err(|e| e.to_string())?;
        if existing_text != formatted_code {
            return Err(format!(
                "{} is stale with respect to authored primitive declarations in {}",
                target_path.display(),
                primitive_root.display()
            ));
        }
        println!("native surface artifact current: {} primitive declarations", ordered_decls.len());
    } else {
        fs::write(&target_path, &formatted_code).map_err(|e| format!("failed to write {}: {e}", target_path.display()))?;
        println!("generated {} primitive surface records to {}", ordered_decls.len(), target_path.display());
    }

    Ok(())
}

fn collect_declarations(
    items: &[syn::Item],
    file: &Path,
    declarations: &mut BTreeMap<(UniverseKey, NativeDispatch, String), NormalizedPrimitiveDecl>,
) -> Result<(), String> {
    for item in items {
        match item {
            syn::Item::Fn(function) => {
                for attribute in function
                    .attrs
                    .iter()
                    .filter(|attribute| attribute.path().segments.last().is_some_and(|segment| segment.ident == "primitive"))
                {
                    let mut declaration = parse_primitive_attribute(attribute).map_err(|error| format!("{}: {error}", file.display()))?;
                    declaration.docs = docs_from_attributes(&function.attrs);
                    let key = (declaration.key.owner, declaration.side, declaration.key.selector.clone());
                    if declarations.insert(key.clone(), declaration).is_some() {
                        return Err(format!("duplicate native key {:?} in {}", key, file.display()));
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

fn generate_surface_source(declarations: &[NormalizedPrimitiveDecl]) -> Result<String, String> {
    let count = declarations.len();
    let mut records = Vec::new();
    for decl in declarations {
        records.push(emit_surface_record(decl)?);
    }

    let file_tokens = quote! {
        //! Generated canonical native surface records.

        /// Number of authored `#[primitive]` declarations scanned by the surface
        /// generator.
        pub const GENERATED_PRIMITIVE_DECLARATION_COUNT: usize = #count;

        use crate::{NativeMemberKind, NativeReturnShape, NativeSurfaceRecord};
        use phalcom_native_meta::*;

        pub static NATIVE_SURFACES: &[NativeSurfaceRecord] = &[
            #(#records),*
        ];
    };

    Ok(file_tokens.to_string())
}

fn emit_surface_record(decl: &NormalizedPrimitiveDecl) -> Result<TokenStream2, String> {
    let owner_ident = format_ident!("{}", decl.key.owner.name());
    let selector_str = &decl.key.selector;
    let selector = Selector::try_decode_exact(selector_str)
        .map_err(|e| format!("invalid selector `{selector_str}` for {owner_ident}: {e}"))?;

    let kind_ident = match selector.kind {
        SelectorKind::Getter => format_ident!("Getter"),
        SelectorKind::Setter => format_ident!("Setter"),
        SelectorKind::Method | SelectorKind::SubscriptGet | SelectorKind::SubscriptSet => format_ident!("Method"),
    };

    let side_ident = match decl.side {
        NativeDispatch::Instance => format_ident!("Instance"),
        NativeDispatch::Class => format_ident!("Class"),
    };

    let is_internal = selector_str.starts_with("_$");
    let vis = decl.visibility.unwrap_or(if is_internal {
        NativeVisibility::Internal
    } else {
        NativeVisibility::Public
    });
    let vis_ident = match vis {
        NativeVisibility::Public => format_ident!("Public"),
        NativeVisibility::Internal => format_ident!("Internal"),
    };

    let stability_ident = match decl.stability {
        NativeStability::Unspecified => format_ident!("Unspecified"),
        NativeStability::Experimental => format_ident!("Experimental"),
        NativeStability::Stable => format_ident!("Stable"),
    };

    let anchor_ident = match decl.anchor {
        NativeAnchorPolicy::Required => format_ident!("Required"),
        NativeAnchorPolicy::Hidden => format_ident!("Hidden"),
    };

    let abi_ident = match decl.abi {
        PrimitiveAbi::Value => format_ident!("Value"),
        PrimitiveAbi::Shape => format_ident!("Shape"),
    };

    let trust_ident = match decl.trust {
        NativeTrust::Ordinary => format_ident!("Ordinary"),
        NativeTrust::Privileged => format_ident!("Privileged"),
    };

    let intrinsic_tokens = match decl.intrinsic.as_deref() {
        Some("BoolAnd") => quote!(Some(::phalcom_native_meta::NativeIntrinsicId::BoolAnd)),
        Some("BoolOr") => quote!(Some(::phalcom_native_meta::NativeIntrinsicId::BoolOr)),
        Some("BoolNot") => quote!(Some(::phalcom_native_meta::NativeIntrinsicId::BoolNot)),
        Some(other) => return Err(format!("unknown intrinsic `{other}` on {selector_str}")),
        None => quote!(None),
    };

    let flow_str = decl.flow.as_deref().unwrap_or("value");
    let flow_tokens = match flow_str {
        "value" => quote!(::phalcom_native_meta::ReturnFlowSpec::Value),
        "receiver" => quote!(::phalcom_native_meta::ReturnFlowSpec::Receiver),
        "never" => quote!(::phalcom_native_meta::ReturnFlowSpec::Never),
        "unknown" => quote!(::phalcom_native_meta::ReturnFlowSpec::Unknown),
        s if s.starts_with("argument(") && s.ends_with(')') => {
            let inner = s["argument(".len()..s.len() - 1].trim();
            let idx: usize = inner.parse().map_err(|_| format!("invalid flow argument index `{inner}`"))?;
            quote!(::phalcom_native_meta::ReturnFlowSpec::Argument(#idx))
        }
        other => return Err(format!("unknown flow spec `{other}` on {selector_str}")),
    };

    let effects_tokens = match decl.effects.as_deref() {
        Some("pure") => quote!(::phalcom_native_meta::EffectSpec::Pure),
        Some("unknown") | None => quote!(::phalcom_native_meta::EffectSpec::Unknown),
        Some(s) if s.starts_with('[') && s.ends_with(']') => {
            let inner = &s[1..s.len() - 1];
            let mut effect_idents = Vec::new();
            for part in inner.split(',') {
                let trimmed = part.trim();
                if trimmed.is_empty() {
                    continue;
                }
                match trimmed {
                    "mutation" => effect_idents.push(quote!(::phalcom_native_meta::NativeEffect::Mutation)),
                    "io" => effect_idents.push(quote!(::phalcom_native_meta::NativeEffect::Io)),
                    "scheduling" => effect_idents.push(quote!(::phalcom_native_meta::NativeEffect::Scheduling)),
                    "reflection" => effect_idents.push(quote!(::phalcom_native_meta::NativeEffect::Reflection)),
                    "nondeterminism" => effect_idents.push(quote!(::phalcom_native_meta::NativeEffect::Nondeterminism)),
                    "blocking" => effect_idents.push(quote!(::phalcom_native_meta::NativeEffect::Blocking)),
                    other => return Err(format!("unknown effect `{other}` on {selector_str}")),
                }
            }
            quote!(::phalcom_native_meta::EffectSpec::Known(&[#(#effect_idents),*]))
        }
        Some(other) => return Err(format!("invalid effects specification `{other}` on {selector_str}")),
    };

    let raises_tokens = match decl.raises.as_deref() {
        Some(s) if s.starts_with('[') && s.ends_with(']') => {
            let inner = &s[1..s.len() - 1];
            let mut ty_tokens = Vec::new();
            for part in inner.split(',') {
                let trimmed = part.trim();
                if trimmed.is_empty() {
                    continue;
                }
                let ty = parse_type_expr(trimmed).map_err(|e| format!("invalid raise type `{trimmed}`: {e}"))?;
                let elem = emit_type_expr_spec(&ty, &[])?;
                ty_tokens.push(quote!(*#elem));
            }
            quote!(::phalcom_native_meta::RaisesSpec::Known(&[#(#ty_tokens),*]))
        }
        _ => quote!(::phalcom_native_meta::RaisesSpec::Unknown),
    };

    let since_tokens = match &decl.since {
        Some(s) => quote!(Some(#s)),
        None => quote!(None),
    };
    let deprecated_since_tokens = match &decl.deprecated_since {
        Some(s) => quote!(Some(#s)),
        None => quote!(None),
    };
    let replacement_tokens = match &decl.replacement {
        Some(s) => quote!(Some(#s)),
        None => quote!(None),
    };

    let docs_tokens = if decl.docs.is_empty() {
        quote!(None)
    } else {
        let joined = decl.docs.join("\n");
        quote!(Some(#joined))
    };

    let conceptual_tokens = match &decl.conceptual {
        Some(c) => quote!(Some(#c)),
        None => quote!(None),
    };

    let (callable_tokens, params_tokens, returns_tokens, return_shape_tokens) = if let Some(types_str) = &decl.types {
        let parsed_callable = parse_callable_type(types_str)
            .map_err(|e| format!("failed to parse types contract `{types_str}` on {owner_ident}>>{selector_str}: {e}"))?;
        let binders: Vec<String> = parsed_callable.type_params.iter().map(|p| p.name.clone()).collect();
        let callable_tok = emit_callable_spec(&parsed_callable)?;
        let params_tok = emit_params_spec(&parsed_callable.params, &binders)?;
        let returns_tok = emit_type_expr_spec(&parsed_callable.return_type, &binders)?;
        let return_shape_tok = compute_return_shape(flow_str, &parsed_callable.return_type);
        (callable_tok, params_tok, returns_tok, return_shape_tok)
    } else {
        let mut positional = Vec::new();
        let mut labeled = Vec::new();
        let mut rest = None;

        if selector.kind == SelectorKind::Setter {
            positional.push(TypeExpr::Unknown);
        } else {
            for slot in selector.slots.iter() {
                match slot {
                    phalcom_common::selector::SelectorSlot::Positional => {
                        positional.push(TypeExpr::Unknown);
                    }
                    phalcom_common::selector::SelectorSlot::Label(l) => {
                        labeled.push(phalcom_type_syntax::LabeledParameter {
                            label: l.clone(),
                            ty: TypeExpr::Unknown,
                        });
                    }
                }
            }
            if selector_str.contains("***") || selector_str.ends_with("...") {
                rest = Some(phalcom_type_syntax::RestParameter { ty: None });
                // If the last slot in slots was rest represented as positional, drop it from positional
                if selector_str.contains("***") && !positional.is_empty() {
                    positional.pop();
                }
            }
        }

        let default_params = ParameterTuple {
            positional,
            labeled,
            rest,
        };
        let default_returns = TypeExpr::Unknown;
        let default_callable = CallableType {
            type_params: Vec::new(),
            params: default_params.clone(),
            return_type: default_returns.clone(),
        };
        let callable_tok = emit_callable_spec(&default_callable)?;
        let params_tok = emit_params_spec(&default_params, &[])?;
        let returns_tok = emit_type_expr_spec(&default_returns, &[])?;
        let return_shape_tok = compute_return_shape(flow_str, &default_returns);
        (callable_tok, params_tok, returns_tok, return_shape_tok)
    };

    Ok(quote! {
        NativeSurfaceRecord {
            surface: PrimitiveSurfaceSpec {
                key: PrimitiveKey {
                    owner: UniverseKey::#owner_ident,
                    side: NativeDispatch::#side_ident,
                    selector: #selector_str,
                },
                visibility: NativeVisibility::#vis_ident,
                stability: NativeStability::#stability_ident,
                anchor: NativeAnchorPolicy::#anchor_ident,
                params: #params_tokens,
                returns: #returns_tokens,
                callable: #callable_tokens,
                raises: #raises_tokens,
                effects: #effects_tokens,
                flow: #flow_tokens,
                termination: TerminationSpec::Unknown,
                since: #since_tokens,
                deprecated_since: #deprecated_since_tokens,
                replacement: #replacement_tokens,
                lifecycle: NativeLifecycleSpec {
                    since: #since_tokens,
                    deprecated_since: #deprecated_since_tokens,
                    replacement: #replacement_tokens,
                },
                intrinsic: #intrinsic_tokens,
                trust: NativeTrust::#trust_ident,
                docs: #docs_tokens,
                conceptual: #conceptual_tokens,
            },
            kind: NativeMemberKind::#kind_ident,
            abi: PrimitiveAbi::#abi_ident,
            return_shape: #return_shape_tokens,
        }
    })
}

fn compute_return_shape(flow: &str, return_type: &TypeExpr) -> TokenStream2 {
    if flow == "receiver" {
        quote!(NativeReturnShape::Receiver)
    } else if flow.starts_with("argument(") && flow.ends_with(')') {
        let inner = flow["argument(".len()..flow.len() - 1].trim();
        if let Ok(idx) = inner.parse::<usize>() {
            quote!(NativeReturnShape::Argument(#idx))
        } else {
            quote!(NativeReturnShape::Unknown)
        }
    } else {
        match return_type {
            TypeExpr::SelfType => quote!(NativeReturnShape::Receiver),
            TypeExpr::Universe(name) => {
                if let Some(key) = UniverseKey::from_name(name) {
                    let name_str = key.name();
                    quote!(NativeReturnShape::Instance(#name_str))
                } else {
                    quote!(NativeReturnShape::Unknown)
                }
            }
            TypeExpr::Named(name) => {
                if let Some(key) = UniverseKey::from_name(name) {
                    let name_str = key.name();
                    quote!(NativeReturnShape::Instance(#name_str))
                } else {
                    quote!(NativeReturnShape::Unknown)
                }
            }
            TypeExpr::Applied { origin, .. } => match origin.as_ref() {
                TypeExpr::Universe(name) | TypeExpr::Named(name) => {
                    if let Some(key) = UniverseKey::from_name(name) {
                        let name_str = key.name();
                        quote!(NativeReturnShape::Instance(#name_str))
                    } else {
                        quote!(NativeReturnShape::Unknown)
                    }
                }
                _ => quote!(NativeReturnShape::Unknown),
            },
            _ => quote!(NativeReturnShape::Unknown),
        }
    }
}

fn emit_type_expr_spec(ty: &TypeExpr, binders: &[String]) -> Result<TokenStream2, String> {
    match ty {
        TypeExpr::Unknown => Ok(quote!(&::phalcom_native_meta::TypeExprSpec::Unknown)),
        TypeExpr::Never => Ok(quote!(&::phalcom_native_meta::TypeExprSpec::Never)),
        TypeExpr::SelfType => Ok(quote!(&::phalcom_native_meta::TypeExprSpec::SelfType)),
        TypeExpr::Universe(name) => {
            let key = UniverseKey::from_name(name).ok_or_else(|| format!("unknown universe type `universe.{name}`"))?;
            let key_ident = format_ident!("{}", key.name());
            Ok(quote!(&::phalcom_native_meta::TypeExprSpec::Universe(::phalcom_native_meta::UniverseKey::#key_ident)))
        }
        TypeExpr::Named(name) => {
            if binders.contains(name) {
                let name_str = name.as_str();
                Ok(quote!(&::phalcom_native_meta::TypeExprSpec::Parameter(#name_str)))
            } else if let Some(key) = UniverseKey::from_name(name) {
                let key_ident = format_ident!("{}", key.name());
                Ok(quote!(&::phalcom_native_meta::TypeExprSpec::Universe(::phalcom_native_meta::UniverseKey::#key_ident)))
            } else {
                Err(format!("unknown type name `{name}`"))
            }
        }
        TypeExpr::Parameter(name) => {
            let name_str = name.as_str();
            Ok(quote!(&::phalcom_native_meta::TypeExprSpec::Parameter(#name_str)))
        }
        TypeExpr::Applied { origin, arguments } => {
            let origin_tokens = emit_type_expr_spec(origin, binders)?;
            let mut arg_tokens = Vec::new();
            for arg in arguments {
                let elem = emit_type_expr_spec(arg, binders)?;
                arg_tokens.push(quote!(*#elem));
            }
            Ok(quote!(&::phalcom_native_meta::TypeExprSpec::Applied {
                origin: #origin_tokens,
                arguments: &[#(#arg_tokens),*],
            }))
        }
        TypeExpr::Union(alts) => {
            let mut alt_tokens = Vec::new();
            for alt in alts {
                let elem = emit_type_expr_spec(alt, binders)?;
                alt_tokens.push(quote!(*#elem));
            }
            Ok(quote!(&::phalcom_native_meta::TypeExprSpec::Union(&[#(#alt_tokens),*])))
        }
        TypeExpr::Tuple(tuple) => {
            let tuple_spec = emit_params_spec(tuple, binders)?;
            Ok(quote!(&::phalcom_native_meta::TypeExprSpec::Tuple(#tuple_spec)))
        }
    }
}

fn emit_params_spec(params: &ParameterTuple, binders: &[String]) -> Result<TokenStream2, String> {
    let mut pos_tokens = Vec::new();
    for pos in &params.positional {
        let elem = emit_type_expr_spec(pos, binders)?;
        pos_tokens.push(quote!(*#elem));
    }

    let mut labeled_tokens = Vec::new();
    for labeled in &params.labeled {
        let label_str = &labeled.label;
        let elem = emit_type_expr_spec(&labeled.ty, binders)?;
        labeled_tokens.push(quote!(::phalcom_native_meta::LabeledParameterSpec {
            label: #label_str,
            ty: #elem,
        }));
    }

    let rest_tokens = match &params.rest {
        Some(rest) => {
            let ty_tokens = match &rest.ty {
                Some(ty) => {
                    let elem = emit_type_expr_spec(ty, binders)?;
                    quote!(Some(#elem))
                }
                None => quote!(None),
            };
            quote!(Some(::phalcom_native_meta::RestParameterSpec { ty: #ty_tokens }))
        }
        None => quote!(None),
    };

    Ok(quote!(&::phalcom_native_meta::ParameterTupleSpec {
        positional: &[#(#pos_tokens),*],
        labeled: &[#(#labeled_tokens),*],
        rest: #rest_tokens,
    }))
}

fn emit_callable_spec(callable: &CallableType) -> Result<TokenStream2, String> {
    let binders: Vec<String> = callable.type_params.iter().map(|p| p.name.clone()).collect();
    let mut type_param_tokens = Vec::new();
    for p in &callable.type_params {
        let name_str = &p.name;
        type_param_tokens.push(quote!(::phalcom_native_meta::TypeParameterSpec {
            name: #name_str,
            kind: ::phalcom_native_meta::KindSpec::Type,
        }));
    }

    let params_spec = emit_params_spec(&callable.params, &binders)?;
    let returns_spec = emit_type_expr_spec(&callable.return_type, &binders)?;

    Ok(quote!(&::phalcom_native_meta::CallableTypeSpec {
        type_params: &[#(#type_param_tokens),*],
        params: #params_spec,
        return_type: #returns_spec,
    }))
}

fn format_source(source: &str) -> Result<String, String> {
    let mut child = Command::new("rustfmt")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("failed to spawn rustfmt: {e}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(source.as_bytes())
            .map_err(|e| format!("failed to write to rustfmt stdin: {e}"))?;
    }

    let output = child.wait_with_output().map_err(|e| format!("failed to wait on rustfmt: {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("rustfmt failed: {stderr}"));
    }

    String::from_utf8(output.stdout).map_err(|e| format!("rustfmt output not utf-8: {e}"))
}
