# Transition 1 migration guide

Declarations changed; call-site `label:` syntax did not.

| Old | New |
|---|---|
| `foo(x)` positional | `foo(_ x)` |
| `foo(label:)` | `foo(label)` |
| `foo(label: local)` | `foo(label local)` |
| `foo=(value)` | `foo=(put value)` |
| `[idx, put:]` | `[_ idx]=(put value)` |
| `[idx, default:, put:]` | `[_ idx, default fallback]=(put value)` |
| `static foo(...)` | `@class` followed by `foo(...)` |
| `static _field` | `@class` followed by `_field` |
| `_helper(...)` as private convention | `@private helper(...)` |
| `size_` internal primitive | `_$size` |
| `__runtimeHook` method | `_$runtimeHook` |

Example:

```phalcom
class Cache {
  @class
  empty() => Cache.new()

  @private
  normalize(_ key) => key.toString

  get(_ key, orElse fallback) { ... }
  [_ key, default fallback]=(put value) { ... }
}

cache.get(key, orElse: { None })
```

`_name` is a source field, `__name` an implementation field, and `_$name` an
implementation selector. Ordinary source cannot declare or access either implementation
namespace. Replace legacy `static`; parser reports a targeted error and does not execute it.
