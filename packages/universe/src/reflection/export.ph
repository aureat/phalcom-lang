@!documentation("Individual reflected module export descriptor.")

from .export_kind import ExportKind
from .module import Module

native class Export is Object {
  native name -> Symbol
  native kind -> ExportKind
  native module -> Module
  native value -> Dynamic
  native isModule -> Bool
  native isBinding -> Bool
}

export Export
