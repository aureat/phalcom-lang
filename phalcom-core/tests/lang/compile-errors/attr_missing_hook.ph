// area: compile-errors
// spec: attribute-classes.md §"A-1"/§"The `Attribute` root and the hook protocol"
// status: NEGATIVE
// contract: a declared Install/Dispatch/Runtime tier with no matching hook
// method implemented is attr.missing_hook (M-ATTR-ROOT)

@On(Method, Install)
class Memoize is Attribute {
  _cache
  @constructor
  new() { _cache = None }
}
