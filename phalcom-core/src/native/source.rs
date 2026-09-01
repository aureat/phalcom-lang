//! Parsed AST source index for native universe classes and members.

use phalcom_ast::ast::{ClassDef, ClassMember, DependencyDecl, ImportDecl, ImportPath, ImportRoot, MemberBody, MethodDef, Program, RestMode, Statement};
use phalcom_common::range::SourceRange;
use phalcom_modules::builtin::UniverseSourceProvider;
use phalcom_modules::identity::{ModuleId, ModulePath};
use phalcom_modules::source::{ModuleKind, ParsedModuleUnit};
use phalcom_native_meta::{NativeDispatch, NativeVisibility, UniverseKey};
use std::collections::HashMap;
use std::sync::Arc;

/// Deterministic VM-free record for one canonical universe source module.
#[derive(Clone, Debug)]
pub struct UniverseModuleRow {
    pub module: ModuleId,
    pub kind: ModuleKind,
    pub source: Option<phalcom_modules::identity::SourceLocation>,
    pub parse_succeeded: bool,
    pub documented: bool,
}

/// Deterministic VM-free record for one class presentation in the universe
/// source corpus. `universe_key` is present for runtime classes and absent for
/// source-only helper classes.
#[derive(Clone, Debug)]
pub struct UniverseClassRow {
    pub module: ModuleId,
    pub name: String,
    pub range: SourceRange,
    pub universe_key: Option<UniverseKey>,
    pub native: bool,
    pub superclass: Option<String>,
    pub documented: bool,
}

/// Deterministic VM-free record for one authored class member.
#[derive(Clone, Debug)]
pub struct UniverseMemberRow {
    pub module: ModuleId,
    pub owner: Option<UniverseKey>,
    pub class_name: String,
    pub side: NativeDispatch,
    pub selector: String,
    pub native: bool,
    pub internal: bool,
    pub declaration_only: bool,
    pub typed: bool,
    pub documented: bool,
    pub range: SourceRange,
}

/// Complete source-corpus census used by migration reports and bootstrap
/// preflight. It deliberately contains no VM handles or executable state.
#[derive(Clone, Debug, Default)]
pub struct UniverseSourceCensus {
    pub modules: Vec<UniverseModuleRow>,
    pub classes: Vec<UniverseClassRow>,
    pub members: Vec<UniverseMemberRow>,
}

/// An AST anchor for a `@native` class declaration.
#[derive(Clone, Debug)]
pub struct NativeClassAnchor {
    pub name: String,
    pub universe_key: UniverseKey,
    pub superclass: Option<String>,
    pub range: SourceRange,
}

/// An AST anchor for a `@native` method, getter, setter, or field.
#[derive(Clone, Debug)]
pub struct NativeMemberAnchor {
    pub key: NativeMemberKey,
    pub class_name: String,
    pub side: NativeDispatch,
    pub selector: String,
    pub visibility: NativeVisibility,
    pub range: SourceRange,
    pub is_declaration: bool,
    pub typed: bool,
}

/// Owned source-side key for a native member.
///
/// Rust descriptors use `&'static str` so they can live in the distributed
/// registry. Source indexes are built from parsed documents and must not leak
/// every selector into the process lifetime merely to reuse that descriptor
/// representation.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct NativeMemberKey {
    pub owner: UniverseKey,
    pub side: NativeDispatch,
    pub selector: String,
}

/// Index of all `@native` classes and members in the canonical universe source tree.
#[derive(Clone, Debug, Default)]
pub struct NativeSourceIndex {
    /// All provider-parsed canonical universe units. Bootstrap compiles these
    /// same units after verification instead of reparsing source text.
    pub units: Vec<Arc<ParsedModuleUnit>>,
    /// Full VM-free migration census for the bundled source corpus.
    pub census: UniverseSourceCensus,
    /// One canonical source presentation for each runtime universe class.
    pub presentations: HashMap<UniverseKey, UniverseClassRow>,
    /// Native class anchors, retained for compatibility with existing native
    /// verifier consumers.
    pub classes: HashMap<UniverseKey, NativeClassAnchor>,
    /// Native member anchors keyed by canonical owner/side/selector identity.
    pub members: HashMap<NativeMemberKey, NativeMemberAnchor>,
}

/// Canonical source index name used by bootstrap and tooling.
pub type UniverseSourceIndex = NativeSourceIndex;

