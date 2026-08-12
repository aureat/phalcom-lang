//! Inferred runtime value knowledge and local fact storage.

use std::collections::BTreeMap;

use phalcom_common::range::SourceRange;

use super::ids::{CallableId, ClassId, DispatchSide, FieldId, ModuleId};
use super::scope::BindingId;

/// Maximum number of incompatible alternatives retained by a union.
pub const MAX_SHAPE_UNION: usize = 8;

/// Advisory runtime value shape. This is deliberately not a language type.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
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
    /// An open method family whose exact selector is chosen at call time.
    Family {
        /// Receiver knowledge retained for later call-site dispatch.
        receiver: Box<ValueShape>,
        /// Base method name before call-site labels are applied.
        base: String,
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
                    base: left_base,
                },
                Self::Family {
                    receiver: right_receiver,
                    base: right_base,
                },
            ) if left_base == right_base => Self::Family {
                receiver: Box::new(left_receiver.join(right_receiver)),
                base: left_base.clone(),
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
    /// Strength of the evidence.
    pub confidence: Confidence,
    /// Source or callable origins, capped to keep facts small.
    pub provenance: Vec<FactOrigin>,
}

impl InferredValue {
    /// Creates an exact syntax fact.
    pub fn exact(shape: ValueShape, range: SourceRange) -> Self {
        Self {
            shape,
            confidence: Confidence::Exact,
            provenance: vec![FactOrigin::Syntax(range)],
        }
    }

    /// Creates a flow fact derived from a binding range.
    pub fn flow(shape: ValueShape, range: SourceRange) -> Self {
        Self {
            shape,
            confidence: Confidence::Flow,
            provenance: vec![FactOrigin::Binding(range)],
        }
    }

    /// Creates an interprocedural fact derived from a resolved call site.
    pub fn interprocedural(shape: ValueShape, range: SourceRange) -> Self {
        Self {
            shape,
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
        Self {
            shape: self.shape.join(&other.shape),
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

#[derive(Clone, Debug, Eq, PartialEq)]
struct BindingFact {
    range: SourceRange,
    value: InferredValue,
}

impl LocalFacts {
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
}
