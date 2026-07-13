// area: decorators
// spec: attribute-classes.md §"Decision"/§"Passive metadata"/§"`@On`"
// status: PASS
// contract: M-ATTR-ROOT's reified-descriptor layer — a passive-metadata
// `Attribute` subclass is instantiated, retained, and reflectable on both a
// class header and a method, via `Behavior#attributes`/`Method#attributes`/
// `attributesOfType(_)`. Positional-only args (parser's attribute-arg-list
// grammar has no label form, `docs/forge/DEFERRED.md`).

@On(Class)
class Author extends Attribute {
  var _name
  construct new(name) { _name = name }
  name => _name
}

@On(Method)
class Tag extends Attribute {
  var _label
  construct new(label) { _label = label }
  label => _label
}

@Author("Ada")
class Engine {
  @Tag("hot")
  run() { return 42 }
}

System.print(Engine.attributesOfType(Author).at(0).name)
System.print(Engine.attributes.size)

let e = Engine.new()
System.print(e.methodFor(Symbol.new("run()")).attributesOfType(Tag).at(0).label)
