use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use phalcom_ast::ast::NormalizedSelectorSpec;
use phalcom_common::range::SourceRange;
pub use phalcom_common::selector::{Selector, SelectorPattern, SelectorSlot};

use super::ids::{CallableId, ClassId, DispatchSide, FieldId, ModuleId};
use super::scope::BindingId;
use super::surface::RestSurface;

/// Maximum number of incompatible alternatives retained by a union.
pub const MAX_SHAPE_UNION: usize = 8;

/// Immutable captured MethodFamily routing snapshot.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CapturedMethodFamilyShape {
    /// Class behavior from which this method family was captured.
    pub source_behavior: ClassId,
    /// The selector pattern used to capture the family.
    pub pattern: SelectorPattern,
    /// Exact selector-to-callable table in deterministic order.
    pub exact: Box<[(Selector, CallableId)]>,
    /// Ordered fallback rest callable candidates (subclass -> superclass).
    pub rest: Box<[(CallableId, RestSurface)]>,
}

impl CapturedMethodFamilyShape {
    /// Resolves a call selector against this captured snapshot.
    pub fn resolve_call(&self, selector: &Selector) -> Option<CallableId> {
        for (exact_sel, callable) in self.exact.iter() {
            if exact_sel == selector {
                return Some(callable.clone());
            }
        }
        let mut positionals = 0;
        let mut labels = Vec::new();
        for slot in selector.slots.iter() {
            match slot {
                SelectorSlot::Positional => positionals += 1,
                SelectorSlot::Label(label) => labels.push(label.clone()),
            }
        }
        for (callable, rest_surface) in self.rest.iter() {
            if rest_surface.accepts(positionals, &labels) {
                return Some(callable.clone());
            }
        }
        None
    }
}

/// Advisory runtime value shape. This is deliberately not a language type.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ValueShape {
    /// No useful runtime knowledge is available.
    Unknown,
    /// An instance of a module-qualified class.
    Instance(ClassId),
    /// A class object for a module-qualified class.
    ClassObject(ClassId),
    /// A first-class module value.
    Module(ModuleId),
    /// A tuple with positional shape knowledge.
    Tuple(Vec<ValueShape>),
    /// A literal list with exact positional shape knowledge.
    ExactList(Vec<ValueShape>),
    /// A record with statically known labels where available.
    Record(Vec<(String, ValueShape)>),
    /// A list and its joined element shape.
    List(Box<ValueShape>),
    /// A set and its joined element shape.
    Set(Box<ValueShape>),
    /// A map and its joined key/value shapes.
    Map {
        /// Joined key shape.
        key: Box<ValueShape>,
        /// Joined value shape.
        value: Box<ValueShape>,
    },
    /// A range and its joined bound shape.
    Range(Box<ValueShape>),
    /// A callable value.
    Callable(CallableId),
    /// A first-class selector value.
    Selector(Selector),
    /// A first-class selector pattern value.
    SelectorPattern(SelectorPattern),
    /// An exact or open method family whose selector is resolved against the receiver.
    Family {
        /// Receiver knowledge retained for later call-site dispatch.
        receiver: Box<ValueShape>,
        /// Exact selector or selector pattern spec.
        spec: NormalizedSelectorSpec,
    },
    /// An unreflected captured exact method.
    Method(CallableId),
    /// An unreflected captured method family snapshot.
    MethodFamily(Arc<CapturedMethodFamilyShape>),
    /// An exact method bound to a receiver value.
    BoundMethod {
        /// Bound receiver shape.
        receiver: Box<ValueShape>,
        /// Captured method identity.
        method: CallableId,
    },
    /// A method family bound to a receiver value.
    BoundMethodFamily {
        /// Bound receiver shape.
        receiver: Box<ValueShape>,
        /// Captured method family snapshot.
        family: Arc<CapturedMethodFamilyShape>,
    },
    /// A bounded set of incompatible alternatives.
    Union(Vec<ValueShape>),
}

