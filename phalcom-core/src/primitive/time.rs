//! Native observations owned by source-declared time capabilities.

use crate::error::PhResult;
use crate::error::RuntimeError;
use crate::heap::{InstanceObject, Object};
use crate::primitive::expect_string;
use crate::primitive::int::expect_int_big;
use crate::value::{Value, normalize_bigint};
use crate::vm::VM;
use num_bigint::BigInt;
use num_traits::{Signed, ToPrimitive};
use phalcom_native_macros::primitive;
use std::time::{SystemTime, UNIX_EPOCH};

/// Signature: `SystemClock.class::monotonicNanoseconds` — exact monotonic
/// nanoseconds elapsed since this VM was created.
#[primitive(SystemClock, "monotonicNanoseconds", params = [], returns = Int, types = "() -> Int", side = class, visibility = internal, effects = [nondeterminism]
)]
pub fn system_clock_monotonic_nanoseconds(vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let nanos = BigInt::from(vm.start_time.elapsed().as_nanos());
    Ok(normalize_bigint(nanos, &mut vm.heap))
}

/// Signature: `SystemWallClock.class::unixNanoseconds` — exact signed Unix/POSIX
/// nanoseconds observed from the host wall clock.
#[primitive(
    SystemWallClock,
    "unixNanoseconds",
    params = [],
    returns = Int,
    types = "() -> Int",
    side = class,
    visibility = internal,
    effects = [nondeterminism]
)]
pub fn system_wall_clock_unix_nanoseconds(vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let nanos = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => BigInt::from(duration.as_secs()) * 1_000_000_000u32 + duration.subsec_nanos(),
        Err(error) => -(BigInt::from(error.duration().as_secs()) * 1_000_000_000u32 + error.duration().subsec_nanos()),
    };
    Ok(normalize_bigint(nanos, &mut vm.heap))
}

fn timezone_class(vm: &mut VM) -> crate::heap::ClassId {
    let symbol = vm.interner.intern("TimeZone");
    let binding = vm.canonical_universe_binding(symbol).expect("TimeZone source binding must be bootstrapped");
    let value = vm
        .heap
        .module(binding.module)
        .get_by_slot(binding.slot as usize)
        .expect("TimeZone source slot must be initialized");
    let object = value.as_obj().expect("TimeZone source binding must be an object");
    matches!(vm.heap.get(object), Object::Class(_))
        .then_some(object)
        .expect("TimeZone source binding must be a class")
}

fn make_timezone(vm: &mut VM, identity: i64) -> Value {
    let class = timezone_class(vm);
    let mut instance = InstanceObject::new(class, vm.heap.class(class).field_count.max(1));
    instance.slots[0] = Value::int(identity);
    Value::obj(vm.heap.alloc(Object::Instance(instance)))
}

pub(crate) fn timestamp_from_value(vm: &mut VM, value: &Value) -> PhResult<jiff::Timestamp> {
    let nanos = expect_int_big(value, vm)?;
    let billion = BigInt::from(1_000_000_000u32);
    let mut seconds = &nanos / &billion;
    let mut remainder = &nanos % &billion;
    if remainder.is_negative() {
        seconds -= 1;
        remainder += &billion;
    }
    let seconds = seconds
        .to_i64()
        .ok_or_else(|| RuntimeError::ArgumentError("timestamp is outside the timezone backend range".into()))?;
    let nanos = remainder
        .to_u32()
        .ok_or_else(|| RuntimeError::ArgumentError("timestamp nanoseconds are outside the backend range".into()))?;
    jiff::Timestamp::new(seconds, nanos as i32)
        .map_err(|error| RuntimeError::ArgumentError(format!("timestamp is outside the timezone backend range: {error}")).into())
}

fn timezone_identity(vm: &VM, receiver: &Value) -> PhResult<i64> {
    let object = receiver.as_obj().ok_or(RuntimeError::Type {
        expected: "TimeZone",
        found: receiver.type_name(),
    })?;
    vm.heap
        .as_instance(object)
        .and_then(|instance| instance.slots.first().copied())
        .and_then(|value| value.as_int())
        .ok_or_else(|| {
            RuntimeError::Type {
                expected: "TimeZone",
                found: "object",
            }
            .into()
        })
}

