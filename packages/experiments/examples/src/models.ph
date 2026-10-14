from universe.functional import Lens, Chain
from experiments.schemas import ModelSchema, Parameter, Codecs

data NoiseModel(rate: Float, noise: Float, family: String)
class Models {
  @class schema -> ModelSchema<NoiseModel, 3> {
    let rate = Parameter<NoiseModel, Float>.new("rate",
      focus: Lens<NoiseModel, Float>.new(get: |model| { model.rate }, set: |model, value| {
        NoiseModel(rate: value, noise: model.noise, family: model.family)
      }), codec: Codecs.positive)
    let noise = Parameter<NoiseModel, Float>.new("noise",
      focus: Lens<NoiseModel, Float>.new(get: |model| { model.noise }, set: |model, value| {
        NoiseModel(rate: model.rate, noise: value, family: model.family)
      }), codec: Codecs.bounded(from: 0.0, to: 1.0))
    let labels = Chain<String>::Empty.prepend("poisson").prepend("normal")
    let family = Parameter<NoiseModel, String>.new("family",
      focus: Lens<NoiseModel, String>.new(get: |model| { model.family }, set: |model, value| {
        NoiseModel(rate: model.rate, noise: model.noise, family: value)
      }), codec: Codecs.categorical(labels))
    ModelSchema<NoiseModel, 0>.empty().append<Float, 1>(rate).append<Float, 2>(noise).append<String, 3>(family)
  }
  @class run() -> Unit {
    let model = NoiseModel(rate: 2.0, noise: 0.25, family: "poisson")
    match Models.schema.encode(model) {
      Result::Ok(coordinates) => {
        System.print(coordinates.values)
        System.print(Models.schema.decode(coordinates, into: model))
      }
      Result::Error(error) => System.print(error)
    }
    ()
  }
}
export NoiseModel, Models