impl ValueShape {
    /// Joins two shapes conservatively, widening oversized unions to unknown.
    pub fn join(&self, other: &Self) -> Self {
        if self == other {
            return self.clone();
        }
        match (self, other) {
            (Self::Unknown, _) | (_, Self::Unknown) => Self::Unknown,
            (Self::List(left), Self::List(right)) => Self::List(Box::new(left.join(right))),
            (Self::ExactList(left), Self::ExactList(right)) if left.len() == right.len() => {
                Self::ExactList(left.iter().zip(right).map(|(a, b)| a.join(b)).collect())
            }
            (Self::ExactList(left), Self::ExactList(right)) => Self::List(Box::new(ValueShape::bounded_union(left.iter().chain(right.iter()).cloned()))),
            (Self::ExactList(elements), Self::List(element)) | (Self::List(element), Self::ExactList(elements)) => {
                let exact_element = ValueShape::bounded_union(elements.iter().cloned());
                let joined = if elements.is_empty() {
                    (**element).clone()
                } else {
                    exact_element.join(element)
                };
                Self::List(Box::new(joined))
            }
            (Self::Set(left), Self::Set(right)) => Self::Set(Box::new(left.join(right))),
            (Self::Range(left), Self::Range(right)) => Self::Range(Box::new(left.join(right))),
            (
                Self::Map {
                    key: left_key,
                    value: left_value,
                },
                Self::Map {
                    key: right_key,
                    value: right_value,
                },
            ) => Self::Map {
                key: Box::new(left_key.join(right_key)),
                value: Box::new(left_value.join(right_value)),
            },
            (Self::Tuple(left), Self::Tuple(right)) if left.len() == right.len() => Self::Tuple(left.iter().zip(right).map(|(a, b)| a.join(b)).collect()),
            (Self::Record(left), Self::Record(right)) if left.len() == right.len() && left.iter().zip(right).all(|(a, b)| a.0 == b.0) => {
                Self::Record(left.iter().zip(right).map(|(a, b)| (a.0.clone(), a.1.join(&b.1))).collect())
            }
            (
                Self::Family {
                    receiver: left_receiver,
                    spec: left_spec,
                },
                Self::Family {
                    receiver: right_receiver,
                    spec: right_spec,
                },
            ) if left_spec == right_spec => Self::Family {
                receiver: Box::new(left_receiver.join(right_receiver)),
                spec: left_spec.clone(),
            },
            (
                Self::BoundMethod {
                    receiver: left_receiver,
                    method: left_method,
                },
                Self::BoundMethod {
                    receiver: right_receiver,
                    method: right_method,
                },
            ) if left_method == right_method => Self::BoundMethod {
                receiver: Box::new(left_receiver.join(right_receiver)),
                method: left_method.clone(),
            },
            (
                Self::BoundMethodFamily {
                    receiver: left_receiver,
                    family: left_family,
                },
                Self::BoundMethodFamily {
                    receiver: right_receiver,
                    family: right_family,
                },
            ) if left_family == right_family => Self::BoundMethodFamily {
                receiver: Box::new(left_receiver.join(right_receiver)),
                family: left_family.clone(),
            },
            _ => Self::bounded_union([self.clone(), other.clone()]),
        }
    }

    /// Builds a bounded union from shapes, flattening nested unions.
    pub fn bounded_union(shapes: impl IntoIterator<Item = Self>) -> Self {
        let mut alternatives = Vec::new();
        for shape in shapes {
            match shape {
                Self::Unknown => return Self::Unknown,
                Self::Union(nested) => {
                    for item in nested {
                        if !alternatives.contains(&item) {
                            alternatives.push(item);
                        }
                    }
                }
                item if !alternatives.contains(&item) => alternatives.push(item),
                _ => {}
            }
            if alternatives.len() > MAX_SHAPE_UNION {
                return Self::Unknown;
            }
        }
        match alternatives.as_slice() {
            [] => Self::Unknown,
            [single] => single.clone(),
            _ => Self::Union(alternatives),
        }
    }

    /// Returns the known element shape of a collection-like value.
    pub fn element_shape(&self) -> Self {
        match self {
            Self::List(element) | Self::Set(element) | Self::Range(element) => (**element).clone(),
            Self::Tuple(elements) => Self::bounded_union(elements.iter().cloned()),
            Self::ExactList(elements) => Self::bounded_union(elements.iter().cloned()),
            _ => Self::Unknown,
        }
    }
}

/// Confidence attached to inferred knowledge.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Confidence {
    /// Direct syntax or an exact semantic guarantee.
    Exact,
    /// Local flow or binding propagation.
    Flow,
    /// Cross-call or cross-file propagation.
    Interprocedural,
    /// Structural use-site heuristic.
    Heuristic,
}

impl Confidence {
    fn join(self, other: Self) -> Self {
        self.min(other)
    }
}

