@!documentation("Public export surface reflection table.")
@native
class ExportTable is Object {
  @native names -> List
  @native keys -> List
  @native size -> Int
  @native contains(_ name: Symbol) -> Bool
  @native descriptor(_ name: Symbol) -> Export
  @native get(_ name: Symbol) -> Option<Dynamic>
}
