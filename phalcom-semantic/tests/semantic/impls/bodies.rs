use phalcom_common::selector::Selector;
use phalcom_modules::identity::{ModuleId, ModulePath, ResolvedProjectId};
use phalcom_modules::interface::LinkedModuleInterface;
use phalcom_modules::linker::{LinkedModule, LinkedProgram, ModuleBindingLayout};
use phalcom_modules::metadata::ModuleMetadata;
use phalcom_modules::source::ModuleKind;
use phalcom_semantic::db::QueryKey;
use phalcom_semantic::identity::{CallableId, DeclarationId, DispatchSide};
use phalcom_semantic::session::SemanticWorkspaceSession;
use phalcom_semantic::source::ParsedModuleUnit;
use phalcom_semantic::types::id::{RecordRowId, StaticDomainId, TypeId, TypeLambdaId, TypeParameterId};
use phalcom_semantic::types::parameter::{GenericConstraint, TypeParameterOwner, TypeTerm};
use phalcom_semantic::types::store::{StaticArgument, StaticDomainData, TypeData, TypeStore};
use phalcom_semantic::workspace::SemanticWorkspaceInput;
use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;

pub(super) fn test_module() -> ModuleId {
    ModuleId::resolved(ResolvedProjectId::from_raw(42), ModulePath::root())
}

pub(super) fn single_module_input(module: ModuleId, source: &str) -> SemanticWorkspaceInput {
    let parsed = phalcom_ast::parse(source, 0);
    assert!(parsed.errors.is_empty(), "parse errors: {:?}", parsed.errors);

    let linked_module = LinkedModule {
        interface: LinkedModuleInterface {
            module: module.clone(),
            kind: ModuleKind::Module,
            exports: BTreeMap::new(),
            metadata: ModuleMetadata::default(),
        },
        bindings: ModuleBindingLayout::default(),
        linked_reads: Vec::new(),
        runtime_dependencies: Vec::new(),
    };
    let mut modules = BTreeMap::new();
    modules.insert(module.clone(), linked_module);
    let linked = Arc::new(LinkedProgram {
        universe: Arc::new(phalcom_modules::project::ProjectUniverse::new()),
        modules,
        graphs: phalcom_modules::graph::ModuleGraphs::default(),
        entry: module.clone(),
        initialization_order: vec![module.clone()],
    });

    let mut sources = BTreeMap::new();
    sources.insert(
        module.clone(),
        Arc::new(ParsedModuleUnit::new(
            module,
            ModuleKind::Module,
            None,
            Arc::from(source),
            Arc::new(parsed.program),
        )),
    );
    SemanticWorkspaceInput::new(linked, sources, 1)
}

struct ImplParameterGraph<'a> {
    store: &'a TypeStore,
    forbidden: &'a HashSet<TypeParameterId>,
    visited_types: HashSet<TypeId>,
    visited_rows: HashSet<RecordRowId>,
    visited_domains: HashSet<StaticDomainId>,
    visited_lambdas: HashSet<TypeLambdaId>,
}

