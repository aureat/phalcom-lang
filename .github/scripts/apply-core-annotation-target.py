from pathlib import Path


def replace_once(path: str, old: str, new: str) -> None:
    p = Path(path)
    text = p.read_text()
    if old not in text:
        raise SystemExit(f"missing patch anchor in {path}: {old[:160]!r}")
    p.write_text(text.replace(old, new, 1))


builder = "phalcom-semantic/src/source_index/builder.rs"
replace_once(
    builder,
    "use std::collections::BTreeMap;\n",
    "use std::collections::{BTreeMap, BTreeSet};\n",
)
replace_once(
    builder,
    "use crate::source_index::site::{SourceSite, SourceSiteKind};\n",
    "use crate::source_index::site::{SourceSite, SourceSiteKind};\nuse crate::types::annotation::TypeResolver;\n",
)
replace_once(
    builder,
    "use phalcom_ast::ast::{BindingKind, BlockExpr, ClassDef, ClassMember, Expr, ForStatement, LetBinding, MemberBody, Pattern, Program, Statement};\n",
    "use phalcom_ast::ast::{\n    BindingKind, BlockExpr, ClassDef, ClassMember, Expr, ForStatement, GenericConstraintSyntax, LetBinding, MemberBody, Pattern, Program, Statement,\n    TypeAnnotation, TypeAnnotationExpr, WhereClauseSyntax,\n};\n",
)
replace_once(
    builder,
    "    /// Canonical callable targets keyed by declaration and exact selector.\n    pub callable_targets: BTreeMap<(DeclarationId, Selector), CallableId>,\n",
    "    /// Canonical callable targets keyed by declaration and exact selector.\n    pub callable_targets: BTreeMap<(DeclarationId, Selector), CallableId>,\n    /// Canonical nominal type references keyed by source module and exact token range.\n    /// Resolution is performed by the compiler type resolver before occurrence\n    /// construction; the source index only publishes the resulting identity.\n    pub type_reference_targets: BTreeMap<(ModuleId, SourceRange), DeclarationId>,\n",
)