impl NativeSourceIndex {
    /// Builds a new index by scanning all parsed units from the canonical universe project.
    pub fn build() -> Result<Self, String> {
        let provider = UniverseSourceProvider::new();
        let mut index = Self::default();

        for node in provider.nodes() {
            let path = ModulePath::from_components(
                node.path
                    .iter()
                    .map(|p| phalcom_modules::ModuleComponent::from_identifier(p).expect("valid component"))
                    .collect::<Vec<_>>(),
            );
            let module_id = ModuleId::universe(path);
            let parsed = provider
                .load_parsed(&module_id)
                .map_err(|e| format!("failed to load parsed universe module {module_id}: {e}"))?;

            index.index_unit(parsed)?;
        }

        Ok(index)
    }

    /// Returns canonical source units in dependency-first initialization order.
    ///
    /// Bootstrap source is already parsed and indexed here, so this derives
    /// only the small runtime graph represented by each unit's preamble. It
    /// deliberately does not use provider enumeration order.
    pub fn initialization_order(&self) -> Result<Vec<Arc<ParsedModuleUnit>>, String> {
        let by_id = self
            .units
            .iter()
            .enumerate()
            .map(|(index, unit)| (unit.id.clone(), index))
            .collect::<HashMap<_, _>>();
        let mut indegree = vec![0usize; self.units.len()];
        let mut dependents = vec![Vec::<usize>::new(); self.units.len()];

        for (importer_index, unit) in self.units.iter().enumerate() {
            let mut dependencies = std::collections::BTreeSet::new();
            for dependency in &unit.program.preamble.dependencies {
                let path = match dependency {
                    DependencyDecl::Import(ImportDecl::Module(decl)) => Some(&decl.path),
                    DependencyDecl::Import(ImportDecl::Selective(decl)) => Some(&decl.path),
                    DependencyDecl::ReExport(decl) => Some(&decl.path),
                    DependencyDecl::Expose(_) => None,
                };
                let Some(path) = path else { continue };
                let Some(target) = universe_dependency_target(unit, path) else { continue };
                let Some(&target_index) = by_id.get(&target) else {
                    return Err(format!("Universe dependency {} referenced by {} is not materialized", target, unit.id));
                };
                if dependencies.insert(target_index) {
                    indegree[importer_index] += 1;
                    dependents[target_index].push(importer_index);
                }
            }
        }

        let mut ready = std::collections::BTreeSet::new();
        for (index, degree) in indegree.iter().enumerate() {
            if *degree == 0 {
                ready.insert((self.units[index].id.clone(), index));
            }
        }

        let mut order = Vec::with_capacity(self.units.len());
        while let Some((_, index)) = ready.pop_first() {
            order.push(self.units[index].clone());
            for dependent in &dependents[index] {
                indegree[*dependent] -= 1;
                if indegree[*dependent] == 0 {
                    ready.insert((self.units[*dependent].id.clone(), *dependent));
                }
            }
        }
        if order.len() != self.units.len() {
            return Err("canonical Universe source initialization cycle".into());
        }
        Ok(order)
    }

    /// Builds an index from one parsed program.
    ///
    /// Kept public so verifier tests can exercise source contracts without
    /// changing the bundled universe corpus.
    pub fn from_program(program: &Program) -> Result<Self, String> {
        Self::from_program_at(&ModuleId::universe_root(), program)
    }

    /// Builds an index from one parsed program with its canonical module
    /// identity. This is the fixture/test equivalent of the provider-backed
    /// path used by [`Self::build`].
    pub fn from_program_at(module: &ModuleId, program: &Program) -> Result<Self, String> {
        let mut index = Self::default();
        index.index_program(module, ModuleKind::Module, None, program)?;
        Ok(index)
    }

    fn index_unit(&mut self, parsed: Arc<ParsedModuleUnit>) -> Result<(), String> {
        self.census.modules.push(UniverseModuleRow {
            module: parsed.id.clone(),
            kind: parsed.kind,
            source: parsed.source.clone(),
            parse_succeeded: true,
            documented: parsed.program.preamble.metadata.iter().any(|attribute| attribute.name == "documentation"),
        });
        self.index_program(&parsed.id, parsed.kind, parsed.source.clone(), &parsed.program)?;
        self.units.push(parsed);
        Ok(())
    }