impl ImplParameterGraph<'_> {
    fn contains_type(&mut self, ty: TypeId) -> bool {
        if !self.visited_types.insert(ty) {
            return false;
        }
        match self.store.get(ty) {
            TypeData::Parameter(parameter) => self.forbidden.contains(parameter),
            TypeData::Applied { origin, arguments } => self.contains_type(*origin) || arguments.iter().any(|argument| self.contains_argument(*argument)),
            TypeData::ExactCase { enum_type, .. } => self.contains_type(*enum_type),
            TypeData::Union(members) | TypeData::TypePack(members) | TypeData::TypePackConcat(members) | TypeData::LabeledTypePackConcat(members) => {
                members.iter().any(|member| self.contains_type(*member))
            }
            TypeData::Tuple(tuple) => {
                tuple.elements.iter().any(|element| self.contains_type(element.ty))
                    || self.contains_tuple_remainder(tuple.positional_remainder)
                    || self.contains_tuple_remainder(tuple.labeled_remainder)
            }
            TypeData::Record(row) => self.contains_row(*row),
            TypeData::LabeledTypePack(elements) => elements.iter().any(|element| self.contains_type(element.element)),
            TypeData::CompleteTypePack { positionals, labeled }
            | TypeData::TupleFromPacks { positionals, labeled }
            | TypeData::PackFromPacks { positionals, labeled } => self.contains_type(*positionals) || self.contains_type(*labeled),
            TypeData::Pack(pack) => {
                pack.elements.iter().any(|element| self.contains_type(element.ty))
                    || self.contains_tuple_remainder(pack.positional_remainder)
                    || self.contains_tuple_remainder(pack.labeled_remainder)
            }
            TypeData::CallableFromPacks {
                positionals,
                labeled,
                return_type,
            } => self.contains_type(*positionals) || self.contains_type(*labeled) || self.contains_type(*return_type),
            TypeData::Callable(callable) => {
                callable.parameters.iter().any(|parameter| self.contains_type(parameter.ty)) || self.contains_type(callable.return_type)
            }
            TypeData::Family(family) => self.store.get_family(*family).members.iter().any(|member| self.contains_type(member.ty)),
            TypeData::Lambda(lambda) => self.contains_lambda(*lambda),
            TypeData::AssociatedProjection(projection) => {
                self.contains_type(projection.subject) || projection.trait_ref.arguments.iter().any(|argument| self.contains_type(*argument))
            }
            TypeData::TypeMemberProjection(projection) => {
                let subject_contains = match projection.subject {
                    phalcom_semantic::types::type_member::ProjectionSubject::Type(ty) => self.contains_type(ty),
                    phalcom_semantic::types::type_member::ProjectionSubject::RecordRow(row) => self.contains_row(row),
                };
                subject_contains
                    || projection.arguments.iter().any(|argument| match argument {
                        phalcom_semantic::types::type_member::TypeMemberArgument::Type(ty) => self.contains_type(*ty),
                        phalcom_semantic::types::type_member::TypeMemberArgument::Const(_) => false,
                        phalcom_semantic::types::type_member::TypeMemberArgument::RecordRow(row) => self.contains_row(*row),
                    })
            }
            TypeData::StaticDomainForm { domain } => self.contains_domain(*domain),
            TypeData::StaticData(term) => self.contains_domain(term.domain) || term.arguments.iter().any(|argument| self.contains_argument(*argument)),
            TypeData::Never | TypeData::Unit | TypeData::ClassObject { .. } | TypeData::Nominal { .. } | TypeData::SelfType(_) | TypeData::ConstInt(_) => false,
        }
    }

    fn contains_tuple_remainder(&mut self, remainder: phalcom_semantic::types::store::AnonymousTupleRemainder) -> bool {
        match remainder {
            phalcom_semantic::types::store::AnonymousTupleRemainder::Typed(ty) => self.contains_type(ty),
            phalcom_semantic::types::store::AnonymousTupleRemainder::None | phalcom_semantic::types::store::AnonymousTupleRemainder::Unconstrained => false,
        }
    }

    fn contains_argument(&mut self, argument: StaticArgument) -> bool {
        match argument {
            StaticArgument::Term(ty) => self.contains_type(ty),
            StaticArgument::RecordRow(row) => self.contains_row(row),
            StaticArgument::DynamicType => false,
        }
    }

    fn contains_row(&mut self, row_id: RecordRowId) -> bool {
        if !self.visited_rows.insert(row_id) {
            return false;
        }
        let row = self.store.record_row(row_id);
        row.fields.iter().any(|field| self.contains_type(field.ty))
            || match row.tail {
                phalcom_semantic::types::row::RecordRowTail::Closed => false,
                phalcom_semantic::types::row::RecordRowTail::Parameter(parameter) => self.forbidden.contains(&parameter),
                phalcom_semantic::types::row::RecordRowTail::Projection(ty) => self.contains_type(ty),
                phalcom_semantic::types::row::RecordRowTail::Anonymous(remainder) => match remainder {
                    phalcom_semantic::types::row::AnonymousRecordRemainder::Unconstrained { key_type } => self.contains_type(key_type),
                    phalcom_semantic::types::row::AnonymousRecordRemainder::Typed { key_type, value_type } => {
                        self.contains_type(key_type) || self.contains_type(value_type)
                    }
                },
            }
    }

    fn contains_domain(&mut self, domain: StaticDomainId) -> bool {
        if !self.visited_domains.insert(domain) {
            return false;
        }
        match self.store.static_domain(domain) {
            Some(StaticDomainData::Associated { owner_arguments, .. }) => owner_arguments.iter().any(|argument| self.contains_argument(*argument)),
            Some(StaticDomainData::ConstInt | StaticDomainData::Nominal { .. }) | None => false,
        }
    }

    fn contains_lambda(&mut self, lambda: TypeLambdaId) -> bool {
        if !self.visited_lambdas.insert(lambda) {
            return false;
        }
        let arena = self.store.arena();
        let mut free_types = Vec::new();
        arena.collect_free_types(arena.get_lambda(lambda).body, &mut free_types);
        free_types.into_iter().any(|ty| self.contains_type(ty))
    }

    fn contains_term(&mut self, term: &TypeTerm) -> bool {
        matches!(term, TypeTerm::Canonical(ty) if self.contains_type(*ty))
    }

    fn contains_constraint(&mut self, constraint: &GenericConstraint) -> bool {
        match constraint {
            GenericConstraint::Subtype { lower, upper } => self.contains_term(lower) || self.contains_term(upper),
            GenericConstraint::Equivalent { left, right } => self.contains_term(left) || self.contains_term(right),
            GenericConstraint::Conforms { subject, trait_ref } => {
                self.contains_term(subject) || trait_ref.arguments.iter().any(|argument| self.contains_type(*argument))
            }
        }
    }

    fn contains_signature(&mut self, signature: &phalcom_semantic::signature::CallableSemanticSignature) -> bool {
        signature
            .parameters
            .iter()
            .filter_map(|parameter| parameter.declared_type.canonical_type())
            .any(|ty| self.contains_type(ty))
            || signature.declared_return.canonical_type().is_some_and(|ty| self.contains_type(ty))
            || signature
                .inferred_return
                .as_ref()
                .and_then(|knowledge| knowledge.ty())
                .is_some_and(|ty| self.contains_type(ty))
            || signature.generics.as_ref().is_some_and(|generics| {
                generics.parameters.iter().any(|parameter| self.forbidden.contains(parameter))
                    || generics.constraints.iter().any(|constraint| self.contains_constraint(constraint))
            })
    }
}

fn signature_contains_impl_parameter(
    store: &TypeStore,
    signature: &phalcom_semantic::signature::CallableSemanticSignature,
    impl_parameters: &HashSet<TypeParameterId>,
) -> bool {
    ImplParameterGraph {
        store,
        forbidden: impl_parameters,
        visited_types: HashSet::new(),
        visited_rows: HashSet::new(),
        visited_domains: HashSet::new(),
        visited_lambdas: HashSet::new(),
    }
    .contains_signature(signature)
}

#[test]
fn test_instance_self_and_private_field_access_in_impl_body() {
    let module = test_module();
    let src = r#"
class User {
  _name: String

  getName() -> String {
    self._name
  }
}

impl User {
  getNameImpl() -> String {
    self._name
  }
}
"#;
    let mut session = SemanticWorkspaceSession::new();
    let update = session.update(single_module_input(module.clone(), src));
    assert!(!update.snapshot.has_errors(), "diagnostics: {:?}", update.snapshot.diagnostics);

    let callable = CallableId::new(
        DeclarationId::new(module, "User".into()),
        Selector::method("getNameImpl", []).unwrap(),
        DispatchSide::Instance,
    );
    let analysis = update.snapshot.callable_analyses.get(&callable).expect("body analysis should be present");
    assert!(analysis.diagnostics.is_empty());
}

