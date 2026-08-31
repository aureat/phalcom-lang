//! Internal pattern value-space representation and exact normalization algebra (Part 05.1).

use crate::identity::VariantId;
use crate::match_semantics::{BranchProofEnvironment, PatternSpaceSummary};
use crate::types::id::TypeId;
use crate::types::relation::{TypeHierarchy, is_subtype};
use crate::types::store::TypeStore;

/// Internal representation of a value space during pattern elimination.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PatternSpace {
    Empty,
    Opaque(TypeId),
    Union(Box<[PatternSpace]>),
    Variant(VariantSpace),
    Tuple(Box<[PatternSpace]>),
    List(ListSpace),
}

/// Space representation for a specific variant case with payload field spaces.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VariantSpace {
    pub variant: VariantId,
    pub exact_case: TypeId,
    pub fields: Box<[PatternSpace]>,
    pub proof: BranchProofEnvironment,
}

/// Space representation for list sequences.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ListSpace {
    pub prefix: Box<[PatternSpace]>,
    pub rest: Option<Box<PatternSpace>>,
}

impl PatternSpace {
    pub fn empty() -> Self {
        Self::Empty
    }

    pub fn opaque(ty: TypeId) -> Self {
        Self::Opaque(ty)
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Self::Empty => true,
            Self::Union(spaces) => spaces.is_empty() || spaces.iter().all(Self::is_empty),
            Self::Variant(v) => v.fields.iter().any(Self::is_empty),
            Self::Tuple(elements) => elements.iter().any(Self::is_empty),
            Self::List(l) => l.prefix.iter().any(Self::is_empty),
            Self::Opaque(_) => false,
        }
    }

    /// Normalizes nested unions, strips empty alternatives, and reduces singletons.
    pub fn normalize(self) -> Self {
        if self.is_empty() {
            return Self::Empty;
        }
        match self {
            Self::Union(spaces) => {
                let mut flat = Vec::new();
                for space in spaces.into_vec() {
                    let norm = space.normalize();
                    if norm.is_empty() {
                        continue;
                    }
                    match norm {
                        Self::Union(nested) => {
                            flat.extend(nested.into_vec());
                        }
                        other => {
                            if !flat.contains(&other) {
                                flat.push(other);
                            }
                        }
                    }
                }
                if flat.is_empty() {
                    Self::Empty
                } else if flat.len() == 1 {
                    flat.pop().unwrap()
                } else {
                    Self::Union(flat.into_boxed_slice())
                }
            }
            Self::Variant(mut v) => {
                let mut fields = Vec::with_capacity(v.fields.len());
                for field in v.fields.into_vec() {
                    let f = field.normalize();
                    if f.is_empty() {
                        return Self::Empty;
                    }
                    fields.push(f);
                }
                v.fields = fields.into_boxed_slice();
                Self::Variant(v)
            }
            Self::Tuple(elements) => {
                let mut norm_elements = Vec::with_capacity(elements.len());
                for element in elements.into_vec() {
                    let e = element.normalize();
                    if e.is_empty() {
                        return Self::Empty;
                    }
                    norm_elements.push(e);
                }
                Self::Tuple(norm_elements.into_boxed_slice())
            }
            Self::List(mut l) => {
                let mut norm_prefix = Vec::with_capacity(l.prefix.len());
                for p in l.prefix.into_vec() {
                    let norm_p = p.normalize();
                    if norm_p.is_empty() {
                        return Self::Empty;
                    }
                    norm_prefix.push(norm_p);
                }
                l.prefix = norm_prefix.into_boxed_slice();
                l.rest = l.rest.map(|r| Box::new(r.normalize()));
                Self::List(l)
            }
            other => other,
        }
    }

    /// Computes the union of two pattern spaces: `self ∪ other`.
    pub fn union(&self, other: &Self) -> Self {
        if self.is_empty() {
            return other.clone().normalize();
        }
        if other.is_empty() {
            return self.clone().normalize();
        }
        Self::Union(vec![self.clone(), other.clone()].into_boxed_slice()).normalize()
    }

    /// Computes the intersection of two pattern spaces: `self ∩ other`.
    pub fn intersect(&self, other: &Self, store: &mut TypeStore, hier: &dyn TypeHierarchy) -> Self {
        if self.is_empty() || other.is_empty() {
            return Self::Empty;
        }
        match (self, other) {
            (Self::Union(members), right) => {
                let mut result = Self::Empty;
                for member in members.iter() {
                    let inter = member.intersect(right, store, hier);
                    result = result.union(&inter);
                }
                result.normalize()
            }
            (left, Self::Union(members)) => {
                let mut result = Self::Empty;
                for member in members.iter() {
                    let inter = left.intersect(member, store, hier);
                    result = result.union(&inter);
                }
                result.normalize()
            }
            (Self::Opaque(t1), Self::Opaque(t2)) => {
                if t1 == t2 {
                    Self::Opaque(*t1)
                } else if is_subtype(store, hier, *t1, *t2) {
                    Self::Opaque(*t1)
                } else if is_subtype(store, hier, *t2, *t1) {
                    Self::Opaque(*t2)
                } else {
                    Self::Empty
                }
            }
            (Self::Variant(v), Self::Opaque(t)) | (Self::Opaque(t), Self::Variant(v)) => {
                if is_subtype(store, hier, v.exact_case, *t) {
                    Self::Variant(v.clone()).normalize()
                } else {
                    Self::Empty
                }
            }
            (Self::Variant(v1), Self::Variant(v2)) => {
                if v1.variant != v2.variant {
                    return Self::Empty;
                }
                if v1.fields.len() != v2.fields.len() {
                    return Self::Empty;
                }
                let mut fields = Vec::with_capacity(v1.fields.len());
                for (f1, f2) in v1.fields.iter().zip(v2.fields.iter()) {
                    let inter = f1.intersect(f2, store, hier);
                    if inter.is_empty() {
                        return Self::Empty;
                    }
                    fields.push(inter);
                }
                let mut proof = v1.proof.clone();
                for (param, ty) in &v2.proof.bindings {
                    proof.bindings.insert(*param, *ty);
                }
                let mut equalities = proof.equalities.into_vec();
                for eq in v2.proof.equalities.iter() {
                    if !equalities.contains(eq) {
                        equalities.push(eq.clone());
                    }
                }
                proof.equalities = equalities.into_boxed_slice();

                Self::Variant(VariantSpace {
                    variant: v1.variant.clone(),
                    exact_case: v1.exact_case,
                    fields: fields.into_boxed_slice(),
                    proof,
                })
                .normalize()
            }
            (Self::Tuple(t1), Self::Tuple(t2)) => {
                if t1.len() != t2.len() {
                    return Self::Empty;
                }
                let mut elements = Vec::with_capacity(t1.len());
                for (e1, e2) in t1.iter().zip(t2.iter()) {
                    let inter = e1.intersect(e2, store, hier);
                    if inter.is_empty() {
                        return Self::Empty;
                    }
                    elements.push(inter);
                }
                Self::Tuple(elements.into_boxed_slice()).normalize()
            }
            (Self::Tuple(t), Self::Opaque(op)) | (Self::Opaque(op), Self::Tuple(t)) => {
                // If the opaque type is a tuple with matching arity:
                if let crate::types::store::TypeData::Tuple(elements) = store.get(*op).clone() {
                    if elements.len() == t.len() {
                        let mut inter_elements = Vec::with_capacity(t.len());
                        for (field, elem) in t.iter().zip(elements.iter()) {
                            let op_field = Self::Opaque(elem.ty);
                            let inter = field.intersect(&op_field, store, hier);
                            if inter.is_empty() {
                                return Self::Empty;
                            }
                            inter_elements.push(inter);
                        }
                        return Self::Tuple(inter_elements.into_boxed_slice()).normalize();
                    }
                }
                Self::Empty
            }
            _ => Self::Empty,
        }
    }

    /// Computes the difference of two pattern spaces: `self \ other`.
    pub fn subtract(&self, other: &Self, store: &mut TypeStore, hier: &dyn TypeHierarchy) -> Self {
        if self.is_empty() {
            return Self::Empty;
        }
        if other.is_empty() {
            return self.clone().normalize();
        }
        match (self, other) {
            (Self::Union(members), right) => {
                let mut result = Self::Empty;
                for member in members.iter() {
                    let diff = member.subtract(right, store, hier);
                    result = result.union(&diff);
                }
                result.normalize()
            }
            (left, Self::Union(members)) => {
                let mut current = left.clone();
                for member in members.iter() {
                    current = current.subtract(member, store, hier);
                    if current.is_empty() {
                        break;
                    }
                }
                current.normalize()
            }
            (Self::Opaque(t1), Self::Opaque(t2)) => {
                if t1 == t2 || is_subtype(store, hier, *t1, *t2) {
                    Self::Empty
                } else {
                    Self::Opaque(*t1)
                }
            }
            (Self::Variant(v), Self::Opaque(t)) => {
                if is_subtype(store, hier, v.exact_case, *t) {
                    Self::Empty
                } else {
                    Self::Variant(v.clone())
                }
            }
            (Self::Opaque(t), Self::Variant(v)) => {
                // If opaque is a general type not resolved to finite enum cases:
                if is_subtype(store, hier, *t, v.exact_case) {
                    Self::Empty
                } else {
                    Self::Opaque(*t)
                }
            }
            (Self::Variant(v1), Self::Variant(v2)) => {
                if v1.variant != v2.variant {
                    return Self::Variant(v1.clone());
                }
                if v1.fields.is_empty() {
                    return Self::Empty;
                }
                if v1.fields.len() != v2.fields.len() {
                    return Self::Variant(v1.clone());
                }

                // Check if v2 strictly covers all fields of v1
                let all_covered = v1.fields.iter().zip(v2.fields.iter()).all(|(f1, f2)| f1.subtract(f2, store, hier).is_empty());
                if all_covered {
                    return Self::Empty;
                }

                // Multi-field Cartesian difference:
                // For (A, B) \ (C, D) = ((A \ C), B) ∪ ((A ∩ C), (B \ D))
                let mut result_spaces = Vec::new();
                let mut accumulated_inter = Vec::new();

                for i in 0..v1.fields.len() {
                    let diff_i = v1.fields[i].subtract(&v2.fields[i], store, hier);
                    if !diff_i.is_empty() {
                        let mut branch_fields = accumulated_inter.clone();
                        branch_fields.push(diff_i);
                        for j in (i + 1)..v1.fields.len() {
                            branch_fields.push(v1.fields[j].clone());
                        }
                        result_spaces.push(Self::Variant(VariantSpace {
                            variant: v1.variant.clone(),
                            exact_case: v1.exact_case,
                            fields: branch_fields.into_boxed_slice(),
                            proof: v1.proof.clone(),
                        }));
                    }

                    let inter_i = v1.fields[i].intersect(&v2.fields[i], store, hier);
                    if inter_i.is_empty() {
                        break;
                    }
                    accumulated_inter.push(inter_i);
                }

                if result_spaces.is_empty() {
                    Self::Empty
                } else if result_spaces.len() == 1 {
                    result_spaces.pop().unwrap().normalize()
                } else {
                    Self::Union(result_spaces.into_boxed_slice()).normalize()
                }
            }
            (Self::Tuple(t1), Self::Tuple(t2)) => {
                if t1.len() != t2.len() {
                    return Self::Tuple(t1.clone());
                }
                let mut result_spaces = Vec::new();
                let mut accumulated_inter = Vec::new();

                for i in 0..t1.len() {
                    let diff_i = t1[i].subtract(&t2[i], store, hier);
                    if !diff_i.is_empty() {
                        let mut branch_fields = accumulated_inter.clone();
                        branch_fields.push(diff_i);
                        for j in (i + 1)..t1.len() {
                            branch_fields.push(t1[j].clone());
                        }
                        result_spaces.push(Self::Tuple(branch_fields.into_boxed_slice()));
                    }

                    let inter_i = t1[i].intersect(&t2[i], store, hier);
                    if inter_i.is_empty() {
                        break;
                    }
                    accumulated_inter.push(inter_i);
                }

                if result_spaces.is_empty() {
                    Self::Empty
                } else if result_spaces.len() == 1 {
                    result_spaces.pop().unwrap().normalize()
                } else {
                    Self::Union(result_spaces.into_boxed_slice()).normalize()
                }
            }
            _ => self.clone().normalize(),
        }
    }

    /// Converts internal `PatternSpace` to public `PatternSpaceSummary`.
    pub fn summarize(&self) -> PatternSpaceSummary {
        match self {
            Self::Empty => PatternSpaceSummary::Empty,
            Self::Opaque(t) => PatternSpaceSummary::Opaque(*t),
            Self::Union(members) => PatternSpaceSummary::Union(members.iter().map(Self::summarize).collect::<Vec<_>>().into_boxed_slice()),
            Self::Variant(v) => PatternSpaceSummary::Variant {
                variant: v.variant.clone(),
                exact_case: v.exact_case,
                fields: v.fields.iter().map(Self::summarize).collect::<Vec<_>>().into_boxed_slice(),
            },
            Self::Tuple(elements) => PatternSpaceSummary::Tuple(elements.iter().map(Self::summarize).collect::<Vec<_>>().into_boxed_slice()),
            Self::List(_) => PatternSpaceSummary::List,
        }
    }
}
