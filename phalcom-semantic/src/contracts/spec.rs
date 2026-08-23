//! Static contract specifications (@requires, @ensures, @invariant).

use crate::identity::ExpressionId;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConditionKind {
    Requires,
    Ensures,
    Invariant,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractCondition {
    pub expression: ExpressionId,
    pub kind: ConditionKind,
    pub label: Option<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ContractSpec {
    pub preconditions: Vec<ContractCondition>,
    pub postconditions: Vec<ContractCondition>,
    pub invariants: Vec<ContractCondition>,
}

impl ContractSpec {
    pub fn is_empty(&self) -> bool {
        self.preconditions.is_empty() && self.postconditions.is_empty() && self.invariants.is_empty()
    }
}