#[test]
fn test_class_side_impl_body_and_self() {
    let module = test_module();
    let src = r#"
class Counter {
  @class
  getCountDirect() -> Int {
    10
  }
}
impl Counter {
  @class
  getCount() -> Int {
    Self.getCountDirect()
  }
}
"#;
    let mut session = SemanticWorkspaceSession::new();
    let update = session.update(single_module_input(module.clone(), src));
    assert!(!update.snapshot.has_errors(), "diagnostics: {:?}", update.snapshot.diagnostics);

    let callable = CallableId::new(
        DeclarationId::new(module, "Counter".into()),
        Selector::method("getCount", []).unwrap(),
        DispatchSide::Class,
    );
    let analysis = update.snapshot.callable_analyses.get(&callable).expect("body analysis should be present");
    assert!(analysis.diagnostics.is_empty());
}

#[test]
fn test_class_side_type_self_and_lowercase_receiver_remain_distinct() {
    let module = test_module();
    let source = r#"
class Counter {
  @class
  direct() -> Int { 10 }
}
impl Counter {
  @class
  throughReceiver() -> Int { self.direct() }
  @class
  throughType() -> Int { Self.direct() }
}
"#;
    let mut session = SemanticWorkspaceSession::new();
    let update = session.update(single_module_input(module.clone(), source));
    assert!(!update.snapshot.has_errors(), "diagnostics: {:?}", update.snapshot.diagnostics);

    let direct = CallableId::new(
        DeclarationId::new(module.clone(), "Counter".into()),
        Selector::method("direct", []).unwrap(),
        DispatchSide::Class,
    );
    let through_receiver = CallableId::new(
        DeclarationId::new(module, "Counter".into()),
        Selector::method("throughReceiver", []).unwrap(),
        DispatchSide::Class,
    );
    let analysis = update.snapshot.callable_analyses.get(&through_receiver).expect("throughReceiver body");
    let receiver = analysis
        .expressions
        .values()
        .find(|expression| source.get(expression.range.start..expression.range.end) == Some("self"))
        .expect("lowercase self expression");
    assert_eq!(receiver.callable, None, "the receiver itself is not a call");
    assert!(
        matches!(
            receiver.knowledge.ty().map(|ty| update.snapshot.store.get(ty)),
            Some(TypeData::ClassObject { declaration }) if declaration.name.as_ref() == "Counter"
        ),
        "lowercase self must be a class object: {receiver:#?}"
    );

    let send = analysis
        .expressions
        .values()
        .find(|expression| expression.callable.as_ref() == Some(&direct))
        .expect("self.direct() must select the class-side direct callable");
    assert_eq!(send.callable.as_ref(), Some(&direct));
}

#[test]
fn test_generic_covering_permutation_body_analysis() {
    let module = test_module();
    let src = r#"
class Pair<X, Y> {
  _first: X
  _second: Y

  first() -> X { self._first }
  second() -> Y { self._second }
}

impl<A, B> Pair<B, A> {
  getFirst() -> B {
    self.first()
  }
  getSecond() -> A {
    self.second()
  }
}
"#;
    let mut session = SemanticWorkspaceSession::new();
    let update = session.update(single_module_input(module.clone(), src));
    assert!(!update.snapshot.has_errors(), "diagnostics: {:?}", update.snapshot.diagnostics);

    let pair_decl = DeclarationId::new(module, "Pair".into());
    let get_first = CallableId::new(pair_decl.clone(), Selector::method("getFirst", []).unwrap(), DispatchSide::Instance);
    let get_second = CallableId::new(pair_decl, Selector::method("getSecond", []).unwrap(), DispatchSide::Instance);

    let analysis1 = update.snapshot.callable_analyses.get(&get_first).expect("getFirst body analysis");
    assert!(analysis1.diagnostics.is_empty());
    let analysis2 = update.snapshot.callable_analyses.get(&get_second).expect("getSecond body analysis");
    assert!(analysis2.diagnostics.is_empty());
}

#[test]
fn test_covering_class_impl_body_uses_impl_generic_coordinates() {
    let module = test_module();
    let src = r#"
data Token()

data Id<Tag>(
  _ token: Token,
)

impl<Tag> Id<Tag> {
  @class
  fresh() -> Id<Tag> {
    Id<Tag>(Token())
  }
}
"#;
    let mut session = SemanticWorkspaceSession::new();
    let update = session.update(single_module_input(module.clone(), src));
    assert!(!update.snapshot.has_errors(), "diagnostics: {:?}", update.snapshot.diagnostics);

    let callable = CallableId::new(
        DeclarationId::new(module, "Id".into()),
        Selector::method("fresh", []).unwrap(),
        DispatchSide::Class,
    );
    let analysis = update.snapshot.callable_analyses.get(&callable).expect("fresh body analysis");
    assert!(analysis.diagnostics.is_empty());
}

#[test]
fn test_generic_covering_class_side_receiver_remains_class_object() {
    let module = test_module();
    let source = r#"
data Id<Tag>()

impl<Tag> Id<Tag> {
  @class
  direct() -> Int { 10 }

  @class
  throughReceiver() -> Int { self.direct() }

  @class
  make() -> Self { Id<Tag>() }
}
"#;
    let mut session = SemanticWorkspaceSession::new();
    let update = session.update(single_module_input(module.clone(), source));
    assert!(!update.snapshot.has_errors(), "diagnostics: {:?}", update.snapshot.diagnostics);

    let owner = DeclarationId::new(module.clone(), "Id".into());
    let direct = CallableId::new(owner.clone(), Selector::method("direct", []).unwrap(), DispatchSide::Class);
    let through_receiver = CallableId::new(owner, Selector::method("throughReceiver", []).unwrap(), DispatchSide::Class);
    let analysis = update.snapshot.callable_analyses.get(&through_receiver).expect("throughReceiver body");
    let receiver = analysis
        .expressions
        .values()
        .find(|expression| source.get(expression.range.start..expression.range.end) == Some("self"))
        .expect("generic class-side self expression");
    assert!(matches!(
        receiver.knowledge.ty().map(|ty| update.snapshot.store.get(ty)),
        Some(TypeData::ClassObject { declaration }) if declaration.name.as_ref() == "Id"
    ));
    assert!(analysis.expressions.values().any(|expression| expression.callable.as_ref() == Some(&direct)));

    let make = CallableId::new(
        DeclarationId::new(module, "Id".into()),
        Selector::method("make", []).unwrap(),
        DispatchSide::Class,
    );
    let definition = &update.snapshot.callable_definitions[&make];
    let context = definition.inherent_body_context.as_ref().expect("covering context");
    let mut store = update.snapshot.store.as_ref().clone();
    let body_signature = context
        .signature_for_body(&mut store, &update.snapshot.declarations, definition.signature.clone())
        .expect("covering signature materializes");
    assert_eq!(body_signature.declared_return.canonical_type(), Some(context.source_target_head));
}

