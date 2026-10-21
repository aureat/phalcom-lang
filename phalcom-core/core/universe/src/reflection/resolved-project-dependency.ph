@!documentation("Resolved dependency inside an active development project.")

from .package_requirement import PackageRequirement
from .package_info import PackageInfo
from .package_object import Package
from .uri import Uri

native class ResolvedProjectDependency is Object {
  native alias -> Symbol
  native requirement -> Option<PackageRequirement>
  native packageInfo -> PackageInfo
  native rootPackage -> Package
  native origin -> Uri
}

export ResolvedProjectDependency