/// Origin of one inferred fact.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum FactOrigin {
    /// A literal or other exact syntax fact.
    Syntax(SourceRange),
    /// A binding initializer or reassignment.
    Binding(SourceRange),
    /// A callable summary.
    Callable(CallableId),
    /// A call-site argument.
    CallSite(SourceRange),
    /// A structural use-site constraint.
    Constraint(SourceRange),
}

/// A shape together with confidence and compact provenance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InferredValue {
    /// Inferred runtime shape.
    pub shape: ValueShape,
    /// Known boolean value when flow analysis can prove it.
    pub(crate) known_boolean: Option<bool>,
    /// Strength of the evidence.
    pub confidence: Confidence,
    /// Source or callable origins, capped to keep facts small.
    pub provenance: Vec<FactOrigin>,
}

impl InferredValue {
    /// Creates an unknown fact.
    pub fn unknown() -> Self {
        Self {
            shape: ValueShape::Unknown,
            known_boolean: None,
            confidence: Confidence::Heuristic,
            provenance: Vec::new(),
        }
    }

    /// Creates an exact syntax fact.
    pub fn exact(shape: ValueShape, range: SourceRange) -> Self {
        Self {
            shape,
            known_boolean: None,
            confidence: Confidence::Exact,
            provenance: vec![FactOrigin::Syntax(range)],
        }
    }

    /// Creates an exact boolean syntax fact while retaining its literal value.
    pub fn exact_boolean(value: bool, range: SourceRange) -> Self {
        Self {
            shape: ValueShape::Instance(ClassId::new(super::ids::ModuleId::new(super::ids::CORE_MODULE_URI), "Bool")),
            known_boolean: Some(value),
            confidence: Confidence::Exact,
            provenance: vec![FactOrigin::Syntax(range)],
        }
    }

    /// Creates a flow fact derived from a binding range.
    pub fn flow(shape: ValueShape, range: SourceRange) -> Self {
        Self {
            shape,
            known_boolean: None,
            confidence: Confidence::Flow,
            provenance: vec![FactOrigin::Binding(range)],
        }
    }

    /// Creates an interprocedural fact derived from a resolved call site.
    pub fn interprocedural(shape: ValueShape, range: SourceRange) -> Self {
        Self {
            shape,
            known_boolean: None,
            confidence: Confidence::Interprocedural,
            provenance: vec![FactOrigin::CallSite(range)],
        }
    }

    /// Joins two values and retains a bounded provenance sample.
    pub fn join(&self, other: &Self) -> Self {
        let mut provenance = self.provenance.clone();
        for origin in &other.provenance {
            if !provenance.contains(origin) && provenance.len() < 4 {
                provenance.push(origin.clone());
            }
        }
        let known_boolean = match (self.known_boolean, other.known_boolean) {
            (Some(left), Some(right)) if left == right => Some(left),
            _ => None,
        };
        Self {
            shape: self.shape.join(&other.shape),
            known_boolean,
            confidence: self.confidence.join(other.confidence),
            provenance,
        }
    }
}

/// Monotonic revision assigned to one source file.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FileRevision(pub u64);

/// Local binding facts collected from one parsed source file.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LocalFacts {
    bindings: BTreeMap<BindingId, Vec<BindingFact>>,
}

/// Inferred class-local field writes and reads.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FieldFacts {
    fields: BTreeMap<FieldId, InferredValue>,
    evidence: BTreeMap<FieldId, Vec<FieldEvidence>>,
}

/// Evidence category for one field value.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum FieldEvidenceKind {
    /// Value supplied by a field declaration initializer.
    DeclarationInitializer,
    /// Value written by a source constructor.
    ConstructorInitialization,
    /// Value written by any other executable assignment.
    GeneralWrite,
}

/// One source-backed field value observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FieldEvidence {
    /// Observed value.
    pub value: InferredValue,
    /// How the value entered field storage.
    pub kind: FieldEvidenceKind,
    /// Exact source site of the observation.
    pub site: SourceRange,
}

/// Call-site facts joined into callable parameters.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ParameterFacts {
    params: BTreeMap<(CallableId, String), InferredValue>,
}

/// Stable identity of one callable parameter slot.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ParameterSlot {
    /// Callable owning the parameter.
    pub callable: CallableId,
    /// Source parameter name.
    pub name: String,
}

