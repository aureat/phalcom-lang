//! AST-to-source-scope construction under compiler ownership.

use std::collections::BTreeMap;

use crate::identity::{
    CallableId, CallableOwnerId, CallableParameterId, DataComponentId, DeclarationId, DispatchSide, FieldId, ModuleId, SemanticTargetId, SourceOwner,
    SourceSiteId, SourceSiteLocalId, StaticCaseId, StaticDomainDeclarationId, VariantFieldId, VariantId,
};
use crate::source_index::scope::{
    CallableSourceInfo, DeclarationSourceInfo, FieldSourceInfo, ImportBindingOrigin, SourceBindingInfo, SourceBindingKind, SourceCallableKind,
    SourceDeclarationKind, SourceScopeId, SourceScopeIndex, TypeMemberSourceInfo,
};
use crate::source_index::site::{SourceSite, SourceSiteKind};
use crate::traits::AssociatedTypeRequirementId;
use crate::types::annotation::TypeResolver;
use phalcom_ast::ast::{
    BehaviorMember, BindingKind, BlockExpr, ClassDef, ClassMember, EnumDef, Expr, ForStatement, GenericConstraintSyntax, LetBinding, MemberBody, Pattern,
    Program, Statement, TypeAnnotation, TypeAnnotationExpr, WhereClauseSyntax,
};
use phalcom_common::range::SourceRange;
use phalcom_common::selector::{Selector, SelectorSlot};
use phalcom_modules::linker::SymbolId;
use phalcom_modules::{ImportSiteId, ImportSiteLocalId};

/// Canonical linked targets available while building source identity.
#[derive(Clone, Debug, Default)]
pub struct SourceIndexContext {
    /// Linked module targets keyed by their canonical source path spelling.
    pub modules: BTreeMap<String, ModuleId>,
    /// Linked exported targets keyed by `(module, exported name)`.
    pub targets: BTreeMap<(ModuleId, String), SemanticTargetId>,
    /// Canonical linked import resolution keyed by importing module and the
    /// written logical path. This prevents source indexing from falling back
    /// to a disconnected default context.
    pub resolved_imports: BTreeMap<(ModuleId, String), ModuleId>,
    /// Exact compiler-owned import products keyed by authored import site.
    /// Production source indexing prefers this identity-keyed projection over
    /// reconstructing meaning from a written path string.
    pub import_products: BTreeMap<ImportSiteId, std::sync::Arc<phalcom_modules::resolver::ImportResolutionProduct>>,
    /// Requires production source indexing to fail closed when an authored
    /// import has no canonical compiler product. Low-level source-index tests
    /// leave this disabled to exercise compatibility-only path maps.
    pub require_canonical_import_products: bool,
    /// Canonical callable targets keyed by declaration and exact selector.
    pub callable_targets: BTreeMap<(DeclarationId, Selector), CallableId>,
    /// Canonical nominal type references keyed by source module and exact token range.
    /// Resolution is performed by the compiler type resolver before occurrence
    /// construction; the source index only publishes the resulting identity.
    pub type_reference_targets: BTreeMap<(ModuleId, SourceRange), DeclarationId>,
    /// Canonical type-member projection targets keyed by source module and the
    /// authored selector range. Owners are resolved through the linked type
    /// resolver; selector spelling alone is never treated as identity.
    pub type_member_projection_targets: BTreeMap<(ModuleId, SourceRange), crate::types::type_member::TypeMemberDeclarationId>,
    /// Product-expansion generic parameter declaration tokens keyed by source
    /// module and exact name range.
    pub type_parameter_declarations: BTreeMap<(ModuleId, SourceRange), String>,
    /// Type-level parameter references keyed by use range and pointing at the
    /// exact declaration-name range in the same source module.
    pub type_parameter_reference_targets: BTreeMap<(ModuleId, SourceRange), SourceRange>,
    /// Exact trait-owned associated declarations keyed by conformance binding
    /// LHS range. The semantic session supplies this identity map.
    pub associated_type_targets: BTreeMap<(ModuleId, SourceRange), AssociatedTypeRequirementId>,
    /// Exact trait-owned associated declarations keyed by contextual
    /// projection name range. The semantic session supplies this identity
    /// map from canonical trait formation products.
    pub associated_projection_targets: BTreeMap<(ModuleId, SourceRange), AssociatedTypeRequirementId>,
}

/// Source ranges for product-expansion parameter declarations and their bound
/// type-level references. Declaration ranges are exact name tokens, not stable
/// identities; the source index resolves them to snapshot-local binding sites.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TypeParameterSourceTargets {
    /// Product-expansion binder name range to authored name.
    pub declarations: BTreeMap<SourceRange, String>,
    /// Reference name range to the declaration's name range.
    pub references: BTreeMap<SourceRange, SourceRange>,
}

struct CallableVisit<'a> {
    parent: SourceScopeId,
    callable: CallableId,
    name_range: SourceRange,
    declaration_range: SourceRange,
    body_range: SourceRange,
    parameters: &'a [phalcom_ast::ast::ParameterDef],
    body: &'a MemberBody,
    parameter_kind: SourceBindingKind,
    kind: SourceCallableKind,
    has_explicit_return_annotation: bool,
}

impl SourceIndexContext {
    /// Adds one canonical module path mapping.
    pub fn with_module(mut self, path: impl Into<String>, module: ModuleId) -> Self {
        self.modules.insert(path.into(), module);
        self
    }

    /// Adds one canonical exported target mapping.
    pub fn with_target(mut self, module: ModuleId, name: impl Into<String>, target: SemanticTargetId) -> Self {
        self.targets.insert((module, name.into()), target);
        self
    }

    pub fn with_resolved_import(mut self, importer: ModuleId, path: impl Into<String>, module: ModuleId) -> Self {
        self.resolved_imports.insert((importer, path.into()), module);
        self
    }
}

/// Resolves nominal type-reference tokens using the compiler's linked type
/// resolver. The result is source identity data only: no type checking or
/// inference is performed here.
pub fn resolve_type_reference_targets(module: &ModuleId, program: &Program, resolver: &dyn TypeResolver) -> BTreeMap<SourceRange, DeclarationId> {
    let mut collector = TypeReferenceTargetCollector {
        module,
        resolver,
        targets: BTreeMap::new(),
        type_member_targets: BTreeMap::new(),
        type_parameter_sources: TypeParameterSourceTargets::default(),
    };
    collector.statement_list(&program.statements, &BoundTypeParameters::default());
    collector.targets
}

/// Collects product-expansion generic declaration/reference ranges for the
/// source index. The resulting declaration identities are assigned by the
/// source-index builder within the current snapshot.
pub fn resolve_type_parameter_source_targets(module: &ModuleId, program: &Program, resolver: &dyn TypeResolver) -> TypeParameterSourceTargets {
    let mut collector = TypeReferenceTargetCollector {
        module,
        resolver,
        targets: BTreeMap::new(),
        type_member_targets: BTreeMap::new(),
        type_parameter_sources: TypeParameterSourceTargets::default(),
    };
    collector.statement_list(&program.statements, &BoundTypeParameters::default());
    collector.type_parameter_sources
}

/// Resolves authored type-member projection selectors to canonical owner and
/// selector identities for source-index occurrence publication.
pub fn resolve_type_member_projection_targets(
    module: &ModuleId,
    program: &Program,
    resolver: &dyn TypeResolver,
) -> BTreeMap<SourceRange, crate::types::type_member::TypeMemberDeclarationId> {
    let mut collector = TypeReferenceTargetCollector {
        module,
        resolver,
        targets: BTreeMap::new(),
        type_member_targets: BTreeMap::new(),
        type_parameter_sources: TypeParameterSourceTargets::default(),
    };
    collector.statement_list(&program.statements, &BoundTypeParameters::default());
    collector.type_member_targets
}

type BoundTypeParameters = BTreeMap<String, Option<SourceRange>>;

struct TypeReferenceTargetCollector<'a> {
    module: &'a ModuleId,
    resolver: &'a dyn TypeResolver,
    targets: BTreeMap<SourceRange, DeclarationId>,
    type_member_targets: BTreeMap<SourceRange, crate::types::type_member::TypeMemberDeclarationId>,
    type_parameter_sources: TypeParameterSourceTargets,
}

