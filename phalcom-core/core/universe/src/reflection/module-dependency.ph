@!documentation("Module runtime dependency reference.")

from .module import Module

native class ModuleDependency is Object {
  native module -> Module
  native phase -> Symbol
  native reason -> String
}