/// Source of one parameter fact contribution.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ContributionSource {
    /// Evidence emitted while analyzing another callable.
    Callable(CallableId),
    /// Evidence emitted by a module-level statement.
    TopLevel(ModuleId),
}

/// Contribution-indexed parameter evidence. Joining happens only within one
/// slot, so replacing one caller can remove exactly its old contribution.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ParameterContributions {
    by_slot: BTreeMap<ParameterSlot, BTreeMap<ContributionSource, InferredValue>>,
    slots_by_source: BTreeMap<ContributionSource, BTreeSet<ParameterSlot>>,
    joined: BTreeMap<ParameterSlot, InferredValue>,
    #[cfg(test)]
    last_recomputed_slots: usize,
}

/// Change to one joined parameter fact caused by replacing one contribution source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParameterFactDelta {
    /// Parameter slot whose joined fact changed.
    pub slot: ParameterSlot,
    /// Joined fact before the source replacement.
    pub before: Option<InferredValue>,
    /// Joined fact after the source replacement.
    pub after: Option<InferredValue>,
}

impl ParameterContributions {
    /// Replaces all evidence from one source and returns changed joined slots.
    pub fn replace_source(&mut self, source: ContributionSource, facts: impl IntoIterator<Item = (ParameterSlot, InferredValue)>) -> Vec<ParameterFactDelta> {
        let replacement = facts.into_iter().collect::<BTreeMap<_, _>>();
        let old_slots = self.slots_by_source.remove(&source).unwrap_or_default();
        let touched_slots = old_slots.iter().cloned().chain(replacement.keys().cloned()).collect::<BTreeSet<_>>();
        #[cfg(test)]
        {
            self.last_recomputed_slots = touched_slots.len();
        }

        for slot in &old_slots {
            let Some(contributions) = self.by_slot.get_mut(slot) else { continue };
            contributions.remove(&source);
            if contributions.is_empty() {
                self.by_slot.remove(slot);
            }
        }

        if !replacement.is_empty() {
            let new_slots = replacement.keys().cloned().collect::<BTreeSet<_>>();
            self.slots_by_source.insert(source.clone(), new_slots);
            for (slot, value) in replacement {
                self.by_slot.entry(slot).or_default().insert(source.clone(), value);
            }
        }

        touched_slots
            .into_iter()
            .filter_map(|slot| {
                let before = self.joined.get(&slot).cloned();
                let after = self
                    .by_slot
                    .get(&slot)
                    .and_then(|contributions| contributions.values().cloned().reduce(|left, right| left.join(&right)));

                if before == after {
                    return None;
                }
                match after.clone() {
                    Some(value) => {
                        self.joined.insert(slot.clone(), value);
                    }
                    None => {
                        self.joined.remove(&slot);
                    }
                }
                Some(ParameterFactDelta { slot, before, after })
            })
            .collect()
    }

    /// Removes all evidence from one source and returns changed joined slots.
    pub fn remove_source(&mut self, source: &ContributionSource) -> Vec<ParameterFactDelta> {
        self.replace_source(source.clone(), std::iter::empty())
    }

    /// Removes every contribution owned by one module.
    pub fn remove_module(&mut self, module: &ModuleId) -> Vec<ParameterFactDelta> {
        let sources = self
            .slots_by_source
            .keys()
            .filter(|source| match source {
                ContributionSource::Callable(callable) => &callable.owner.module == module,
                ContributionSource::TopLevel(owner) => owner == module,
            })
            .cloned()
            .collect::<Vec<_>>();
        let mut deltas = Vec::new();
        for source in sources {
            deltas.extend(self.remove_source(&source));
        }
        deltas
    }

    /// Returns the cached joined fact for one parameter slot.
    pub fn get(&self, slot: &ParameterSlot) -> Option<&InferredValue> {
        self.joined.get(slot)
    }

    /// Iterates over cached joined parameter facts in deterministic slot order.
    pub fn joined_iter(&self) -> impl Iterator<Item = (&ParameterSlot, &InferredValue)> {
        self.joined.iter()
    }

