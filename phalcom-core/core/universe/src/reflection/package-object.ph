@!documentation("Package reflection and exposure namespace object.")
from .module import Module
from .package_info import PackageInfo
from .child_module_table import ChildModuleTable

native primitive class Package is Module {
  native package -> Package
  native parentPackage -> Option<Package>
  native rootPackage -> Package
  native packageInfo -> PackageInfo
  native children -> ChildModuleTable
  native isRoot -> Bool
  native __parent__ -> Option<Package>
  native __children__ -> ChildModuleTable
  native __version__ -> String
  native __namespace__ -> Symbol
  native toString -> String
}

export Package
