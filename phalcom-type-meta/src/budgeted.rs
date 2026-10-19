//! Current-schema JSON construction. Lexical inspection and unknown-field skipping
//! borrow input; every owned string and typed collection reserves before allocation.
use crate::allocation::{AllocationBudget, AllocationError};

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum DecodeError {
    #[error("{0}")]
    Syntax(&'static str),
    #[error(transparent)]
    Allocation(#[from] AllocationError),
}

#[derive(Clone, Copy)]
pub struct Text<'a>(&'a str);
impl Text<'_> {
    fn chars(&self) -> Chars<'_> {
        Chars { raw: self.0, pos: 0 }
    }
    pub fn equals(&self, other: &str) -> bool {
        self.chars().eq(other.chars())
    }
    fn len(&self) -> usize {
        self.chars().map(char::len_utf8).sum()
    }
}
#[derive(Clone)]
struct Chars<'a> {
    raw: &'a str,
    pos: usize,
}
impl Iterator for Chars<'_> {
    type Item = char;
    fn next(&mut self) -> Option<char> {
        let rest = self.raw.get(self.pos..)?;
        let c = rest.chars().next()?;
        self.pos += c.len_utf8();
        if c != '\\' {
            return Some(c);
        }
        let escape = *self.raw.as_bytes().get(self.pos)?;
        self.pos += 1;
        Some(match escape {
            b'"' => '"',
            b'\\' => '\\',
            b'/' => '/',
            b'b' => '\u{0008}',
            b'f' => '\u{000c}',
            b'n' => '\n',
            b'r' => '\r',
            b't' => '\t',
            b'u' => {
                let h = u16::from_str_radix(self.raw.get(self.pos..self.pos + 4)?, 16).ok()?;
                self.pos += 4;
                let cp = if (0xd800..=0xdbff).contains(&h) {
                    self.pos += 2;
                    let l = u16::from_str_radix(self.raw.get(self.pos..self.pos + 4)?, 16).ok()?;
                    self.pos += 4;
                    0x10000 + ((h as u32 - 0xd800) << 10) + (l as u32 - 0xdc00)
                } else {
                    h as u32
                };
                char::from_u32(cp)?
            }
            _ => return None,
        })
    }
}

