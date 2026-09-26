@!documentation("Reified message invocation object.")
native class Message is Object {
  native selector -> Symbol
  native name -> String
  native labels -> List
  /// Runtime message arguments retain their call-site-selected shape.
  native args -> Dynamic
}

export Message