    fn index_program(
        &mut self,
        module: &ModuleId,
        kind: ModuleKind,
        source: Option<phalcom_modules::identity::SourceLocation>,
        program: &Program,
    ) -> Result<(), String> {
        if self.census.modules.iter().all(|row| row.module != *module) {
            self.census.modules.push(UniverseModuleRow {
                module: module.clone(),
                kind,
                source,
                parse_succeeded: true,
                documented: program.preamble.metadata.iter().any(|attribute| attribute.name == "documentation"),
            });
        }
        for stmt in &program.statements {
            match stmt {
                Statement::Class(class_def) => self.index_class(module, class_def)?,
                Statement::Enum(enum_def) => self.index_enum(module, enum_def)?,
                _ => {}
            }
        }
        Ok(())
    }

    fn index_enum(&mut self, module: &ModuleId, enum_def: &phalcom_ast::ast::EnumDef) -> Result<(), String> {
        let has_native_attr = enum_def.attributes.iter().any(|a| a.name == "native");
        let named_key = UniverseKey::from_name(&enum_def.name);
        let universe_key = named_key.filter(|key| {
            let expected = ModulePath::from_components(
                key.source_path()
                    .iter()
                    .map(|part| phalcom_modules::ModuleComponent::from_identifier(part).expect("valid component"))
                    .collect::<Vec<_>>(),
            );
            module.path == expected
        });
        if has_native_attr && named_key.is_some() && universe_key.is_none() {
            return Err(format!("native enum {} is declared outside canonical source module", enum_def.name));
        }

        if let Some(key) = universe_key {
            let row = UniverseClassRow {
                module: module.clone(),
                name: enum_def.name.clone(),
                range: enum_def.range,
                universe_key: Some(key),
                native: has_native_attr,
                superclass: Some("Object".to_string()),
                documented: false,
            };
            if self.census.classes.iter().any(|existing| existing.universe_key == Some(key)) {
                return Err(format!("duplicate universe class presentation for {}", key.name()));
            }
            self.presentations.insert(key, row.clone());
            self.census.classes.push(row);
        } else {
            self.census.classes.push(UniverseClassRow {
                module: module.clone(),
                name: enum_def.name.clone(),
                range: enum_def.range,
                universe_key: None,
                native: has_native_attr,
                superclass: Some("Object".to_string()),
                documented: false,
            });
        }

        if has_native_attr {
            if let Some(uk) = universe_key {
                let anchor = NativeClassAnchor {
                    name: enum_def.name.clone(),
                    universe_key: uk,
                    superclass: Some("Object".to_string()),
                    range: enum_def.range,
                };
                if self.classes.insert(uk, anchor).is_some() {
                    return Err(format!("duplicate @native class anchor for {}", uk.name()));
                }
            }
        }

        let Some(owner_key) = universe_key else {
            return Ok(());
        };

        for member in &enum_def.members {
            match member {
                phalcom_ast::ast::EnumMember::Behavior(b) => {
                    let class_member = match b {
                        phalcom_ast::ast::EnumBehaviorMember::Method(m) => ClassMember::Method(m.clone()),
                        phalcom_ast::ast::EnumBehaviorMember::Getter(g) => ClassMember::Getter(g.clone()),
                        phalcom_ast::ast::EnumBehaviorMember::Setter(s) => ClassMember::Setter(s.clone()),
                        phalcom_ast::ast::EnumBehaviorMember::Index(i) => ClassMember::Index(i.clone()),
                    };
                    self.index_member(module, owner_key, &enum_def.name, &class_member)?;
                }
                phalcom_ast::ast::EnumMember::Variant(variant) => {
                    // Only Native Option variants have support-class
                    // presentations and implicit constructor descriptors.
                    // Result and Ordering variants are semantic ADT variants,
                    // not top-level Universe classes.
                    if owner_key != UniverseKey::Option {
                        continue;
                    }
                    let Some(variant_key) = UniverseKey::from_name(&variant.name) else {
                        continue;
                    };
                    let row = UniverseClassRow {
                        module: module.clone(),
                        name: variant.name.clone(),
                        range: variant.range,
                        universe_key: Some(variant_key),
                        native: has_native_attr,
                        superclass: Some("Option".to_owned()),
                        documented: false,
                    };
                    if self.presentations.insert(variant_key, row.clone()).is_some() {
                        return Err(format!("duplicate universe variant presentation for {}", variant.name));
                    }
                    self.census.classes.push(row);

                    // Native Option's payload variant has hidden runtime
                    // constructor surfaces on its support class. They are
                    // generated descriptors, so retain matching source anchors
                    // even though constructors are implicit in enum syntax.
                    if !has_native_attr || variant.payload.is_none() {
                        continue;
                    }
                    for selector in ["call(_)".to_owned(), "new(_)".to_owned()] {
                        let key = NativeMemberKey {
                            owner: variant_key,
                            side: NativeDispatch::Class,
                            selector: selector.clone(),
                        };
                        self.census.members.push(UniverseMemberRow {
                            module: module.clone(),
                            owner: Some(variant_key),
                            class_name: variant.name.clone(),
                            side: NativeDispatch::Class,
                            selector: selector.clone(),
                            native: true,
                            internal: false,
                            declaration_only: false,
                            typed: true,
                            documented: false,
                            range: variant.range,
                        });
                        let anchor = NativeMemberAnchor {
                            key: key.clone(),
                            class_name: variant.name.clone(),
                            side: NativeDispatch::Class,
                            selector,
                            visibility: NativeVisibility::Public,
                            range: variant.range,
                            is_declaration: false,
                            typed: true,
                        };
                        if self.members.insert(key.clone(), anchor).is_some() {
                            return Err(format!("duplicate implicit native variant anchor for {}", key.owner.name()));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn index_class(&mut self, module: &ModuleId, class_def: &ClassDef) -> Result<(), String> {
        let has_native_attr = class_def.attributes.iter().any(|a| a.name == "native");
        let named_key = UniverseKey::from_name(&class_def.name);
        let universe_key = named_key.filter(|key| {
            let expected = ModulePath::from_components(
                key.source_path()
                    .iter()
                    .map(|part| phalcom_modules::ModuleComponent::from_identifier(part).expect("valid component"))
                    .collect::<Vec<_>>(),
            );
            module.path == expected
        });
        if has_native_attr && named_key.is_some() && universe_key.is_none() {
            return Err(format!("native class {} is declared outside canonical source module", class_def.name));
        }
        let superclass = class_def.superclass_ref().map(|reference| reference.leaf_name().to_owned());

        if let Some(key) = universe_key {
            let row = UniverseClassRow {
                module: module.clone(),
                name: class_def.name.clone(),
                range: class_def.range,
                universe_key: Some(key),
                native: has_native_attr,
                superclass,
                documented: false,
            };
            if self.census.classes.iter().any(|existing| existing.universe_key == Some(key)) {
                return Err(format!("duplicate universe class presentation for {}", key.name()));
            }
            self.presentations.insert(key, row.clone());
            self.census.classes.push(row);
        } else {
            self.census.classes.push(UniverseClassRow {
                module: module.clone(),
                name: class_def.name.clone(),
                range: class_def.range,
                universe_key: None,
                native: has_native_attr,
                superclass,
                documented: false,
            });
        }

        if has_native_attr {
            if let Some(uk) = universe_key {
                let anchor = NativeClassAnchor {
                    name: class_def.name.clone(),
                    universe_key: uk,
                    superclass: class_def.superclass_ref().map(|reference| reference.leaf_name().to_owned()),
                    range: class_def.range,
                };
                if self.classes.insert(uk, anchor).is_some() {
                    return Err(format!("duplicate @native class anchor for {}", uk.name()));
                }
            }
        }

        let Some(owner_key) = universe_key else {
            return Ok(());
        };

        for member in &class_def.members {
            self.index_member(module, owner_key, &class_def.name, member)?;
        }

        Ok(())
    }

    fn index_member(&mut self, module: &ModuleId, owner_key: UniverseKey, class_name: &str, member: &ClassMember) -> Result<(), String> {
        let (is_native, has_internal, is_internal, side, selector, is_declaration, typed, range) = match member {
            ClassMember::Method(m) => {
                let is_native = m.attributes.iter().any(|a| a.name == "native");
                let has_internal = m.attributes.iter().any(|a| a.name == "internal");
                let is_internal = has_internal || m.name.starts_with("_$");
                let side = if m.is_static || m.attributes.iter().any(|a| a.name == "class") {
                    NativeDispatch::Class
                } else {
                    NativeDispatch::Instance
                };
                let selector = source_method_selector(m);
                let is_declaration = matches!(m.body, MemberBody::Declaration);
                let typed = m.return_annotation.is_some() && m.params.iter().all(|param| param.annotation.is_some());
                (is_native, has_internal, is_internal, side, selector, is_declaration, typed, m.range)
            }
            ClassMember::Getter(g) => {
                let is_native = g.attributes.iter().any(|a| a.name == "native");
                let has_internal = g.attributes.iter().any(|a| a.name == "internal");
                let is_internal = has_internal || g.name.starts_with("_$");
                let side = if g.is_static || g.attributes.iter().any(|a| a.name == "class") {
                    NativeDispatch::Class
                } else {
                    NativeDispatch::Instance
                };
                let selector = phalcom_ast::selector::selector_from_getter(g).encode();
                let is_declaration = matches!(g.body, MemberBody::Declaration);
                (
                    is_native,
                    has_internal,
                    is_internal,
                    side,
                    selector,
                    is_declaration,
                    g.return_annotation.is_some(),
                    g.range,
                )
            }
            ClassMember::Setter(s) => {
                let is_native = s.attributes.iter().any(|a| a.name == "native");
                let has_internal = s.attributes.iter().any(|a| a.name == "internal");
                let is_internal = has_internal || s.name.starts_with("_$");
                let side = if s.is_static || s.attributes.iter().any(|a| a.name == "class") {
                    NativeDispatch::Class
                } else {
                    NativeDispatch::Instance
                };
                let selector = phalcom_ast::selector::selector_from_setter(s).encode();
                let is_declaration = matches!(s.body, MemberBody::Declaration);
                (
                    is_native,
                    has_internal,
                    is_internal,
                    side,
                    selector,
                    is_declaration,
                    s.param.annotation.is_some() && s.return_annotation.is_some(),
                    s.range,
                )
            }
            ClassMember::Field(f) => {
                let is_native = f.attributes.iter().any(|a| a.name == "native");
                let has_internal = f.attributes.iter().any(|a| a.name == "internal");
                let is_internal = has_internal || f.name.starts_with("__");
                let side = if f.is_static || f.attributes.iter().any(|a| a.name == "class") {
                    NativeDispatch::Class
                } else {
                    NativeDispatch::Instance
                };
                let selector = phalcom_ast::selector::selector_from_field(f).encode();
                (is_native, has_internal, is_internal, side, selector, true, f.annotation.is_some(), f.range)
            }
            ClassMember::Index(ix) => {
                let is_native = ix.attributes.iter().any(|a| a.name == "native");
                let has_internal = ix.attributes.iter().any(|a| a.name == "internal");
                let is_internal = has_internal;
                let side = NativeDispatch::Instance;
                let selector = phalcom_ast::selector::selector_from_index(ix).encode();
                let typed = ix.return_annotation.is_some() && ix.params.iter().all(|param| param.annotation.is_some());
                (is_native, has_internal, is_internal, side, selector, false, typed, ix.range)
            }
            ClassMember::Variant(_) => return Ok(()),
        };

        self.census.members.push(UniverseMemberRow {
            module: module.clone(),
            owner: Some(owner_key),
            class_name: class_name.to_owned(),
            side,
            selector: selector.clone(),
            native: is_native,
            internal: is_internal,
            declaration_only: is_declaration,
            typed,
            documented: false,
            range,
        });

        if !is_native {
            return Ok(());
        }

        let visibility = if is_internal { NativeVisibility::Internal } else { NativeVisibility::Public };

        if selector.starts_with("_$") && !has_internal {
            return Err(format!("native implementation selector '{selector}' must carry @internal"));
        }

        let key = NativeMemberKey {
            owner: owner_key,
            side,
            selector: selector.clone(),
        };

        let anchor = NativeMemberAnchor {
            key: key.clone(),
            class_name: class_name.to_string(),
            side,
            selector,
            visibility,
            range,
            is_declaration,
            typed,
        };
        if self.members.insert(key.clone(), anchor).is_some() {
            return Err(format!(
                "duplicate @native member anchor for {} {:?} '{}'",
                key.owner.name(),
                key.side,
                key.selector
            ));
        }

        Ok(())
    }
}

fn universe_dependency_target(importer: &ParsedModuleUnit, path: &ImportPath) -> Option<ModuleId> {
    let mut components = match &path.root {
        ImportRoot::Absolute(root) if root.name == "universe" => Vec::new(),
        ImportRoot::Absolute(_) => return None,
        ImportRoot::Relative { dots, .. } => {
            let mut base = if importer.kind == ModuleKind::Package {
                importer.id.path.clone()
            } else {
                importer.id.path.parent().unwrap_or_else(ModulePath::root)
            };
            for _ in 1..usize::from(*dots) {
                base = base.parent().unwrap_or_else(ModulePath::root);
            }
            base.components().to_vec()
        }
    };
    components.extend(
        path.segments
            .iter()
            .map(|segment| phalcom_modules::ModuleComponent::from_identifier(&segment.name).expect("canonical Universe component")),
    );
    Some(ModuleId::universe(ModulePath::from_components(components)))
}

fn source_method_selector(method: &MethodDef) -> String {
    let Some(_) = method.params.iter().find(|param| param.rest_mode != RestMode::None) else {
        return phalcom_ast::selector::selector_from_method(method).encode();
    };

    let slots = method
        .params
        .iter()
        .map(|param| match param.rest_mode {
            RestMode::None => param.label.clone().unwrap_or_else(|| "_".to_owned()),
            RestMode::Positional => "*".to_owned(),
            RestMode::Labeled => "**".to_owned(),
            RestMode::Complete => "***".to_owned(),
        })
        .collect::<Vec<_>>();
    format!("{}({})", method.name, slots.join(","))
}

#[cfg(test)]
mod tests {
    use super::*;
    use phalcom_ast::ast::{ClassMember, MemberBody, Statement};

    fn parse_program(source: &str) -> Program {
        let parsed = phalcom_ast::parser::parse(source, 0);
        assert!(parsed.errors.is_empty(), "parse errors: {:?}", parsed.errors);
        parsed.program
    }

    #[test]
    fn source_anchor_uses_owned_selector_key_and_preserves_body_kind() {
        let program = parse_program("class String {\n  @native\n  @internal\n  _$byteAt(_ index: Int) -> String\n}\n");
        let module = ModuleId::universe(ModulePath::from_components(
            ["scalar", "string"]
                .into_iter()
                .map(|part| phalcom_modules::ModuleComponent::from_identifier(part).unwrap())
                .collect(),
        ));
        let index = NativeSourceIndex::from_program_at(&module, &program).expect("source index builds");
        let key = NativeMemberKey {
            owner: UniverseKey::String,
            side: NativeDispatch::Instance,
            selector: "_$byteAt(_)".to_owned(),
        };
        let anchor = index.members.get(&key).expect("native anchor indexed");
        assert_eq!(anchor.visibility, NativeVisibility::Internal);
        assert!(anchor.is_declaration);
        assert!(matches!(
            program.statements.first(),
            Some(Statement::Class(class))
                if matches!(class.members.first(), Some(ClassMember::Method(method))
                    if matches!(method.body, MemberBody::Declaration))
        ));
    }

    #[test]
    fn duplicate_source_anchor_is_rejected() {
        let program = parse_program("class String {\n  @native\n  @internal\n  _$byteCount\n  @native\n  @internal\n  _$byteCount\n}\n");
        let module = ModuleId::universe(ModulePath::from_components(
            ["scalar", "string"]
                .into_iter()
                .map(|part| phalcom_modules::ModuleComponent::from_identifier(part).unwrap())
                .collect(),
        ));
        let error = NativeSourceIndex::from_program_at(&module, &program).expect_err("duplicate anchor must fail");
        assert!(error.contains("duplicate @native member anchor"), "{error}");
    }

    #[test]
    fn bundled_provider_corpus_is_indexable() {
        NativeSourceIndex::build().expect("bundled universe source must parse");
    }

    #[test]
    fn native_class_with_familiar_name_at_wrong_path_is_not_associated() {
        let program = parse_program("@native\nclass String { }\n");
        let error = NativeSourceIndex::from_program(&program).expect_err("wrong-path native class must fail closed");
        assert!(error.contains("outside canonical source module"), "{error}");
    }

    #[test]
    fn implementation_anchor_requires_explicit_internal_attribute() {
        let program = parse_program("class String {\n  @native\n  _$byteCount\n}\n");
        let module = ModuleId::universe(ModulePath::from_components(
            ["scalar", "string"]
                .into_iter()
                .map(|part| phalcom_modules::ModuleComponent::from_identifier(part).unwrap())
                .collect(),
        ));
        let error = NativeSourceIndex::from_program_at(&module, &program).expect_err("implementation anchor must be explicit");
        assert!(error.contains("must carry @internal"), "{error}");
    }
}
