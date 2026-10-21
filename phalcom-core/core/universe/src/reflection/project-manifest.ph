from .package-author import PackageAuthor
from .package-requirement import PackageRequirement
from .uri import Uri

@!documentation("Validated development project manifest representation.")
native class ProjectManifest is Object {
  native name -> Symbol
  native namespace -> Symbol
  native version -> String
  native authors -> List<PackageAuthor>
  native description -> String
  native license -> String
  native homepage -> Uri
  native repository -> Uri
  native source -> Uri
  native entry -> String
  native defaultEntry -> String
  native dependencyDeclarations -> List<PackageRequirement>
  native dependencies -> List<PackageRequirement>
  native toString -> String
}
