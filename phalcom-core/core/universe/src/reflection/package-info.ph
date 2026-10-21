from .package-author import PackageAuthor
from .package-identity import PackageIdentity
from .uri import Uri

@!documentation("Durable package artifact metadata.")
native class PackageInfo is Object {
  native name -> Symbol
  native namespace -> Symbol
  native version -> String
  native authors -> Tuple<PackageAuthor>
  native description -> String
  native license -> String
  native homepage -> Uri
  native repository -> Uri
  native requirements -> List
  native defaultEntry -> String
  native identity -> PackageIdentity
  native toString -> String
}