#[primitive(TimeZone, "utcNative", params = [], returns = TimeZone, types = "() -> TimeZone", side = class, visibility = internal
)]
pub fn timezone_utc_native(vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let zone = vm.timezone_adapter.utc();
    let id = vm.timezone_adapter.register_handle(zone);
    Ok(make_timezone(vm, id))
}

#[phalcom_native_macros::primitive(TimeZone, "fixedNative(_)", params = [Int], returns = TimeZone, types = "(Int) -> TimeZone", side = class, visibility = internal
)]
pub fn timezone_fixed_native(vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let seconds = expect_int_big(&args[0], vm)?.to_i64().ok_or(RuntimeError::Type {
        expected: "bounded Int",
        found: "Int",
    })?;
    if !(-93_599..=93_599).contains(&seconds) {
        return Err(RuntimeError::ArgumentError("timezone offset is outside the P1 UtcOffset bound".into()).into());
    }
    let zone = vm.timezone_adapter.fixed(seconds as i32);
    let id = vm.timezone_adapter.register_handle(zone);
    Ok(make_timezone(vm, id))
}

#[phalcom_native_macros::primitive(TimeZone, "namedNative(_)", params = [String], returns = TimeZone, types = "(String) -> TimeZone", side = class, visibility = internal
)]
pub fn timezone_named_native(vm: &mut VM, _receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let identifier = expect_string(vm, &args[0])?;
    let zone = vm
        .timezone_adapter
        .named(&identifier)
        .map_err(|error| RuntimeError::ArgumentError(error.to_string()))?;
    let identity = vm.timezone_adapter.register_handle(zone);
    Ok(make_timezone(vm, identity))
}

#[phalcom_native_macros::primitive(TimeZone, "systemNative", params = [], returns = TimeZone, types = "() -> TimeZone", side = class, visibility = internal
)]
pub fn timezone_system_native(vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let zone = vm.timezone_adapter.system().map_err(|error| RuntimeError::ArgumentError(error.to_string()))?;
    let identity = vm.timezone_adapter.register_handle(zone);
    Ok(make_timezone(vm, identity))
}

#[phalcom_native_macros::primitive(TimeZone, "offsetSecondsNative(_)", params = [Int], returns = Int, types = "(Int) -> Int", visibility = internal
)]
pub fn timezone_offset_seconds_native(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let identity = timezone_identity(vm, receiver)?;
    let timestamp = timestamp_from_value(vm, &args[0])?;
    let zone = vm
        .timezone_adapter
        .handle(identity)
        .ok_or(RuntimeError::ArgumentError("stale timezone handle".into()))?;
    Ok(Value::int(vm.timezone_adapter.offset_at(zone, timestamp).into()))
}

#[phalcom_native_macros::primitive(TimeZone, "identityNative", params = [], returns = Int, types = "() -> Int", visibility = internal)]
pub fn timezone_identity_native(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    Ok(Value::int(timezone_identity(vm, receiver)?))
}

#[phalcom_native_macros::primitive(TimeZone, "displayNative", params = [], returns = String, types = "() -> String", visibility = internal)]
pub fn timezone_display_native(vm: &mut VM, receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    let identity = timezone_identity(vm, receiver)?;
    let display = vm
        .timezone_adapter
        .display_name(identity)
        .ok_or(RuntimeError::ArgumentError("stale timezone handle".into()))?;
    Ok(vm.alloc_string_value(display))
}

#[phalcom_native_macros::primitive(TimeZone, "localNative(_)", params = [Int], returns = List, types = "(Int) -> List", visibility = internal)]
pub fn timezone_local_native(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let identity = timezone_identity(vm, receiver)?;
    let timestamp = timestamp_from_value(vm, &args[0])?;
    let zone = vm
        .timezone_adapter
        .handle(identity)
        .ok_or(RuntimeError::ArgumentError("stale timezone handle".into()))?;
    let local = vm.timezone_adapter.local_at(zone, timestamp);
    Ok(Value::obj(vm.heap.alloc_list(vec![
        Value::int(local.year() as i64),
        Value::int(local.month() as i64),
        Value::int(local.day() as i64),
        Value::int(local.hour() as i64),
        Value::int(local.minute() as i64),
        Value::int(local.second() as i64),
        Value::int(local.nanosecond() as i64),
    ])))
}