impl TypeReferenceTargetCollector<'_> {
    fn statement_list(&mut self, statements: &[Statement], bound: &BoundTypeParameters) {
        for statement in statements {
            self.statement(statement, bound);
        }
    }

    fn generic_parameters(&mut self, bound: &BoundTypeParameters, parameters: &[phalcom_ast::ast::GenericParameterSyntax]) -> BoundTypeParameters {
        let mut next = bound.clone();
        for parameter in parameters {
            let declaration = parameter.mode.map(|_| parameter.name_range);
            if declaration.is_some() {
                self.type_parameter_sources.declarations.insert(parameter.name_range, parameter.name.clone());
            }
            next.insert(parameter.name.clone(), declaration);
        }
        next
    }

    fn type_member_parameters(&self, bound: &BoundTypeParameters, parameters: &[phalcom_ast::ast::TypeMemberParameterSyntax]) -> BoundTypeParameters {
        let mut next = bound.clone();
        next.extend(parameters.iter().map(|parameter| (parameter.name.clone(), None)));
        next
    }

    fn statement(&mut self, statement: &Statement, bound: &BoundTypeParameters) {
        match statement {
            Statement::Class(class) => {
                let class_bound = self.generic_parameters(bound, &class.generic_parameters);
                if let Some(superclass) = &class.superclass {
                    self.annotation(superclass, &class_bound);
                }
                self.where_clause(class.where_clause.as_ref(), &class_bound);
                for member in &class.members {
                    match member {
                        ClassMember::Method(method) => {
                            let method_bound = self.generic_parameters(&class_bound, &method.generic_parameters);
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
                            let index_bound = self.generic_parameters(&class_bound, &index.generic_parameters);
                            for parameter in &index.params {
                                if let Some(annotation) = &parameter.annotation {
                                    self.annotation(annotation, &index_bound);
                                }
                            }
                            if let phalcom_ast::ast::IndexAccessor::Set { value } = &index.accessor
                                && let Some(annotation) = &value.annotation
                            {
                                self.annotation(annotation, &index_bound);
                            }
                            if let Some(annotation) = &index.return_annotation {
                                self.annotation(annotation, &index_bound);
                            }
                            self.where_clause(index.where_clause.as_ref(), &index_bound);
                        }
                        ClassMember::TypeMember(type_member) => {
                            let type_member_bound = self.type_member_parameters(&class_bound, &type_member.parameters);
                            for parameter in &type_member.parameters {
                                match &parameter.domain {
                                    phalcom_ast::ast::TypeMemberParameterDomainSyntax::Kind(_) => {}
                                    phalcom_ast::ast::TypeMemberParameterDomainSyntax::ConstType(t) => {
                                        self.annotation(t, &type_member_bound);
                                    }
                                }
                            }
                            match &type_member.result {
                                phalcom_ast::ast::TypeMemberResultSyntax::Kind(_) => {}
                                phalcom_ast::ast::TypeMemberResultSyntax::Type(t) => {
                                    self.annotation(t, &type_member_bound);
                                }
                            }
                            self.where_clause(type_member.where_clause.as_ref(), &type_member_bound);
                        }
                        ClassMember::Variant(_) | ClassMember::Delegation(_) => {}
                    }
                }
            }
            Statement::Data(data_def) => {
                if data_def.is_sum() {
                    let data_bound = self.generic_parameters(bound, &data_def.generic_parameters);
                    self.sum_data_definition(data_def, &data_bound);
                } else {
                    let data_bound = self.generic_parameters(bound, &data_def.generic_parameters);
                    self.where_clause(data_def.where_clause.as_ref(), &data_bound);
                    if let Some(components) = data_def.components() {
                        for comp in components {
                            self.annotation(&comp.annotation, &data_bound);
                        }
                    }
                }
            }
            Statement::Trait(trait_def) => {
                let trait_bound = self.generic_parameters(bound, &trait_def.generic_parameters);
                self.where_clause(trait_def.where_clause.as_ref(), &trait_bound);
                for trait_member in &trait_def.members {
                    if let Some(member) = trait_member.behavior() {
                        self.behavior_members(std::slice::from_ref(member), &trait_bound);
                    }
                    if let phalcom_ast::ast::TraitMember::Property(property) = trait_member {
                        self.annotation(&property.annotation, &trait_bound);
                    }
                }
            }
            Statement::TypeAlias(alias) => {
                let alias_bound = self.generic_parameters(bound, &alias.generic_parameters);
                self.where_clause(alias.where_clause.as_ref(), &alias_bound);
                self.annotation(&alias.body, &alias_bound);
            }
            Statement::Let(binding) => {
                if let Some(annotation) = &binding.annotation {
                    self.annotation(annotation, bound);
                }
            }
            Statement::Impl(impl_def) => {
                let impl_bound = self.generic_parameters(bound, &impl_def.generic_parameters);
                self.where_clause(impl_def.where_clause.as_ref(), &impl_bound);
                if let phalcom_ast::ast::ImplKind::Conformance { trait_ref, .. } = &impl_def.kind {
                    self.annotation(trait_ref, &impl_bound);
                }
                self.annotation(&impl_def.target, &impl_bound);
                if matches!(impl_def.kind, phalcom_ast::ast::ImplKind::Conformance { .. }) {
                    return;
                }
                for impl_member in &impl_def.members {
                    if let phalcom_ast::ast::ImplMember::TypeMember(type_member) = impl_member {
                        let type_member_bound = self.type_member_parameters(&impl_bound, &type_member.parameters);
                        for parameter in &type_member.parameters {
                            if let phalcom_ast::ast::TypeMemberParameterDomainSyntax::ConstType(annotation) = &parameter.domain {
                                self.annotation(annotation, &type_member_bound);
                            }
                        }
                        if let phalcom_ast::ast::TypeMemberResultSyntax::Type(annotation) = &type_member.result {
                            self.annotation(annotation, &type_member_bound);
                        }
                        self.where_clause(type_member.where_clause.as_ref(), &type_member_bound);
                        continue;
                    }
                    let Some(member) = impl_member.behavior() else {
                        continue;
                    };
                    match member {
                        phalcom_ast::ast::BehaviorMember::Method(method) => {
                            let method_bound = self.generic_parameters(&impl_bound, &method.generic_parameters);
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
                        phalcom_ast::ast::BehaviorMember::Getter(getter) => {
                            let getter_bound = self.generic_parameters(&impl_bound, &getter.generic_parameters);
                            if let Some(annotation) = &getter.return_annotation {
                                self.annotation(annotation, &getter_bound);
                            }
                            self.where_clause(getter.where_clause.as_ref(), &getter_bound);
                        }
                        phalcom_ast::ast::BehaviorMember::Setter(setter) => {
                            let setter_bound = self.generic_parameters(&impl_bound, &setter.generic_parameters);
                            if let Some(annotation) = &setter.param.annotation {
                                self.annotation(annotation, &setter_bound);
                            }
                            if let Some(annotation) = &setter.return_annotation {
                                self.annotation(annotation, &setter_bound);
                            }
                            self.where_clause(setter.where_clause.as_ref(), &setter_bound);
                        }
                        phalcom_ast::ast::BehaviorMember::Index(index) => {
                            let index_bound = self.generic_parameters(&impl_bound, &index.generic_parameters);
                            for parameter in &index.params {
                                if let Some(annotation) = &parameter.annotation {
                                    self.annotation(annotation, &index_bound);
                                }
                            }
                            if let phalcom_ast::ast::IndexAccessor::Set { value } = &index.accessor
                                && let Some(annotation) = &value.annotation
                            {
                                self.annotation(annotation, &index_bound);
                            }
                            if let Some(annotation) = &index.return_annotation {
                                self.annotation(annotation, &index_bound);
                            }
                            self.where_clause(index.where_clause.as_ref(), &index_bound);
                        }
                    }
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

    fn where_clause(&mut self, clause: Option<&WhereClauseSyntax>, bound: &BoundTypeParameters) {
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
                GenericConstraintSyntax::Conforms { subject, bounds, .. } => {
                    self.annotation(subject, bound);
                    for b in bounds {
                        self.trait_annotation(&b.trait_type, bound);
                        for spec in &b.associated_specializations {
                            self.annotation(&spec.value, bound);
                        }
                    }
                }
                GenericConstraintSyntax::KindFact { .. } => {}
                GenericConstraintSyntax::Invalid { .. } => {}
            }
        }
    }

    fn behavior_members(&mut self, members: &[phalcom_ast::ast::BehaviorMember], bound: &BoundTypeParameters) {
        for member in members {
            match member {
                phalcom_ast::ast::BehaviorMember::Method(method) => {
                    let member_bound = self.generic_parameters(bound, &method.generic_parameters);
                    for parameter in &method.params {
                        if let Some(annotation) = &parameter.annotation {
                            self.annotation(annotation, &member_bound);
                        }
                    }
                    if let Some(annotation) = &method.return_annotation {
                        self.annotation(annotation, &member_bound);
                    }
                    self.where_clause(method.where_clause.as_ref(), &member_bound);
                }
                phalcom_ast::ast::BehaviorMember::Getter(getter) => {
                    let member_bound = self.generic_parameters(bound, &getter.generic_parameters);
                    if let Some(annotation) = &getter.return_annotation {
                        self.annotation(annotation, &member_bound);
                    }
                    self.where_clause(getter.where_clause.as_ref(), &member_bound);
                }
                phalcom_ast::ast::BehaviorMember::Setter(setter) => {
                    let member_bound = self.generic_parameters(bound, &setter.generic_parameters);
                    if let Some(annotation) = &setter.param.annotation {
                        self.annotation(annotation, &member_bound);
                    }
                    if let Some(annotation) = &setter.return_annotation {
                        self.annotation(annotation, &member_bound);
                    }
                    self.where_clause(setter.where_clause.as_ref(), &member_bound);
                }
                phalcom_ast::ast::BehaviorMember::Index(index) => {
                    let member_bound = self.generic_parameters(bound, &index.generic_parameters);
                    for parameter in &index.params {
                        if let Some(annotation) = &parameter.annotation {
                            self.annotation(annotation, &member_bound);
                        }
                    }
                    if let phalcom_ast::ast::IndexAccessor::Set { value } = &index.accessor
                        && let Some(annotation) = &value.annotation
                    {
                        self.annotation(annotation, &member_bound);
                    }
                    if let Some(annotation) = &index.return_annotation {
                        self.annotation(annotation, &member_bound);
                    }
                    self.where_clause(index.where_clause.as_ref(), &member_bound);
                }
            }
        }
    }

    fn sum_data_definition(&mut self, data_def: &EnumDef, bound: &BoundTypeParameters) {
        let enum_bound = self.generic_parameters(bound, &data_def.generic_parameters);
        self.where_clause(data_def.where_clause.as_ref(), &enum_bound);

        for variant in data_def.cases().unwrap_or_default() {
            let variant_bound = self.generic_parameters(&enum_bound, &variant.generic_parameters);
            self.where_clause(variant.where_clause.as_ref(), &variant_bound);
            match &variant.payload {
                phalcom_ast::ast::CasePayloadSyntax::Tuple { parameters, .. } => {
                    for parameter in parameters {
                        if let Some(annotation) = &parameter.annotation {
                            self.annotation(annotation, &variant_bound);
                        }
                    }
                }
                phalcom_ast::ast::CasePayloadSyntax::Record { components, .. } => {
                    for comp in components {
                        self.annotation(&comp.annotation, &variant_bound);
                    }
                }
                phalcom_ast::ast::CasePayloadSyntax::Singleton => {}
            }
            if let Some(annotation) = &variant.result_annotation {
                self.annotation(annotation, &variant_bound);
            }
        }
    }

    fn annotation(&mut self, annotation: &TypeAnnotation, bound: &BoundTypeParameters) {
        match &annotation.expr {
            TypeAnnotationExpr::Reference(symbol) => {
                if symbol.members.is_empty()
                    && let Some(parameter_range) = bound.get(&symbol.root)
                {
                    if let Some(parameter_range) = parameter_range {
                        self.type_parameter_sources.references.insert(symbol.root_range, *parameter_range);
                    }
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
                    match argument {
                        phalcom_ast::ast::GenericArgumentSyntax::Single(annotation) => self.annotation(annotation, bound),
                        phalcom_ast::ast::GenericArgumentSyntax::Expansion(expansion) => self.annotation(&expansion.operand, bound),
                    }
                }
            }
            TypeAnnotationExpr::Union { members, .. } => {
                for member in members {
                    self.annotation(member, bound);
                }
            }
            TypeAnnotationExpr::Tuple { elements, .. } => {
                for element in elements {
                    match element {
                        phalcom_ast::ast::TypeTupleItem::Element(element) => self.annotation(&element.ty, bound),
                        phalcom_ast::ast::TypeTupleItem::Expansion(expansion) => self.annotation(&expansion.operand, bound),
                        phalcom_ast::ast::TypeTupleItem::Remainder(remainder) => {
                            if let phalcom_ast::ast::AnonymousRemainderSyntax::Typed { constraint, .. } = &remainder.remainder {
                                self.annotation(constraint, bound);
                            }
                        }
                    }
                }
            }
            TypeAnnotationExpr::Callable { parameters, result, .. } => {
                for parameter in parameters {
                    match parameter {
                        phalcom_ast::ast::TypeCallableItem::Parameter(parameter) => self.annotation(&parameter.ty, bound),
                        phalcom_ast::ast::TypeCallableItem::Expansion(expansion) => self.annotation(&expansion.operand, bound),
                        phalcom_ast::ast::TypeCallableItem::Remainder(_) => {}
                    }
                }
                self.annotation(result, bound);
            }
            TypeAnnotationExpr::Record { fields, tail, .. } => {
                for field in fields {
                    self.annotation(&field.ty, bound);
                }
                if let Some(phalcom_ast::ast::RecordRowTailSyntax::Anonymous(phalcom_ast::ast::AnonymousRemainderSyntax::Typed { constraint, .. })) = tail {
                    self.annotation(constraint, bound);
                }
            }
            TypeAnnotationExpr::TypeLambda { parameters, body, .. } => {
                let mut lambda_bound = bound.clone();
                lambda_bound.extend(parameters.iter().map(|parameter| (parameter.name.clone(), None)));
                self.annotation(body, &lambda_bound);
            }
            TypeAnnotationExpr::ExactEnumCase {
                enum_target,
                variant_name,
                variant_name_range,
                generic_arguments,
                payload_shape,
                ..
            } => {
                // `Owner::Member` is intentionally parsed as an exact-case candidate.
                // Only publish it as a type-member reference when the checked
                // associated surface selects a source declaration. Ordinary enum
                // cases remain owned by the static-case catalog.
                if generic_arguments.is_empty() && payload_shape.is_none() {
                    self.record_type_member_target(
                        enum_target,
                        crate::types::type_member::TypeMemberSelector::named(variant_name.as_str()),
                        *variant_name_range,
                    );
                }
                self.annotation(enum_target, bound);
                for argument in generic_arguments {
                    self.annotation(argument, bound);
                }
            }
            TypeAnnotationExpr::AssociatedTypeProjection { subject, .. } => self.annotation(subject, bound),
            TypeAnnotationExpr::QualifiedAssociatedTypeProjection { subject, trait_ref, .. } => {
                self.annotation(subject, bound);
                self.trait_annotation(&trait_ref.trait_type, bound);
                for spec in &trait_ref.associated_specializations {
                    self.annotation(&spec.value, bound);
                }
            }
            TypeAnnotationExpr::TypeMemberProjection {
                subject, selector, arguments, ..
            } => {
                self.record_type_member_target(subject, crate::types::type_member::TypeMemberSelector::from_syntax(selector), selector.range());
                self.annotation(subject, bound);
                for argument in arguments {
                    if let phalcom_ast::ast::TypeMemberArgumentSyntax::TypeForm(ty) = argument {
                        self.annotation(ty, bound);
                    }
                }
            }
            TypeAnnotationExpr::Unit { .. }
            | TypeAnnotationExpr::Dynamic { .. }
            | TypeAnnotationExpr::Never { .. }
            | TypeAnnotationExpr::SelfType { .. }
            | TypeAnnotationExpr::Invalid { .. } => {}
        }
    }

    fn trait_annotation(&mut self, annotation: &TypeAnnotation, bound: &BoundTypeParameters) {
        match &annotation.expr {
            TypeAnnotationExpr::Reference(symbol) => {
                let members = symbol.members.iter().map(|member| member.name.clone()).collect::<Vec<_>>();
                if let Some(declaration) = self.resolver.resolve_trait_name(self.module, &symbol.root, &members) {
                    let range = symbol.members.last().map_or(symbol.root_range, |member| member.range);
                    self.targets.insert(range, declaration);
                }
            }
            TypeAnnotationExpr::Application { origin, arguments, .. } => {
                self.trait_annotation(origin, bound);
                for argument in arguments {
                    match argument {
                        phalcom_ast::ast::GenericArgumentSyntax::Single(annotation) => self.annotation(annotation, bound),
                        phalcom_ast::ast::GenericArgumentSyntax::Expansion(expansion) => self.annotation(&expansion.operand, bound),
                    }
                }
            }
            _ => self.annotation(annotation, bound),
        }
    }

    fn record_type_member_target(&mut self, subject: &TypeAnnotation, selector: crate::types::type_member::TypeMemberSelector, range: SourceRange) {
        let Some(owner) = type_member_subject_owner(subject, self.module, self.resolver) else {
            return;
        };
        if let Some(product) = self.resolver.resolve_associated_type_member(&owner, &selector) {
            self.type_member_targets.insert(range, product.id.clone());
        }
    }
}

fn type_member_subject_owner(annotation: &TypeAnnotation, module: &ModuleId, resolver: &dyn TypeResolver) -> Option<DeclarationId> {
    match &annotation.expr {
        TypeAnnotationExpr::Reference(reference) => {
            let members = reference.members.iter().map(|member| member.name.clone()).collect::<Vec<_>>();
            resolver.resolve_type_name(module, &reference.root, &members)
        }
        TypeAnnotationExpr::Application { origin, .. } => type_member_subject_owner(origin, module, resolver),
        _ => None,
    }
}

/// Builds the compiler-owned lexical source index for one parsed module.
pub fn build_source_scope_index(module: ModuleId, program: &Program, context: &SourceIndexContext) -> SourceScopeIndex {
    let mut builder = SourceScopeBuilder {
        index: SourceScopeIndex::new(module.clone(), statements_range(&program.statements)),
        context,
        next_scope: 1,
        next_site: BTreeMap::new(),
        current_owner: SourceOwner::Module(module.clone()),
    };

    for ((_, range), name) in context.type_parameter_declarations.iter().filter(|((owner, _), _)| owner == &module) {
        let site = builder.allocate_site(SourceOwner::Module(module.clone()), *range, SourceSiteKind::BindingDeclaration);
        builder.index.register_target(site.clone(), SemanticTargetId::Binding(site.clone()));
        builder.index.register_binding(SourceBindingInfo {
            declaration_site: site,
            scope: builder.index.root,
            name: name.clone().into(),
            kind: SourceBindingKind::TypeParameter,
            declaration_range: *range,
            has_explicit_annotation: true,
            mutable: false,
            redeclaration_of: None,
        });
    }

    for statement in &program.statements {
        if let Statement::Class(class) = statement {
            let declaration = builder.declaration_id(class);
            builder.index.register_class(class.name.clone(), declaration);
        } else if let Statement::TypeAlias(alias) = statement {
            let declaration = DeclarationId::new(module.clone(), alias.name.clone().into());
            builder.index.register_class(alias.name.clone(), declaration);
        } else if let Statement::Data(data_def) = statement {
            let declaration = DeclarationId::new(module.clone(), data_def.name.clone().into());
            builder.index.register_class(data_def.name.clone(), declaration);
        } else if let Statement::Trait(trait_def) = statement {
            let declaration = DeclarationId::new(module.clone(), trait_def.name.clone().into());
            builder.index.register_class(trait_def.name.clone(), declaration);
        }
    }
    builder.visit_imports(program);
    builder.visit_statements(builder.index.root, &program.statements, true);
    builder.index.finish_scope_order();
    builder.index
}

struct SourceScopeBuilder<'a> {
    index: SourceScopeIndex,
    context: &'a SourceIndexContext,
    next_scope: u32,
    next_site: BTreeMap<SourceOwner, u32>,
    current_owner: SourceOwner,
}

impl SourceScopeBuilder<'_> {
    fn declaration_id(&self, class: &ClassDef) -> DeclarationId {
        DeclarationId::new(self.index.module.clone(), class.name.clone().into())
    }

    fn type_reference_target(&self, annotation: &TypeAnnotation) -> Option<DeclarationId> {
        let range = match &annotation.expr {
            TypeAnnotationExpr::Reference(symbol) => symbol.members.last().map_or(symbol.root_range, |member| member.range),
            TypeAnnotationExpr::Application { origin, .. } => return self.type_reference_target(origin),
            TypeAnnotationExpr::ExactEnumCase { enum_target, .. } => return self.type_reference_target(enum_target),
            _ => return None,
        };
        self.context.type_reference_targets.get(&(self.index.module.clone(), range)).cloned()
    }

    fn type_reference_semantic_target(&self, annotation: &TypeAnnotation) -> Option<SemanticTargetId> {
        match &annotation.expr {
            TypeAnnotationExpr::ExactEnumCase {
                enum_target,
                variant_name,
                payload_shape,
                ..
            } => {
                let declaration = self.type_reference_target(enum_target)?;
                Some(SemanticTargetId::Variant(VariantId::new(
                    declaration,
                    phalcom_ast::selector::selector_from_exact_case_target(variant_name, payload_shape.as_ref()),
                )))
            }
            _ => self.type_reference_target(annotation).map(SemanticTargetId::Declaration),
        }
    }

    fn visit_annotation_projection_targets(&mut self, annotation: &TypeAnnotation) {
        match &annotation.expr {
            TypeAnnotationExpr::AssociatedTypeProjection { subject, name_range, .. } => {
                self.visit_annotation_projection_targets(subject);
                if let Some(requirement) = self
                    .context
                    .associated_projection_targets
                    .get(&(self.index.module.clone(), *name_range))
                    .cloned()
                {
                    let site = self.allocate_site(self.current_owner.clone(), *name_range, SourceSiteKind::Occurrence);
                    self.index.register_target(site, SemanticTargetId::AssociatedType(requirement));
                }
            }
            TypeAnnotationExpr::QualifiedAssociatedTypeProjection {
                subject,
                trait_ref,
                name_range,
                ..
            } => {
                self.visit_annotation_projection_targets(subject);
                self.visit_annotation_projection_targets(&trait_ref.trait_type);
                for spec in &trait_ref.associated_specializations {
                    self.visit_annotation_projection_targets(&spec.value);
                }
                if let Some(requirement) = self
                    .context
                    .associated_projection_targets
                    .get(&(self.index.module.clone(), *name_range))
                    .cloned()
                {
                    let site = self.allocate_site(self.current_owner.clone(), *name_range, SourceSiteKind::Occurrence);
                    self.index.register_target(site, SemanticTargetId::AssociatedType(requirement));
                }
            }
            TypeAnnotationExpr::Application { origin, arguments, .. } => {
                self.visit_annotation_projection_targets(origin);
                for argument in arguments {
                    match argument {
                        phalcom_ast::ast::GenericArgumentSyntax::Single(annotation) => self.visit_annotation_projection_targets(annotation),
                        phalcom_ast::ast::GenericArgumentSyntax::Expansion(expansion) => self.visit_annotation_projection_targets(&expansion.operand),
                    }
                }
            }
            TypeAnnotationExpr::Union { members, .. } => {
                for member in members {
                    self.visit_annotation_projection_targets(member);
                }
            }
            TypeAnnotationExpr::Tuple { elements, .. } => {
                for element in elements {
                    match element {
                        phalcom_ast::ast::TypeTupleItem::Element(element) => self.visit_annotation_projection_targets(&element.ty),
                        phalcom_ast::ast::TypeTupleItem::Expansion(expansion) => self.visit_annotation_projection_targets(&expansion.operand),
                        phalcom_ast::ast::TypeTupleItem::Remainder(remainder) => {
                            if let phalcom_ast::ast::AnonymousRemainderSyntax::Typed { constraint, .. } = &remainder.remainder {
                                self.visit_annotation_projection_targets(constraint);
                            }
                        }
                    }
                }
            }
            TypeAnnotationExpr::Record { fields, tail, .. } => {
                for field in fields {
                    self.visit_annotation_projection_targets(&field.ty);
                }
                if let Some(phalcom_ast::ast::RecordRowTailSyntax::Anonymous(phalcom_ast::ast::AnonymousRemainderSyntax::Typed { constraint, .. })) = tail {
                    self.visit_annotation_projection_targets(constraint);
                }
            }
            TypeAnnotationExpr::Callable { parameters, result, .. } => {
                for parameter in parameters {
                    match parameter {
                        phalcom_ast::ast::TypeCallableItem::Parameter(parameter) => self.visit_annotation_projection_targets(&parameter.ty),
                        phalcom_ast::ast::TypeCallableItem::Expansion(expansion) => self.visit_annotation_projection_targets(&expansion.operand),
                        phalcom_ast::ast::TypeCallableItem::Remainder(_) => {}
                    }
                }
                self.visit_annotation_projection_targets(result);
            }
            TypeAnnotationExpr::TypeLambda { body, .. } => self.visit_annotation_projection_targets(body),
            TypeAnnotationExpr::ExactEnumCase {
                enum_target,
                generic_arguments,
                ..
            } => {
                self.visit_annotation_projection_targets(enum_target);
                for argument in generic_arguments {
                    self.visit_annotation_projection_targets(argument);
                }
            }
            TypeAnnotationExpr::TypeMemberProjection { subject, arguments, .. } => {
                self.visit_annotation_projection_targets(subject);
                for argument in arguments {
                    if let phalcom_ast::ast::TypeMemberArgumentSyntax::TypeForm(ty) = argument {
                        self.visit_annotation_projection_targets(ty);
                    }
                }
            }
            TypeAnnotationExpr::Reference(_)
            | TypeAnnotationExpr::Unit { .. }
            | TypeAnnotationExpr::Dynamic { .. }
            | TypeAnnotationExpr::Never { .. }
            | TypeAnnotationExpr::SelfType { .. }
            | TypeAnnotationExpr::Invalid { .. } => {}
        }
    }

    fn new_scope(&mut self, parent: SourceScopeId, range: SourceRange) -> SourceScopeId {
        let id = SourceScopeId(self.next_scope);
        self.next_scope += 1;
        self.index.add_scope(id, parent, range);
        id
    }

    fn allocate_site(&mut self, owner: SourceOwner, range: SourceRange, kind: SourceSiteKind) -> SourceSiteId {
        let next = self.next_site.entry(owner.clone()).or_default();
        let site = SourceSite::new(owner, SourceSiteLocalId(*next), range, kind);
        *next += 1;
        let id = site.id.clone();
        self.index.register_site(site);
        id
    }

    fn declare(&mut self, scope: SourceScopeId, name: impl Into<Box<str>>, kind: SourceBindingKind, range: SourceRange, mutable: bool) -> SourceSiteId {
        self.declare_with_annotation(scope, name, kind, range, mutable, false)
    }

    fn declare_with_annotation(
        &mut self,
        scope: SourceScopeId,
        name: impl Into<Box<str>>,
        kind: SourceBindingKind,
        range: SourceRange,
        mutable: bool,
        has_explicit_annotation: bool,
    ) -> SourceSiteId {
        let name = name.into();
        let first = self.index.scopes.get(&scope).and_then(|scope| scope.bindings.get(&name)).cloned();
        let site = self.allocate_site(self.current_owner.clone(), range, SourceSiteKind::BindingDeclaration);
        let primary = first.clone().unwrap_or_else(|| site.clone());
        let target = match kind {
            SourceBindingKind::TopLevelLet | SourceBindingKind::TopLevelConst => SemanticTargetId::ModuleBinding(SymbolId {
                module: self.index.module.clone(),
                name: name.clone(),
            }),
            _ => SemanticTargetId::Binding(primary.clone()),
        };
        self.index.register_target(site.clone(), target);
        self.index.register_binding(SourceBindingInfo {
            declaration_site: site.clone(),
            scope,
            name: name.clone(),
            kind,
            declaration_range: range,
            has_explicit_annotation,
            mutable,
            redeclaration_of: first,
        });
        if let Some(scope_info) = self.index.scopes.get_mut(&scope)
            && !scope_info.bindings.contains_key(&name)
        {
            scope_info.bindings.insert(name, site.clone());
        }
        site
    }

    fn visit_imports(&mut self, program: &Program) {
        let mut import_site_local = 0u32;
        for dependency in &program.preamble.dependencies {
            match dependency {
                phalcom_ast::ast::DependencyDecl::Import(import) => match import {
                    phalcom_ast::ast::ImportDecl::Module(module_import) => {
                        let import_site = ImportSiteId::new(self.index.module.clone(), ImportSiteLocalId::new(import_site_local));
                        import_site_local = import_site_local.saturating_add(1);
                        let name = module_import
                            .alias
                            .as_ref()
                            .map(|alias| alias.name.clone())
                            .or_else(|| module_import.path.segments.last().map(|segment| segment.name.clone()))
                            .or_else(|| match &module_import.path.root {
                                phalcom_ast::ast::ImportRoot::Absolute(segment) => Some(segment.name.clone()),
                                phalcom_ast::ast::ImportRoot::Relative { .. } => None,
                            });
                        let Some(name) = name else { continue };
                        let range = module_import.alias.as_ref().map_or(module_import.range, |alias| alias.range);
                        let site = self.declare(self.index.root, name.clone(), SourceBindingKind::Import, range, false);
                        let module = match self.context.import_products.get(&import_site) {
                            Some(product) => product.target.as_ref().ok().cloned(),
                            None if self.context.require_canonical_import_products => None,
                            None => self
                                .context
                                .resolved_imports
                                .get(&(self.index.module.clone(), module_import.path.to_string()))
                                .or_else(|| self.context.modules.get(&module_import.path.to_string()))
                                .cloned(),
                        };
                        if let Some(module) = module {
                            self.index.register_module(name, module.clone());
                            self.index.register_import_origin(ImportBindingOrigin {
                                local_binding: site,
                                remote_target: SemanticTargetId::Module(module.clone()),
                            });
                        }
                    }
                    phalcom_ast::ast::ImportDecl::Selective(selective_import) => {
                        let import_site = ImportSiteId::new(self.index.module.clone(), ImportSiteLocalId::new(import_site_local));
                        import_site_local = import_site_local.saturating_add(1);
                        let module = match self.context.import_products.get(&import_site) {
                            Some(product) => product.target.as_ref().ok().cloned(),
                            None if self.context.require_canonical_import_products => None,
                            None => self
                                .context
                                .resolved_imports
                                .get(&(self.index.module.clone(), selective_import.path.to_string()))
                                .or_else(|| self.context.modules.get(&selective_import.path.to_string()))
                                .cloned(),
                        };
                        for item in &selective_import.items {
                            let name = item.alias.as_ref().map_or_else(|| item.name.clone(), |alias| alias.name.clone());
                            let range = item.alias.as_ref().map_or(item.name_range, |alias| alias.range);
                            let site = self.declare(self.index.root, name, SourceBindingKind::Import, range, false);
                            if let Some(module) = module.as_ref()
                                && let Some(target) = self.context.targets.get(&(module.clone(), item.name.clone()))
                            {
                                self.index.register_import_origin(ImportBindingOrigin {
                                    local_binding: site,
                                    remote_target: target.clone(),
                                });
                            }
                        }
                    }
                },
                phalcom_ast::ast::DependencyDecl::ReExport(_) => {
                    import_site_local = import_site_local.saturating_add(1);
                }
                phalcom_ast::ast::DependencyDecl::Expose(_) => {}
            }
        }
    }

    fn visit_statements(&mut self, scope: SourceScopeId, statements: &[Statement], top_level: bool) {
        for (statement_index, statement) in statements.iter().enumerate() {
            match statement {
                Statement::Class(class) => self.visit_class(scope, class),
                Statement::TypeAlias(alias) => self.visit_type_alias(scope, alias),
                Statement::Data(data_def) => {
                    if data_def.phase == phalcom_ast::ast::AlgebraicPhase::TypeOnly {
                        self.type_data_definition(data_def);
                    } else if data_def.is_sum() {
                        self.visit_sum_data(scope, data_def);
                    } else {
                        self.visit_data(scope, data_def);
                    }
                }
                Statement::Trait(trait_def) => self.visit_trait(scope, trait_def),
                Statement::Let(binding) => self.visit_let(scope, binding, top_level),
                Statement::Return(return_statement) => {
                    if let Some(value) = &return_statement.value {
                        self.visit_expr(scope, value);
                    }
                }
                Statement::Expr { expr, .. } => self.visit_expr(scope, expr),
                Statement::For(for_statement) => self.visit_for(scope, for_statement),
                Statement::Throw { expr, .. } => self.visit_expr(scope, expr),
                Statement::Impl(impl_def) => self.visit_impl(scope, statement_index, impl_def),
                Statement::Export(_) | Statement::Break { .. } | Statement::Continue { .. } => {}
            }
        }
    }

    fn visit_impl(&mut self, parent: SourceScopeId, statement_index: usize, impl_def: &phalcom_ast::ast::ImplDef) {
        if matches!(impl_def.kind, phalcom_ast::ast::ImplKind::Inherent)
            && let Some(SemanticTargetId::Declaration(owner)) = self.type_reference_semantic_target(&impl_def.target)
        {
            self.visit_associated_type_data(&owner, &impl_def.associated_type_data);
        }
        if matches!(impl_def.kind, phalcom_ast::ast::ImplKind::Conformance { .. }) {
            if let phalcom_ast::ast::ImplKind::Conformance { trait_ref, .. } = &impl_def.kind
                && let Some(target) = self.type_reference_semantic_target(trait_ref)
            {
                let site = self.allocate_site(self.current_owner.clone(), trait_ref.range, SourceSiteKind::Occurrence);
                self.index.register_target(site, target);
            }
            if let Some(target) = self.type_reference_semantic_target(&impl_def.target) {
                let site = self.allocate_site(self.current_owner.clone(), impl_def.target.range, SourceSiteKind::Occurrence);
                self.index.register_target(site, target);
            }
            let owner = CallableOwnerId::Conformance(crate::identity::ImplId::new(
                self.index.module.clone(),
                crate::identity::ImplLocalId(statement_index as u32),
            ));
            for impl_member in &impl_def.members {
                if let phalcom_ast::ast::ImplMember::AssociatedTypeBinding(binding) = impl_member {
                    if let Some(requirement) = self
                        .context
                        .associated_type_targets
                        .get(&(self.index.module.clone(), binding.name_range))
                        .cloned()
                    {
                        let site = self.allocate_site(self.current_owner.clone(), binding.name_range, SourceSiteKind::Occurrence);
                        self.index.register_target(site, SemanticTargetId::AssociatedType(requirement));
                    }
                    self.visit_annotation_projection_targets(&binding.value);
                    continue;
                }
                let Some(member) = impl_member.behavior() else {
                    continue;
                };
                self.visit_behavior_member(parent, owner.clone(), member, behavior_side(member));
            }
            return;
        }
        // The semantic session publishes the resolved target for the exact
        // target annotation range. Source indexing consumes that canonical
        // identity rather than resolving an impl target from a written name.
        // The local-class fallback keeps the low-level source-index builder
        // useful in isolated tests that intentionally provide no semantic
        // context.
        let owner = match &impl_def.target.expr {
            phalcom_ast::ast::TypeAnnotationExpr::ExactEnumCase {
                enum_target,
                variant_name,
                variant_name_range,
                payload_shape,
                ..
            } => {
                let enum_decl = self
                    .context
                    .type_reference_targets
                    .get(&(self.index.module.clone(), enum_target.range))
                    .cloned()
                    .or_else(|| {
                        enum_target.origin_symbol_ref().and_then(|reference| {
                            reference
                                .members
                                .is_empty()
                                .then(|| self.index.declaration_for_name(&reference.root).cloned())
                                .flatten()
                        })
                    });
                if let Some(enum_decl) = enum_decl {
                    let selector = phalcom_ast::selector::selector_from_exact_case_target(variant_name, payload_shape.as_ref());
                    let variant_id = VariantId::new(enum_decl, selector);
                    let variant_site = self.allocate_site(
                        SourceOwner::Module(self.index.module.clone()),
                        *variant_name_range,
                        SourceSiteKind::Variant(variant_id.clone()),
                    );
                    self.index.register_target(variant_site, SemanticTargetId::Variant(variant_id.clone()));
                    CallableOwnerId::Variant(variant_id)
                } else {
                    return;
                }
            }
            _ => {
                let target = self
                    .context
                    .type_reference_targets
                    .get(&(self.index.module.clone(), impl_def.target_range))
                    .cloned()
                    .or_else(|| {
                        impl_def.target.origin_symbol_ref().and_then(|reference| {
                            reference
                                .members
                                .is_empty()
                                .then(|| self.index.declaration_for_name(&reference.root).cloned())
                                .flatten()
                        })
                    });
                let Some(target) = target else { return };
                CallableOwnerId::Declaration(target)
            }
        };
        for impl_member in &impl_def.members {
            if let phalcom_ast::ast::ImplMember::TypeMember(definition) = impl_member {
                let CallableOwnerId::Declaration(declaration) = &owner else { continue };
                let id = crate::types::type_member::TypeMemberDeclarationId::new(
                    declaration.clone(),
                    crate::types::type_member::TypeMemberSelector::from_syntax(&definition.selector),
                );
                let name_range = definition.selector.range();
                let site = self.allocate_site(
                    SourceOwner::Module(self.index.module.clone()),
                    name_range,
                    SourceSiteKind::TypeMember(id.clone()),
                );
                self.index.register_target(site.clone(), SemanticTargetId::TypeMember(id.clone()));
                self.index.type_member_sources.insert(
                    id.clone(),
                    TypeMemberSourceInfo {
                        id,
                        declaration_site: site,
                        name_range,
                        declaration_range: definition.range,
                    },
                );
                continue;
            }
            let Some(member) = impl_member.behavior() else {
                continue;
            };
            let side = behavior_side(member);
            self.visit_behavior_member(parent, owner.clone(), member, side);
        }
    }

    fn visit_class(&mut self, parent: SourceScopeId, class: &ClassDef) {
        let declaration = self.declaration_id(class);
        let site = self.allocate_site(
            SourceOwner::Module(self.index.module.clone()),
            class.name_range,
            SourceSiteKind::Declaration(declaration.clone()),
        );
        self.index.register_class(class.name.clone(), declaration.clone());
        self.index.register_target(site.clone(), SemanticTargetId::Declaration(declaration.clone()));
        self.index.declaration_sources.insert(
            declaration.clone(),
            DeclarationSourceInfo {
                id: declaration.clone(),
                name: class.name.clone().into(),
                kind: SourceDeclarationKind::Class,
                declaration_site: site,
                name_range: class.name_range,
                declaration_range: class.range,
            },
        );
        self.visit_associated_type_data(&declaration, &class.associated_type_data);
        for member in &class.members {
            self.visit_member(parent, &declaration, member);
        }
    }

    fn visit_trait(&mut self, parent: SourceScopeId, trait_def: &phalcom_ast::ast::TraitDef) {
        let declaration = DeclarationId::new(self.index.module.clone(), trait_def.name.clone().into());
        let site = self.allocate_site(
            SourceOwner::Module(self.index.module.clone()),
            trait_def.name_range,
            SourceSiteKind::Declaration(declaration.clone()),
        );
        self.index.register_class(trait_def.name.clone(), declaration.clone());
        self.index.register_target(site.clone(), SemanticTargetId::Declaration(declaration.clone()));
        self.index.declaration_sources.insert(
            declaration.clone(),
            DeclarationSourceInfo {
                id: declaration.clone(),
                name: trait_def.name.clone().into(),
                kind: SourceDeclarationKind::Trait,
                declaration_site: site,
                name_range: trait_def.name_range,
                declaration_range: trait_def.range,
            },
        );
        let mut associated_type_index = 0u32;
        for trait_member in &trait_def.members {
            if let phalcom_ast::ast::TraitMember::AssociatedType(declaration_member) = trait_member {
                let requirement = AssociatedTypeRequirementId::new(declaration.clone(), associated_type_index);
                associated_type_index += 1;
                let site = self.allocate_site(
                    SourceOwner::Module(self.index.module.clone()),
                    declaration_member.name_range,
                    SourceSiteKind::AssociatedType(requirement.clone()),
                );
                self.index.register_target(site, SemanticTargetId::AssociatedType(requirement));
                continue;
            }
            if let phalcom_ast::ast::TraitMember::Property(property) = trait_member {
                self.visit_annotation_projection_targets(&property.annotation);
            }
            let Some(member) = trait_member.behavior() else {
                continue;
            };
            self.visit_behavior_member(parent, CallableOwnerId::Declaration(declaration.clone()), member, behavior_side(member));
        }
    }

    fn visit_type_alias(&mut self, _parent: SourceScopeId, alias: &phalcom_ast::ast::TypeAliasDef) {
        let declaration = DeclarationId::new(self.index.module.clone(), alias.name.clone().into());
        let site = self.allocate_site(
            SourceOwner::Module(self.index.module.clone()),
            alias.name_range,
            SourceSiteKind::Declaration(declaration.clone()),
        );
        self.index.register_target(site.clone(), SemanticTargetId::Declaration(declaration.clone()));
        self.index.declaration_sources.insert(
            declaration.clone(),
            DeclarationSourceInfo {
                id: declaration,
                name: alias.name.clone().into(),
                kind: SourceDeclarationKind::TypeAlias,
                declaration_site: site,
                name_range: alias.name_range,
                declaration_range: alias.range,
            },
        );
    }

    fn visit_sum_data(&mut self, _parent: SourceScopeId, data_def: &EnumDef) {
        let declaration = DeclarationId::new(self.index.module.clone(), data_def.name.clone().into());
        let declaration_site = self.allocate_site(
            SourceOwner::Module(self.index.module.clone()),
            data_def.name_range,
            SourceSiteKind::Declaration(declaration.clone()),
        );
        self.index
            .register_target(declaration_site.clone(), SemanticTargetId::Declaration(declaration.clone()));
        self.index.declaration_sources.insert(
            declaration.clone(),
            DeclarationSourceInfo {
                id: declaration.clone(),
                name: data_def.name.clone().into(),
                kind: SourceDeclarationKind::Data,
                declaration_site,
                name_range: data_def.name_range,
                declaration_range: data_def.range,
            },
        );

        for variant in data_def.cases().unwrap_or_default() {
            let variant_id = VariantId::new(declaration.clone(), phalcom_ast::selector::selector_from_variant(variant));
            let variant_site = self.allocate_site(
                SourceOwner::Module(self.index.module.clone()),
                variant.name_range,
                SourceSiteKind::Variant(variant_id.clone()),
            );
            self.index.register_target(variant_site, SemanticTargetId::Variant(variant_id.clone()));
            if let Some(family) = variant_id.family() {
                let family_site = self.allocate_site(
                    SourceOwner::Module(self.index.module.clone()),
                    variant.name_range,
                    SourceSiteKind::VariantFamily(family.clone()),
                );
                self.index.register_target(family_site, SemanticTargetId::VariantFamily(family));
            }
            match &variant.payload {
                phalcom_ast::ast::CasePayloadSyntax::Tuple { parameters, .. } => {
                    for (index, parameter) in parameters.iter().enumerate() {
                        let field = VariantFieldId::new(variant_id.clone(), index as u32);
                        let site = self.allocate_site(
                            SourceOwner::Module(self.index.module.clone()),
                            parameter.name_range,
                            SourceSiteKind::VariantField(field.clone()),
                        );
                        self.index.register_target(site, SemanticTargetId::VariantField(field));
                    }
                }
                phalcom_ast::ast::CasePayloadSyntax::Record { components, .. } => {
                    for (index, component) in components.iter().enumerate() {
                        let field = VariantFieldId::new(variant_id.clone(), index as u32);
                        let site = self.allocate_site(
                            SourceOwner::Module(self.index.module.clone()),
                            component.name_range,
                            SourceSiteKind::VariantField(field.clone()),
                        );
                        self.index.register_target(site, SemanticTargetId::VariantField(field));
                    }
                }
                phalcom_ast::ast::CasePayloadSyntax::Singleton => {}
            }
        }
    }

    fn type_data_definition(&mut self, data_def: &EnumDef) {
        let declaration = DeclarationId::new(self.index.module.clone(), data_def.name.clone().into());
        let domain = StaticDomainDeclarationId::Nominal(declaration.clone());
        let site = self.allocate_site(
            SourceOwner::Module(self.index.module.clone()),
            data_def.name_range,
            SourceSiteKind::Declaration(declaration.clone()),
        );
        self.index.register_target(site.clone(), SemanticTargetId::StaticDomain(domain.clone()));
        self.index.declaration_sources.insert(
            declaration.clone(),
            DeclarationSourceInfo {
                id: declaration.clone(),
                name: data_def.name.clone().into(),
                kind: SourceDeclarationKind::TypeData,
                declaration_site: site,
                name_range: data_def.name_range,
                declaration_range: data_def.range,
            },
        );
        self.visit_static_cases(&domain, data_def);
        for (index, component) in data_def.components().unwrap_or_default().iter().enumerate() {
            let component_id = DataComponentId::new(declaration.clone(), index as u32);
            let site = self.allocate_site(
                SourceOwner::Module(self.index.module.clone()),
                component.name_range,
                SourceSiteKind::DataComponent(component_id.clone()),
            );
            self.index.register_target(site, SemanticTargetId::DataComponent(component_id));
        }
    }

    fn visit_associated_type_data(&mut self, owner: &DeclarationId, definitions: &[EnumDef]) {
        for definition in definitions {
            let domain = StaticDomainDeclarationId::Associated(crate::identity::AssociatedStaticDomainDeclarationId::new(
                owner.clone(),
                definition.name.clone(),
            ));
            let site = self.allocate_site(
                SourceOwner::Module(self.index.module.clone()),
                definition.name_range,
                SourceSiteKind::StaticDomain(domain.clone()),
            );
            self.index.register_target(site, SemanticTargetId::StaticDomain(domain.clone()));
            self.visit_static_cases(&domain, definition);
        }
    }

    fn visit_static_cases(&mut self, domain: &StaticDomainDeclarationId, definition: &EnumDef) {
        for case in definition.cases().unwrap_or_default() {
            let id = StaticCaseId::with_selector(domain.clone(), phalcom_ast::selector::selector_from_variant(case));
            let site = self.allocate_site(
                SourceOwner::Module(self.index.module.clone()),
                case.name_range,
                SourceSiteKind::StaticCase(id.clone()),
            );
            self.index.register_target(site, SemanticTargetId::StaticCase(id));
        }
    }

    fn visit_data(&mut self, _parent: SourceScopeId, data_def: &phalcom_ast::ast::DataDef) {
        let declaration = DeclarationId::new(self.index.module.clone(), data_def.name.clone().into());
        let declaration_site = self.allocate_site(
            SourceOwner::Module(self.index.module.clone()),
            data_def.name_range,
            SourceSiteKind::Declaration(declaration.clone()),
        );
        self.index
            .register_target(declaration_site.clone(), SemanticTargetId::Declaration(declaration.clone()));
        self.index.declaration_sources.insert(
            declaration.clone(),
            DeclarationSourceInfo {
                id: declaration.clone(),
                name: data_def.name.clone().into(),
                kind: SourceDeclarationKind::Data,
                declaration_site,
                name_range: data_def.name_range,
                declaration_range: data_def.range,
            },
        );
        let components = data_def.components().unwrap_or_default();
        for (index, component) in components.iter().enumerate() {
            let component_id = DataComponentId::new(declaration.clone(), index as u32);
            let site = self.allocate_site(
                SourceOwner::Module(self.index.module.clone()),
                component.name_range,
                SourceSiteKind::DataComponent(component_id.clone()),
            );
            self.index.register_target(site, SemanticTargetId::DataComponent(component_id));
        }
    }

    fn visit_behavior_member(&mut self, parent: SourceScopeId, owner: CallableOwnerId, behavior: &phalcom_ast::ast::BehaviorMember, side: DispatchSide) {
        match behavior {
            BehaviorMember::Method(method) => {
                if let Some(annotation) = &method.return_annotation {
                    self.visit_annotation_projection_targets(annotation);
                }
            }
            BehaviorMember::Getter(getter) => {
                if let Some(annotation) = &getter.return_annotation {
                    self.visit_annotation_projection_targets(annotation);
                }
            }
            BehaviorMember::Setter(setter) => {
                if let Some(annotation) = &setter.return_annotation {
                    self.visit_annotation_projection_targets(annotation);
                }
            }
            BehaviorMember::Index(index) => {
                if let Some(annotation) = &index.return_annotation {
                    self.visit_annotation_projection_targets(annotation);
                }
            }
        }
        let syntax = crate::checker::declaration_signature::CallableSyntaxRef::from(behavior);
        let Some(callable) = crate::checker::declaration_signature::callable_id_for_syntax(&owner, syntax, side) else {
            return;
        };
        let declaration_range = behavior
            .attributes()
            .first()
            .map_or(behavior.range(), |attribute| SourceRange::new(attribute.range.start, behavior.range().end));
        match behavior {
            BehaviorMember::Method(method) => self.visit_callable(CallableVisit {
                parent,
                callable,
                name_range: method.name_range,
                declaration_range,
                body_range: method.range,
                parameters: &method.params,
                body: &method.body,
                parameter_kind: SourceBindingKind::MethodParameter,
                kind: if method.is_constructor || method.attributes.iter().any(|attribute| attribute.name == "constructor") {
                    SourceCallableKind::Constructor
                } else {
                    SourceCallableKind::Method
                },
                has_explicit_return_annotation: method.return_annotation.is_some(),
            }),
            BehaviorMember::Getter(getter) => self.visit_callable(CallableVisit {
                parent,
                callable,
                name_range: getter.name_range,
                declaration_range,
                body_range: getter.range,
                parameters: &[],
                body: &getter.body,
                parameter_kind: SourceBindingKind::MethodParameter,
                kind: SourceCallableKind::Getter,
                has_explicit_return_annotation: getter.return_annotation.is_some(),
            }),
            BehaviorMember::Setter(setter) => self.visit_callable(CallableVisit {
                parent,
                callable,
                name_range: setter.name_range,
                declaration_range,
                body_range: setter.range,
                parameters: std::slice::from_ref(&setter.param),
                body: &setter.body,
                parameter_kind: SourceBindingKind::SetterParameter,
                kind: SourceCallableKind::Setter,
                has_explicit_return_annotation: setter.return_annotation.is_some(),
            }),
            BehaviorMember::Index(index) => {
                let mut parameters = index.params.clone();
                if let phalcom_ast::ast::IndexAccessor::Set { value } = &index.accessor {
                    parameters.push((**value).clone());
                }
                self.visit_callable(CallableVisit {
                    parent,
                    callable,
                    name_range: index.name_range,
                    declaration_range,
                    body_range: index.range,
                    parameters: &parameters,
                    body: &index.body,
                    parameter_kind: SourceBindingKind::IndexParameter,
                    kind: match &index.accessor {
                        phalcom_ast::ast::IndexAccessor::Get => SourceCallableKind::IndexGet,
                        phalcom_ast::ast::IndexAccessor::Set { .. } => SourceCallableKind::IndexSet,
                    },
                    has_explicit_return_annotation: index.return_annotation.is_some(),
                });
            }
        }
    }

    fn visit_member(&mut self, parent: SourceScopeId, declaration: &DeclarationId, member: &ClassMember) {
        let member_side = crate::checker::declaration::member_side(member);
        let member_range = member.range();
        let declaration_range = member
            .attributes()
            .first()
            .map_or(member_range, |attribute| SourceRange::new(attribute.range.start, member_range.end));
        match member {
            ClassMember::Method(method) => {
                let is_constructor = method.is_constructor || method.attributes.iter().any(|attribute| attribute.name == "constructor");
                let Some(callable) = self.method_callable(declaration, method, is_constructor, member_side) else {
                    return;
                };
                self.visit_callable(CallableVisit {
                    parent,
                    callable,
                    name_range: method.name_range,
                    declaration_range,
                    body_range: method.range,
                    parameters: &method.params,
                    body: &method.body,
                    parameter_kind: SourceBindingKind::MethodParameter,
                    kind: if is_constructor {
                        SourceCallableKind::Constructor
                    } else {
                        SourceCallableKind::Method
                    },
                    has_explicit_return_annotation: method.return_annotation.is_some(),
                });
            }
            ClassMember::Getter(getter) => {
                let Some(callable) = Selector::getter(&getter.name)
                    .ok()
                    .map(|selector| CallableId::new(declaration.clone(), selector, member_side))
                else {
                    return;
                };
                self.visit_callable(CallableVisit {
                    parent,
                    callable,
                    name_range: getter.name_range,
                    declaration_range,
                    body_range: getter.range,
                    parameters: &[],
                    body: &getter.body,
                    parameter_kind: SourceBindingKind::MethodParameter,
                    kind: SourceCallableKind::Getter,
                    has_explicit_return_annotation: getter.return_annotation.is_some(),
                });
            }
            ClassMember::Setter(setter) => {
                let Some(callable) = Selector::setter(&setter.name)
                    .ok()
                    .map(|selector| CallableId::new(declaration.clone(), selector, member_side))
                else {
                    return;
                };
                self.visit_callable(CallableVisit {
                    parent,
                    callable,
                    name_range: setter.name_range,
                    declaration_range,
                    body_range: setter.range,
                    parameters: std::slice::from_ref(&setter.param),
                    body: &setter.body,
                    parameter_kind: SourceBindingKind::SetterParameter,
                    kind: SourceCallableKind::Setter,
                    has_explicit_return_annotation: setter.return_annotation.is_some(),
                });
            }
            ClassMember::Index(index) => {
                let slots = index.params.iter().map(parameter_slot).collect::<Vec<_>>();
                let selector = match index.accessor {
                    phalcom_ast::ast::IndexAccessor::Get => Selector::subscript_get(slots),
                    phalcom_ast::ast::IndexAccessor::Set { .. } => Selector::subscript_set(slots),
                };
                let Some(selector) = selector.ok() else { return };
                let callable = CallableId::new(declaration.clone(), selector, member_side);
                let mut parameters = index.params.clone();
                if let phalcom_ast::ast::IndexAccessor::Set { value } = &index.accessor {
                    parameters.push((**value).clone());
                }
                self.visit_callable(CallableVisit {
                    parent,
                    callable,
                    name_range: index.name_range,
                    declaration_range,
                    body_range: index.range,
                    parameters: &parameters,
                    body: &index.body,
                    parameter_kind: SourceBindingKind::IndexParameter,
                    kind: match &index.accessor {
                        phalcom_ast::ast::IndexAccessor::Get => SourceCallableKind::IndexGet,
                        phalcom_ast::ast::IndexAccessor::Set { .. } => SourceCallableKind::IndexSet,
                    },
                    has_explicit_return_annotation: index.return_annotation.is_some(),
                });
            }
            ClassMember::Field(field) => {
                let field_id = FieldId::new(declaration.clone(), field.name.clone(), member_side);
                let site = self.allocate_site(
                    SourceOwner::Module(self.index.module.clone()),
                    field.name_range,
                    SourceSiteKind::Field(field_id.clone()),
                );
                self.index.register_target(site.clone(), SemanticTargetId::Field(field_id.clone()));
                self.index.field_sources.insert(
                    field_id.clone(),
                    FieldSourceInfo {
                        id: field_id,
                        declaration_site: site,
                        name_range: field.name_range,
                        declaration_range: field.range,
                        has_explicit_annotation: field.annotation.is_some(),
                    },
                );
                if let Some(default) = &field.default {
                    self.visit_expr(parent, default);
                }
            }
            ClassMember::TypeMember(definition) => {
                let selector = crate::types::type_member::TypeMemberSelector::from_syntax(&definition.selector);
                let id = crate::types::type_member::TypeMemberDeclarationId::new(declaration.clone(), selector);
                let name_range = definition.selector.range();
                let site = self.allocate_site(
                    SourceOwner::Module(self.index.module.clone()),
                    name_range,
                    SourceSiteKind::TypeMember(id.clone()),
                );
                self.index.register_target(site.clone(), SemanticTargetId::TypeMember(id.clone()));
                self.index.type_member_sources.insert(
                    id.clone(),
                    TypeMemberSourceInfo {
                        id,
                        declaration_site: site,
                        name_range,
                        declaration_range: definition.range,
                    },
                );
            }
            ClassMember::Variant(_) | ClassMember::Delegation(_) => {}
        }
    }

    fn method_callable(
        &self,
        declaration: &DeclarationId,
        method: &phalcom_ast::ast::MethodDef,
        is_constructor: bool,
        member_side: DispatchSide,
    ) -> Option<CallableId> {
        let slots = method
            .params
            .iter()
            .filter(|p| p.rest_mode == phalcom_ast::ast::RestMode::None)
            .map(parameter_slot)
            .collect::<Vec<_>>();
        Selector::method(&method.name, slots)
            .ok()
            .map(|selector| CallableId::new(declaration.clone(), selector, if is_constructor { DispatchSide::Class } else { member_side }))
    }

    fn visit_callable(&mut self, visit: CallableVisit<'_>) {
        let CallableVisit {
            parent,
            callable,
            name_range,
            declaration_range,
            body_range,
            parameters,
            body,
            parameter_kind,
            kind,
            has_explicit_return_annotation,
        } = visit;
        let declaration_site = self.allocate_site(
            SourceOwner::Module(self.index.module.clone()),
            name_range,
            SourceSiteKind::Callable(callable.clone()),
        );
        self.index
            .register_target(declaration_site.clone(), SemanticTargetId::Callable(callable.clone()));
        self.index.callable_sources.insert(
            callable.clone(),
            CallableSourceInfo {
                id: callable.clone(),
                kind,
                declaration_site,
                name_range,
                declaration_range,
                parameter_sites: BTreeMap::new(),
                has_explicit_return_annotation,
            },
        );
        self.index.callable_body_ranges.insert(callable.clone(), body_range);
        let previous_owner = std::mem::replace(&mut self.current_owner, SourceOwner::Callable(callable.clone()));
        let scope = self.new_scope(parent, body_range);
        let mut parameter_sites = BTreeMap::new();
        for (index, parameter) in parameters.iter().enumerate() {
            if let Some(annotation) = &parameter.annotation {
                self.visit_annotation_projection_targets(annotation);
            }
            let site = self.declare_with_annotation(
                scope,
                parameter.name.clone(),
                parameter_kind,
                parameter.name_range,
                true,
                parameter.annotation.is_some(),
            );
            parameter_sites.insert(CallableParameterId::new(callable.clone(), index as u32), site);
        }
        if let Some(source) = self.index.callable_sources.get_mut(&callable) {
            source.parameter_sites = parameter_sites;
        }
        if let Some(statements) = body.statements() {
            self.visit_statements(scope, statements, false);
        }
        self.current_owner = previous_owner;
    }

    fn visit_let(&mut self, scope: SourceScopeId, binding: &LetBinding, top_level: bool) {
        if let Some(value) = &binding.value {
            self.visit_expr(scope, value);
        }
        let kind = match (top_level, binding.kind) {
            (true, BindingKind::Let) => SourceBindingKind::TopLevelLet,
            (true, BindingKind::Const) => SourceBindingKind::TopLevelConst,
            (false, BindingKind::Let) => SourceBindingKind::LocalLet,
            (false, BindingKind::Const) => SourceBindingKind::LocalConst,
        };
        self.declare_pattern_with_annotation(scope, &binding.pattern, kind, binding.kind == BindingKind::Let, binding.annotation.is_some());
    }

    fn declare_pattern(&mut self, scope: SourceScopeId, pattern: &Pattern, kind: SourceBindingKind, mutable: bool) {
        self.declare_pattern_with_annotation(scope, pattern, kind, mutable, false);
    }

    fn declare_pattern_with_annotation(
        &mut self,
        scope: SourceScopeId,
        pattern: &Pattern,
        kind: SourceBindingKind,
        mutable: bool,
        has_explicit_annotation: bool,
    ) {
        match pattern {
            Pattern::Name { name, range } => {
                self.declare_with_annotation(scope, name.clone(), kind, *range, mutable, has_explicit_annotation);
            }
            Pattern::Tuple { elements, .. } => {
                for element in elements {
                    self.declare_pattern_with_annotation(scope, element, SourceBindingKind::Destructure, mutable, has_explicit_annotation);
                }
            }
            Pattern::List { elements, rest, .. } => {
                for element in elements {
                    self.declare_pattern_with_annotation(scope, element, SourceBindingKind::Destructure, mutable, has_explicit_annotation);
                }
                if let Some(rest) = rest {
                    self.declare_pattern_with_annotation(scope, rest, SourceBindingKind::Destructure, mutable, has_explicit_annotation);
                }
            }
            Pattern::Wildcard { .. } => {}
            Pattern::Or { alternatives, .. } => {
                for alt in alternatives {
                    self.declare_pattern_with_annotation(scope, alt, SourceBindingKind::Destructure, mutable, has_explicit_annotation);
                }
            }
            Pattern::Variant(variant) => match &variant.mode {
                phalcom_ast::ast::VariantPatternMode::ExactCall { arguments } => {
                    for argument in arguments {
                        self.declare_pattern_with_annotation(scope, &argument.pattern, SourceBindingKind::Destructure, mutable, has_explicit_annotation);
                    }
                }
                phalcom_ast::ast::VariantPatternMode::CallablePattern { prefix, suffix, .. } => {
                    for argument in prefix.iter().chain(suffix.iter()) {
                        self.declare_pattern_with_annotation(scope, &argument.pattern, SourceBindingKind::Destructure, mutable, has_explicit_annotation);
                    }
                }
                phalcom_ast::ast::VariantPatternMode::Singleton | phalcom_ast::ast::VariantPatternMode::WholeFamily { .. } => {}
            },
            Pattern::Record { entries, rest, .. } => {
                for entry in entries {
                    self.declare_pattern_with_annotation(scope, &entry.pattern, SourceBindingKind::Destructure, mutable, has_explicit_annotation);
                }
                if let Some(rest) = rest {
                    self.declare_pattern_with_annotation(scope, rest, SourceBindingKind::Destructure, mutable, has_explicit_annotation);
                }
            }
            Pattern::Map { entries, .. } => {
                for entry in entries {
                    self.declare_pattern_with_annotation(scope, &entry.pattern, SourceBindingKind::Destructure, mutable, has_explicit_annotation);
                }
            }
        }
    }

    fn visit_for(&mut self, parent: SourceScopeId, statement: &ForStatement) {
        for lane in &statement.lanes {
            self.visit_expr(parent, &lane.iter);
        }
        let scope = self.new_scope(parent, statements_range(&statement.body));
        for lane in &statement.lanes {
            self.declare_pattern(scope, &lane.pattern, SourceBindingKind::ForBinding, true);
            if let Some(index) = &lane.index {
                self.declare(scope, index.name.clone(), SourceBindingKind::ForBinding, index.range, true);
            }
        }
        self.visit_statements(scope, &statement.body, false);
    }

    fn visit_expr(&mut self, scope: SourceScopeId, expr: &Expr) {
        match &self.current_owner {
            SourceOwner::Module(_) => {
                self.allocate_site(self.current_owner.clone(), expr.range(), SourceSiteKind::Expression);
            }
            SourceOwner::Callable(callable) => {
                self.index.callable_expression_ranges.entry(callable.clone()).or_default().push(expr.range());
            }
        }
        match expr {
            Expr::Assignment(assignment) => {
                self.visit_expr(scope, &assignment.name);
                self.visit_expr(scope, &assignment.value);
            }
            Expr::Range(range) => {
                if let Some(lower) = &range.lower {
                    self.visit_expr(scope, lower);
                }
                if let Some(upper) = &range.upper {
                    self.visit_expr(scope, upper);
                }
            }
            Expr::Unary(unary) => self.visit_expr(scope, &unary.expr),
            Expr::Binary(binary) => {
                self.visit_expr(scope, &binary.left);
                self.visit_expr(scope, &binary.right);
            }
            Expr::ComparisonChain(chain) => {
                for operand in &chain.operands {
                    self.visit_expr(scope, operand);
                }
            }
            Expr::IfLet(if_let) => {
                self.visit_expr(scope, &if_let.value);
                let then_scope = self.new_scope(scope, if_let.then_body.range);
                self.declare_pattern(then_scope, &if_let.pattern, SourceBindingKind::Destructure, true);
                self.visit_statements(then_scope, &if_let.then_body.body, false);
                if let Some(else_body) = &if_let.else_body {
                    let else_scope = self.new_scope(scope, else_body.range);
                    self.visit_statements(else_scope, &else_body.body, false);
                }
            }
            Expr::If(if_expr) => {
                self.visit_expr(scope, &if_expr.condition);
                let then_scope = self.new_scope(scope, if_expr.then_body.range);
                self.visit_statements(then_scope, &if_expr.then_body.body, false);
                if let Some(else_body) = &if_expr.else_body {
                    match else_body {
                        phalcom_ast::ast::IfElseBody::Block(block) => {
                            let else_scope = self.new_scope(scope, block.range);
                            self.visit_statements(else_scope, &block.body, false);
                        }
                        phalcom_ast::ast::IfElseBody::ElseIf(expr) => self.visit_expr(scope, expr),
                    }
                }
            }
            Expr::WhileLet(while_let) => {
                let loop_scope = self.new_scope(scope, while_let.range);
                self.visit_expr(loop_scope, &while_let.value);
                self.declare_pattern(loop_scope, &while_let.pattern, SourceBindingKind::Destructure, true);
                self.visit_statements(loop_scope, &while_let.body, false);
            }
            Expr::Membership(membership) => {
                self.visit_expr(scope, &membership.left);
                self.visit_expr(scope, &membership.right);
            }
            Expr::IsMembership(membership) => {
                self.visit_expr(scope, &membership.left);
                self.visit_expr(scope, &membership.candidates);
            }
            Expr::UnqualifiedCall(call) => self.visit_pack_items(scope, &call.args),
            Expr::MethodCall(call) => {
                self.visit_expr(scope, &call.object);
                self.visit_pack_items(scope, &call.args);
            }
            Expr::GetProperty(property) => self.visit_expr(scope, &property.object),
            Expr::SetProperty(property) => {
                self.visit_expr(scope, &property.object);
                self.visit_expr(scope, &property.value);
            }
            Expr::Index(index) => {
                self.visit_expr(scope, &index.object);
                self.visit_pack_items(scope, &index.args);
            }
            Expr::SetIndex(index) => {
                self.visit_expr(scope, &index.object);
                self.visit_pack_items(scope, &index.args);
                self.visit_expr(scope, &index.value);
            }
            Expr::Block(block) => self.visit_block(scope, block),
            Expr::TupleLiteral(tuple) => {
                for entry in &tuple.entries {
                    match entry {
                        phalcom_ast::ast::TupleLiteralEntry::Positional { expr, .. } | phalcom_ast::ast::TupleLiteralEntry::Expand { expr, .. } => {
                            self.visit_expr(scope, expr)
                        }
                        phalcom_ast::ast::TupleLiteralEntry::Labeled { label, value, .. } => {
                            self.visit_product_label(scope, label);
                            self.visit_expr(scope, value);
                        }
                    }
                }
            }
            Expr::RecordLiteral(record) => {
                for entry in &record.entries {
                    match entry {
                        phalcom_ast::ast::RecordLiteralEntry::Field(field) => {
                            self.visit_product_label(scope, &field.label);
                            self.visit_expr(scope, &field.value);
                        }
                        phalcom_ast::ast::RecordLiteralEntry::Expansion { expr, .. } => self.visit_expr(scope, expr),
                    }
                }
            }
            Expr::MapLiteral(map) => {
                for entry in &map.entries {
                    match entry {
                        phalcom_ast::ast::MapLiteralEntry::Association { key, value, .. } => {
                            if let phalcom_ast::ast::MapLiteralKey::Computed { expr, .. } = key {
                                self.visit_expr(scope, expr);
                            }
                            self.visit_expr(scope, value);
                        }
                        phalcom_ast::ast::MapLiteralEntry::Expansion { expr, .. } => self.visit_expr(scope, expr),
                    }
                }
            }
            Expr::SetLiteral(set) => {
                for entry in &set.entries {
                    match entry {
                        phalcom_ast::ast::SetLiteralEntry::Element { expr, .. } | phalcom_ast::ast::SetLiteralEntry::Expansion { expr, .. } => {
                            self.visit_expr(scope, expr)
                        }
                    }
                }
            }
            Expr::ListLiteral(list) => {
                for element in &list.elements {
                    match element {
                        phalcom_ast::ast::ListLiteralElement::Element { expr, .. } | phalcom_ast::ast::ListLiteralElement::Expansion { expr, .. } => {
                            self.visit_expr(scope, expr)
                        }
                    }
                }
            }
            Expr::AssociatedLookup(lookup) => self.visit_expr(scope, &lookup.receiver),
            Expr::AssociatedInvoke(invoke) => {
                self.visit_expr(scope, &invoke.receiver);
                self.visit_pack_items(scope, &invoke.args);
            }
            Expr::CallableReference(reference) => match &reference.target {
                phalcom_ast::ast::CallableReferenceTarget::Bound { receiver, .. } | phalcom_ast::ast::CallableReferenceTarget::Associated { receiver, .. } => {
                    self.visit_expr(scope, receiver)
                }
                phalcom_ast::ast::CallableReferenceTarget::PrimaryDataConstructor { receiver, .. } => self.visit_expr(scope, receiver),
            },
            Expr::Match(match_expr) => {
                self.visit_expr(scope, &match_expr.value);
                for arm in &match_expr.arms {
                    let arm_scope = self.new_scope(scope, arm.range);
                    self.declare_pattern(arm_scope, &arm.pattern, SourceBindingKind::Destructure, false);
                    match &arm.branch {
                        phalcom_ast::ast::MatchArmBody::Expression(expr) => self.visit_expr(arm_scope, expr),
                        phalcom_ast::ast::MatchArmBody::Block(block) => self.visit_statements(arm_scope, &block.body, false),
                    }
                }
            }
            Expr::RecordConstruction(record) => {
                for entry in &record.entries {
                    self.visit_expr(scope, &entry.value);
                }
            }
            Expr::Int { .. }
            | Expr::Float { .. }
            | Expr::String { .. }
            | Expr::Boolean { .. }
            | Expr::Var { .. }
            | Expr::Field { .. }
            | Expr::SelfVar { .. }
            | Expr::SuperVar { .. }
            | Expr::Symbol { .. }
            | Expr::Ellipsis { .. }
            | Expr::TypeForm(_) => {}
        }
    }

    fn visit_block(&mut self, parent: SourceScopeId, block: &BlockExpr) {
        let scope = self.new_scope(parent, block.range);
        self.visit_block_contents(scope, block);
    }

    fn visit_block_contents(&mut self, scope: SourceScopeId, block: &BlockExpr) {
        for parameter in &block.params.fixed {
            self.declare(scope, parameter.name.clone(), SourceBindingKind::ClosureParameter, parameter.range, true);
        }
        if let Some(parameter) = &block.params.positional_rest {
            self.declare(scope, parameter.name.clone(), SourceBindingKind::ClosureParameter, parameter.range, true);
        }
        self.visit_statements(scope, &block.body, false);
    }

    fn visit_pack_items(&mut self, scope: SourceScopeId, items: &[phalcom_ast::ast::PackItem]) {
        for item in items {
            match item {
                phalcom_ast::ast::PackItem::Positional { expr, .. }
                | phalcom_ast::ast::PackItem::Expand { expr, .. }
                | phalcom_ast::ast::PackItem::Labeled { value: expr, .. } => self.visit_expr(scope, expr),
            }
        }
    }

    fn visit_product_label(&mut self, scope: SourceScopeId, label: &phalcom_ast::ast::ProductLabel) {
        if let phalcom_ast::ast::ProductLabel::Computed { expr, .. } = label {
            self.visit_expr(scope, expr);
        }
    }
}