#[test]
fn test_conditional_class_side_receiver_remains_class_object() {
    let module = test_module();
    let source = r#"
class Box<T> {
  @class
  direct() -> Int { 10 }
}

impl<T> Box<T> where T <: Int {
  @class
  throughReceiver() -> Int { self.direct() }
}
"#;
    let mut session = SemanticWorkspaceSession::new();
    let update = session.update(single_module_input(module.clone(), source));
    assert!(!update.snapshot.has_errors(), "diagnostics: {:?}", update.snapshot.diagnostics);

    let owner = DeclarationId::new(module, "Box".into());
    let direct = CallableId::new(owner.clone(), Selector::method("direct", []).unwrap(), DispatchSide::Class);
    let through_receiver = CallableId::new(owner, Selector::method("throughReceiver", []).unwrap(), DispatchSide::Class);
    let analysis = update
        .snapshot
        .callable_analyses
        .get(&through_receiver)
        .expect("conditional throughReceiver body");
    let receiver = analysis
        .expressions
        .values()
        .find(|expression| source.get(expression.range.start..expression.range.end) == Some("self"))
        .expect("conditional class-side self expression");
    assert!(matches!(
        receiver.knowledge.ty().map(|ty| update.snapshot.store.get(ty)),
        Some(TypeData::ClassObject { declaration }) if declaration.name.as_ref() == "Box"
    ));
    assert!(analysis.expressions.values().any(|expression| expression.callable.as_ref() == Some(&direct)));
}

#[test]
fn test_covering_class_impl_body_specializes_bare_self() {
    let module = test_module();
    let src = r#"
data Token()

data Id<Tag>(
  _ token: Token,
)

impl<Tag> Id<Tag> {
  @class
  fresh() -> Self {
    Id<Tag>(Token())
  }
}
"#;
    let mut session = SemanticWorkspaceSession::new();
    let update = session.update(single_module_input(module.clone(), src));
    assert!(!update.snapshot.has_errors(), "diagnostics: {:?}", update.snapshot.diagnostics);

    let callable = CallableId::new(
        DeclarationId::new(module, "Id".into()),
        Selector::method("fresh", []).unwrap(),
        DispatchSide::Class,
    );
    let analysis = update.snapshot.callable_analyses.get(&callable).expect("fresh body analysis");
    assert!(analysis.diagnostics.is_empty());
}

#[test]
fn test_covering_permutation_body_specializes_parameters_and_return() {
    let module = test_module();
    let src = r#"
data Pair<X, Y>(
  _ first: X,
  _ second: Y,
)

impl<A, B> Pair<B, A> {
  @class
  duplicate(_ b: B, _ a: A) -> Pair<B, A> {
    Pair<B, A>(b, a)
  }
}
"#;
    let mut session = SemanticWorkspaceSession::new();
    let update = session.update(single_module_input(module.clone(), src));
    assert!(!update.snapshot.has_errors(), "diagnostics: {:?}", update.snapshot.diagnostics);

    let callable = CallableId::new(
        DeclarationId::new(module, "Pair".into()),
        Selector::method(
            "duplicate",
            [
                phalcom_common::selector::SelectorSlot::Positional,
                phalcom_common::selector::SelectorSlot::Positional,
            ],
        )
        .unwrap(),
        DispatchSide::Class,
    );
    let analysis = update.snapshot.callable_analyses.get(&callable).expect("duplicate body analysis");
    assert!(analysis.diagnostics.is_empty());
}

