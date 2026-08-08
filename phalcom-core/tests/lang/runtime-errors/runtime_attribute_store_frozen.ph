// area: decorators
// spec: attribute-classes.md "Deferred to v0.3" (A-5)
// status: NEGATIVE
// contract: a class's attribute-retention store is frozen once, at the end
// of its own class-definition (M-ATTR-ROOT) — a further `Object#__attach(_)`
// against it raises `attr.frozen` rather than silently mutating.

@On(Class)
class Author is Attribute {
  _name
  @constructor
  new(_ name) { _name = name }
  name => _name
}

@Author("Ada")
class Engine {}

Engine.__attach(Author.new("Bob"))
