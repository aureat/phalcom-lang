//! Inferred runtime value knowledge and local fact storage.

use std::collections::BTreeMap;

use phalcom_common::range::SourceRange;

use super::ids::{CallableId, ClassId, ModuleId};

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
    bindings: BTreeMap<String, Vec<BindingFact>>,
}

/// Inferred class-local field writes and reads.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FieldFacts {
    fields: BTreeMap<(ClassId, String), InferredValue>,
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
    pub fn record(&mut self, name: impl Into<String>, range: SourceRange, value: InferredValue) {
        self.bindings.entry(name.into()).or_default().push(BindingFact { range, value });
    }

    /// Returns the most recent fact visible before `offset`.
    pub fn binding_at(&self, name: &str, offset: usize) -> Option<&InferredValue> {
        self.bindings
            .get(name)?
            .iter()
            .filter(|fact| fact.range.start < offset)
            .max_by_key(|fact| fact.range.start)
            .map(|fact| &fact.value)
    }

    /// Returns every binding name recorded in this file.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.bindings.keys().map(String::as_str)
    }
}

impl FieldFacts {
    /// Records or joins one field write.
    pub fn record(&mut self, class: ClassId, name: impl Into<String>, value: InferredValue) {
        let key = (class, name.into());
        self.fields.entry(key).and_modify(|old| *old = old.join(&value)).or_insert(value);
    }

    /// Returns the joined fact for one class-local field.
    pub fn get(&self, class: &ClassId, name: &str) -> Option<&InferredValue> {
        self.fields.get(&(class.clone(), name.to_string()))
    }

    /// Iterates over class-qualified field facts for publication in the database.
    pub fn iter(&self) -> impl Iterator<Item = (&(ClassId, String), &InferredValue)> {
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
}