#[test]
fn test_covering_body_coordinates_all_three_parameter_permutations() {
    let permutations = [[0, 1, 2], [0, 2, 1], [1, 0, 2], [1, 2, 0], [2, 0, 1], [2, 1, 0]];
    let names = ["A", "B", "C"];

    for permutation in permutations {
        let target = permutation.map(|index| names[index]).join(", ");
        let source = format!(
            r#"
data Triple<X, Y, Z>()
impl<A, B, C> Triple<{target}> {{
  @class
  choose(_ a: A, _ b: B, _ c: C) -> A {{ a }}
}}
"#
        );
        let module = test_module();
        let mut session = SemanticWorkspaceSession::new();
        let update = session.update(single_module_input(module.clone(), &source));
        assert!(
            !update.snapshot.has_errors(),
            "permutation {permutation:?} diagnostics: {:?}",
            update.snapshot.diagnostics
        );

        let owner = DeclarationId::new(module, "Triple".into());
        let callable = CallableId::new(
            owner.clone(),
            Selector::method(
                "choose",
                [
                    phalcom_common::selector::SelectorSlot::Positional,
                    phalcom_common::selector::SelectorSlot::Positional,
                    phalcom_common::selector::SelectorSlot::Positional,
                ],
            )
            .unwrap(),
            DispatchSide::Class,
        );
        let definition = update.snapshot.callable_definitions.get(&callable).expect("accepted covering definition");
        let context = definition.inherent_body_context.as_ref().expect("covering body context");
        let implementation_parameters = context.generic_signature.as_ref().expect("impl binders").parameters.as_ref();
        let declaration_parameters = update
            .snapshot
            .declarations
            .generic_signature(&owner)
            .expect("declaration binders")
            .parameters
            .as_ref();
        let covering = context.covering.as_ref().expect("permutation must use a covering substitution");
        let mut declaration_slot_for_impl = [usize::MAX; 3];
        for (declaration_slot, &impl_slot) in permutation.iter().enumerate() {
            declaration_slot_for_impl[impl_slot] = declaration_slot;
        }
        assert!(declaration_slot_for_impl.iter().all(|slot| *slot != usize::MAX));
        assert_eq!(covering.impl_to_decl.len(), 3, "forward map for {permutation:?}");
        assert_eq!(covering.decl_to_impl.len(), 3, "inverse map for {permutation:?}");
        for impl_slot in 0..3 {
            let implementation_parameter = implementation_parameters[impl_slot];
            let declaration_parameter = declaration_parameters[declaration_slot_for_impl[impl_slot]];
            assert_eq!(covering.impl_to_decl.get(&implementation_parameter), Some(&declaration_parameter));
            assert_eq!(covering.decl_to_impl.get(&declaration_parameter), Some(&implementation_parameter));
        }

        let public_signature = update.snapshot.callable_signatures.get(&callable).expect("published canonical signature");
        assert_eq!(public_signature.callable, definition.signature.callable);
        assert_eq!(public_signature.generics, definition.signature.generics);
        assert_eq!(
            public_signature
                .parameters
                .iter()
                .map(|parameter| parameter.declared_type.canonical_type())
                .collect::<Vec<_>>(),
            definition
                .signature
                .parameters
                .iter()
                .map(|parameter| parameter.declared_type.canonical_type())
                .collect::<Vec<_>>(),
            "effective definition and published product must agree on declaration-canonical parameter types"
        );
        assert_eq!(
            public_signature.declared_return.canonical_type(),
            definition.signature.declared_return.canonical_type()
        );
        let implementation_parameter_set = implementation_parameters.iter().copied().collect::<HashSet<_>>();
        assert!(
            !signature_contains_impl_parameter(&update.snapshot.store, public_signature, &implementation_parameter_set),
            "published signature leaked an impl parameter for {permutation:?}: {public_signature:#?}"
        );

        let mut store = update.snapshot.store.as_ref().clone();
        let body_signature = context
            .signature_for_body(&mut store, &update.snapshot.declarations, public_signature.clone())
            .expect("source-relative signature materializes");
        for (slot, implementation_parameter) in implementation_parameters.iter().enumerate() {
            let public_type = public_signature.parameters[slot].declared_type.canonical_type().expect("public parameter type");
            assert!(matches!(store.get(public_type), TypeData::Parameter(parameter) if *parameter == declaration_parameters[declaration_slot_for_impl[slot]]));
            let body_type = body_signature.parameters[slot].declared_type.canonical_type().expect("body parameter type");
            assert!(matches!(store.get(body_type), TypeData::Parameter(parameter) if parameter == implementation_parameter));
        }
        let public_return = public_signature.declared_return.canonical_type().expect("public return type");
        assert!(matches!(store.get(public_return), TypeData::Parameter(parameter) if *parameter == declaration_parameters[declaration_slot_for_impl[0]]));
        let body_return = body_signature.declared_return.canonical_type().expect("body return type");
        assert!(matches!(store.get(body_return), TypeData::Parameter(parameter) if *parameter == implementation_parameters[0]));
    }
}

#[test]
fn test_covering_body_nested_shapes_publish_no_impl_parameters() {
    let module = test_module();
    let source = r#"
trait HasItem { type Item }
data Box<T>()
data Envelope<**Rows: RecordFields> { **Rows }
data Triple<X, Y, Z, **Rows: RecordFields> { **Rows }

impl<A, B, C, **Fields: RecordFields> Triple<B, C, A, **Fields> {
  @class
  inspect<U>(
    _ applied: Box<A>,
    _ tuple: (A, B),
    _ pack: Pack<*: A>,
    _ callback: (A, B) -> C,
    _ row: { item: A, **Fields },
    _ applied_row: Envelope<**{ value: A, **Fields }>,
    _ projection: <A as HasItem>::Item,
    _ result: A
  ) -> A where U <: Box<A>, A |= HasItem { result }
}
"#;
    let mut session = SemanticWorkspaceSession::new();
    let update = session.update(single_module_input(module.clone(), source));
    assert!(!update.snapshot.has_errors(), "diagnostics: {:?}", update.snapshot.diagnostics);

    let owner = DeclarationId::new(module, "Triple".into());
    let callable = CallableId::new(
        owner.clone(),
        Selector::method("inspect", vec![phalcom_common::selector::SelectorSlot::Positional; 8]).unwrap(),
        DispatchSide::Class,
    );
    let definition = update.snapshot.callable_definitions.get(&callable).expect("nested-shape covering definition");
    let context = definition.inherent_body_context.as_ref().expect("covering body context");
    let implementation_parameters = context
        .generic_signature
        .as_ref()
        .expect("impl generic signature")
        .parameters
        .iter()
        .copied()
        .collect::<HashSet<_>>();
    let public_signature = update.snapshot.callable_signatures.get(&callable).expect("published signature");
    assert!(
        !signature_contains_impl_parameter(&update.snapshot.store, public_signature, &implementation_parameters),
        "nested signature, constraints, or rows leaked an impl parameter: {public_signature:#?}"
    );

    let declaration_parameters = update
        .snapshot
        .declarations
        .generic_signature(&owner)
        .expect("declaration signature")
        .parameters
        .clone();
    let [x, y, z, rows] = declaration_parameters.as_ref() else {
        panic!("Triple should publish three type parameters and one RecordFields parameter: {declaration_parameters:?}");
    };
    let public_generics = public_signature.generics.as_ref().expect("generic callable constraints");
    let declaration_parameter_set = HashSet::from([*z]);
    let mut declaration_parameter_graph = ImplParameterGraph {
        store: &update.snapshot.store,
        forbidden: &declaration_parameter_set,
        visited_types: HashSet::new(),
        visited_rows: HashSet::new(),
        visited_domains: HashSet::new(),
        visited_lambdas: HashSet::new(),
    };
    assert!(
        public_generics
            .constraints
            .iter()
            .any(|constraint| declaration_parameter_graph.contains_constraint(constraint)),
        "the public callable constraints should retain the declaration-relative applied argument"
    );
    let mut store = update.snapshot.store.as_ref().clone();
    let body_signature = context
        .signature_for_body(&mut store, &update.snapshot.declarations, public_signature.clone())
        .expect("nested source-relative signature materializes");
    let body_generics = body_signature.generics.as_ref().expect("body-relative generic constraints");
    let implementation_parameter_set = HashSet::from([context.generic_signature.as_ref().expect("impl generic signature").parameters[0]]);
    let mut implementation_parameter_graph = ImplParameterGraph {
        store: &store,
        forbidden: &implementation_parameter_set,
        visited_types: HashSet::new(),
        visited_rows: HashSet::new(),
        visited_domains: HashSet::new(),
        visited_lambdas: HashSet::new(),
    };
    assert!(
        body_generics
            .constraints
            .iter()
            .any(|constraint| implementation_parameter_graph.contains_constraint(constraint)),
        "the body-relative callable constraints should restore the impl parameter coordinate"
    );
    let expected_public_parameters = vec![
        vec![*z],
        vec![*z, *x],
        vec![*z],
        vec![*z, *x, *y],
        vec![*z, *rows],
        vec![*z, *rows],
        vec![*z],
        vec![*z],
    ];
    for (index, expected) in expected_public_parameters.iter().enumerate() {
        let ty = public_signature.parameters[index].declared_type.canonical_type().expect("annotated parameter");
        for parameter in expected {
            let expected_parameter_set = HashSet::from([*parameter]);
            let mut graph = ImplParameterGraph {
                store: &store,
                forbidden: &expected_parameter_set,
                visited_types: HashSet::new(),
                visited_rows: HashSet::new(),
                visited_domains: HashSet::new(),
                visited_lambdas: HashSet::new(),
            };
            assert!(graph.contains_type(ty), "parameter {index} should retain declaration parameter {parameter:?}");
        }
    }
    assert_eq!(body_signature.parameters.len(), 8);
    assert_eq!(
        body_signature.declared_return.canonical_type(),
        body_signature.parameters[7].declared_type.canonical_type()
    );
    let implementation_parameters = context.generic_signature.as_ref().expect("impl generic signature").parameters.as_ref();
    let expected_body_parameters = vec![
        vec![implementation_parameters[0]],
        vec![implementation_parameters[0], implementation_parameters[1]],
        vec![implementation_parameters[0]],
        vec![implementation_parameters[0], implementation_parameters[1], implementation_parameters[2]],
        vec![implementation_parameters[0], implementation_parameters[3]],
        vec![implementation_parameters[0], implementation_parameters[3]],
        vec![implementation_parameters[0]],
        vec![implementation_parameters[0]],
    ];
    for (index, expected) in expected_body_parameters.iter().enumerate() {
        let ty = body_signature.parameters[index].declared_type.canonical_type().expect("body parameter type");
        for parameter in expected {
            let expected_parameter_set = HashSet::from([*parameter]);
            let mut graph = ImplParameterGraph {
                store: &store,
                forbidden: &expected_parameter_set,
                visited_types: HashSet::new(),
                visited_rows: HashSet::new(),
                visited_domains: HashSet::new(),
                visited_lambdas: HashSet::new(),
            };
            assert!(graph.contains_type(ty), "body parameter {index} should use impl coordinate {parameter:?}");
        }
    }
}