#[derive(Clone, Copy)]
pub struct Cursor<'a> {
    input: &'a str,
    pos: usize,
    limit: usize,
}
impl<'a> Cursor<'a> {
    pub fn new(input: &'a str, limit: usize) -> Self {
        Self { input, pos: 0, limit }
    }
    fn space(&mut self) {
        while self.input.as_bytes().get(self.pos).is_some_and(|b| matches!(b, b' ' | b'\n' | b'\r' | b'\t')) {
            self.pos += 1;
        }
    }
    pub fn peek(&mut self) -> Option<u8> {
        self.space();
        self.input.as_bytes().get(self.pos).copied()
    }
    pub fn expect(&mut self, c: u8) -> Result<(), DecodeError> {
        if self.peek() != Some(c) {
            return Err(DecodeError::Syntax("unexpected metadata JSON token"));
        }
        self.pos += 1;
        Ok(())
    }
    pub fn finish(&mut self) -> Result<(), DecodeError> {
        if self.peek().is_some() {
            return Err(DecodeError::Syntax("trailing metadata JSON"));
        }
        Ok(())
    }
    pub fn string_token(&mut self) -> Result<Text<'a>, DecodeError> {
        self.expect(b'"')?;
        let start = self.pos;
        loop {
            let c = *self.input.as_bytes().get(self.pos).ok_or(DecodeError::Syntax("unterminated JSON string"))?;
            match c {
                b'"' => {
                    let end = self.pos;
                    self.pos += 1;
                    return Ok(Text(&self.input[start..end]));
                }
                0..=31 => return Err(DecodeError::Syntax("control byte in JSON string")),
                b'\\' => {
                    self.pos += 1;
                    let e = *self.input.as_bytes().get(self.pos).ok_or(DecodeError::Syntax("unterminated JSON escape"))?;
                    self.pos += 1;
                    match e {
                        b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't' => {}
                        b'u' => {
                            let h = self.hex()?;
                            if (0xd800..=0xdbff).contains(&h) {
                                if self.input.as_bytes().get(self.pos..self.pos + 2) != Some(b"\\u") {
                                    return Err(DecodeError::Syntax("missing low surrogate"));
                                }
                                self.pos += 2;
                                let l = self.hex()?;
                                if !(0xdc00..=0xdfff).contains(&l) {
                                    return Err(DecodeError::Syntax("invalid low surrogate"));
                                }
                            } else if (0xdc00..=0xdfff).contains(&h) {
                                return Err(DecodeError::Syntax("unpaired low surrogate"));
                            }
                        }
                        _ => return Err(DecodeError::Syntax("invalid JSON escape")),
                    }
                }
                _ => {
                    let c = self.input[self.pos..].chars().next().ok_or(DecodeError::Syntax("invalid UTF8"))?;
                    self.pos += c.len_utf8();
                }
            }
        }
    }
    fn hex(&mut self) -> Result<u16, DecodeError> {
        let text = self.input.get(self.pos..self.pos + 4).ok_or(DecodeError::Syntax("short Unicode escape"))?;
        if !text.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(DecodeError::Syntax("invalid Unicode escape"));
        }
        self.pos += 4;
        u16::from_str_radix(text, 16).map_err(|_| DecodeError::Syntax("invalid Unicode escape"))
    }
    pub fn object_key(&mut self, first: &mut bool) -> Result<Option<Text<'a>>, DecodeError> {
        if self.peek() == Some(b'}') {
            self.pos += 1;
            return Ok(None);
        }
        if !*first {
            self.expect(b',')?;
        }
        *first = false;
        let key = self.string_token()?;
        self.expect(b':')?;
        Ok(Some(key))
    }
    fn atom(&mut self) -> Result<&'a str, DecodeError> {
        self.space();
        let start = self.pos;
        while self
            .input
            .as_bytes()
            .get(self.pos)
            .is_some_and(|b| !matches!(b, b',' | b']' | b'}' | b' ' | b'\t' | b'\r' | b'\n'))
        {
            self.pos += 1;
        }
        if start == self.pos {
            return Err(DecodeError::Syntax("missing JSON scalar"));
        }
        Ok(&self.input[start..self.pos])
    }
    pub fn skip_value(&mut self) -> Result<(), DecodeError> {
        self.skip(0)
    }
    fn skip(&mut self, depth: usize) -> Result<(), DecodeError> {
        if depth >= self.limit {
            return Err(DecodeError::Syntax("metadata nesting limit"));
        }
        match self.peek() {
            Some(b'"') => {
                self.string_token()?;
            }
            Some(b'{') => {
                self.pos += 1;
                let mut first = true;
                while self.object_key(&mut first)?.is_some() {
                    self.skip(depth + 1)?;
                }
            }
            Some(b'[') => {
                self.pos += 1;
                if self.peek() != Some(b']') {
                    loop {
                        self.skip(depth + 1)?;
                        if self.peek() != Some(b',') {
                            break;
                        }
                        self.pos += 1;
                    }
                }
                self.expect(b']')?;
            }
            Some(_) => {
                let a = self.atom()?;
                if !matches!(a, "null" | "true" | "false") {
                    validate_number(a)?;
                }
            }
            None => return Err(DecodeError::Syntax("missing JSON value")),
        }
        Ok(())
    }
    fn array_count(&self) -> Result<usize, DecodeError> {
        let mut p = *self;
        p.expect(b'[')?;
        let mut count = 0usize;
        if p.peek() != Some(b']') {
            loop {
                p.skip_value()?;
                count = count.checked_add(1).ok_or(AllocationError::Overflow)?;
                if p.peek() != Some(b',') {
                    break;
                }
                p.pos += 1;
            }
        }
        p.expect(b']')?;
        Ok(count)
    }
}
fn validate_number(s: &str) -> Result<(), DecodeError> {
    let b = s.as_bytes();
    let mut i = usize::from(b.first() == Some(&b'-'));
    match b.get(i) {
        Some(b'0') => i += 1,
        Some(b'1'..=b'9') => {
            i += 1;
            while b.get(i).is_some_and(u8::is_ascii_digit) {
                i += 1;
            }
        }
        _ => return Err(DecodeError::Syntax("invalid JSON number")),
    }
    if b.get(i) == Some(&b'.') {
        i += 1;
        let start = i;
        while b.get(i).is_some_and(u8::is_ascii_digit) {
            i += 1;
        }
        if i == start {
            return Err(DecodeError::Syntax("invalid JSON number"));
        }
    }
    if matches!(b.get(i), Some(b'e' | b'E')) {
        i += 1;
        if matches!(b.get(i), Some(b'+' | b'-')) {
            i += 1;
        }
        let start = i;
        while b.get(i).is_some_and(u8::is_ascii_digit) {
            i += 1;
        }
        if i == start {
            return Err(DecodeError::Syntax("invalid JSON number"));
        }
    }
    if i != b.len() {
        return Err(DecodeError::Syntax("invalid JSON number"));
    }
    Ok(())
}

