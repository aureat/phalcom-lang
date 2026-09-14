# Phalcom Semantic Inference / Proof / Mismatch Integration Test Matrix

These should be Rust integration tests using non-trivial inline Phalcom programs. Each test should verify the inferred fact itself and, where applicable, the proof or diagnostic produced from it.

1. **Constructor return propagates through a class factory**

   ```phalcom
   class CellNum {
     @constructor new() {}

     @class
     of() {
       CellNum.new()
     }
   }

   const x = CellNum.of()
   ```

   Assert:

  * `CellNum.new()` → `CellNum`
  * inferred return of `CellNum.of()` → `CellNum`
  * inferred type of `x` → `CellNum`
  * no diagnostics

2. **Wrong annotation cannot override constructor/factory inference**

   ```phalcom
   const x: Int = CellNum.of()
   ```

   Assert:

  * initializer actual type → `CellNum`
  * declared type → `Int`
  * `BindingInitializerMismatch`
  * diagnostic expected = `Int`
  * diagnostic actual = `CellNum`
  * semantic type retained for the initializer is still `CellNum`

3. **Correct supertype annotation accepts inferred subtype**

   ```phalcom
   class Number {}
   class CellNum is Number {
     @constructor new() {}
   }

   const x: Number = CellNum.new()
   ```

   Assert:

  * initializer → `CellNum`
  * proof succeeds: `CellNum <: Number`
  * annotation accepted
  * no mismatch diagnostic
  * do not rewrite the initializer itself to `Number`

4. **Unrelated nominal annotation is rejected**

   ```phalcom
   class User {}
   class CellNum {
     @constructor new() {}
   }

   const x: User = CellNum.new()
   ```

   Assert:

  * actual → `CellNum`
  * expected → `User`
  * subtype proof fails
  * exactly one binding mismatch diagnostic

5. **Multi-hop factory inference**

   ```phalcom
   class Value {
     @constructor new() {}

     @class make() {
       Value.new()
     }

     @class of() {
       Value.make()
     }

     @class default() {
       Value.of()
     }
   }

   const x = Value.default()
   ```

   Assert every hop:

  * `new()` → `Value`
  * `make()` → `Value`
  * `of()` → `Value`
  * `default()` → `Value`
  * `x` → `Value`

6. **Explicit wrong method return annotation conflicts with inferred body**

   ```phalcom
   class Cell {
     @constructor new() {}
   }

   class Factory {
     @class
     make() -> Int {
       Cell.new()
     }
   }
   ```

   Assert:

  * body expression → `Cell`
  * declared return → `Int`
  * `ReturnMismatch`
  * actual = `Cell`
  * expected = `Int`

7. **Explicit supertype method return accepts subtype body**

   ```phalcom
   class Animal {}
   class Dog is Animal {
     @constructor new() {}
   }

   class Factory {
     @class
     make() -> Animal {
       Dog.new()
     }
   }
   ```

   Assert:

  * body → `Dog`
  * proof `Dog <: Animal` succeeds
  * method return contract accepted
  * no error

8. **Branch join infers common supertype**

   ```phalcom
   class Animal {}
   class Dog is Animal {
     @constructor new() {}
   }
   class Cat is Animal {
     @constructor new() {}
   }

   choose(flag: Bool) {
     if flag {
       Dog.new()
     } else {
       Cat.new()
     }
   }

   const x = choose(true)
   ```

   Assert:

  * branch 1 → `Dog`
  * branch 2 → `Cat`
  * join → `Animal` if nominal LUB inference is supported
  * `x` → `Animal`

9. **Branch mismatch against narrower annotation**

   ```phalcom
   const x: Dog =
     if condition {
       Dog.new()
     } else {
       Cat.new()
     }
   ```

   Assert:

  * inferred joined type is not provably `Dog`
  * annotation proof fails
  * mismatch diagnostic
  * diagnostic should retain branch-derived actual type/provenance

10. **Inherited constructor specializes `Self`, ordinary class method does not**

    ```phalcom
    class Base {
      @constructor new() {}

      @class
      ordinary() -> Base {
        Base.new()
      }
    }

    class Derived is Base {}

    const a = Derived.new()
    const b = Derived.ordinary()
    ```

    Assert:

  * `a` → `Derived`
  * `b` → `Base`
  * constructor `Self` specialization occurs
  * ordinary explicit return is not rewritten to `Derived`

11. **Inherited specialized constructor checked against annotation**

    ```phalcom
    class Base {
      @constructor new() {}
    }

    class Derived is Base {}

    const good: Base = Derived.new()
    const bad: String = Derived.new()
    ```

    Assert:

  * both initializers infer `Derived`
  * `Derived <: Base` proved
  * `good` valid
  * `bad` produces expected `String`, actual `Derived`

