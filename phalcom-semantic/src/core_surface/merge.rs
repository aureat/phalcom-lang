//! Surface merge between source and native declarations.

use super::source::{SourceClassRecord, SourceMemberRecord};
use crate::identity::{DeclarationId, DispatchSide};
use phalcom_native_surface::NativeSurfaceRecord;
use std::collections::HashMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SurfaceMergeOutcome<'a> {
    SourceOnly(&'a SourceMemberRecord),
    NativeOnly(&'a NativeSurfaceRecord),
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
pub struct MergedClassSurface<'a> {
    pub declaration_id: DeclarationId,
    pub name: String,
    pub superclass: Option<String>,
    pub source_class: Option<&'a SourceClassRecord>,
    pub members: HashMap<(DispatchSide, String), SurfaceMergeOutcome<'a>>,
}

/// Merges source class records and native surface records.
pub fn merge_surfaces<'a>(source_classes: &'a [SourceClassRecord], native_records: &'a [NativeSurfaceRecord]) -> Vec<MergedClassSurface<'a>> {
    let mut by_name: HashMap<String, MergedClassSurface<'a>> = HashMap::new();

    // 1. Ingest source classes
    for s_class in source_classes {
        let entry = by_name.entry(s_class.name.clone()).or_insert_with(|| MergedClassSurface {
            declaration_id: s_class.declaration_id.clone(),
            name: s_class.name.clone(),
            superclass: s_class.superclass.clone(),
            source_class: Some(s_class),
            members: HashMap::new(),
        });
        entry.source_class = Some(s_class);
        if entry.superclass.is_none() {
            entry.superclass = s_class.superclass.clone();
        }

        for m in &s_class.members {
            let key = (m.side, m.selector_raw.clone());
            entry.members.insert(key, SurfaceMergeOutcome::SourceOnly(m));
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

        let entry = by_name.entry(owner_name.to_string()).or_insert_with(|| MergedClassSurface {
            declaration_id: DeclarationId::new(crate::identity::ModuleId::core(), owner_name.into()),
            name: owner_name.to_string(),
            superclass: None,
            source_class: None,
            members: HashMap::new(),
        });

        match entry.members.get(&key) {
            Some(SurfaceMergeOutcome::SourceOnly(s_mem)) => {
                let outcome = SurfaceMergeOutcome::SourceDeclarationNativeImplementation { source: *s_mem, native: n_rec };
                entry.members.insert(key, outcome);
            }
            None => {
                entry.members.insert(key, SurfaceMergeOutcome::NativeOnly(n_rec));
            }
            _ => {}
        }
    }

    let mut result: Vec<_> = by_name.into_values().collect();
    result.sort_by(|a, b| a.name.cmp(&b.name));
    result
}