pub struct Decoder<'a, 'b> {
    pub cursor: Cursor<'a>,
    pub budget: &'b mut AllocationBudget,
    depth: usize,
}
impl<'a, 'b> Decoder<'a, 'b> {
    pub fn new(input: &'a str, budget: &'b mut AllocationBudget, depth: usize) -> Result<Self, DecodeError> {
        let mut inspection = Cursor::new(input, depth);
        inspection.skip_value()?;
        inspection.finish()?;
        Ok(Self {
            cursor: Cursor::new(input, depth),
            budget,
            depth: 0,
        })
    }
    pub fn nested<T>(&mut self, f: impl FnOnce(&mut Self) -> Result<T, DecodeError>) -> Result<T, DecodeError> {
        if self.depth >= self.cursor.limit {
            return Err(DecodeError::Syntax("metadata type nesting limit"));
        }
        self.depth += 1;
        let r = f(self);
        self.depth -= 1;
        r
    }
}
pub trait BudgetDecode: Sized {
    fn decode(json: &mut Decoder<'_, '_>) -> Result<Self, DecodeError>;
}
macro_rules! integer{($($t:ty),*)=>{$(impl BudgetDecode for $t{fn decode(json:&mut Decoder<'_, '_>)->Result<Self,DecodeError>{let text=json.cursor.atom()?;validate_number(text)?;text.parse().map_err(|_|DecodeError::Syntax("metadata integer out of range"))}})*};}
integer!(u8, u16, u32, u64, u128, i32, i64, usize);
impl BudgetDecode for bool {
    fn decode(json: &mut Decoder<'_, '_>) -> Result<Self, DecodeError> {
        match json.cursor.atom()? {
            "true" => Ok(true),
            "false" => Ok(false),
            _ => Err(DecodeError::Syntax("invalid boolean")),
        }
    }
}
impl<T: BudgetDecode> BudgetDecode for Option<T> {
    fn decode(json: &mut Decoder<'_, '_>) -> Result<Self, DecodeError> {
        if json.cursor.peek() == Some(b'n') {
            if json.cursor.atom()? != "null" {
                return Err(DecodeError::Syntax("invalid null"));
            }
            Ok(None)
        } else {
            T::decode(json).map(Some)
        }
    }
}
impl BudgetDecode for String {
    fn decode(json: &mut Decoder<'_, '_>) -> Result<Self, DecodeError> {
        let t = json.cursor.string_token()?;
        let len = t.len();
        json.budget.reserve_array::<u8>(len)?;
        let mut s = String::new();
        s.try_reserve_exact(len).map_err(|_| AllocationError::Failed)?;
        s.extend(t.chars());
        Ok(s)
    }
}
impl BudgetDecode for Box<str> {
    fn decode(json: &mut Decoder<'_, '_>) -> Result<Self, DecodeError> {
        String::decode(json).map(String::into_boxed_str)
    }
}
impl<T: BudgetDecode> BudgetDecode for Box<[T]> {
    fn decode(json: &mut Decoder<'_, '_>) -> Result<Self, DecodeError> {
        json.nested(|json| {
            let count = json.cursor.array_count()?;
            let mut v = json.budget.vector::<T>(count)?;
            json.cursor.expect(b'[')?;
            for i in 0..count {
                if i != 0 {
                    json.cursor.expect(b',')?;
                }
                v.push(T::decode(json)?);
            }
            json.cursor.expect(b']')?;
            Ok(v.into_boxed_slice())
        })
    }
}
impl<T: BudgetDecode> BudgetDecode for Box<T> {
    fn decode(json: &mut Decoder<'_, '_>) -> Result<Self, DecodeError> {
        json.budget.reserve_array::<T>(1)?;
        Box::try_new(T::decode(json)?).map_err(|_| AllocationError::Failed.into())
    }
}
impl<const N: usize> BudgetDecode for [u8; N] {
    fn decode(json: &mut Decoder<'_, '_>) -> Result<Self, DecodeError> {
        let mut a = [0; N];
        json.cursor.expect(b'[')?;
        for (i, v) in a.iter_mut().enumerate() {
            if i != 0 {
                json.cursor.expect(b',')?;
            }
            *v = u8::decode(json)?;
        }
        json.cursor.expect(b']')?;
        Ok(a)
    }
}

