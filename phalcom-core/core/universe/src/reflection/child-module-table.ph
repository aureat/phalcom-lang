@!documentation("Exposed child modules table for a Package.")
@native
class ChildModuleTable is Object {
  @native names -> List
  @native size -> Int
  @native contains(_ name: Symbol) -> Bool
  @native get(_ name: Symbol) -> Option<Module>
}
