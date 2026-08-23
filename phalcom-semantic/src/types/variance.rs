//! Declaration-site variance algebra and occurrence path validation.

use super::id::{KindId, TypeId, TypeParameterId};
use super::parameter::TypeParameterData;
use super::store::{TypeData, TypeStore};
use crate::identity::DeclarationId;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Variance {
    Covariant,
    Contravariant,
    Invariant,
}

impl Variance {
    /// Polarity composition law:
    /// + ∘ + = +
    /// + ∘ - = -
    /// - ∘ + = -
    /// - ∘ - = +
    /// 0 ∘ x = 0
    /// x ∘ 0 = 0
    #[inline]
    pub fn compose(self, other: Self) -> Self {
        match (self, other) {
            (Self::Invariant, _) | (_, Self::Invariant) => Self::Invariant,
            (Self::Covariant, Self::Covariant) => Self::Covariant,
            (Self::Covariant, Self::Contravariant) => Self::Contravariant,
            (Self::Contravariant, Self::Covariant) => Self::Contravariant,
            (Self::Contravariant, Self::Contravariant) => Self::Covariant,
        }
    }

    /// Inverts polarity (e.g. For callable parameter positions).
    #[inline]
    pub fn invert(self) -> Self {
        match self {
            Self::Covariant => Self::Contravariant,
            Self::Contravariant => Self::Covariant,
            Self::Invariant => Self::Invariant,
        }
    }

    /// Checks if this variance satisfies the declared requirement.
    /// In particular, Invariant is satisfied only by Invariant (or no occurrence).
    /// Covariant requires only Covariant occurrences (+).
    /// Contravariant requires only Contravariant occurrences (-).
    pub fn satisfies(self, required: Self) -> bool {
        match (self, required) {
            (Self::Covariant, Self::Covariant) => true,
            (Self::Contravariant, Self::Contravariant) => true,
            (Self::Invariant, Self::Invariant) => true,
            _ => false,
        }
    }
}

/// Path step explaining variance violation for diagnostics.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VarianceStep {
    Field {
        name: Box<str>,
        mutable: bool,
    },
    CallableParameter {
        index: u32,
        label: Option<Box<str>>,
    },
    CallableReturn,
    AppliedArgument {
        origin: DeclarationId,
        index: u32,
        param_variance: Variance,
    },
    TupleElement {
        index: u32,
    },
    RecordField {
        name: Box<str>,
    },
    UnionMember {
        index: u32,
    },
}

/// Detailed diagnostic trace when a declared variance fails validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VarianceDiagnostic {
    pub parameter: TypeParameterId,
    pub parameter_name: Box<str>,
    pub declared: Variance,
    pub actual_polarity: Variance,
    pub path: Vec<VarianceStep>,
}

