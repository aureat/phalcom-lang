@!documentation("Durable package artifact metadata.")

from .package_author import PackageAuthor
from .package_identity import PackageIdentity
from .package_requirement import PackageRequirement
from .uri import Uri

native class PackageInfo is Object {
  native name -> String
  native namespace -> Symbol
  native version -> Option<String>
  native authors -> (*: PackageAuthor)
  native description -> Option<String>
  native license -> Option<String>
  native homepage -> Option<Uri>
  native repository -> Option<Uri>
  native requirements -> (*: PackageRequirement)
  native defaultEntry -> Option<String>
  native identity -> PackageIdentity
  native toString -> String
}

export PackageInfo
