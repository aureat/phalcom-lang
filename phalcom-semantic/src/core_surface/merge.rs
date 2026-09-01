//! Surface merge between source and native declarations.

use super::source::{SourceClassRecord, SourceDeclarationRecord, SourceEnumRecord, SourceMemberRecord, SourceNativeBindingRole};
use crate::identity::{DeclarationId, DispatchSide};
use phalcom_native_surface::NativeSurfaceRecord;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SurfaceMergeOutcome<'a> {
    SourceOnly(&'a SourceMemberRecord),
    NativeOnly(&'a NativeSurfaceRecord),
    Generated(&'a NativeSurfaceRecord),
    SourceDeclarationNativeImplementation {
        source: &'a SourceMemberRecord,
        native: &'a NativeSurfaceRecord,
    },
    SourceWrapperOverNative {
        source: &'a SourceMemberRecord,
        native: &'a NativeSurfaceRecord,
    },
    Conflict {
        source: &'a SourceMemberRecord,
        native: &'a NativeSurfaceRecord,
        reason: &'static str,
    },
}

#[derive(Clone, Debug)]
pub enum MergedDeclarationSource<'a> {
    Class(&'a SourceClassRecord),
    Enum(&'a SourceEnumRecord),
}

#[derive(Clone, Debug)]
pub struct MergedDeclarationSurface<'a> {
    pub declaration_id: DeclarationId,
    pub name: String,
    pub superclass: Option<String>,
    pub source: Option<MergedDeclarationSource<'a>>,
    pub members: BTreeMap<(DispatchSide, String), SurfaceMergeOutcome<'a>>,
}

impl<'a> MergedDeclarationSurface<'a> {
    pub fn source_class(&self) -> Option<&'a SourceClassRecord> {
        match self.source {
            Some(MergedDeclarationSource::Class(c)) => Some(c),
            _ => None,
        }
    }

    pub fn source_enum(&self) -> Option<&'a SourceEnumRecord> {
        match self.source {
            Some(MergedDeclarationSource::Enum(e)) => Some(e),
            _ => None,
        }
    }

    pub fn doc_comment(&self) -> Option<&str> {
        match self.source {
            Some(MergedDeclarationSource::Class(c)) => c.doc_comment.as_deref(),
            Some(MergedDeclarationSource::Enum(e)) => e.doc_comment.as_deref(),
            None => None,
        }
    }
}

pub type MergedClassSurface<'a> = MergedDeclarationSurface<'a>;

/// Merges source declaration records (classes and enums) and native surface records.
pub fn merge_surfaces<'a>(source_declarations: &'a [SourceDeclarationRecord], native_records: &'a [NativeSurfaceRecord]) -> Vec<MergedDeclarationSurface<'a>> {
    let mut by_id: BTreeMap<DeclarationId, MergedDeclarationSurface<'a>> = BTreeMap::new();

    // 1. Ingest source declarations
    for s_decl in source_declarations {
        match s_decl {
            SourceDeclarationRecord::Class(s_class) => {
                let entry = by_id.entry(s_class.declaration_id.clone()).or_insert_with(|| MergedDeclarationSurface {
                    declaration_id: s_class.declaration_id.clone(),
                    name: s_class.name.clone(),
                    superclass: s_class.superclass.clone(),
                    source: Some(MergedDeclarationSource::Class(s_class)),
                    members: BTreeMap::new(),
                });
                entry.source = Some(MergedDeclarationSource::Class(s_class));
                if entry.superclass.is_none() {
                    entry.superclass = s_class.superclass.clone();
                }

                for m in &s_class.members {
                    let key = (m.side, m.selector_raw.clone());
                    entry.members.insert(key, SurfaceMergeOutcome::SourceOnly(m));
                }
            }
            SourceDeclarationRecord::Enum(s_enum) => {
                let entry = by_id.entry(s_enum.declaration_id.clone()).or_insert_with(|| MergedDeclarationSurface {
                    declaration_id: s_enum.declaration_id.clone(),
                    name: s_enum.name.clone(),
                    superclass: None,
                    source: Some(MergedDeclarationSource::Enum(s_enum)),
                    members: BTreeMap::new(),
                });
                entry.source = Some(MergedDeclarationSource::Enum(s_enum));

                for m in &s_enum.members {
                    let key = (m.side, m.selector_raw.clone());
                    entry.members.insert(key, SurfaceMergeOutcome::SourceOnly(m));
                }
            }
        }
    }

    // 2. Ingest native records
    for n_rec in native_records {
        let owner_name = n_rec.owner().name();
        let side = match n_rec.side() {
            phalcom_native_meta::NativeDispatch::Instance => DispatchSide::Instance,
            phalcom_native_meta::NativeDispatch::Class => DispatchSide::Class,
        };
        let key = (side, n_rec.selector().to_string());

        let native_declaration = crate::core_surface::universe_declaration(n_rec.owner());
        let entry = by_id.entry(native_declaration.clone()).or_insert_with(|| MergedDeclarationSurface {
            declaration_id: native_declaration,
            name: owner_name.to_string(),
            superclass: None,
            source: None,
            members: BTreeMap::new(),
        });

        match entry.members.get(&key) {
            Some(SurfaceMergeOutcome::SourceOnly(s_mem)) => {
                let outcome = match s_mem.binding_role {
                    SourceNativeBindingRole::DeclarationImplementation => {
                        SurfaceMergeOutcome::SourceDeclarationNativeImplementation { source: s_mem, native: n_rec }
                    }
                    SourceNativeBindingRole::WrapperOverNative => SurfaceMergeOutcome::SourceWrapperOverNative { source: s_mem, native: n_rec },
                    SourceNativeBindingRole::None => SurfaceMergeOutcome::Conflict {
                        source: s_mem,
                        native: n_rec,
                        reason: "source/native collision has no explicit binding role",
                    },
                };
                entry.members.insert(key, outcome);
            }
            None => {
                entry.members.insert(key, SurfaceMergeOutcome::NativeOnly(n_rec));
            }
            _ => {}
        }
    }

    let mut result: Vec<_> = by_id.into_values().collect();
    result.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.declaration_id.cmp(&b.declaration_id)));
    result
}
