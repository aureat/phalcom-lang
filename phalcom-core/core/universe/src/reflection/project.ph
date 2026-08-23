@!documentation("Project reflection and build boundary object.")
@native
class Project is Object {
  @native name -> Symbol
  @native namespace -> Symbol
  @native manifest -> ProjectManifest
  @native rootPackage -> Package
  @native dependencies -> List
  @native developmentEntry -> Module
  @native identity -> ProjectIdentity
  @native toString -> String
}
