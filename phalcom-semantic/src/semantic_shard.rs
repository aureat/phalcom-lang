//! Retained source-local semantic structure for incremental workspace updates.

use crate::identity::{DeclarationId, ModuleId};
use crate::source::ParsedModuleUnit;
use phalcom_ast::ast::Statement;
use phalcom_modules::declaration::{DeclarationBlueprint, DeclarationKind};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// Source-local declaration structure retained between semantic snapshots.
///
/// This shard contains syntax that can be extracted without solving cross-module
/// meaning. TypeStore forms, resolved aliases, hierarchy edges, and callable
/// products remain owned by the shared semantic workspace and its DB queries.
#[derive(Clone, Debug)]
pub struct ModuleSemanticStructureShard {
    /// Canonical module identity owned by this shard.
    pub module: ModuleId,
    /// Parsed source retained for query inputs and source provenance.
    pub source: Arc<ParsedModuleUnit>,
    /// Stable declaration blueprints needed for predeclaration.
    pub declarations: Arc<[DeclarationBlueprint]>,
    /// Alias identities declared by this module.
    pub aliases: Arc<[DeclarationId]>,
    /// Source identity used to decide whether this shard can be retained.
    pub source_fingerprint: u64,
}

impl ModuleSemanticStructureShard {
    /// Extracts source-local declaration structure once for one parsed module.
    pub fn from_source(source: Arc<ParsedModuleUnit>) -> Arc<Self> {
        let mut declarations = Vec::new();
        let mut aliases = Vec::new();
        for statement in &source.program.statements {
            let (name, kind) = match statement {
                Statement::Class(class_def) => (&class_def.name, DeclarationKind::Class),
                Statement::Enum(enum_def) => (&enum_def.name, DeclarationKind::Class),
                Statement::TypeAlias(alias) => (&alias.name, DeclarationKind::Alias),
                _ => continue,
            };
            let declaration = DeclarationId::new(source.id.clone(), name.clone().into());
            declarations.push(DeclarationBlueprint { id: declaration.clone(), kind });
            if kind == DeclarationKind::Alias {
                aliases.push(declaration);
            }
        }

        let mut hasher = DefaultHasher::new();
        source.id.hash(&mut hasher);
        source.text.hash(&mut hasher);

        Arc::new(Self {
            module: source.id.clone(),
            source,
            declarations: Arc::from(declarations.into_boxed_slice()),
            aliases: Arc::from(aliases.into_boxed_slice()),
            source_fingerprint: hasher.finish(),
        })
    }

    /// Retains source-local declaration structure while replacing the parsed
    /// source used by body and provenance queries. Module interfaces decide
    /// whether this contribution is structurally unchanged; body-only edits
    /// must not force declaration-shard reconstruction.
    pub fn with_source(previous: &Arc<Self>, source: Arc<ParsedModuleUnit>) -> Arc<Self> {
        Arc::new(Self {
            module: previous.module.clone(),
            source,
            declarations: previous.declarations.clone(),
            aliases: previous.aliases.clone(),
            source_fingerprint: previous.source_fingerprint,
        })
    }
}
