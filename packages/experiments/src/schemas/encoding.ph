import universe.math as math
from universe.functional.chain import Chain

data Encoding {
  @variant Continuous
  @variant PositiveLog
  @variant BoundedLogit(_ lower: Float, _ upper: Float)
  @variant Categorical(_ labels: Chain<String>)
}
data CodecError(_ message: String)

/// A reversible numerical representation chosen explicitly by the caller.
/// Successful round trips are laws of the supplied functions. One numerical
/// coordinate is used here; categorical coordinates are discrete, not gradients.
data Codec<T> private(
  _ encoding: Encoding,
  _ encodeValue: (T) -> Result<Float, CodecError>,
  _ decodeValue: (Float) -> Result<T, CodecError>
)
impl<T> Codec<T> {
  type Value = T
  @class new(encoding: Encoding, encode: (T) -> Result<Float, CodecError>, decode: (Float) -> Result<T, CodecError>) -> Codec<T> {
    Codec<T>(encoding, encode, decode)
  }
  encoding -> Encoding { self.encoding }
  encode(_ value: T) -> Result<Float, CodecError> { self.encodeValue.call(value) }
  decode(_ value: Float) -> Result<T, CodecError> {
    if not value.isFinite { return Result::Error(CodecError("Coordinate must be finite")) }
    self.decodeValue.call(value)
  }
}

class Codecs {
  @class continuous -> Codec<Float> {
    Codec<Float>.new(encoding: Encoding::Continuous,
      encode: |value| {
        if not value.isFinite { return Result::Error(CodecError("Continuous parameter must be finite")) }
        Result::Ok(value)
      }, decode: |value| { Result::Ok(value) })
  }
  @class positive -> Codec<Float> {
    Codec<Float>.new(encoding: Encoding::PositiveLog,
      encode: |value| {
        if not value.isFinite or value <= 0.0 { return Result::Error(CodecError("Positive parameter must be finite and greater than zero")) }
        Result::Ok(math.ln(value))
      }, decode: |value| {
        let decoded = math.exp(value)
        if not decoded.isFinite or decoded <= 0.0 { return Result::Error(CodecError("Positive transform overflowed or underflowed")) }
        Result::Ok(decoded)
      })
  }
  @class bounded(from lower: Float, to upper: Float) -> Codec<Float> {
    if not lower.isFinite or not upper.isFinite or lower >= upper or not (upper - lower).isFinite {
      ArgumentError.new("Bounded codec requires finite ordered endpoints and width").raise()
    }
    Codec<Float>.new(encoding: Encoding::BoundedLogit(lower, upper),
      encode: |value| {
        if not value.isFinite or value <= lower or value >= upper { return Result::Error(CodecError("Bounded parameter must lie strictly between endpoints")) }
        let encoded = math.ln(value - lower) - math.ln(upper - value)
        if not encoded.isFinite { return Result::Error(CodecError("Bounded coordinate is outside numerical range")) }
        Result::Ok(encoded)
      }, decode: |value| {
        let fraction = if value >= 0.0 { 1.0 / (1.0 + math.exp(-value)) } else {
          let e = math.exp(value)
          e / (1.0 + e)
        }
        let decoded = lower + (upper - lower) * fraction
        if decoded <= lower or decoded >= upper { return Result::Error(CodecError("Bounded transform rounded to an excluded endpoint")) }
        Result::Ok(decoded)
      })
  }
  /// Stable label order is part of the codec, never inferred from hash iteration.
  @class categorical(_ labels: Chain<String>) -> Codec<String> {
    if labels.isEmpty { ArgumentError.new("Categorical codec needs labels").raise() }
    let mut remaining = labels
    while not remaining.isEmpty {
      let label = remaining.first.unwrap
      if not remaining.tail.filter(|other| { other == label }).isEmpty {
        ArgumentError.new("Categorical codec labels must be unique").raise()
      }
      remaining = remaining.tail
    }
    Codec<String>.new(encoding: Encoding::Categorical(labels),
      encode: |value| {
        let mut cursor = labels
        let mut index = 0
        while not cursor.isEmpty {
          if cursor.first.unwrap == value { return Result::Ok(Float.new(index)) }
          cursor = cursor.tail
          index = index + 1
        }
        Result::Error(CodecError("Unknown category"))
      }, decode: |value| {
        if value < 0.0 or value >= Float.new(labels.size) { return Result::Error(CodecError("Category index outside labels")) }
        let index = value.truncated
        if Float.new(index) != value { return Result::Error(CodecError("Category index must be integral")) }
        Result::Ok(labels.at(index).unwrap)
      })
  }
}
export Encoding, CodecError, Codec, Codecs
