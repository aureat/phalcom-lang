//! Canonical module interface fingerprints.
//!
//! Provides deterministic input and product fingerprint hashing for unlinked and
//! linked module interfaces.

use crate::interface::{
    ImportSurface, LinkedExportTarget, LinkedModuleInterface, UnlinkedExportTarget, UnlinkedModuleInterface,
};
use crate::metadata::{MetadataTarget, ModuleMetadata};
use crate::source::ModuleKind;
use phalcom_ast::ast::{ImportPath, ImportRoot, MetadataLiteral};
use phalcom_common::range::SourceRange;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Deterministic semantic product fingerprint for an unlinked module interface.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct InterfaceFingerprint(pub u64);

impl InterfaceFingerprint {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// Deterministic semantic product fingerprint for a linked module interface.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LinkedInterfaceFingerprint(pub u64);

impl LinkedInterfaceFingerprint {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// Deterministic private linkage dependency fingerprint for a linked module.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LinkedDependencyFingerprint(pub u64);

impl LinkedDependencyFingerprint {
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    pub const fn raw(self) -> u64 {
        self.0
    }
}

fn hash_range(range: SourceRange, hasher: &mut impl Hasher) {
    range.start.hash(hasher);
    range.end.hash(hasher);
}

fn hash_module_kind(kind: ModuleKind, hasher: &mut impl Hasher) {
    match kind {
        ModuleKind::Module => 0u8.hash(hasher),
        ModuleKind::Package => 1u8.hash(hasher),
    }
}

fn hash_import_path(path: &ImportPath, include_ranges: bool, hasher: &mut impl Hasher) {
    match &path.root {
        ImportRoot::Absolute(segment) => {
            0u8.hash(hasher);
            segment.name.hash(hasher);
            if include_ranges {
                hash_range(segment.range, hasher);
            }
        }
        ImportRoot::Relative { dots, range } => {
            1u8.hash(hasher);
            dots.hash(hasher);
            if include_ranges {
                hash_range(*range, hasher);
            }
        }
    }
    path.segments.len().hash(hasher);
    for segment in &path.segments {
        segment.name.hash(hasher);
        if include_ranges {
            hash_range(segment.range, hasher);
        }
    }
    if include_ranges {
        hash_range(path.range, hasher);
    }
}

fn hash_metadata_literal(literal: &MetadataLiteral, hasher: &mut impl Hasher) {
    match literal {
        MetadataLiteral::Unit => 0u8.hash(hasher),
        MetadataLiteral::Bool(value) => {
            1u8.hash(hasher);
            value.hash(hasher);
        }
        MetadataLiteral::Int(value) => {
            2u8.hash(hasher);
            value.hash(hasher);
        }
        MetadataLiteral::Float(value) => {
            3u8.hash(hasher);
            value.to_bits().hash(hasher);
        }
        MetadataLiteral::String(value) => {
            4u8.hash(hasher);
            value.hash(hasher);
        }
        MetadataLiteral::Symbol(value) => {
            5u8.hash(hasher);
            value.hash(hasher);
        }
        MetadataLiteral::Tuple(values) => {
            6u8.hash(hasher);
            values.len().hash(hasher);
            for value in values {
                hash_metadata_literal(value, hasher);
            }
        }
        MetadataLiteral::Record(fields) => {
            7u8.hash(hasher);
            fields.len().hash(hasher);
            for (name, value) in fields {
                name.hash(hasher);
                hash_metadata_literal(value, hasher);
            }
        }
    }
}

fn hash_metadata(metadata: &ModuleMetadata, include_ranges: bool, hasher: &mut impl Hasher) {
    metadata.attributes.len().hash(hasher);
    for attribute in &metadata.attributes {
        match attribute.target {
            MetadataTarget::Module => 0u8.hash(hasher),
            MetadataTarget::Package => 1u8.hash(hasher),
            MetadataTarget::Project => 2u8.hash(hasher),
        }
        attribute.name.hash(hasher);
        attribute.arguments.len().hash(hasher);
        for argument in &attribute.arguments {
            hash_metadata_literal(argument, hasher);
        }
        if include_ranges {
            hash_range(attribute.range, hasher);
        }
    }
}

/// Hashes an unlinked module interface with or without source range information.
pub fn hash_unlinked_interface(interface: &UnlinkedModuleInterface, include_ranges: bool, hasher: &mut impl Hasher) {
    interface.id.hash(hasher);
    hash_module_kind(interface.kind, hasher);

    interface.declarations.len().hash(hasher);
    for (name, declaration) in &interface.declarations {
        name.hash(hasher);
        declaration.name.hash(hasher);
        declaration.is_const.hash(hasher);
        if include_ranges {
            hash_range(declaration.range, hasher);
        }
    }

    interface.imports.len().hash(hasher);
    for import in &interface.imports {
        match import {
            ImportSurface::Module(module) => {
                0u8.hash(hasher);
                hash_import_path(&module.path, include_ranges, hasher);
                module.alias.as_ref().map(|alias| alias.name.as_str()).hash(hasher);
                if include_ranges {
                    if let Some(alias) = &module.alias {
                        hash_range(alias.range, hasher);
                    }
                    hash_range(module.range, hasher);
                }
            }
            ImportSurface::Selective(selective) => {
                1u8.hash(hasher);
                hash_import_path(&selective.path, include_ranges, hasher);
                selective.items.len().hash(hasher);
                for item in &selective.items {
                    item.name.hash(hasher);
                    item.alias.as_ref().map(|alias| alias.name.as_str()).hash(hasher);
                    if include_ranges {
                        hash_range(item.name_range, hasher);
                        if let Some(alias) = &item.alias {
                            hash_range(alias.range, hasher);
                        }
                        hash_range(item.range, hasher);
                    }
                }
                if include_ranges {
                    hash_range(selective.range, hasher);
                }
            }
            ImportSurface::ReExport(reexport) => {
                2u8.hash(hasher);
                hash_import_path(&reexport.path, include_ranges, hasher);
                reexport.items.len().hash(hasher);
                for item in &reexport.items {
                    item.local_or_remote_name.hash(hasher);
                    item.alias.as_ref().map(|alias| alias.name.as_str()).hash(hasher);
                    if include_ranges {
                        hash_range(item.name_range, hasher);
                        if let Some(alias) = &item.alias {
                            hash_range(alias.range, hasher);
                        }
                        hash_range(item.range, hasher);
                    }
                }
                if include_ranges {
                    hash_range(reexport.range, hasher);
                }
            }
        }
    }

    interface.exports.len().hash(hasher);
    for (name, export) in &interface.exports {
        name.hash(hasher);
        export.exported_name.hash(hasher);
        export.internal_name.hash(hasher);
        match &export.target {
            UnlinkedExportTarget::Local(local) => {
                0u8.hash(hasher);
                local.hash(hasher);
            }
            UnlinkedExportTarget::ReExport { path, remote } => {
                1u8.hash(hasher);
                hash_import_path(path, include_ranges, hasher);
                remote.hash(hasher);
            }
            UnlinkedExportTarget::CanonicalDeclaration { module, name } => {
                2u8.hash(hasher);
                module.hash(hasher);
                name.hash(hasher);
            }
        }
        if include_ranges {
            hash_range(export.range, hasher);
        }
    }

    interface.exposed_children.len().hash(hasher);
    for child in &interface.exposed_children {
        child.hash(hasher);
    }
    hash_metadata(&interface.metadata, include_ranges, hasher);
}

/// Hashes a linked module interface with or without source range information.
pub fn hash_linked_interface(interface: &LinkedModuleInterface, include_ranges: bool, hasher: &mut impl Hasher) {
    interface.module.hash(hasher);
    hash_module_kind(interface.kind, hasher);
    interface.exports.len().hash(hasher);
    for (name, export) in &interface.exports {
        name.hash(hasher);
        export.public_name.hash(hasher);
        match &export.target {
            LinkedExportTarget::Binding(symbol) => {
                0u8.hash(hasher);
                symbol.hash(hasher);
            }
            LinkedExportTarget::Module(module) => {
                1u8.hash(hasher);
                module.hash(hasher);
            }
        }
        if include_ranges {
            hash_range(export.range, hasher);
        }
    }
    hash_metadata(&interface.metadata, include_ranges, hasher);
}

/// Computes the semantic product fingerprint for an unlinked module interface.
pub fn interface_fingerprint(interface: &UnlinkedModuleInterface) -> InterfaceFingerprint {
    let mut hasher = DefaultHasher::new();
    hash_unlinked_interface(interface, false, &mut hasher);
    InterfaceFingerprint::new(hasher.finish())
}

/// Computes the source/provenance-sensitive input fingerprint for an unlinked module interface.
pub fn unlinked_interface_input_fingerprint(interface: &UnlinkedModuleInterface) -> InterfaceFingerprint {
    let mut hasher = DefaultHasher::new();
    hash_unlinked_interface(interface, true, &mut hasher);
    InterfaceFingerprint::new(hasher.finish())
}

/// Computes the semantic product fingerprint for a linked module interface.
pub fn linked_interface_fingerprint(interface: &LinkedModuleInterface) -> LinkedInterfaceFingerprint {
    let mut hasher = DefaultHasher::new();
    hash_linked_interface(interface, false, &mut hasher);
    LinkedInterfaceFingerprint::new(hasher.finish())
}

/// Computes the source/provenance-sensitive input fingerprint for a linked module interface.
pub fn linked_interface_input_fingerprint(interface: &LinkedModuleInterface) -> LinkedInterfaceFingerprint {
    let mut hasher = DefaultHasher::new();
    hash_linked_interface(interface, true, &mut hasher);
    LinkedInterfaceFingerprint::new(hasher.finish())
}

/// Computes the private linkage dependency fingerprint for a linked module.
pub fn linked_dependency_fingerprint(module: &crate::linker::LinkedModule) -> LinkedDependencyFingerprint {
    let mut hasher = DefaultHasher::new();
    module.interface.module.hash(&mut hasher);

    module.bindings.local_globals.len().hash(&mut hasher);
    for (name, id) in &module.bindings.local_globals {
        name.hash(&mut hasher);
        id.0.hash(&mut hasher);
    }

    module.bindings.imports.len().hash(&mut hasher);
    for (name, id) in &module.bindings.imports {
        name.hash(&mut hasher);
        id.0.hash(&mut hasher);
    }

    module.linked_reads.len().hash(&mut hasher);
    for read in &module.linked_reads {
        match read {
            crate::linker::LinkedReadSpec::Module(id) => {
                0u8.hash(&mut hasher);
                id.hash(&mut hasher);
            }
            crate::linker::LinkedReadSpec::Binding(symbol) => {
                1u8.hash(&mut hasher);
                symbol.module.hash(&mut hasher);
                symbol.name.hash(&mut hasher);
            }
        }
    }

    module.runtime_dependencies.len().hash(&mut hasher);
    for dep in &module.runtime_dependencies {
        dep.hash(&mut hasher);
    }

    LinkedDependencyFingerprint::new(hasher.finish())
}
