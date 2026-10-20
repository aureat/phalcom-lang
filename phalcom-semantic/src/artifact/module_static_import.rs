//! Certified PERF003 P7 module-static binding join adapted to ARTF001.
//!
//! P7 materialized a live `ModuleStaticBindingIndex` by deep-cloning each
//! callable binding state. ARTF001 already stages that checked state once in
//! `ImportedCallableBindingAuthority`, so this module retains only the exact
//! relationship from stable module symbols to those canonical binding rows.
//! T4 owns the single live owning materialization.

use super::callable_binding_import::{ImportedCallableBindingAuthority, ImportedCallableBindingSourceAuthority};
use super::identity::StableOwnerCatalog;
use crate::identity::{BindingId, CallableId, ModuleId};
use crate::module_bindings::ModuleBindingPhase;
use phalcom_common::range::SourceRange;
use phalcom_compiler_artifact::semantic::world::{ModuleStaticBindingIndexV1, ModuleStaticBindingPhaseV1, StaticConstantSymbolV1};
use phalcom_compiler_artifact::{ArtifactError, ValidationError};
use phalcom_modules::linker::SymbolId;
use phalcom_type_meta::allocation::AllocationBudget;

/// One checked module binding, referencing the unique imported callable-binding
/// state rather than copying it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportedModuleStaticBindingFact {
    pub symbol: SymbolId,
    pub source_body: CallableId,
    pub source_binding: BindingId,
    pub phase: ModuleBindingPhase,
    pub declaration_range: SourceRange,
    pub initializer_range: Option<SourceRange>,
}

/// One module's stable binding/visible-name layout.
///
/// Both slices are sorted and duplicate-free. Visible names point only at
/// symbols present in `bindings`.
#[derive(Clone, Debug)]
pub struct ImportedModuleStaticBindingIndex {
    pub module: ModuleId,
    pub bindings: Box<[ImportedModuleStaticBindingFact]>,
    pub visible_names: Box<[(Box<str>, SymbolId)]>,
}

/// Snapshot authority for all imported module-static bindings.
#[derive(Clone, Debug, Default)]
pub struct ImportedModuleStaticBindingAuthority {
    modules: Box<[ImportedModuleStaticBindingIndex]>,
}

impl ImportedModuleStaticBindingAuthority {
    pub fn module_len(&self) -> usize {
        self.modules.len()
    }

    pub fn binding_len(&self) -> usize {
        self.modules.iter().map(|module| module.bindings.len()).sum()
    }

    pub fn visible_name_len(&self) -> usize {
        self.modules.iter().map(|module| module.visible_names.len()).sum()
    }

    pub(crate) fn module(&self, module: &ModuleId) -> Option<&ImportedModuleStaticBindingIndex> {
        self.modules
            .binary_search_by(|candidate| candidate.module.cmp(module))
            .ok()
            .map(|index| &self.modules[index])
    }
}

