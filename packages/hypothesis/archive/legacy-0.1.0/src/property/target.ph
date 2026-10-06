// Invocation adapters consumed by engine PropertySpec values.

class _InvocationTarget {
  invoke(arguments: List<Any>) -> Any {
    throw Error.new("_InvocationTarget.invoke(_) is abstract")
  }
}

class _MethodTarget is _InvocationTarget {
  @constructor
  new(method: Method, receiver: Any) {
    _method = method
    _receiver = receiver
  }

  invoke(arguments: List<Any>) -> Any {
    return _method.invokeOn(_receiver, arguments)
  }

  method -> Method { _method }
  receiver -> Any { _receiver }
}

class _BlockTarget is _InvocationTarget {
  @constructor
  new(block: Closure) {
    _block = block
  }

  invoke(arguments: List<Any>) -> Any {
    return _block.callWith(arguments)
  }
}
