@!documentation("Reified message invocation object.")
native class Message is Object {
  native selector -> Symbol
  native name -> String
  native labels -> List
  native args -> Tuple
}

export Message
