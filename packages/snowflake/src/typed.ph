from .id import SnowflakeId
from .layout import SnowflakeLayout
from .node import SnowflakeNode
from .errors import SnowflakeParseError

/// Domain and format are caller-chosen nominal tags. The stored layout, not a
/// global setting, decodes this value's fields.
data TypedSnowflake<Domain, Format> private(_ value: SnowflakeId, _ layout: SnowflakeLayout)

impl<Domain, Format> TypedSnowflake<Domain, Format> {
  /// Explicitly assert the domain and format of a validated 63-bit ID.
  @class
  fromId(_ value: SnowflakeId, _ layout: SnowflakeLayout) -> TypedSnowflake<Domain, Format> {
    TypedSnowflake<Domain, Format>(value, layout)
  }

  @class
  parse(_ text: String, _ layout: SnowflakeLayout) -> Result<TypedSnowflake<Domain, Format>, SnowflakeParseError> {
    match SnowflakeId.parse(text) {
      Result::Ok(value) => Result::Ok(TypedSnowflake<Domain, Format>(value, layout))
      Result::Error(error) => Result::Error(error)
    }
  }

  value -> SnowflakeId { self._value }
  layout -> SnowflakeLayout { self._layout }
  timeMillis -> Int { self._value.timeMillis(using: self._layout) }
  node -> Int { self._value.node(using: self._layout) }
  step -> Int { self._value.step(using: self._layout) }
  toString -> String { self._value.toString }

  ==(_ other: Dynamic) -> Bool {
    if other is not TypedSnowflake<Domain, Format> { return false }
    self._value == other.value and
      self._layout.epochMillis == other.layout.epochMillis and
      self._layout.nodeBits == other.layout.nodeBits and
      self._layout.sequenceBits == other.layout.sequenceBits
  }

  hash -> Int {
    (((self._value.hash * 31 + self._layout.epochMillis.hash) * 31 +
      self._layout.nodeBits.hash) * 31 + self._layout.sequenceBits.hash)
  }
}

/// Captures exactly one runtime layout and returns values with its nominal
/// domain/format tags. Separate configured nodes need not share bit settings.
class TypedSnowflakeNode<Domain, Format> {
  _node: SnowflakeNode
  _layout: SnowflakeLayout

  @constructor
  private create(_ node: SnowflakeNode, _ layout: SnowflakeLayout) {
    self._node = node
    self._layout = layout
  }

  @class
  configured(_ nodeId: Int, _ layout: SnowflakeLayout) -> TypedSnowflakeNode<Domain, Format> {
    TypedSnowflakeNode<Domain, Format>.create(SnowflakeNode.configured(nodeId: nodeId, layout: layout), layout)
  }

  @class
  twitter(_ nodeId: Int) -> TypedSnowflakeNode<Domain, Format> {
    TypedSnowflakeNode<Domain, Format>.configured(nodeId, SnowflakeLayout.twitter)
  }

  nodeId -> Int { self._node.nodeId }
  layout -> SnowflakeLayout { self._layout }

  generate() -> TypedSnowflake<Domain, Format> {
    TypedSnowflake<Domain, Format>.fromId(self._node.generate(), self._layout)
  }

  generateAt(_ unixMillis: Int) -> TypedSnowflake<Domain, Format> {
    TypedSnowflake<Domain, Format>.fromId(self._node.generateAt(unixMillis), self._layout)
  }
}

export TypedSnowflake, TypedSnowflakeNode
