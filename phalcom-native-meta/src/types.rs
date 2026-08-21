//! Static symbolic type and callable specification structures.

use crate::universe::UniverseKey;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum TypeExprSpec {
    Unknown,
    Never,
    SelfType,
    Universe(UniverseKey),
    Parameter(&'static str),
    Applied {
        origin: &'static TypeExprSpec,
        arguments: &'static [TypeExprSpec],
    },
    Union(&'static [TypeExprSpec]),
    Tuple(&'static ParameterTupleSpec),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct LabeledParameterSpec {
    pub label: &'static str,
    pub ty: &'static TypeExprSpec,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct RestParameterSpec {
    pub ty: Option<&'static TypeExprSpec>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct ParameterTupleSpec {
    pub positional: &'static [TypeExprSpec],
    pub labeled: &'static [LabeledParameterSpec],
    pub rest: Option<RestParameterSpec>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct TypeParameterSpec {
    pub name: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct CallableTypeSpec {
    pub type_params: &'static [TypeParameterSpec],
    pub params: &'static ParameterTupleSpec,
    pub return_type: &'static TypeExprSpec,
}
