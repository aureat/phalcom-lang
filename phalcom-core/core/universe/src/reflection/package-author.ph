@!documentation("Structured package authorship metadata.")

from .uri import Uri

native class PackageAuthor is Object {
  native name -> String
  native email -> String
  native url -> Uri
}
