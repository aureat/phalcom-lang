@!documentation("Project reflection and build boundary object.")

from .project_manifest import ProjectManifest
from .package_object import Package
from .module import Module
from .project_identity import ProjectIdentity
from .resolved_project_dependency import ResolvedProjectDependency

native class Project is Object {
  native name -> Symbol
  native namespace -> Symbol
  native manifest -> ProjectManifest
  native rootPackage -> Package
  native dependencies -> (*: ResolvedProjectDependency)
  native developmentEntry -> Module
  native identity -> ProjectIdentity
  native toString -> String
}
