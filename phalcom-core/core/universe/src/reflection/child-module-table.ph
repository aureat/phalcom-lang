from .module import Module

@!documentation("Exposed child modules table for a Package.")
native class ChildModuleTable is Object {
  native names -> (*: Symbol)
  native size -> Int
  native contains(_ name: Symbol) -> Bool
  native get(_ name: Symbol) -> Option<Module>
}
