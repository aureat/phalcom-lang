@!documentation("Project reflection and build boundary object.")

from .project-manifest import ProjectManifest
from .package-object import Package
from .module import Module
from .project-identity import ProjectIdentity
from .resolved-project-dependency import ResolvedProjectDependency

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
