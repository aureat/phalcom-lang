//
// Identity
//

data VariableIdentity()

type VariableId =
  Id<VariableIdentity>

impl VariableId {
  toString -> String {
    "VariableId(\(self.id))"
  }
}


data PredicateIdentity()

type PredicateId =
  Id<PredicateIdentity>

impl PredicateId {
  toString -> String {
    "PredicateId(\(self.id))"
  }
}


data VariableScopeIdentity()

type VariableScopeId =
  Id<VariableScopeIdentity>

impl VariableScopeId {
  toString -> String {
    "VariableScopeId(\(self.id))"
  }
}


export (
  VariableId,
  PredicateId,
  VariableScopeId
)