marker = "}\n\n/// Builds the compiler-owned lexical source index for one parsed module.\npub fn build_source_scope_index"
helper = r''' }

/// Resolves nominal type-reference tokens using the compiler's linked type
/// resolver. The result is source identity data only: no type checking or
/// inference is performed here.
pub fn resolve_type_reference_targets(
    module: &ModuleId,
    program: &Program,
    resolver: &dyn TypeResolver,
) -> BTreeMap<SourceRange, DeclarationId> {
    let mut collector = TypeReferenceTargetCollector {
        module,
        resolver,
        targets: BTreeMap::new(),
    };
    let bound = BTreeSet::new();
    for statement in &program.statements {
        collector.statement(statement, &bound);
    }
    collector.targets
}

struct TypeReferenceTargetCollector<'a> {
    module: &'a ModuleId,
    resolver: &'a dyn TypeResolver,
    targets: BTreeMap<SourceRange, DeclarationId>,
}

impl TypeReferenceTargetCollector<'_> {
    fn statement(&mut self, statement: &Statement, bound: &BTreeSet<String>) {
        match statement {
            Statement::Class(class) => {
                let mut class_bound = bound.clone();
                class_bound.extend(class.generic_parameters.iter().map(|parameter| parameter.name.clone()));
                if let Some(superclass) = &class.superclass {
                    self.annotation(superclass, &class_bound);
                }
                self.where_clause(class.where_clause.as_ref(), &class_bound);
                for member in &class.members {
                    match member {
                        ClassMember::Method(method) => {
                            let mut method_bound = class_bound.clone();
                            method_bound.extend(method.generic_parameters.iter().map(|parameter| parameter.name.clone()));
                            for parameter in &method.params {
                                if let Some(annotation) = &parameter.annotation {
                                    self.annotation(annotation, &method_bound);
                                }
                            }
                            if let Some(annotation) = &method.return_annotation {
                                self.annotation(annotation, &method_bound);
                            }
                            self.where_clause(method.where_clause.as_ref(), &method_bound);
                        }
                        ClassMember::Getter(getter) => {
                            if let Some(annotation) = &getter.return_annotation {
                                self.annotation(annotation, &class_bound);
                            }
                        }
                        ClassMember::Setter(setter) => {
                            if let Some(annotation) = &setter.param.annotation {
                                self.annotation(annotation, &class_bound);
                            }
                            if let Some(annotation) = &setter.return_annotation {
                                self.annotation(annotation, &class_bound);
                            }
                        }
                        ClassMember::Field(field) => {
                            if let Some(annotation) = &field.annotation {
                                self.annotation(annotation, &class_bound);
                            }
                        }
                        ClassMember::Index(index) => {
                            for parameter in &index.params {
                                if let Some(annotation) = &parameter.annotation {
                                    self.annotation(annotation, &class_bound);
                                }
                            }
                            if let phalcom_ast::ast::IndexAccessor::Set { put } = &index.accessor
                                && let Some(annotation) = &put.annotation
                            {
                                self.annotation(annotation, &class_bound);
                            }
                            if let Some(annotation) = &index.return_annotation {
                                self.annotation(annotation, &class_bound);
                            }
                        }
                        ClassMember::Variant(_) => {}
                    }
                }
            }
            Statement::TypeAlias(alias) => {
                let mut alias_bound = bound.clone();
                alias_bound.extend(alias.generic_parameters.iter().map(|parameter| parameter.name.clone()));
                self.where_clause(alias.where_clause.as_ref(), &alias_bound);
                self.annotation(&alias.body, &alias_bound);
            }
            Statement::Let(binding) => {
                if let Some(annotation) = &binding.annotation {
                    self.annotation(annotation, bound);
                }
            }
            Statement::Return(_)
            | Statement::Expr { .. }
            | Statement::For(_)
            | Statement::Break { .. }
            | Statement::Continue { .. }
            | Statement::Throw { .. }
            | Statement::Export(_) => {}
        }
    }

    fn where_clause(&mut self, clause: Option<&WhereClauseSyntax>, bound: &BTreeSet<String>) {
        let Some(clause) = clause else { return };
        for constraint in &clause.constraints {
            match constraint {
                GenericConstraintSyntax::Subtype { lower, upper, .. } => {
                    self.annotation(lower, bound);
                    self.annotation(upper, bound);
                }
                GenericConstraintSyntax::Equivalent { left, right, .. } => {
                    self.annotation(left, bound);
                    self.annotation(right, bound);
                }
                GenericConstraintSyntax::Invalid { .. } => {}
            }
        }
    }

    fn annotation(&mut self, annotation: &TypeAnnotation, bound: &BTreeSet<String>) {
        match &annotation.expr {
            TypeAnnotationExpr::Reference(symbol) => {
                if symbol.members.is_empty() && bound.contains(&symbol.root) {
                    return;
                }
                let members = symbol.members.iter().map(|member| member.name.clone()).collect::<Vec<_>>();
                if let Some(declaration) = self.resolver.resolve_type_name(self.module, &symbol.root, &members) {
                    let range = symbol.members.last().map_or(symbol.root_range, |member| member.range);
                    self.targets.insert(range, declaration);
                }
            }
            TypeAnnotationExpr::Application { origin, arguments, .. } => {
                self.annotation(origin, bound);
                for argument in arguments {
                    self.annotation(argument, bound);
                }
            }
            TypeAnnotationExpr::Union { members, .. } => {
                for member in members {
                    self.annotation(member, bound);
                }
            }
            TypeAnnotationExpr::Tuple { elements, .. } => {
                for element in elements {
                    self.annotation(&element.ty, bound);
                }
            }
            TypeAnnotationExpr::Callable { parameters, result, .. } => {
                for parameter in parameters {
                    self.annotation(&parameter.ty, bound);
                }
                self.annotation(result, bound);
            }
            TypeAnnotationExpr::Record { fields, .. } => {
                for field in fields {
                    self.annotation(&field.ty, bound);
                }
            }
            TypeAnnotationExpr::TypeLambda { parameters, body, .. } => {
                let mut lambda_bound = bound.clone();
                lambda_bound.extend(parameters.iter().map(|parameter| parameter.name.clone()));
                self.annotation(body, &lambda_bound);
            }
            TypeAnnotationExpr::Unit { .. }
            | TypeAnnotationExpr::Dynamic { .. }
            | TypeAnnotationExpr::Never { .. }
            | TypeAnnotationExpr::SelfType { .. }
            | TypeAnnotationExpr::Invalid { .. } => {}
        }
    }
}

/// Builds the compiler-owned lexical source index for one parsed module.
pub fn build_source_scope_index'''
replace_once(builder, marker, helper)