impl BudgetDecode for crate::scalar::StaticInt {
    fn decode(json: &mut Decoder<'_, '_>) -> Result<Self, DecodeError> {
        use crate::scalar::StaticInt;
        use num_bigint::{BigInt, Sign};
        // Decimal text is the current-schema representation; legacy numeric
        // values remain accepted without allocating their source spelling.
        let token = if json.cursor.peek() == Some(b'"') {
            json.cursor.string_token()?
        } else {
            Text(json.cursor.atom()?)
        };
        let mut chars = token.chars().peekable();
        let negative = chars.peek() == Some(&'-');
        if negative {
            chars.next();
        }
        let digits = chars.clone().count();
        if digits == 0 || !chars.clone().all(|c| c.is_ascii_digit()) {
            return Err(DecodeError::Syntax("invalid exact integer"));
        }
        let mut small = Some(0u64);
        for c in chars.clone() {
            small = small.and_then(|v| v.checked_mul(10)?.checked_add(c as u64 - '0' as u64));
        }
        if let Some(value) = small {
            if negative && value <= 1u64 << 63 {
                return Ok(StaticInt::Small(value.wrapping_neg() as i64));
            }
            if !negative && value <= i64::MAX as u64 {
                return Ok(StaticInt::Small(value as i64));
            }
        }
        // Each decimal digit requires fewer than four bits. Reserve the full
        // limb destination once; multiply/add below never grows its capacity.
        let capacity = digits.checked_add(7).ok_or(AllocationError::Overflow)? / 8;
        let mut limbs = json.budget.vector::<u32>(capacity)?;
        limbs.push(0);
        for digit in chars {
            let mut carry = digit as u64 - '0' as u64;
            for limb in &mut limbs {
                let v = *limb as u64 * 10 + carry;
                *limb = v as u32;
                carry = v >> 32;
            }
            if carry != 0 {
                if limbs.len() == capacity {
                    return Err(DecodeError::Syntax("exact integer limb bound"));
                }
                limbs.push(carry as u32);
            }
        }
        // num-bigint 0.4.8 BigUint::new consumes u32 limbs on 32-bit targets;
        // on 64-bit targets its ExactSizeIterator creates exactly ceil(n/2)
        // u64 limbs, with zero/one limb retained inline (big_digit.rs::extend).
        #[cfg(target_pointer_width = "64")]
        {
            let native = limbs.len().div_ceil(2);
            if native > 1 {
                json.budget.reserve_array::<u64>(native)?;
            }
        }
        let value = BigInt::new(if negative { Sign::Minus } else { Sign::Plus }, limbs);
        let layout = std::alloc::Layout::new::<[std::sync::atomic::AtomicUsize; 2]>()
            .extend(std::alloc::Layout::new::<BigInt>())
            .map_err(|_| AllocationError::Overflow)?
            .0
            .pad_to_align();
        json.budget.reserve_layout(layout)?;
        Ok(StaticInt::Large(std::sync::Arc::try_new(value).map_err(|_| AllocationError::Failed)?))
    }
}

