@!documentation("Public export surface reflection table.")

from .export import Export

native class ExportTable is Object {
  native names -> (*: Symbol)
  native keys -> (*: Symbol)
  native size -> Int
  native contains(_ name: Symbol) -> Bool
  native descriptor(_ name: Symbol) -> Export
  native get(_ name: Symbol) -> Option<Dynamic>
}
