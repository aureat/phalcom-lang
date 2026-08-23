@!documentation("Module runtime dependency reference.")
@native
class ModuleDependency is Object {
  @native module -> Module
  @native phase -> Symbol
  @native reason -> String
}
