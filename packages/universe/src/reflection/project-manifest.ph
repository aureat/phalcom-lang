@!documentation("Validated development project manifest representation.")

from .package_author import PackageAuthor
from .package_requirement import PackageRequirement
from .uri import Uri

native class ProjectManifest is Object {
  native name -> String
  native namespace -> Symbol
  native version -> Option<String>
  native authors -> (*: PackageAuthor)
  native description -> Option<String>
  native license -> Option<String>
  native homepage -> Option<Uri>
  native repository -> Option<Uri>
  native source -> Uri
  native entry -> Option<String>
  native defaultEntry -> Option<String>
  native dependencyDeclarations -> (*: PackageRequirement)
  native dependencies -> (*: PackageRequirement)
  native toString -> String
}

export ProjectManifest
