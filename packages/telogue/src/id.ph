from .identity_token import Id

data VariableIdentity()
type VariableId = Id<VariableIdentity>

data PredicateIdentity()
type PredicateId = Id<PredicateIdentity>

data VariableScopeIdentity()
type VariableScopeId = Id<VariableScopeIdentity>

export (VariableId, PredicateId, VariableScopeId)
