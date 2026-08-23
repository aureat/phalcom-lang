@!documentation("Validated development project manifest representation.")
@native
class ProjectManifest is Object {
  @native name -> Symbol
  @native namespace -> Symbol
  @native version -> String
  @native authors -> List
  @native description -> String
  @native license -> String
  @native homepage -> Uri
  @native repository -> Uri
  @native source -> Uri
  @native entry -> String
  @native defaultEntry -> String
  @native dependencyDeclarations -> List
  @native dependencies -> List
  @native toString -> String
}
