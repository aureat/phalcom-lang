/// Semantic-token fixture.
/*@token.class*/
class TokenSample {
  _value: Int = 1

  /*@token.method*/read(_ /*@token.parameter*/fallback: Int) -> Int {
    let /*@token.local*/current = _value

    if current > 0 { 
      current
    } else { 
      fallback
    }
  }
}

export TokenSample
