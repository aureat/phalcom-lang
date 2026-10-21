@!documentation("Durable package artifact metadata.")

from .package_author import PackageAuthor
from .package_identity import PackageIdentity
from .package_requirement import PackageRequirement
from .uri import Uri

native class PackageInfo is Object {
  native name -> Symbol
  native namespace -> Symbol
  native version -> String
  native authors -> (*: PackageAuthor)
  native description -> String
  native license -> String
  native homepage -> Uri
  native repository -> Uri
  native requirements -> (*: PackageRequirement)
  native defaultEntry -> String
  native identity -> PackageIdentity
  native toString -> String
}

export PackageInfo