/// Port of certified P7 `import_module_static_binding_indexes`.
///
/// The old importer duplicated `BindingState` into each module fact. Here the
/// exact `(source_body, source_binding)` relation is validated against the
/// canonical callable-binding staging authority and retained for T4.
pub(crate) fn reconstruct_module_static_bindings(
    records: &[ModuleStaticBindingIndexV1],
    callable_bindings: &ImportedCallableBindingAuthority,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<ImportedModuleStaticBindingAuthority, ArtifactError> {
    let mut modules = allocations
        .vector::<ImportedModuleStaticBindingIndex>(records.len())
        .map_err(ArtifactError::Allocation)?;

    for record in records {
        let module = owners.module(&record.module)?;
        let mut bindings = allocations
            .vector::<ImportedModuleStaticBindingFact>(record.bindings.len())
            .map_err(ArtifactError::Allocation)?;

        for binding in &record.bindings {
            let source_body = owners.callable(&binding.source_body)?;
            let source_row = callable_bindings
                .binding(&source_body, binding.source_binding_ordinal)
                .ok_or_else(|| invalid("module static binding references an absent callable binding row"))?;
            if !matches!(
                source_row.source_authority,
                ImportedCallableBindingSourceAuthority::BodyLocalSource(_) | ImportedCallableBindingSourceAuthority::BodyLocalGenerated(_)
            ) {
                return Err(invalid("module static binding does not resolve to a body-local binding authority"));
            }

            let declaration_range = source_range(&binding.declaration_source, "module static-binding declaration")?;
            let initializer_range = binding
                .initializer_source
                .as_ref()
                .map(|range| source_range(range, "module static-binding initializer"))
                .transpose()?;

            bindings.push(ImportedModuleStaticBindingFact {
                symbol: static_symbol(&binding.symbol, owners, allocations)?,
                source_body,
                source_binding: source_row.state.binding,
                phase: match binding.phase {
                    ModuleStaticBindingPhaseV1::StaticConstant => ModuleBindingPhase::StaticConstant,
                    ModuleStaticBindingPhaseV1::Runtime => ModuleBindingPhase::Runtime,
                },
                declaration_range,
                initializer_range,
            });
        }

        bindings.sort_by(|left, right| left.symbol.cmp(&right.symbol));
        if bindings.windows(2).any(|pair| pair[0].symbol == pair[1].symbol) {
            return Err(invalid("module static-binding symbol is duplicated after stable-owner relocation"));
        }

        let mut visible_names = allocations
            .vector::<(Box<str>, SymbolId)>(record.visible_names.len())
            .map_err(ArtifactError::Allocation)?;
        for visible in &record.visible_names {
            let name = allocations.boxed_str(&visible.name).map_err(ArtifactError::Allocation)?;
            let symbol = static_symbol(&visible.symbol, owners, allocations)?;
            if bindings.binary_search_by(|binding| binding.symbol.cmp(&symbol)).is_err() {
                return Err(invalid("module visible static-binding name references an absent binding fact"));
            }
            visible_names.push((name, symbol));
        }
        visible_names.sort_by(|left, right| left.0.cmp(&right.0));
        if visible_names.windows(2).any(|pair| pair[0].0 == pair[1].0) {
            return Err(invalid("module visible static-binding name is duplicated after relocation"));
        }

        modules.push(ImportedModuleStaticBindingIndex {
            module,
            bindings: allocations.boxed_slice(bindings).map_err(ArtifactError::Allocation)?,
            visible_names: allocations.boxed_slice(visible_names).map_err(ArtifactError::Allocation)?,
        });
    }

    modules.sort_by(|left, right| left.module.cmp(&right.module));
    if modules.windows(2).any(|pair| pair[0].module == pair[1].module) {
        return Err(invalid("module static-binding index owner is duplicated after stable-owner relocation"));
    }

    Ok(ImportedModuleStaticBindingAuthority {
        modules: allocations.boxed_slice(modules).map_err(ArtifactError::Allocation)?,
    })
}

fn static_symbol(symbol: &StaticConstantSymbolV1, owners: &StableOwnerCatalog<'_>, allocations: &mut AllocationBudget) -> Result<SymbolId, ArtifactError> {
    Ok(SymbolId {
        module: owners.module(&symbol.module)?,
        name: allocations.boxed_str(&symbol.name).map_err(ArtifactError::Allocation)?,
    })
}

fn source_range(range: &phalcom_compiler_artifact::module_world::SpanV1, family: &'static str) -> Result<SourceRange, ArtifactError> {
    if range.start > range.end {
        return Err(invalid(format!("{family} range is inverted")));
    }
    Ok(SourceRange {
        start: usize::try_from(range.start).map_err(|_| invalid(format!("{family} start exceeds address space")))?,
        end: usize::try_from(range.end).map_err(|_| invalid(format!("{family} end exceeds address space")))?,
    })
}

fn invalid(message: impl Into<String>) -> ArtifactError {
    ValidationError::ReferentialIntegrity(message.into()).into()
}