occurrence = "phalcom-semantic/src/source_index/occurrence.rs"
replace_once(
    occurrence,
    "        for statement in &program.statements {\n            visitor.statement(statement);\n        }\n        result = Self::new(visitor.occurrences, visitor.targets);\n",
    "        for statement in &program.statements {\n            visitor.statement(statement);\n        }\n        if let Some(context) = context {\n            let module = visitor.scopes.module.clone();\n            let references = context\n                .type_reference_targets\n                .iter()\n                .filter(|((owner, _), _)| owner == &module)\n                .map(|((_, range), declaration)| (*range, declaration.clone()))\n                .collect::<Vec<_>>();\n            for (range, declaration) in references {\n                visitor.record_targeted(\n                    range,\n                    OccurrenceKind::Declaration,\n                    OccurrenceRole::Reference,\n                    None,\n                    Some(SemanticTargetId::Declaration(declaration)),\n                );\n            }\n        }\n        result = Self::new(visitor.occurrences, visitor.targets);\n",
)

modrs = "phalcom-semantic/src/source_index/mod.rs"
replace_once(
    modrs,
    "pub use builder::{SourceIndexContext, build_source_scope_index};\n",
    "pub use builder::{SourceIndexContext, build_source_scope_index, resolve_type_reference_targets};\n",
)

session = "phalcom-semantic/src/session.rs"
replace_once(
    session,
    "use crate::source_index::{SourceIndexContext, SourceSemanticIndex, build_source_scope_index};\n",
    "use crate::source_index::{SourceIndexContext, SourceSemanticIndex, build_source_scope_index, resolve_type_reference_targets};\n",
)
replace_once(
    session,
    "        let mut source_index = build_source_semantic_index(&input.sources, &callable_analyses, &resolved_imports_map, input.linked.as_ref());\n",
    "        let mut source_index = build_source_semantic_index(\n            &input.sources,\n            &callable_analyses,\n            &resolved_imports_map,\n            input.linked.as_ref(),\n            &resolver,\n        );\n",
)
replace_once(
    session,
    "fn build_source_semantic_index(\n    sources: &BTreeMap<ModuleId, Arc<ParsedModuleUnit>>,\n    callable_analyses: &HashMap<crate::identity::CallableId, Arc<crate::checker::CallableAnalysis>>,\n    resolved_imports: &BTreeMap<(ModuleId, String), ModuleId>,\n    linked: &LinkedProgram,\n) -> SourceSemanticIndex {\n",
    "fn build_source_semantic_index(\n    sources: &BTreeMap<ModuleId, Arc<ParsedModuleUnit>>,\n    callable_analyses: &HashMap<crate::identity::CallableId, Arc<crate::checker::CallableAnalysis>>,\n    resolved_imports: &BTreeMap<(ModuleId, String), ModuleId>,\n    linked: &LinkedProgram,\n    type_resolver: &dyn TypeResolver,\n) -> SourceSemanticIndex {\n",
)
replace_once(
    session,
    "    let mut context = SourceIndexContext {\n        resolved_imports: resolved_imports.clone(),\n        ..SourceIndexContext::default()\n    };\n",
    "    let mut context = SourceIndexContext {\n        resolved_imports: resolved_imports.clone(),\n        ..SourceIndexContext::default()\n    };\n    for (module, source) in sources {\n        for (range, declaration) in resolve_type_reference_targets(module, &source.program, type_resolver) {\n            context.type_reference_targets.insert((module.clone(), range), declaration);\n        }\n    }\n",
)