fn parameter_slot(parameter: &phalcom_ast::ast::ParameterDef) -> SelectorSlot {
    parameter
        .label
        .as_ref()
        .map_or(SelectorSlot::Positional, |label| SelectorSlot::Label(label.clone()))
}

fn behavior_side(member: &phalcom_ast::ast::BehaviorMember) -> DispatchSide {
    if member.attributes().iter().any(|attribute| attribute.name == "class") {
        return DispatchSide::Class;
    }
    match member {
        phalcom_ast::ast::BehaviorMember::Method(method) => {
            if method.is_static {
                DispatchSide::Class
            } else {
                DispatchSide::Instance
            }
        }
        phalcom_ast::ast::BehaviorMember::Getter(getter) => {
            if getter.is_static {
                DispatchSide::Class
            } else {
                DispatchSide::Instance
            }
        }
        phalcom_ast::ast::BehaviorMember::Setter(setter) => {
            if setter.is_static {
                DispatchSide::Class
            } else {
                DispatchSide::Instance
            }
        }
        phalcom_ast::ast::BehaviorMember::Index(_) => DispatchSide::Instance,
    }
}

fn statements_range(statements: &[Statement]) -> SourceRange {
    statements
        .iter()
        .map(statement_range)
        .reduce(|left, right| left.merge(&right))
        .unwrap_or_default()
}

fn statement_range(statement: &Statement) -> SourceRange {
    match statement {
        Statement::Class(class) => class.range,
        Statement::Data(data_def) => data_def.range,
        Statement::Trait(trait_def) => trait_def.range,
        Statement::Let(binding) => binding.range,
        Statement::Return(return_statement) => return_statement.range,
        Statement::Expr { range, .. } => *range,
        Statement::For(for_statement) => for_statement.range,
        Statement::Break { range } | Statement::Continue { range } => *range,
        Statement::Throw { range, .. } => *range,
        Statement::Export(export_decl) => export_decl.range,
        Statement::TypeAlias(type_alias) => type_alias.range,
        Statement::Impl(impl_def) => impl_def.range,
    }
}