#[phalcom_native_macros::primitive(TimeZone, "resolveNative(_,_,_,_,_,_,_)", params = [Int, Int, Int, Int, Int, Int, Int], returns = List, types = "(Int, Int, Int, Int, Int, Int, Int) -> List", visibility = internal)]
pub fn timezone_resolve_native(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    let identity = timezone_identity(vm, receiver)?;
    let values = args
        .iter()
        .map(|arg| {
            expect_int_big(arg, vm).and_then(|value| {
                value.to_i64().ok_or_else(|| {
                    RuntimeError::Type {
                        expected: "bounded Int",
                        found: "Int",
                    }
                    .into()
                })
            })
        })
        .collect::<PhResult<Vec<_>>>()?;
    let [year, month, day, hour, minute, second, nanosecond] = values.as_slice() else {
        return Err(RuntimeError::Arity {
            signature: "TimeZone.resolveNative(_,_,_,_,_,_,_)",
            expected: 7,
            found: values.len(),
        }
        .into());
    };
    let datetime = jiff::civil::DateTime::new(
        *year as i16,
        *month as i8,
        *day as i8,
        *hour as i8,
        *minute as i8,
        *second as i8,
        *nanosecond as i32,
    )
    .map_err(|error| RuntimeError::ArgumentError(format!("invalid local datetime: {error}")))?;
    let zone = vm
        .timezone_adapter
        .handle(identity)
        .ok_or(RuntimeError::ArgumentError("stale timezone handle".into()))?;
    let fact = vm
        .timezone_adapter
        .resolve_local(zone, datetime)
        .map_err(|error| RuntimeError::ArgumentError(error.to_string()))?;
    let mut encoded = Vec::new();
    match fact {
        crate::timezone::LocalFact::Unique { timestamp } => encoded.extend([
            Value::int(0),
            Value::int(timestamp.as_second()),
            Value::int(timestamp.subsec_nanosecond() as i64),
        ]),
        crate::timezone::LocalFact::Fold { earlier, later } => encoded.extend([
            Value::int(1),
            Value::int(earlier.as_second()),
            Value::int(earlier.subsec_nanosecond() as i64),
            Value::int(later.as_second()),
            Value::int(later.subsec_nanosecond() as i64),
        ]),
        crate::timezone::LocalFact::Gap { before, after } => encoded.extend([
            Value::int(2),
            Value::int(before.as_second()),
            Value::int(before.subsec_nanosecond() as i64),
            Value::int(after.as_second()),
            Value::int(after.subsec_nanosecond() as i64),
        ]),
    }
    Ok(Value::obj(vm.heap.alloc_list(encoded)))
}

fn transition_native(vm: &mut VM, receiver: &Value, args: &[Value], before: bool) -> PhResult<Value> {
    let identity = timezone_identity(vm, receiver)?;
    let timestamp = timestamp_from_value(vm, &args[0])?;
    let zone = vm
        .timezone_adapter
        .handle(identity)
        .ok_or(RuntimeError::ArgumentError("stale timezone handle".into()))?;
    let fact = if before {
        vm.timezone_adapter.transition_before(zone, timestamp)
    } else {
        vm.timezone_adapter.transition_after(zone, timestamp)
    };
    let Some(fact) = fact else {
        return Ok(Value::obj(vm.heap.alloc_list(Vec::new())));
    };
    Ok(Value::obj(vm.heap.alloc_list(vec![
        Value::int(fact.timestamp.as_second()),
        Value::int(fact.timestamp.subsec_nanosecond() as i64),
        Value::int(fact.offset_before as i64),
        Value::int(fact.offset_after as i64),
    ])))
}

#[phalcom_native_macros::primitive(TimeZone, "transitionBeforeNative(_)", params = [Int], returns = List, types = "(Int) -> List", visibility = internal)]
pub fn timezone_transition_before_native(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    transition_native(vm, receiver, args, true)
}

#[phalcom_native_macros::primitive(TimeZone, "transitionAfterNative(_)", params = [Int], returns = List, types = "(Int) -> List", visibility = internal)]
pub fn timezone_transition_after_native(vm: &mut VM, receiver: &Value, args: &[Value]) -> PhResult<Value> {
    transition_native(vm, receiver, args, false)
}
