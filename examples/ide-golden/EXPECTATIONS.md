# IDE Golden Expectations

## Baseline

Open this directory in VS Code. The workspace must converge to `Ready` with no
parser, module, or formal semantic diagnostics. `phalcom check` and compilation
must succeed. Running `ide_golden.main` must print the transcript in
`expectations/runtime.toml` exactly.

## Completion

- `completion.parcel`: instance completion must include `id`, `destination`, and `weight`.
- `completion.units.path`: import-path completion must include `distance` and `weight`, and exclude `internal`.
- `completion.geo.path`: import-path completion must include `point` and `route`, and exclude `internal`.

## Hover and inlays

- `hover.int` must report `Int`.
- `hover.point` must report `Point`.
- `hover.flow.refined` must report the refined `ExpressShipment` receiver context.
- `inlay.value.inferred` may show `: Int`; `inlay.value.explicit` must not duplicate the explicit annotation.
- The same suppression rule applies to `inlay.parameter.explicit`.

## Navigation

- `navigation.parcel.use` -> `definition.parcel` in `src/domain/parcel.ph`.
- `navigation.point.cross_project` -> `navigation.point.definition` in `deps/geo/src/point.ph`.
- `navigation.distance.direct` -> `navigation.distance.definition` in `deps/units/src/distance.ph`.
- `navigation.core.int` -> the selected physical universe/core declaration for `Int`.

## Semantic tokens

Anchors `token.class`, `token.method`, `token.parameter`, and `token.local` must
be covered by the corresponding semantic-token categories.

## Mutations

- `diagnostic.binding_mismatch`: replace `42` at `mutation.binding_mismatch` with `"wrong"`; expect `type.binding.initializer_mismatch`.
- `diagnostic.parser_recovery`: replace `42` at `mutation.parser.expression` with an incomplete expression; expect parser diagnostics and no stale formal diagnostic from the previous revision.
- Every mutation must restore the original source and return the workspace to zero Problems and `Ready`.
