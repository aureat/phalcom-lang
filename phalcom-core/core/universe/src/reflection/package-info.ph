@!documentation("Durable package artifact metadata.")
@native
class PackageInfo is Object {
  @native name -> Symbol
  @native namespace -> Symbol
  @native version -> String
  @native authors -> List
  @native description -> String
  @native license -> String
  @native homepage -> Uri
  @native repository -> Uri
  @native requirements -> List
  @native defaultEntry -> String
  @native identity -> PackageIdentity
  @native toString -> String
}