#[test]
fn test_covering_body_specializes_callable_generic_self_constraint() {
    let module = test_module();
    let src = r#"
data Pair<X, Y>(
  _ first: X,
  _ second: Y,
)

impl<A, B> Pair<B, A> {
  @class
  retain<T>(_ value: T) -> T where T <: Self { value }
}
"#;
    let mut session = SemanticWorkspaceSession::new();
    let update = session.update(single_module_input(module.clone(), src));
    assert!(!update.snapshot.has_errors(), "diagnostics: {:?}", update.snapshot.diagnostics);

    let callable = CallableId::new(
        DeclarationId::new(module, "Pair".into()),
        Selector::method("retain", [phalcom_common::selector::SelectorSlot::Positional]).unwrap(),
        DispatchSide::Class,
    );
    let definition = update.snapshot.callable_definitions.get(&callable).expect("accepted inherent definition");
    let context = definition.inherent_body_context.as_ref().expect("covering impl body context");
    let mut store = update.snapshot.store.as_ref().clone();
    let body_signature = context
        .signature_for_body(&mut store, &update.snapshot.declarations, definition.signature.clone())
        .expect("accepted covering signature materializes for its source body");
    let generics = body_signature.generics.as_ref().expect("retain generic signature");
    let parameter = generics.parameters[0];
    assert!(matches!(
        &generics.constraints[0],
        GenericConstraint::Subtype {
            lower: TypeTerm::Canonical(lower),
            upper: TypeTerm::Canonical(upper),
        } if matches!(store.get(*lower), TypeData::Parameter(actual) if *actual == parameter)
            && *upper == context.source_target_head
    ));
    assert!(matches!(
        store.type_parameter(parameter).owner,
        TypeParameterOwner::Callable(ref owner) if owner == &callable
    ));
    assert!(
        update
            .snapshot
            .callable_analyses
            .get(&callable)
            .expect("retain body analysis")
            .diagnostics
            .is_empty()
    );
}

#[test]
fn test_covering_body_consumes_callable_generic_self_equivalence() {
    let module = test_module();
    let source = r#"
data Box<T>(_ value: T)
impl<T> Box<T> {
  identity<U>(_ value: U) -> Self where U == Self { value }
}
"#;
    let mut session = SemanticWorkspaceSession::new();
    let update = session.update(single_module_input(module, source));
    assert!(!update.snapshot.has_errors(), "diagnostics: {:?}", update.snapshot.diagnostics);
}

