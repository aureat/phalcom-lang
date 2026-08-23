@!documentation("Reified message invocation object.")
@native
class Message is Object {
  @native selector -> Selector
  @native name -> Symbol
  @native labels -> Tuple
  @native args -> Tuple
}
