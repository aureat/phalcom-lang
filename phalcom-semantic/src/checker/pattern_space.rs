//! Internal pattern value-space representation and exact normalization algebra (Part 05.1).

use crate::identity::VariantId;
use crate::match_semantics::{BranchProofEnvironment, PatternSpaceSummary};
use crate::types::id::TypeId;
use crate::types::relation::{TypeHierarchy, is_subtype};
use crate::types::store::TypeStore;
use phalcom_ast::ast::MapPatternKey;
use phalcom_native_meta::UniverseKey;

/// Internal representation of a value space during pattern elimination.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PatternSpace {
    Empty,
    Opaque(TypeId),
    Union(Box<[PatternSpace]>),
    Variant(VariantSpace),
    Tuple(Box<[PatternSpace]>),
    List(ListSpace),
    /// A refutable open record-pattern space. Kept distinct from `Opaque` so
    /// an open field requirement can never prove an arbitrary object domain
    /// exhaustive.
    Record(RecordSpace),
    /// A refutable open map-pattern space. Required keys are runtime tests,
    /// not a claim that every map contains those keys.
    Map(MapSpace),
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordSpace {
    pub ty: TypeId,
    pub fields: Box<[(Box<str>, PatternSpace)]>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MapSpace {
    pub ty: TypeId,
    pub entries: Box<[(MapPatternKey, PatternSpace)]>,
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
            Self::List(l) => l.prefix.iter().any(Self::is_empty) || l.rest.as_ref().is_some_and(|rest| rest.is_empty()),
            Self::Record(record) => record.fields.iter().any(|(_, space)| space.is_empty()),
            Self::Map(map) => map.entries.iter().any(|(_, space)| space.is_empty()),
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
            Self::Record(mut record) => {
                let mut fields = Vec::with_capacity(record.fields.len());
                for (label, space) in record.fields.into_vec() {
                    let normalized = space.normalize();
                    if normalized.is_empty() {
                        return Self::Empty;
                    }
                    fields.push((label, normalized));
                }
                record.fields = fields.into_boxed_slice();
                Self::Record(record)
            }
            Self::Map(map) => {
                let mut entries = Vec::with_capacity(map.entries.len());
                for (key, space) in map.entries.into_vec() {
                    let normalized = space.normalize();
                    if normalized.is_empty() {
                        return Self::Empty;
                    }
                    entries.push((key, normalized));
                }
                let map = MapSpace {
                    ty: map.ty,
                    entries: entries.into_boxed_slice(),
                };
                Self::Map(map)
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
                if t1 == t2 || (t1.index() < store.len() && t2.index() < store.len() && is_subtype(store, hier, *t1, *t2)) {
                    Self::Opaque(*t1)
                } else if t1.index() < store.len() && t2.index() < store.len() && is_subtype(store, hier, *t2, *t1) {
                    Self::Opaque(*t2)
                } else {
                    Self::Empty
                }
            }
            (Self::Variant(v), Self::Opaque(t)) | (Self::Opaque(t), Self::Variant(v)) => {
                if v.exact_case.index() < store.len() && t.index() < store.len() && is_subtype(store, hier, v.exact_case, *t) {
                    Self::Variant(v.clone()).normalize()
                } else {
                    Self::Empty
                }
            }
            (Self::Variant(v1), Self::Variant(v2)) => {
                if v1.variant != v2.variant {
                    return Self::Empty;
                }
                if !crate::checker::gadt_proof::exact_cases_compatible(store, v1.exact_case, v2.exact_case) {
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
                let proof = match crate::checker::gadt_proof::merge_branch_proofs(store, &v1.proof, &v2.proof) {
                    crate::checker::gadt_proof::ProofMerge::Compatible(proof) => proof,
                    crate::checker::gadt_proof::ProofMerge::Contradictory => return Self::Empty,
                };

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
            (Self::List(left), Self::Opaque(ty)) | (Self::Opaque(ty), Self::List(left)) => {
                if canonical_list_element(store, *ty).is_some() {
                    Self::List(left.clone()).normalize()
                } else {
                    Self::Empty
                }
            }
            (Self::List(left), Self::List(right)) => intersect_list_spaces(left, right, store, hier),
            (Self::Opaque(_), Self::Record(record)) | (Self::Record(record), Self::Opaque(_)) => Self::Record(record.clone()).normalize(),
            (Self::Opaque(_), Self::Map(map)) | (Self::Map(map), Self::Opaque(_)) => Self::Map(map.clone()).normalize(),
            (Self::Record(left), Self::Record(_right)) => Self::Record(left.clone()).normalize(),
            (Self::Map(left), Self::Map(_right)) => Self::Map(left.clone()).normalize(),
            (Self::Tuple(t), Self::Opaque(op)) | (Self::Opaque(op), Self::Tuple(t)) => {
                // If the opaque type is a tuple with matching arity:
                if op.index() < store.len() {
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
                if t1 == t2 || (t1.index() < store.len() && t2.index() < store.len() && is_subtype(store, hier, *t1, *t2)) {
                    Self::Empty
                } else {
                    Self::Opaque(*t1)
                }
            }
            (Self::Variant(v), Self::Opaque(t)) => {
                if v.exact_case.index() < store.len() && t.index() < store.len() && is_subtype(store, hier, v.exact_case, *t) {
                    Self::Empty
                } else {
                    Self::Variant(v.clone())
                }
            }
            (Self::Opaque(t), Self::Variant(v)) => {
                // If opaque is a general type not resolved to finite enum cases:
                if t.index() < store.len() && v.exact_case.index() < store.len() && is_subtype(store, hier, *t, v.exact_case) {
                    Self::Empty
                } else {
                    Self::Opaque(*t)
                }
            }
            (Self::Variant(v1), Self::Variant(v2)) => {
                if v1.variant != v2.variant {
                    return Self::Variant(v1.clone());
                }
                if !crate::checker::gadt_proof::exact_cases_compatible(store, v1.exact_case, v2.exact_case)
                    || matches!(
                        crate::checker::gadt_proof::merge_branch_proofs(store, &v1.proof, &v2.proof),
                        crate::checker::gadt_proof::ProofMerge::Contradictory
                    )
                {
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
            (Self::Opaque(ty), Self::List(list)) => subtract_opaque_list(*ty, list, store),
            (Self::List(list), Self::Opaque(ty)) => {
                if canonical_list_element(store, *ty).is_some() {
                    Self::Empty
                } else {
                    Self::List(list.clone())
                }
            }
            (Self::List(left), Self::List(right)) => subtract_list_spaces(left, right, store, hier),
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
            Self::Record(record) => PatternSpaceSummary::Opaque(record.ty),
            Self::Map(map) => PatternSpaceSummary::Opaque(map.ty),
        }
    }
}

/// Returns the element type only for the canonical universe `List<T>`.
///
/// Pattern-space algebra has no checking context, so it derives identity from
/// the same canonical declaration used by core surface registration. A user
/// declaration merely named `List` must not acquire sequence semantics.
fn canonical_list_element(store: &TypeStore, ty: TypeId) -> Option<TypeId> {
    let (declaration, arguments) = store.applied_nominal_parts(ty)?;
    (declaration == crate::core_surface::universe_declaration(UniverseKey::List) && arguments.len() == 1).then(|| arguments[0])
}

fn opaque_tail(space: Option<&PatternSpace>) -> bool {
    matches!(space, Some(PatternSpace::Opaque(_)))
}

fn intersect_list_spaces(left: &ListSpace, right: &ListSpace, store: &mut TypeStore, hier: &dyn TypeHierarchy) -> PatternSpace {
    let left_exact = left.rest.is_none();
    let right_exact = right.rest.is_none();

    if left_exact && right_exact {
        if left.prefix.len() != right.prefix.len() {
            return PatternSpace::Empty;
        }
        let mut prefix = Vec::with_capacity(left.prefix.len());
        for (left_field, right_field) in left.prefix.iter().zip(right.prefix.iter()) {
            let field = left_field.intersect(right_field, store, hier);
            if field.is_empty() {
                return PatternSpace::Empty;
            }
            prefix.push(field);
        }
        return PatternSpace::List(ListSpace {
            prefix: prefix.into_boxed_slice(),
            rest: None,
        })
        .normalize();
    }

    if left_exact != right_exact {
        let (exact, at_least) = if left_exact { (left, right) } else { (right, left) };
        if exact.prefix.len() < at_least.prefix.len() {
            return PatternSpace::Empty;
        }
        let mut prefix = Vec::with_capacity(exact.prefix.len());
        for (exact_field, required_field) in exact.prefix.iter().zip(at_least.prefix.iter()) {
            let field = exact_field.intersect(required_field, store, hier);
            if field.is_empty() {
                return PatternSpace::Empty;
            }
            prefix.push(field);
        }
        if exact.prefix.len() == at_least.prefix.len() && opaque_tail(at_least.rest.as_deref()) {
            prefix.extend(exact.prefix[at_least.prefix.len()..].iter().cloned());
            return PatternSpace::List(ListSpace {
                prefix: prefix.into_boxed_slice(),
                rest: None,
            })
            .normalize();
        }

        // A constrained tail can still be intersected when the exact list has
        // no remaining elements. For deeper constrained tails, retain a
        // conservative list product rather than inventing a length proof.
        if exact.prefix.len() == at_least.prefix.len() {
            let tail = PatternSpace::List(ListSpace {
                prefix: Box::new([]),
                rest: None,
            })
            .intersect(at_least.rest.as_deref().expect("at-least list has a rest"), store, hier);
            if tail.is_empty() {
                return PatternSpace::Empty;
            }
            return PatternSpace::List(ListSpace {
                prefix: prefix.into_boxed_slice(),
                rest: None,
            })
            .normalize();
        }

        return PatternSpace::List(exact.clone()).normalize();
    }

    // Both patterns have an unbounded tail. Exact prefix alignment is enough
    // for current source syntax (`[head, *tail]`); differing prefix lengths
    // retain a conservative over-approximation.
    if left.prefix.len() != right.prefix.len() {
        return PatternSpace::List(left.clone()).normalize();
    }
    let mut prefix = Vec::with_capacity(left.prefix.len());
    for (left_field, right_field) in left.prefix.iter().zip(right.prefix.iter()) {
        let field = left_field.intersect(right_field, store, hier);
        if field.is_empty() {
            return PatternSpace::Empty;
        }
        prefix.push(field);
    }
    let rest =
        left.rest
            .as_deref()
            .expect("unbounded left list has a rest")
            .intersect(right.rest.as_deref().expect("unbounded right list has a rest"), store, hier);
    if rest.is_empty() {
        return PatternSpace::Empty;
    }
    PatternSpace::List(ListSpace {
        prefix: prefix.into_boxed_slice(),
        rest: Some(Box::new(rest)),
    })
    .normalize()
}

fn subtract_opaque_list(ty: TypeId, list: &ListSpace, store: &mut TypeStore) -> PatternSpace {
    let Some(element) = canonical_list_element(store, ty) else {
        return PatternSpace::Opaque(ty);
    };

    if list.rest.is_some() && opaque_tail(list.rest.as_deref()) {
        // A wildcard tail pattern `[p1, ..., pn, *rest]` covers every list
        // whose length is at least n. The finite residual consists of shorter
        // exact lists, which is representable for the current partition form.
        let mut residual = Vec::with_capacity(list.prefix.len());
        for length in 0..list.prefix.len() {
            residual.push(PatternSpace::List(ListSpace {
                prefix: vec![PatternSpace::Opaque(element); length].into_boxed_slice(),
                rest: None,
            }));
        }
        return PatternSpace::Union(residual.into_boxed_slice()).normalize();
    }

    if list.prefix.is_empty() && list.rest.is_none() {
        // Removing `[]` leaves all non-empty lists.
        return PatternSpace::List(ListSpace {
            prefix: Box::new([PatternSpace::Opaque(element)]),
            rest: Some(Box::new(PatternSpace::Opaque(ty))),
        })
        .normalize();
    }

    // Excluding one exact non-empty length would require an unbounded union of
    // length partitions. Keep the canonical list domain as a conservative
    // residual instead of claiming coverage we cannot represent.
    PatternSpace::Opaque(ty)
}

fn subtract_list_spaces(left: &ListSpace, right: &ListSpace, store: &mut TypeStore, hier: &dyn TypeHierarchy) -> PatternSpace {
    if left.rest.is_none() && right.rest.is_some() {
        if left.prefix.len() < right.prefix.len() {
            return PatternSpace::List(left.clone());
        }
        if opaque_tail(right.rest.as_deref()) {
            let covered = left
                .prefix
                .iter()
                .zip(right.prefix.iter())
                .all(|(left_field, right_field)| left_field.subtract(right_field, store, hier).is_empty());
            if covered {
                return PatternSpace::Empty;
            }
        }
        return PatternSpace::List(left.clone());
    }

    if left.rest.is_some() && right.rest.is_none() {
        // Removing one exact length from an unbounded tail is not finitely
        // representable in this product. Preserve residual conservatively.
        return PatternSpace::List(left.clone());
    }

    if left.rest.is_none() && right.rest.is_none() {
        if left.prefix.len() != right.prefix.len() {
            return PatternSpace::List(left.clone());
        }
        let covered = left
            .prefix
            .iter()
            .zip(right.prefix.iter())
            .all(|(left_field, right_field)| left_field.subtract(right_field, store, hier).is_empty());
        return if covered { PatternSpace::Empty } else { PatternSpace::List(left.clone()) };
    }

    if left.prefix.len() == right.prefix.len() && opaque_tail(right.rest.as_deref()) {
        let covered = left
            .prefix
            .iter()
            .zip(right.prefix.iter())
            .all(|(left_field, right_field)| left_field.subtract(right_field, store, hier).is_empty());
        if covered {
            return PatternSpace::Empty;
        }
    }
    PatternSpace::List(left.clone())
}