#[test]
fn test_covering_body_preserves_callable_generic_shadowing() {
    let module = test_module();
    let source = r#"
data Box<T>(_ value: T)
impl<T> Box<T> {
  identity<T>(_ value: T) -> T { value }
}
"#;
    let mut session = SemanticWorkspaceSession::new();
    let update = session.update(single_module_input(module.clone(), source));
    assert!(!update.snapshot.has_errors(), "diagnostics: {:?}", update.snapshot.diagnostics);
    let callable = CallableId::new(
        DeclarationId::new(module, "Box".into()),
        Selector::method("identity", [phalcom_common::selector::SelectorSlot::Positional]).unwrap(),
        DispatchSide::Instance,
    );
    let definition = &update.snapshot.callable_definitions[&callable];
    let context = definition.inherent_body_context.as_ref().unwrap();
    let declaration_parameter = update.snapshot.declarations.generic_signature(callable.declaration_owner()).unwrap().parameters[0];
    let impl_parameter = context.generic_signature.as_ref().unwrap().parameters[0];
    let callable_parameter = definition.signature.generics.as_ref().unwrap().parameters[0];
    assert_ne!(declaration_parameter, impl_parameter);
    assert_ne!(declaration_parameter, callable_parameter);
    assert_ne!(impl_parameter, callable_parameter);
    for parameter in [declaration_parameter, impl_parameter, callable_parameter] {
        assert_eq!(update.snapshot.store.type_parameter(parameter).name.as_ref(), "T");
    }
    assert_eq!(
        update.snapshot.store.type_parameter(callable_parameter).owner,
        TypeParameterOwner::Callable(callable.clone())
    );
    let mut store = update.snapshot.store.as_ref().clone();
    let body = context
        .signature_for_body(&mut store, &update.snapshot.declarations, definition.signature.clone())
        .unwrap();
    for ty in [
        body.parameters[0].declared_type.canonical_type().unwrap(),
        body.declared_return.canonical_type().unwrap(),
    ] {
        assert!(matches!(store.get(ty), TypeData::Parameter(parameter) if *parameter == callable_parameter));
    }
}

#[test]
fn test_covering_body_rejects_unrelated_callable_binder() {
    let module = test_module();
    let source = r#"
data Id<T>()
impl<T> Id<T> {
  bad<U>(_ value: Id<U>) -> Id<T> { value }
}
"#;
    let mut session = SemanticWorkspaceSession::new();
    let update = session.update(single_module_input(module, source));
    assert!(
        update
            .snapshot
            .diagnostics
            .values()
            .flat_map(|diagnostics| diagnostics.iter())
            .any(|diagnostic| diagnostic.code == phalcom_semantic::diagnostic::DiagnosticCode::ReturnMismatch),
        "unrelated binders must conflict: {:?}",
        update.snapshot.diagnostics
    );
}

#[test]
fn test_covering_body_rejects_self_type_application() {
    let module = test_module();
    let source = r#"
data Id<T>()
impl<T> Id<T> {
  bad(_ value: Self<T>) -> Unit { () }
}
"#;
    let mut session = SemanticWorkspaceSession::new();
    let update = session.update(single_module_input(module, source));
    assert!(
        update
            .snapshot
            .diagnostics
            .values()
            .flat_map(|diagnostics| diagnostics.iter())
            .any(|diagnostic| diagnostic.code == phalcom_semantic::diagnostic::DiagnosticCode::ApplicationNotConstructor),
        "Self is a proper type: {:?}",
        update.snapshot.diagnostics
    );
}

#[test]
fn test_covering_body_context_only_permutation_changes_context_not_public_contract() {
    let module = test_module();
    let source_v1 = r#"
data Pair<X, Y>()
impl<A, B> Pair<A, B> {
  ping() -> Unit { () }
}
"#;
    let source_v2 = r#"
data Pair<X, Y>()
impl<A, B> Pair<B, A> {
  ping() -> Unit { () }
}
"#;
    let callable = CallableId::new(
        DeclarationId::new(module.clone(), "Pair".into()),
        Selector::method("ping", []).unwrap(),
        DispatchSide::Instance,
    );
    let definition_key = QueryKey::CallableDefinition(callable.clone());
    let body_key = QueryKey::CallableBody(callable.clone());
    let mut session = SemanticWorkspaceSession::new();
    let first = session.update(single_module_input(module.clone(), source_v1));
    let first_def = first.snapshot.callable_definitions.get(&callable).expect("first definition");
    let first_context = first_def.inherent_body_context.as_ref().unwrap().clone();
    let first_definition_fp = session.db().ready_product_fingerprint(&definition_key).expect("first definition product");
    assert!(
        session
            .db()
            .index()
            .dependencies_of(&body_key)
            .is_some_and(|edges| edges.iter().any(|edge| edge.dependency == definition_key)),
        "callable body must record its accepted callable definition dependency"
    );
    let first_public_fp = phalcom_semantic::db::fingerprint::callable_signature_product_fingerprint(first.snapshot.callable_signatures.get(&callable).unwrap());
    let second = session.update(single_module_input(module.clone(), source_v2));
    let second_def = second.snapshot.callable_definitions.get(&callable).expect("second definition");
    let second_context = second_def.inherent_body_context.as_ref().unwrap();
    let second_definition_fp = session.db().ready_product_fingerprint(&definition_key).expect("second definition product");
    let second_public_fp =
        phalcom_semantic::db::fingerprint::callable_signature_product_fingerprint(second.snapshot.callable_signatures.get(&callable).unwrap());
    assert_ne!(first_definition_fp, second_definition_fp);
    assert_ne!(first_context.source_target_head, second_context.source_target_head);
    assert_eq!(first_public_fp, second_public_fp);
    assert_eq!(
        session.db().query_state(&body_key).and_then(|state| state.revision()),
        Some(second.snapshot.id.revision())
    );
    assert!(
        second.stats.callable_bodies_recomputed > 0 || second.stats.query_products_revalidated > 0,
        "body query must be recomputed or revalidated: {:?}",
        second.stats
    );
    let mut cold_session = SemanticWorkspaceSession::new();
    let cold = cold_session.update(single_module_input(module, source_v2));
    let incremental_body = second.snapshot.callable_analyses.get(&callable).unwrap();
    let cold_body = cold.snapshot.callable_analyses.get(&callable).unwrap();
    assert_eq!(incremental_body.dependency_fingerprint, cold_body.dependency_fingerprint);
    assert_eq!(incremental_body.return_validation, cold_body.return_validation);
    assert_eq!(incremental_body.diagnostics, cold_body.diagnostics);
}

