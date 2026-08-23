@!documentation("Module reflection and execution boundary object.")
@native
class Module is Object {
  @class @native new() -> Module
  @native doesNotUnderstand(_ message: Message) -> Dynamic
  @native name -> Symbol
  @native namespace -> Symbol
  @native package -> Package
  @native rootPackage -> Package
  @native packageInfo -> PackageInfo
  @native exports -> ExportTable
  @native metadata -> Map
  @native dependencies -> List
  @native uri -> Uri
  @native identity -> ModuleIdentity
  @native __exports__ -> ExportTable
  @native __export__(_ name: Symbol) -> Export
  @native __understands__(_ selector: Selector) -> Bool
  @native __metadata__ -> Map
  @native __dependencies__ -> List
  @native __uri__ -> Uri
  @native __name__ -> Symbol
  @native __id__ -> ModuleIdentity
  @native __path__ -> String
  @native toString -> String
}