12. **Field default inference and mismatch**

    ```phalcom
    class Cell {
      @constructor new() {}
    }

    class Holder {
      _cell: Cell = Cell.new()
      _wrong: Int = Cell.new()
    }
    ```

    Assert:

  * both initializer expressions → `Cell`
  * `_cell` valid
  * `_wrong` → `FieldMismatch`
  * expected `Int`, actual `Cell`

13. **Assignment checks new value against established binding type**

    ```phalcom
    let x: Int = 1
    x = 2
    x = "invalid"
    ```

    Assert:

  * initial value → `Int`
  * second assignment provably compatible
  * third assignment → `String`
  * assignment mismatch diagnostic only for the third write
  * prior valid semantic state remains available

14. **Parameter types propagate through method body and return inference**

    ```phalcom
    identity(x: CellNum) {
      x
    }

    const result = identity(CellNum.new())
    ```

    Assert:

  * parameter binding → `CellNum`
  * body expression → `CellNum`
  * inferred method return → `CellNum`
  * call result → `CellNum`
  * `result` → `CellNum`

15. **Argument mismatch proves call invalid without destroying return knowledge**

    ```phalcom
    class Factory {
      @class
      fromInt(value: Int) -> CellNum {
        CellNum.new()
      }
    }

    const x = Factory.fromInt("wrong")
    ```

    Assert:

  * argument actual → `String`
  * parameter expected → `Int`
  * argument mismatch diagnostic
  * callable still resolves to `Factory.fromInt`
  * declared return remains known as `CellNum`
  * analyzer does not collapse the entire call to unknown merely because one argument is invalid

16. **Nested expression inference catches mismatch several levels outward**

    ```phalcom
    class Box {
      @constructor new(value: CellNum) {}
    }

    make() {
      CellNum.new()
    }

    const x: String = Box.new(make())
    ```

    Assert:

  * `make()` → `CellNum`
  * argument to `Box.new` proved compatible with `CellNum`
  * `Box.new(...)` → `Box`
  * outer annotation expects `String`
  * final mismatch expected `String`, actual `Box`

17. **Imported factory return type propagates across module boundary**

    ```phalcom
    // values.ph
    export CellNum

    class CellNum {
      @constructor new() {}

      @class
      of() {
        CellNum.new()
      }
    }

    // main.ph
    from .values import CellNum

    const x = CellNum.of()
    ```

    Assert:

  * imported `CellNum` resolves to the original declaration
  * class-side `of` resolves across the module boundary
  * `of()` → original `CellNum`
  * `x` → same nominal `CellNum` identity, not a reconstructed/local fake type

18. **Re-exported declaration retains semantic identity and inference**

    ```phalcom
    // value.ph
    class Cell {
      @constructor new() {}
    }
    export Cell

    // api.ph
    from .value import Cell
    export Cell

    // main.ph
    from .api import Cell
    const x: Int = Cell.new()
    ```

    Assert:

  * imported/re-exported `Cell` points to the original declaration
  * constructor → original `Cell`
  * mismatch expected `Int`, actual original `Cell`
  * re-export does not create a second nominal type identity

19. **Wrong type annotation must not poison downstream member inference**

    ```phalcom
    class CellNum {
      @constructor new() {}

      cellOnly() -> Int {
        1
      }
    }

    const x: String = CellNum.new()

    const y = x.cellOnly()
    ```

    Assert:

  * `x` initializer actual → `CellNum`
  * mismatch against `String`
  * receiver of `x.cellOnly()` remains semantically `CellNum`
  * member resolves successfully
  * `y` → `Int`
  * the programmer's wrong annotation must not cause a false “method not found on String”

20. **Conflicting evidence produces precise proof failure rather than cascading errors**

    ```phalcom
    class Animal {}
    class Dog is Animal {
      @constructor new() {}

      bark() -> String {
        "woof"
      }
    }

    build() {
      Dog.new()
    }

    const animal: Animal = build()
    const wrong: Int = animal
    const sound = animal.bark()
    ```

    Assert:

  * `build()` → `Dog`
  * initializer of `animal` → `Dog`
  * `Dog <: Animal` proves the declaration valid
  * `wrong` tests the statically exposed/declared `Animal` relationship appropriately
  * member lookup on `animal` obeys the language's chosen distinction between actual inferred type and declared/static boundary
  * diagnostics are intentional and non-cascading
  * no secondary nonsense errors caused by losing the original `Dog` provenance

The highest-value tests here are #2, #5, #10, #15, #18, #19, and #20 because they force multiple semantic mechanisms to compose instead of merely proving that literal typing works.

A useful rule for the suite is: every mismatch test should assert at least four things—`actual type`, `expected type`, `proof result`, and `diagnostic code`. Every inference test should assert the intermediate expression/callable types as well as the final binding type. That prevents a test from passing because two incorrect inference steps happened to cancel each other out.
