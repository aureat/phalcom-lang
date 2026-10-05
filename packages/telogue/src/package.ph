expose .id
expose .variable
expose .term
expose .goal
expose .clause
expose .predicate
expose .relationship
expose .rule
expose .query
expose .base

export (
  VariableId,
  PredicateId,
  VariableScopeId,
) from .id
export Variable from .variable
export Term from .term
export Goal from .goal
export Clause from .clause
export Predicate from .predicate
export Relationship from .relationship
export Rule from .rule
export Query from .query
export KnowledgeBase from .base