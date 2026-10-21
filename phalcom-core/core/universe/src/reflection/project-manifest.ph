@!documentation("Validated development project manifest representation.")

from .package_author import PackageAuthor
from .package_requirement import PackageRequirement
from .uri import Uri

native class ProjectManifest is Object {
  native name -> Symbol
  native namespace -> Symbol
  native version -> String
  native authors -> (*: PackageAuthor)
  native description -> String
  native license -> String
  native homepage -> Uri
  native repository -> Uri
  native source -> Uri
  native entry -> String
  native defaultEntry -> String
  native dependencyDeclarations -> (*: PackageRequirement)
  native dependencies -> (*: PackageRequirement)
  native toString -> String
}

export ProjectManifest