/// Read the current header without constructing keys or skipped values.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HeaderProbe {
    pub schema_version: u32,
    pub profile: crate::header::MetadataProfile,
    pub identity_scheme: crate::header::ArtifactIdentityScheme,
}

pub fn probe_header(input: &str) -> Result<HeaderProbe, DecodeError> {
    let mut cursor = Cursor::new(input, 256);
    cursor.expect(b'{')?;
    let mut first = true;
    while let Some(key) = cursor.object_key(&mut first)? {
        if key.equals("header") {
            cursor.expect(b'{')?;
            let mut header_first = true;
            let mut schema_version = None;
            let mut profile = None;
            let mut identity_scheme = None;
            while let Some(hkey) = cursor.object_key(&mut header_first)? {
                if hkey.equals("schema_version") {
                    let val = cursor.atom()?;
                    schema_version = Some(val.parse().map_err(|_| DecodeError::Syntax("invalid schema version"))?);
                } else if hkey.equals("profile") {
                    let token = cursor.string_token()?;
                    profile = Some(match () {
                        _ if token.equals("RuntimeMinimal") => crate::header::MetadataProfile::RuntimeMinimal,
                        _ if token.equals("RuntimePublic") => crate::header::MetadataProfile::RuntimePublic,
                        _ if token.equals("ToolingDebug") => crate::header::MetadataProfile::ToolingDebug,
                        _ if token.equals("Proof") => crate::header::MetadataProfile::Proof,
                        _ => return Err(DecodeError::Syntax("unknown profile in header")),
                    });
                } else if hkey.equals("identity_scheme") {
                    let token = cursor.string_token()?;
                    identity_scheme = Some(match () {
                        _ if token.equals("V1Standard") => crate::header::ArtifactIdentityScheme::V1Standard,
                        _ if token.equals("SessionLocal") => crate::header::ArtifactIdentityScheme::SessionLocal,
                        _ => return Err(DecodeError::Syntax("unknown identity scheme in header")),
                    });
                } else {
                    cursor.skip_value()?;
                }
            }
            let schema_version = schema_version.ok_or(DecodeError::Syntax("missing schema version"))?;
            let profile = profile.ok_or(DecodeError::Syntax("missing profile in header"))?;
            let identity_scheme = identity_scheme.ok_or(DecodeError::Syntax("missing identity scheme in header"))?;
            return Ok(HeaderProbe {
                schema_version,
                profile,
                identity_scheme,
            });
        }
        cursor.skip_value()?;
    }
    Err(DecodeError::Syntax("missing metadata header"))
}

pub fn schema_version(input: &str) -> Result<u32, DecodeError> {
    let mut cursor = Cursor::new(input, 256);
    cursor.expect(b'{')?;
    let mut first = true;
    while let Some(key) = cursor.object_key(&mut first)? {
        if key.equals("header") {
            cursor.expect(b'{')?;
            let mut first = true;
            while let Some(key) = cursor.object_key(&mut first)? {
                if key.equals("schema_version") {
                    let value = cursor.atom()?;
                    return value.parse().map_err(|_| DecodeError::Syntax("invalid schema version"));
                }
                cursor.skip_value()?;
            }
            return Err(DecodeError::Syntax("missing schema version"));
        }
        cursor.skip_value()?;
    }
    Err(DecodeError::Syntax("missing metadata header"))
}
