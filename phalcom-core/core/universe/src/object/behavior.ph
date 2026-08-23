// `Behavior#attributes`/`#attributesOfType(_)` (object-model.md's metaclass
// tower superclass of `Class`+`Metaclass`) — the reflection surface over the
// native `_attributes` store every class object carries (M-ATTR-ROOT).
// Method-only reopen (no new fields) — safe on a bootstrap class (a
// reopen-with-fields would trip read-before-write).
@native
class Behavior is Object {
  @native
  superclass -> Dynamic
  @native
  superclass=(put value: Dynamic) -> Dynamic
  @native
  name -> String
  @native
  methods -> Dynamic
  @native
  >>(_ selector: Dynamic) -> Dynamic
  attributes { self._$attributes }
  attributesOfType(_ cls) { self._$attributes.filter |a| { a.is(cls) } }
}
