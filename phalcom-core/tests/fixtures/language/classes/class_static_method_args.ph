// area: classes
// spec: classes.md; object-model.md
// status: PASS

class MathUtil {
  @class
  square(_ n) {
    return n * n;
  }
}
System.print(MathUtil.square(6))
