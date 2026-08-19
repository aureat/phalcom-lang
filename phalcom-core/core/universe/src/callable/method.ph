// `Method#attributes`/`#attributesOfType(_)` — the same reflection surface
// as `Behavior` above, for the reified `Method` object a class's method
// dictionary holds.
class Method {
  attributes { self._$attributes }
  attributesOfType(_ cls) { self._$attributes.filter |a| { a.isA(cls) } }
}
