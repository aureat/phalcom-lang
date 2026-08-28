//! Bounded advisory runtime-shape domain.

use std::sync::Arc;

use phalcom_ast::ast::NormalizedSelectorSpec;
use phalcom_common::selector::{Selector, SelectorPattern};

use crate::identity::{CallableId, DeclarationId, ModuleId};

/// Maximum number of incompatible alternatives retained by a union.
pub const MAX_SHAPE_UNION: usize = 8;

/// Immutable captured method-family routing evidence.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CapturedMethodFamilyShape {
    /// Canonical declaration whose behavior supplied this family.
    pub source_behavior: DeclarationId,
    /// Selector pattern used to capture the family.
    pub pattern: SelectorPattern,
    /// Exact selector-to-callable entries in canonical order.
    pub exact: Box<[(Selector, CallableId)]>,
    /// Rest candidates in compiler dispatch order.
    pub rest_candidates: Box<[CallableId]>,
}

impl CapturedMethodFamilyShape {
    /// Returns an exact captured target. Rest acceptance belongs to the
    /// compiler signature/dispatch products and is intentionally not copied
    /// into this advisory shape.
    pub fn exact_target(&self, selector: &Selector) -> Option<CallableId> {
        self.exact
            .iter()
            .find(|(candidate, _)| candidate == selector)
            .map(|(_, callable)| callable.clone())
    }

    fn canonicalize(&self) -> Self {
        let mut exact = self.exact.to_vec();
        exact.sort();
        exact.dedup();
        let mut rest_candidates = self.rest_candidates.to_vec();
        rest_candidates.sort();
        rest_candidates.dedup();
        Self {
            source_behavior: self.source_behavior.clone(),
            pattern: self.pattern.clone(),
            exact: exact.into_boxed_slice(),
            rest_candidates: rest_candidates.into_boxed_slice(),
        }
    }
}

/// Advisory runtime-value shape. This is deliberately not a language type.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ValueShape {
    /// No useful runtime knowledge is available.
    Unknown,
    /// Canonical bottom result; contributes no runtime value to a join.
    Never,
    /// Canonical unit runtime value.
    Unit,
    /// Instance of canonical declaration.
    Instance(DeclarationId),
    /// Class object for canonical declaration.
    ClassObject(DeclarationId),
    /// First-class module value.
    Module(ModuleId),
    /// Tuple with positional shape knowledge.
    Tuple(Arc<[ValueShape]>),
    /// Literal list with exact positional shape knowledge.
    ExactList(Arc<[ValueShape]>),
    /// Record with canonical label ordering.
    Record(Arc<[(Box<str>, ValueShape)]>),
    /// List and joined element shape.
    List(Box<ValueShape>),
    /// Set and joined element shape.
    Set(Box<ValueShape>),
    /// Map and joined key/value shapes.
    Map { key: Box<ValueShape>, value: Box<ValueShape> },
    /// Range and joined bound shape.
    Range(Box<ValueShape>),
    /// Callable value.
    Callable(CallableId),
    /// First-class selector value.
    Selector(Selector),
    /// First-class selector pattern value.
    SelectorPattern(SelectorPattern),
    /// Open or exact method family retained for later dispatch.
    Family { receiver: Box<ValueShape>, spec: NormalizedSelectorSpec },
    /// Captured exact method.
    Method(CallableId),
    /// Captured method-family snapshot.
    MethodFamily(Arc<CapturedMethodFamilyShape>),
    /// Exact method bound to a receiver.
    BoundMethod { receiver: Box<ValueShape>, method: CallableId },
    /// Method family bound to a receiver.
    BoundMethodFamily {
        receiver: Box<ValueShape>,
        family: Arc<CapturedMethodFamilyShape>,
    },
    /// Bounded set of incompatible alternatives.
    Union(Arc<[ValueShape]>),
}

impl ValueShape {
    /// Builds a canonical record shape, sorting labels and joining duplicate
    /// labels instead of preserving parser/hash-map traversal order.
    pub fn record(fields: impl IntoIterator<Item = (impl Into<Box<str>>, ValueShape)>) -> Self {
        let mut fields = fields
            .into_iter()
            .map(|(label, value)| (label.into(), value.canonicalize()))
            .collect::<Vec<_>>();
        fields.sort_by(|left, right| left.0.cmp(&right.0));
        let mut canonical: Vec<(Box<str>, ValueShape)> = Vec::with_capacity(fields.len());
        for (label, value) in fields {
            if let Some((previous, previous_value)) = canonical.last_mut() {
                if *previous == label {
                    *previous_value = previous_value.join(&value);
                    continue;
                }
            }
            canonical.push((label, value));
        }
        Self::Record(Arc::from(canonical.into_boxed_slice()))
    }