#[test]
fn test_covering_body_incremental_permutation_recomputes_and_matches_cold_analysis() {
    let source_v1 = r#"
data Pair<X, Y>(_ first: X, _ second: Y)
impl<A, B> Pair<A, B> {
  @class
  value(_ a: A, _ b: B) -> A { a }
}
"#;
    let source_v2 = r#"
data Pair<X, Y>(_ first: X, _ second: Y)
impl<A, B> Pair<B, A> {
  @class
  value(_ a: A, _ b: B) -> A { a }
}
"#;
    let module = test_module();
    let callable = CallableId::new(
        DeclarationId::new(module.clone(), "Pair".into()),
        Selector::method(
            "value",
            [
                phalcom_common::selector::SelectorSlot::Positional,
                phalcom_common::selector::SelectorSlot::Positional,
            ],
        )
        .unwrap(),
        DispatchSide::Class,
    );
    let definition_key = QueryKey::CallableDefinition(callable.clone());
    let body_key = QueryKey::CallableBody(callable.clone());
    let mut incremental = SemanticWorkspaceSession::new();
    let first = incremental.update(single_module_input(module.clone(), source_v1));
    assert!(!first.snapshot.has_errors(), "v1 diagnostics: {:?}", first.snapshot.diagnostics);
    let first_context_fingerprint = incremental
        .db()
        .query_state(&definition_key)
        .and_then(|state| state.input_fingerprint())
        .expect("initial callable definition fingerprint");
    let second = incremental.update(single_module_input(module.clone(), source_v2));
    assert!(!second.snapshot.has_errors(), "incremental diagnostics: {:?}", second.snapshot.diagnostics);
    let second_context_fingerprint = incremental
        .db()
        .query_state(&definition_key)
        .and_then(|state| state.input_fingerprint())
        .expect("permuted callable definition fingerprint");
    assert_ne!(
        first_context_fingerprint, second_context_fingerprint,
        "covering-map permutation must change body-context input"
    );
    assert_eq!(
        incremental.db().query_state(&body_key).and_then(|state| state.revision()),
        Some(second.snapshot.id.revision()),
        "the callable body must be recomputed in the permutation revision",
    );

    let mut cold = SemanticWorkspaceSession::new();
    let cold = cold.update(single_module_input(module.clone(), source_v2));
    assert_eq!(
        second.snapshot.diagnostics, cold.snapshot.diagnostics,
        "incremental and cold diagnostics must agree"
    );
    let incremental_body = second.snapshot.callable_analyses.get(&callable).expect("incremental Pair.value body");
    let cold_body = cold.snapshot.callable_analyses.get(&callable).expect("cold Pair.value body");
    assert_eq!(
        incremental_body.dependency_fingerprint, cold_body.dependency_fingerprint,
        "incremental and cold callable body semantic products must agree",
    );
    assert_eq!(incremental_body.return_validation, cold_body.return_validation);
    assert_eq!(incremental_body.diagnostics, cold_body.diagnostics);
    assert_eq!(
        second.snapshot.callable_signatures.get(&callable),
        cold.snapshot.callable_signatures.get(&callable),
        "incremental and cold public callable signatures must agree",
    );
    let public_signature = second.snapshot.callable_signatures.get(&callable).expect("public Pair.value signature");
    let return_ty = public_signature.declared_return.canonical_type().expect("declared return type");
    let TypeData::Parameter(return_parameter) = second.snapshot.store.get(return_ty) else {
        panic!("public return must remain declaration-parameter-owned: {return_ty:?}");
    };
    assert!(matches!(
        second.snapshot.store.type_parameter(*return_parameter).owner,
        TypeParameterOwner::Declaration(ref owner) if owner == &DeclarationId::new(module, "Pair".into())
    ));
}

#[test]
fn test_subclass_inherits_inherent_impl_method() {
    let module = test_module();
    let src = r#"
class Base {}

impl Base {
  greet() -> String {
    "hello from base"
  }
}

class Child is Base {
  testCall() -> String {
    self.greet()
  }
}
"#;
    let mut session = SemanticWorkspaceSession::new();
    let update = session.update(single_module_input(module.clone(), src));
    assert!(!update.snapshot.has_errors(), "diagnostics: {:?}", update.snapshot.diagnostics);

    let child_decl = DeclarationId::new(module, "Child".into());
    let test_call = CallableId::new(child_decl, Selector::method("testCall", []).unwrap(), DispatchSide::Instance);
    let analysis = update.snapshot.callable_analyses.get(&test_call).expect("testCall body analysis");
    assert!(analysis.diagnostics.is_empty());
}

#[test]
fn test_data_declaration_inherent_impl_body() {
    let module = test_module();
    let src = r#"
data Point(_ x: Int, _ y: Int)

impl Point {
  sum() -> Int {
    100
  }
}
"#;
    let mut session = SemanticWorkspaceSession::new();
    let update = session.update(single_module_input(module.clone(), src));
    assert!(!update.snapshot.has_errors(), "diagnostics: {:?}", update.snapshot.diagnostics);

    let point_decl = DeclarationId::new(module, "Point".into());
    let sum_callable = CallableId::new(point_decl, Selector::method("sum", []).unwrap(), DispatchSide::Instance);
    let analysis = update.snapshot.callable_analyses.get(&sum_callable).expect("sum body analysis");
    assert!(analysis.diagnostics.is_empty());
}

#[test]
fn test_enum_root_inherent_impl_body() {
    let module = test_module();
    let src = r#"
data Status {
  @variant Active
  @variant Inactive
}

impl Status {
  isStatus() -> Bool {
    true
  }
}
"#;
    let mut session = SemanticWorkspaceSession::new();
    let update = session.update(single_module_input(module.clone(), src));
    assert!(!update.snapshot.has_errors(), "diagnostics: {:?}", update.snapshot.diagnostics);

    let status_decl = DeclarationId::new(module, "Status".into());
    let is_status = CallableId::new(status_decl, Selector::method("isStatus", []).unwrap(), DispatchSide::Instance);
    let analysis = update.snapshot.callable_analyses.get(&is_status).expect("isStatus body analysis");
    assert!(analysis.diagnostics.is_empty());
}
