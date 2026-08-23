@!documentation("First-class closure capturing lexical scope.")
@native
class Closure is Function {
  @native arity -> Int
  @native name -> Symbol
  @native whileTrue(_ body: Dynamic) -> Dynamic
  @native on(_ error: Dynamic, _ handler: Dynamic) -> Dynamic
  @native ensure(_ cleanup: Dynamic) -> Dynamic
}
