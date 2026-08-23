@!documentation("Resolved dependency inside an active development project.")
@native
class ResolvedProjectDependency is Object {
  @native alias -> Symbol
  @native requirement -> PackageRequirement
  @native packageInfo -> PackageInfo
  @native rootPackage -> Package
  @native origin -> Uri
}
