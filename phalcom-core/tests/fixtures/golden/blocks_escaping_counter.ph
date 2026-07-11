let makeCounter = {
  let count = 0
  { count = count + 1 }
}

let counter = makeCounter.call()
System.print(counter.call())
System.print(counter.call())
System.print(counter.call())