    /// Returns a snapshot of all cached joined parameter facts.
    pub fn joined(&self) -> BTreeMap<ParameterSlot, InferredValue> {
        self.joined.clone()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct BindingFact {
    range: SourceRange,
    value: InferredValue,
}

impl LocalFacts {
    /// Captures compact per-binding lengths for a speculative nested-flow pass.
    pub(crate) fn checkpoint(&self) -> BTreeMap<BindingId, usize> {
        self.bindings.iter().map(|(binding, facts)| (*binding, facts.len())).collect()
    }

    /// Rolls back facts recorded after a speculative nested-flow pass.
    pub(crate) fn rollback(&mut self, checkpoint: &BTreeMap<BindingId, usize>) {
        self.bindings.retain(|binding, facts| {
            let Some(length) = checkpoint.get(binding) else { return false };
            facts.truncate(*length);
            true
        });
    }

    /// Records one binding write in source order.
    pub fn record(&mut self, binding: BindingId, range: SourceRange, value: InferredValue) {
        self.bindings.entry(binding).or_default().push(BindingFact { range, value });
    }

    /// Returns the most recent fact for one lexical binding before `offset`.
    pub fn value_before(&self, binding: BindingId, offset: usize) -> Option<&InferredValue> {
        self.bindings
            .get(&binding)?
            .iter()
            .filter(|fact| fact.range.start < offset)
            .max_by_key(|fact| fact.range.start)
            .map(|fact| &fact.value)
    }

    /// Returns all facts for one binding in source order.
    pub fn facts_for(&self, binding: BindingId) -> impl Iterator<Item = &InferredValue> {
        self.bindings.get(&binding).into_iter().flat_map(|facts| facts.iter().map(|fact| &fact.value))
    }
}

impl FieldFacts {
    /// Records or joins one field write.
    pub fn record(&mut self, class: ClassId, name: impl Into<String>, side: DispatchSide, value: InferredValue) {
        let site = value.provenance.first().map_or(Default::default(), |origin| match origin {
            super::facts::FactOrigin::Syntax(range)
            | super::facts::FactOrigin::Binding(range)
            | super::facts::FactOrigin::CallSite(range)
            | super::facts::FactOrigin::Constraint(range) => *range,
            super::facts::FactOrigin::Callable(_) => Default::default(),
        });
        self.record_evidence(class, name, side, FieldEvidenceKind::GeneralWrite, site, value);
    }

    /// Records one field observation with its semantic evidence category.
    pub fn record_evidence(
        &mut self,
        class: ClassId,
        name: impl Into<String>,
        side: DispatchSide,
        kind: FieldEvidenceKind,
        site: SourceRange,
        value: InferredValue,
    ) {
        let key = FieldId {
            owner: class,
            name: name.into(),
            side,
        };
        self.evidence.entry(key.clone()).or_default().push(FieldEvidence {
            value: value.clone(),
            kind,
            site,
        });
        self.fields.entry(key).and_modify(|old| *old = old.join(&value)).or_insert(value);
    }

    /// Returns the joined fact for one class-local field.
    pub fn get(&self, class: &ClassId, name: &str, side: DispatchSide) -> Option<&InferredValue> {
        self.fields.get(&FieldId {
            owner: class.clone(),
            name: name.to_string(),
            side,
        })
    }

    /// Returns source-backed observations for one class-qualified field.
    pub fn evidence(&self, class: &ClassId, name: &str, side: DispatchSide) -> &[FieldEvidence] {
        self.evidence
            .get(&FieldId {
                owner: class.clone(),
                name: name.to_string(),
                side,
            })
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    /// Iterates over class-qualified field facts for publication in the database.
    pub fn iter(&self) -> impl Iterator<Item = (&FieldId, &InferredValue)> {
        self.fields.iter()
    }
}

impl ParameterFacts {
    /// Records or joins one resolved call-site argument fact.
    pub fn record(&mut self, callable: CallableId, name: impl Into<String>, value: InferredValue) {
        let key = (callable, name.into());
        self.params.entry(key).and_modify(|old| *old = old.join(&value)).or_insert(value);
    }

    /// Returns the joined fact observed for one callable parameter.
    pub fn get(&self, callable: &CallableId, name: &str) -> Option<&InferredValue> {
        self.params.get(&(callable.clone(), name.to_string()))
    }

    /// Iterates over parameter facts for publication in the database.
    pub fn iter(&self) -> impl Iterator<Item = (&(CallableId, String), &InferredValue)> {
        self.params.iter()
    }

    /// Joins every contribution from `other` into this aggregate.
    pub fn merge_from(&mut self, other: &Self) {
        for ((callable, name), value) in other.iter() {
            self.record(callable.clone(), name.clone(), value.clone());
        }
    }

    /// Widen every retained fact for defensive solver recovery.
    pub(crate) fn widen_all(&mut self) {
        for value in self.params.values_mut() {
            *value = InferredValue::flow(ValueShape::Unknown, Default::default());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_callable(name: impl Into<String>) -> CallableId {
        let name = name.into();
        CallableId {
            owner: ClassId::new(ModuleId::new(format!("file:///{name}.ph")), "Service"),
            selector: "consume(_)".to_string(),
            side: super::super::DispatchSide::Instance,
        }
    }

    fn test_slot(name: impl Into<String>) -> ParameterSlot {
        ParameterSlot {
            callable: test_callable(name),
            name: "value".to_string(),
        }
    }

    fn test_value(class: &str) -> InferredValue {
        InferredValue::flow(
            ValueShape::Instance(ClassId::new(ModuleId::new(format!("file:///{class}.ph")), class)),
            Default::default(),
        )
    }

    #[test]
    fn parameter_facts_merge_joins_contributions() {
        let callable = CallableId {
            owner: ClassId::new(ModuleId::new("file:///service.ph"), "Service"),
            selector: "consume(_)".to_string(),
            side: super::super::DispatchSide::Instance,
        };
        let cat = ClassId::new(ModuleId::new("file:///cat.ph"), "Cat");
        let dog = ClassId::new(ModuleId::new("file:///dog.ph"), "Dog");
        let mut left = ParameterFacts::default();
        let mut right = ParameterFacts::default();
        left.record(
            callable.clone(),
            "value",
            InferredValue::flow(ValueShape::Instance(cat.clone()), Default::default()),
        );
        right.record(
            callable.clone(),
            "value",
            InferredValue::flow(ValueShape::Instance(dog.clone()), Default::default()),
        );

        left.merge_from(&right);

        assert!(
            matches!(left.get(&callable, "value").unwrap().shape, ValueShape::Union(ref values) if values.contains(&ValueShape::Instance(cat)) && values.contains(&ValueShape::Instance(dog)))
        );
    }

    #[test]
    fn parameter_contributions_replace_source_reports_only_old_and_new_slots() {
        let tracked = ContributionSource::Callable(test_callable("tracked"));
        let old_a = test_slot("old-a");
        let old_b = test_slot("old-b");
        let new_c = test_slot("new-c");
        let mut contributions = ParameterContributions::default();

        contributions.replace_source(tracked.clone(), [(old_a.clone(), test_value("Cat")), (old_b.clone(), test_value("Dog"))]);

        for index in 0..1_000 {
            contributions.replace_source(
                ContributionSource::Callable(test_callable(format!("unrelated-{index}"))),
                [(test_slot(format!("unrelated-slot-{index}")), test_value("Other"))],
            );
        }

        let deltas = contributions.replace_source(tracked, [(old_a.clone(), test_value("Dog")), (new_c.clone(), test_value("Cat"))]);
        let touched = deltas.iter().map(|delta| delta.slot.clone()).collect::<BTreeSet<_>>();

        assert_eq!(touched, BTreeSet::from([old_a.clone(), old_b.clone(), new_c.clone()]));
        assert_eq!(deltas.len(), 3);
        assert_eq!(contributions.last_recomputed_slots, 3);
        assert_eq!(contributions.joined().len(), 1_002);
        assert_eq!(contributions.get(&old_b).map(|value| &value.shape), None);
    }

    #[test]
    fn parameter_contributions_preserve_joining_and_remove_one_source_locally() {
        let slot = test_slot("shared");
        let left_source = ContributionSource::Callable(test_callable("left"));
        let right_source = ContributionSource::Callable(test_callable("right"));
        let mut contributions = ParameterContributions::default();

        contributions.replace_source(left_source.clone(), [(slot.clone(), test_value("Cat"))]);
        contributions.replace_source(right_source.clone(), [(slot.clone(), test_value("Dog"))]);
        assert!(matches!(contributions.get(&slot).unwrap().shape, ValueShape::Union(ref values) if values.len() == 2));

        let deltas = contributions.remove_source(&left_source);
        assert_eq!(deltas.len(), 1);
        assert!(matches!(deltas[0].before.as_ref().unwrap().shape, ValueShape::Union(ref values) if values.len() == 2));
        assert!(matches!(deltas[0].after.as_ref().unwrap().shape, ValueShape::Instance(_)));
        assert!(matches!(contributions.get(&slot).unwrap().shape, ValueShape::Instance(_)));

        let deltas = contributions.remove_source(&right_source);
        assert_eq!(deltas[0].after, None);
        assert_eq!(contributions.get(&slot), None);
    }

    #[test]
    fn parameter_contributions_omit_unchanged_replacements_and_order_joined_facts() {
        let source = ContributionSource::Callable(test_callable("source"));
        let first = test_slot("first");
        let second = test_slot("second");
        let value = test_value("Stable");
        let mut contributions = ParameterContributions::default();

        contributions.replace_source(source.clone(), [(second.clone(), value.clone()), (first.clone(), value.clone())]);
        assert!(
            contributions
                .replace_source(source, [(first.clone(), value.clone()), (second.clone(), value)])
                .is_empty()
        );

        let ordered = contributions.joined_iter().map(|(slot, _)| slot.clone()).collect::<Vec<_>>();
        assert_eq!(ordered, vec![first, second]);
    }

    #[test]
    fn family_and_method_family_joins() {
        let cat = ValueShape::Instance(ClassId::new(ModuleId::new("file:///cat.ph"), "Cat"));
        let dog = ValueShape::Instance(ClassId::new(ModuleId::new("file:///dog.ph"), "Dog"));
        let exact_foo = NormalizedSelectorSpec::Exact(Selector::method("foo", [SelectorSlot::Positional]).unwrap());

        let family_a = ValueShape::Family {
            receiver: Box::new(cat.clone()),
            spec: exact_foo.clone(),
        };
        let family_b = ValueShape::Family {
            receiver: Box::new(dog.clone()),
            spec: exact_foo.clone(),
        };
        let joined_family = family_a.join(&family_b);
        assert_eq!(
            joined_family,
            ValueShape::Family {
                receiver: Box::new(ValueShape::Union(vec![cat.clone(), dog.clone()])),
                spec: exact_foo,
            }
        );

        let pattern = SelectorPattern::named_method("foo", [], [], true).unwrap();
        let callable_a = CallableId {
            owner: ClassId::new(ModuleId::new("file:///cat.ph"), "Cat"),
            selector: "foo(_)".into(),
            side: super::super::DispatchSide::Instance,
        };
        let callable_b = CallableId {
            owner: ClassId::new(ModuleId::new("file:///dog.ph"), "Dog"),
            selector: "foo(_)".into(),
            side: super::super::DispatchSide::Instance,
        };
        let mf_1 = ValueShape::MethodFamily(Arc::new(CapturedMethodFamilyShape {
            source_behavior: ClassId::new(ModuleId::new("file:///cat.ph"), "Cat"),
            pattern: pattern.clone(),
            exact: Box::new([(Selector::method("foo", [SelectorSlot::Positional]).unwrap(), callable_a)]),
            rest: Box::new([]),
        }));
        let mf_2 = ValueShape::MethodFamily(Arc::new(CapturedMethodFamilyShape {
            source_behavior: ClassId::new(ModuleId::new("file:///dog.ph"), "Dog"),
            pattern,
            exact: Box::new([(Selector::method("foo", [SelectorSlot::Positional]).unwrap(), callable_b)]),
            rest: Box::new([]),
        }));
        let joined_mf = mf_1.join(&mf_2);
        assert_eq!(joined_mf, ValueShape::Union(vec![mf_1, mf_2]));
    }

    #[test]
    fn exact_list_join_preserves_positions_until_shape_must_widen() {
        let int = ValueShape::Instance(ClassId::new(ModuleId::new("file:///core.ph"), "Int"));
        let string = ValueShape::Instance(ClassId::new(ModuleId::new("file:///core.ph"), "String"));
        assert_eq!(
            ValueShape::ExactList(vec![int.clone(), string.clone()]).join(&ValueShape::ExactList(vec![int.clone(), int.clone()])),
            ValueShape::ExactList(vec![int.clone(), ValueShape::Union(vec![string, int.clone()])])
        );
        assert_eq!(
            ValueShape::ExactList(vec![int.clone()]).join(&ValueShape::ExactList(vec![int.clone(), int.clone()])),
            ValueShape::List(Box::new(int.clone()))
        );
        assert_eq!(
            ValueShape::ExactList(Vec::new()).join(&ValueShape::List(Box::new(int.clone()))),
            ValueShape::List(Box::new(int))
        );
    }
}
