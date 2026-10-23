@!documentation("Opaque logical module identity.")

from .uri import Uri

native class ModuleIdentity is Object {
  native uri -> Uri
  native toString -> String
}

export ModuleIdentity
