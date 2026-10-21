@!documentation("Module reflection and execution boundary object.")

from .message import Message
from .package_info import PackageInfo
from .uri import Uri
from .module_identity import ModuleIdentity
from .module_dependency import ModuleDependency
from .export_table import ExportTable
from .package_object import Package

native primitive class Module is Object {
  @class
  native new() -> Module
  native doesNotUnderstand(_ message: Message) -> Dynamic
  native name -> Symbol
  native namespace -> Symbol
  native package -> Option<Package>
  native rootPackage -> Option<Package>
  native packageInfo -> PackageInfo
  native exports -> ExportTable
  native metadata -> Map
  native dependencies -> (*: ModuleDependency)
  native uri -> Uri
  native identity -> ModuleIdentity
  native __exports__ -> ExportTable
  native __export__(_ name: Symbol) -> Option<Dynamic>
  native __understands__(_ selector: Selector) -> Bool
  native __metadata__ -> Map
  native __dependencies__ -> (*: ModuleDependency)
  native __uri__ -> Uri
  native __name__ -> Symbol
  native __id__ -> ModuleIdentity
  native __path__ -> String
  native toString -> String
}

export Module
