"""Regenerate independent constants: Python + mpmath 1.4.1, no production code.

Run with a separately installed mpmath; Cargo tests consume only the TSV.
"""
from pathlib import Path
import mpmath as m

m.mp.dps = 100
rows = []

def put(expression, value, tolerance=3e-12):
    rows.append(f"{expression}\t{float(value):.17g}\t{tolerance}")

def lit(value):
    return repr(float(value))

for a in [1e-300, 1e-8, .1, .5, .99, 1, 1.01, 1.5, 2, 5, 16, 100, 10000, 1e6]:
    put(f"special.Gamma.log({lit(a)})", m.loggamma(a))
for a, b in [(.1,.2), (.5,.5), (1,1), (2,3), (10,20), (10000,10000), (1e6,1e6), (1e-300,2), (1,1e6)]:
    put(f"special.Beta.log({lit(a)}, {lit(b)})", m.log(m.beta(a,b)))
for a, x in [(1e-300,.5), (1e-8,.1), (.1,1e-10), (.1,.01), (.5,.1), (.5,1), (.5,1000), (1,1e-300), (1,1), (1,1000), (2,3), (5,2), (5,10), (100,100), (10000,10000), (10000,9500), (10000,10500)]:
    p = m.gammainc(a, 0, x, regularized=True)
    q = m.gammainc(a, x, m.inf, regularized=True)
    for name, value in [("p",p), ("q",q), ("logP",m.log(p)), ("logQ",m.log(q))]:
        put(f"special.Gamma.{name}({lit(a)}, {lit(x)})", value, 3e-10 if a >= 10000 else 3e-12)
for a, b, x in [(.1,.2,1e-20), (.5,.5,.1), (1,1,.25), (2,3,.1), (2,3,.9), (5,10,.3), (100,100,.5), (100,200,.2), (1000,1000,.5), (1000,2,1e-3), (2,1000,.999), (1e-8,.1,.9)]:
    xx = m.mpf(x)
    p = m.betainc(a, b, 0, xx, regularized=True)
    q = m.betainc(b, a, 0, 1-xx, regularized=True)
    for name, value in [("regularized",p), ("complement",q), ("logRegularized",m.log(p)), ("logComplement",m.log(q))]:
        put(f"special.Beta.{name}({lit(x)}, {lit(a)}, {lit(b)})", value, 3e-10 if max(a,b) >= 1000 else 3e-12)
for x in [-30,-3,-.1,-1e-300,0,1e-300,.1,1,5,20,27,27.1,27.2,30]:
    for name, value in [("erf",m.erf(x)), ("erfc",m.erfc(x)), ("logErfc",m.log1p(-m.erf(x)) if abs(x) <= .5 else m.log(m.erfc(x)))]:
        put(f"special.{name}({lit(x)})", value)
for p in [1e-300,1e-20,.001,.025,.25,.5,.75,.975,.999,1-1e-12]:
    if p < 1e-90:
        value = -m.findroot(lambda z: m.log(m.erfc(z/m.sqrt(2))/2)-m.log(p), (30,40))
    else:
        value = -m.sqrt(2)*m.erfinv(1-2*m.mpf(p))
    put(f"special.Normal.quantile({lit(p)})", value)
for a, b, x in [(.5,.5,1e-20), (.5,.5,.1), (2,3,.1), (2,5,.9), (100,100,.45)]:
    p = float(m.betainc(a,b,0,x,regularized=True))
    put(f"special.Beta.inverse({lit(p)}, {lit(a)}, {lit(b)})", x, 3e-10)
for p in [1e-300, .1, .5, .99, 1-1e-12]:
    put(f"special.Gamma.inverseP(1.0, {lit(p)})", -m.log1p(-m.mpf(p)), 3e-12)
for a, x in [(.5,.1), (2,3), (10,10), (100,90)]:
    p = float(m.gammainc(a,0,x,regularized=True))
    put(f"special.Gamma.inverseP({lit(a)}, {lit(p)})", x, 3e-10)
for n in [0, 1, 10, 31, 32, 10000, 9007199254740991]:
    put(f"special.logFactorial({n})", m.loggamma(n+1))
for n, k in [(0,0), (5,6), (100,1), (100,31), (100,32), (10000,5000), (9007199254740991,1), (9007199254740991,31), (9007199254740991,32), (9007199254740991,4503599627370495)]:
    put(f"special.logChoose({n}, {k})", m.log(m.binomial(n,k)))
for expression, value in [
    ("special.Gamma.p(2.0, 0.0)", 0),
    ("special.Gamma.q(2.0, 0.0)", 1),
    ("special.Gamma.logP(2.0, 0.0)", -m.inf),
    ("special.Gamma.logQ(2.0, 1.0 / 0.0)", -m.inf),
    ("special.Beta.regularized(0.0, 2.0, 3.0)", 0),
    ("special.Beta.complement(1.0, 2.0, 3.0)", 0),
    ("special.Normal.quantile(0.0)", -m.inf),
    ("special.Normal.quantile(1.0)", m.inf),
    ("special.Gamma.inverseP(2.0, 1.0)", m.inf),
    ("special.Beta.inverse(0.0, 2.0, 3.0)", 0),
    ("special.Beta.inverse(1.0, 2.0, 3.0)", 1),
]:
    put(expression, value)
path = Path(__file__).resolve().parents[1] / "fixtures/math_special_reference.tsv"
path.write_text(f"# mpmath {m.__version__}; 100 decimal digits; values rounded once to binary64.\n" + "\n".join(rows) + "\n")
print(f"{len(rows)} independent reference cases")