/// Computes the net variance occurrence of `param` inside `ty`.
pub fn compute_variance_occurrence(
    store: &TypeStore,
    param: TypeParameterId,
    ty: TypeId,
    context_polarity: Variance,
    path: &mut Vec<VarianceStep>,
) -> Option<(Variance, Vec<VarianceStep>)> {
    match store.get(ty) {
        TypeData::Parameter(p) => {
            if *p == param {
                Some((context_polarity, path.clone()))
            } else {
                None
            }
        }
        TypeData::Applied { origin, arguments } => {
            let mut result_variance = None;
            for (idx, &arg) in arguments.iter().enumerate() {
                // If origin is a nominal declaration, look up parameter variance; default to invariant
                let param_var = if let TypeData::Nominal { declaration } = store.get(*origin) {
                    store.get_parameter_variance(declaration, idx as u32).unwrap_or(Variance::Invariant)
                } else {
                    Variance::Invariant
                };

                let effective_polarity = context_polarity.compose(param_var);
                let origin_decl = if let TypeData::Nominal { declaration } = store.get(*origin) {
                    declaration.clone()
                } else {
                    DeclarationId::new(phalcom_modules::identity::ModuleId::core(), "AppliedOrigin".into())
                };

                path.push(VarianceStep::AppliedArgument {
                    origin: origin_decl,
                    index: idx as u32,
                    param_variance: param_var,
                });
                if let Some((occ, p)) = compute_variance_occurrence(store, param, arg, effective_polarity, path) {
                    result_variance = match result_variance {
                        None => Some((occ, p)),
                        Some((prev_occ, prev_p)) => {
                            if prev_occ == occ {
                                Some((prev_occ, prev_p))
                            } else {
                                Some((Variance::Invariant, p))
                            }
                        }
                    };
                }
                path.pop();
            }
            result_variance
        }
        TypeData::Union(members) => {
            let mut result_variance = None;
            for (idx, &m) in members.iter().enumerate() {
                path.push(VarianceStep::UnionMember { index: idx as u32 });
                if let Some((occ, p)) = compute_variance_occurrence(store, param, m, context_polarity, path) {
                    result_variance = match result_variance {
                        None => Some((occ, p)),
                        Some((prev_occ, prev_p)) => {
                            if prev_occ == occ {
                                Some((prev_occ, prev_p))
                            } else {
                                Some((Variance::Invariant, p))
                            }
                        }
                    };
                }
                path.pop();
            }
            result_variance
        }
        TypeData::Tuple(elems) => {
            let mut result_variance = None;
            for (idx, e) in elems.iter().enumerate() {
                path.push(VarianceStep::TupleElement { index: idx as u32 });
                if let Some((occ, p)) = compute_variance_occurrence(store, param, e.ty, context_polarity, path) {
                    result_variance = match result_variance {
                        None => Some((occ, p)),
                        Some((prev_occ, prev_p)) => {
                            if prev_occ == occ {
                                Some((prev_occ, prev_p))
                            } else {
                                Some((Variance::Invariant, p))
                            }
                        }
                    };
                }
                path.pop();
            }
            result_variance
        }
        TypeData::Record(row_id) => {
            let row = store.record_row(*row_id);
            let mut result_variance = None;
            for f in row.fields.iter() {
                path.push(VarianceStep::RecordField { name: f.name.clone() });
                if let Some((occ, p)) = compute_variance_occurrence(store, param, f.ty, context_polarity, path) {
                    result_variance = match result_variance {
                        None => Some((occ, p)),
                        Some((prev_occ, prev_p)) => {
                            if prev_occ == occ {
                                Some((prev_occ, prev_p))
                            } else {
                                Some((Variance::Invariant, p))
                            }
                        }
                    };
                }
                path.pop();
            }
            result_variance
        }
        TypeData::Callable(call) => {
            let mut result_variance = None;
            // Parameters are in contravariant position
            let param_polarity = context_polarity.invert();
            for (idx, p) in call.parameters.iter().enumerate() {
                path.push(VarianceStep::CallableParameter {
                    index: idx as u32,
                    label: p.label.clone(),
                });
                if let Some((occ, p_trace)) = compute_variance_occurrence(store, param, p.ty, param_polarity, path) {
                    result_variance = match result_variance {
                        None => Some((occ, p_trace)),
                        Some((prev_occ, prev_p)) => {
                            if prev_occ == occ {
                                Some((prev_occ, prev_p))
                            } else {
                                Some((Variance::Invariant, p_trace))
                            }
                        }
                    };
                }
                path.pop();
            }

            // Return type is in covariant position
            path.push(VarianceStep::CallableReturn);
            if let Some((occ, ret_trace)) = compute_variance_occurrence(store, param, call.return_type, context_polarity, path) {
                result_variance = match result_variance {
                    None => Some((occ, ret_trace)),
                    Some((prev_occ, prev_p)) => {
                        if prev_occ == occ {
                            Some((prev_occ, prev_p))
                        } else {
                            Some((Variance::Invariant, ret_trace))
                        }
                    }
                };
            }
            path.pop();

            result_variance
        }
        _ => None,
    }
}
