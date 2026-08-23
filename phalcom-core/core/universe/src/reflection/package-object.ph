@!documentation("Package reflection and exposure namespace object.")
@native
class Package is Module {
  @native package -> Package
  @native parentPackage -> Package
  @native rootPackage -> Package
  @native packageInfo -> PackageInfo
  @native children -> ChildModuleTable
  @native isRoot -> Bool
  @native __parent__ -> Package
  @native __children__ -> ChildModuleTable
  @native __version__ -> String
  @native __namespace__ -> Symbol
  @native toString -> String
}