    /// Builds a canonical bounded union.
    pub fn bounded_union(shapes: impl IntoIterator<Item = Self>) -> Self {
        let mut alternatives = Vec::new();
        for shape in shapes {
            match shape {
                Self::Unknown => return Self::Unknown,
                Self::Never => continue,
                Self::Union(nested) => alternatives.extend(nested.iter().map(ValueShape::canonicalize)),
                other => alternatives.push(other.canonicalize()),
            }
        }
        alternatives.sort();
        alternatives.dedup();
        if alternatives.len() > MAX_SHAPE_UNION {
            return Self::Unknown;
        }
        match alternatives.as_slice() {
            [] => Self::Never,
            [single] => single.clone(),
            _ => Self::Union(Arc::from(alternatives.into_boxed_slice())),
        }
    }

    /// Joins two advisory shapes conservatively and deterministically.
    pub fn join(&self, other: &Self) -> Self {
        let left = self.canonicalize();
        let right = other.canonicalize();
        if left == right {
            return left;
        }
        match (&left, &right) {
            (Self::Unknown, _) | (_, Self::Unknown) => Self::Unknown,
            (Self::Never, other) | (other, Self::Never) => other.clone(),
            (Self::List(left), Self::List(right)) => Self::List(Box::new(left.join(right))),
            (Self::ExactList(left), Self::ExactList(right)) if left.len() == right.len() => Self::ExactList(Arc::from(
                left.iter().zip(right.iter()).map(|(a, b)| a.join(b)).collect::<Vec<_>>().into_boxed_slice(),
            )),
            (Self::ExactList(left), Self::ExactList(right)) => Self::List(Box::new(Self::bounded_union(left.iter().chain(right.iter()).cloned()))),
            (Self::ExactList(elements), Self::List(element)) | (Self::List(element), Self::ExactList(elements)) => {
                let element_shape = Self::bounded_union(elements.iter().cloned());
                let joined = if elements.is_empty() {
                    (**element).clone()
                } else {
                    element_shape.join(element)
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
            (Self::Tuple(left), Self::Tuple(right)) if left.len() == right.len() => Self::Tuple(Arc::from(
                left.iter().zip(right.iter()).map(|(a, b)| a.join(b)).collect::<Vec<_>>().into_boxed_slice(),
            )),
            (Self::Record(left), Self::Record(right)) if left.len() == right.len() && left.iter().zip(right.iter()).all(|(a, b)| a.0 == b.0) => {
                Self::record(left.iter().zip(right.iter()).map(|(left, right)| (left.0.clone(), left.1.join(&right.1))))
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
            _ => Self::bounded_union([left, right]),
        }
    }

    /// Recursively canonicalizes collection, record, union, and family data.
    pub fn canonicalize(&self) -> Self {
        match self {
            Self::Unknown
            | Self::Never
            | Self::Unit
            | Self::Instance(_)
            | Self::ClassObject(_)
            | Self::Module(_)
            | Self::Callable(_)
            | Self::Selector(_)
            | Self::SelectorPattern(_) => self.clone(),
            Self::Tuple(elements) => Self::Tuple(Arc::from(elements.iter().map(Self::canonicalize).collect::<Vec<_>>().into_boxed_slice())),
            Self::ExactList(elements) => Self::ExactList(Arc::from(elements.iter().map(Self::canonicalize).collect::<Vec<_>>().into_boxed_slice())),
            Self::Record(fields) => Self::record(fields.iter().map(|(label, value)| (label.clone(), value.canonicalize()))),
            Self::List(element) => Self::List(Box::new(element.canonicalize())),
            Self::Set(element) => Self::Set(Box::new(element.canonicalize())),
            Self::Map { key, value } => Self::Map {
                key: Box::new(key.canonicalize()),
                value: Box::new(value.canonicalize()),
            },
            Self::Range(element) => Self::Range(Box::new(element.canonicalize())),
            Self::Family { receiver, spec } => Self::Family {
                receiver: Box::new(receiver.canonicalize()),
                spec: spec.clone(),
            },
            Self::Method(_) => self.clone(),
            Self::MethodFamily(family) => Self::MethodFamily(Arc::new(family.canonicalize())),
            Self::BoundMethod { receiver, method } => Self::BoundMethod {
                receiver: Box::new(receiver.canonicalize()),
                method: method.clone(),
            },
            Self::BoundMethodFamily { receiver, family } => Self::BoundMethodFamily {
                receiver: Box::new(receiver.canonicalize()),
                family: Arc::new(family.canonicalize()),
            },
            Self::Union(alternatives) => Self::bounded_union(alternatives.iter().cloned()),
        }
    }

    /// Returns joined element knowledge for collection-like shapes.
    pub fn element_shape(&self) -> Self {
        match self {
            Self::List(element) | Self::Set(element) | Self::Range(element) => (**element).clone(),
            Self::Tuple(elements) | Self::ExactList(elements) => Self::bounded_union(elements.iter().cloned()),
            _ => Self::Unknown,
        }
    }
}
