from .export import Export

@!documentation("Public export surface reflection table.")
native class ExportTable is Object {
  native names -> List<Symbol>
  native keys -> List<Symbol>
  native size -> Int
  native contains(_ name: Symbol) -> Bool
  native descriptor(_ name: Symbol) -> Export
  native get(_ name: Symbol) -> Option<Dynamic>
}
