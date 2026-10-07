This file is a merged representation of a subset of the codebase, containing specifically included files, combined into a single document by Repomix.
The content has been processed where empty lines have been removed, line numbers have been added, security check has been disabled.

# File Summary

## Purpose
This file contains a packed representation of a subset of the repository's contents that is considered the most important context.
It is designed to be easily consumable by AI systems for analysis, code review,
or other automated processes.

## File Format
The content is organized as follows:
1. This summary section
2. Repository information
3. Directory structure
4. Repository files (if enabled)
5. Multiple file entries, each consisting of:
  a. A header with the file path (## File: path/to/file)
  b. The full contents of the file in a code block

## Usage Guidelines
- This file should be treated as read-only. Any changes should be made to the
  original repository files, not this packed version.
- When processing this file, use the file path to distinguish
  between different files in the repository.
- Be aware that this file may contain sensitive information. Handle it with
  the same level of security as you would the original repository.

## Notes
- Some files may have been excluded based on .gitignore rules and Repomix's configuration
- Binary files are not included in this packed representation. Please refer to the Repository Structure section for a complete list of file paths, including binary files
- Only files matching these patterns are included: src/**/*.rs, examples/**/*.rs
- Files matching patterns in .gitignore are excluded
- Files matching default ignore patterns are excluded
- Empty lines have been removed from all files
- Line numbers have been added to the beginning of each line
- Security check has been disabled - content may contain sensitive information
- Files are sorted by Git change count (files with more changes are at the bottom)

# Directory Structure
````
src/
  map/
    entry.rs
    iter.rs
    mutable.rs
    raw_entry_v1.rs
    rayon.rs
    serde_seq.rs
    slice.rs
    tests.rs
  set/
    iter.rs
    mutable.rs
    rayon.rs
    slice.rs
    tests.rs
  arbitrary.rs
  borsh.rs
  lib.rs
  macros.rs
  map.rs
  serde.rs
  set.rs
  sval.rs
````

# Files

## File: src/map/entry.rs
````rust
  1: use core::cmp::Ordering;
  2: use core::fmt;
  3: use indexmap::map as ix;
  4: #[cfg(doc)]
  5: use alloc::vec::Vec;
  6: /// Entry for an existing key-value pair in an [`OrderMap`][crate::OrderMap]
  7: /// or a vacant location to insert one.
  8: pub enum Entry<'a, K, V> {
  9:     /// Existing slot with equivalent key.
 10:     Occupied(OccupiedEntry<'a, K, V>),
 11:     /// Vacant slot (no equivalent key in the map).
 12:     Vacant(VacantEntry<'a, K, V>),
 13: }
 14: impl<'a, K, V> Entry<'a, K, V> {
 15:     pub(super) fn new(entry: ix::Entry<'a, K, V>) -> Self {
 16:         match entry {
 17:             ix::Entry::Occupied(inner) => Self::Occupied(OccupiedEntry { inner }),
 18:             ix::Entry::Vacant(inner) => Self::Vacant(VacantEntry { inner }),
 19:         }
 20:     }
 21:     /// Return the index where the key-value pair exists or will be inserted.
 22:     pub fn index(&self) -> usize {
 23:         match *self {
 24:             Entry::Occupied(ref entry) => entry.index(),
 25:             Entry::Vacant(ref entry) => entry.index(),
 26:         }
 27:     }
 28:     /// Sets the value of the entry (after inserting if vacant), and returns an `OccupiedEntry`.
 29:     ///
 30:     /// Computes in **O(1)** time (amortized average).
 31:     pub fn insert_entry(self, value: V) -> OccupiedEntry<'a, K, V> {
 32:         match self {
 33:             Entry::Occupied(mut entry) => {
 34:                 entry.insert(value);
 35:                 entry
 36:             }
 37:             Entry::Vacant(entry) => entry.insert_entry(value),
 38:         }
 39:     }
 40:     /// Inserts the given default value in the entry if it is vacant and returns a mutable
 41:     /// reference to it. Otherwise a mutable reference to an already existent value is returned.
 42:     ///
 43:     /// Computes in **O(1)** time (amortized average).
 44:     pub fn or_insert(self, default: V) -> &'a mut V {
 45:         match self {
 46:             Entry::Occupied(entry) => entry.into_mut(),
 47:             Entry::Vacant(entry) => entry.insert(default),
 48:         }
 49:     }
 50:     /// Inserts the result of the `call` function in the entry if it is vacant and returns a mutable
 51:     /// reference to it. Otherwise a mutable reference to an already existent value is returned.
 52:     ///
 53:     /// Computes in **O(1)** time (amortized average).
 54:     pub fn or_insert_with<F>(self, call: F) -> &'a mut V
 55:     where
 56:         F: FnOnce() -> V,
 57:     {
 58:         match self {
 59:             Entry::Occupied(entry) => entry.into_mut(),
 60:             Entry::Vacant(entry) => entry.insert(call()),
 61:         }
 62:     }
 63:     /// Inserts the result of the `call` function with a reference to the entry's key if it is
 64:     /// vacant, and returns a mutable reference to the new value. Otherwise a mutable reference to
 65:     /// an already existent value is returned.
 66:     ///
 67:     /// Computes in **O(1)** time (amortized average).
 68:     pub fn or_insert_with_key<F>(self, call: F) -> &'a mut V
 69:     where
 70:         F: FnOnce(&K) -> V,
 71:     {
 72:         match self {
 73:             Entry::Occupied(entry) => entry.into_mut(),
 74:             Entry::Vacant(entry) => {
 75:                 let value = call(&entry.key());
 76:                 entry.insert(value)
 77:             }
 78:         }
 79:     }
 80:     /// Gets a reference to the entry's key, either within the map if occupied,
 81:     /// or else the new key that was used to find the entry.
 82:     pub fn key(&self) -> &K {
 83:         match *self {
 84:             Entry::Occupied(ref entry) => entry.key(),
 85:             Entry::Vacant(ref entry) => entry.key(),
 86:         }
 87:     }
 88:     /// Modifies the entry if it is occupied.
 89:     pub fn and_modify<F>(mut self, f: F) -> Self
 90:     where
 91:         F: FnOnce(&mut V),
 92:     {
 93:         if let Entry::Occupied(entry) = &mut self {
 94:             f(entry.get_mut());
 95:         }
 96:         self
 97:     }
 98:     /// Inserts a default-constructed value in the entry if it is vacant and returns a mutable
 99:     /// reference to it. Otherwise a mutable reference to an already existent value is returned.
100:     ///
101:     /// Computes in **O(1)** time (amortized average).
102:     pub fn or_default(self) -> &'a mut V
103:     where
104:         V: Default,
105:     {
106:         match self {
107:             Entry::Occupied(entry) => entry.into_mut(),
108:             Entry::Vacant(entry) => entry.insert(V::default()),
109:         }
110:     }
111: }
112: impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for Entry<'_, K, V> {
113:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
114:         let mut tuple = f.debug_tuple("Entry");
115:         match self {
116:             Entry::Vacant(v) => tuple.field(v),
117:             Entry::Occupied(o) => tuple.field(o),
118:         };
119:         tuple.finish()
120:     }
121: }
122: /// A view into an occupied entry in an [`OrderMap`][crate::OrderMap].
123: /// It is part of the [`Entry`] enum.
124: pub struct OccupiedEntry<'a, K, V> {
125:     pub(crate) inner: ix::OccupiedEntry<'a, K, V>,
126: }
127: impl<'a, K, V> OccupiedEntry<'a, K, V> {
128:     /// Return the index of the key-value pair
129:     #[inline]
130:     pub fn index(&self) -> usize {
131:         self.inner.index()
132:     }
133:     /// Gets a reference to the entry's key in the map.
134:     ///
135:     /// Note that this is not the key that was used to find the entry. There may be an observable
136:     /// difference if the key type has any distinguishing features outside of `Hash` and `Eq`, like
137:     /// extra fields or the memory address of an allocation.
138:     pub fn key(&self) -> &K {
139:         self.inner.key()
140:     }
141:     /// Gets a reference to the entry's value in the map.
142:     pub fn get(&self) -> &V {
143:         self.inner.get()
144:     }
145:     /// Gets a mutable reference to the entry's value in the map.
146:     ///
147:     /// If you need a reference which may outlive the destruction of the
148:     /// [`Entry`] value, see [`into_mut`][Self::into_mut].
149:     pub fn get_mut(&mut self) -> &mut V {
150:         self.inner.get_mut()
151:     }
152:     /// Converts into a mutable reference to the entry's value in the map,
153:     /// with a lifetime bound to the map itself.
154:     pub fn into_mut(self) -> &'a mut V {
155:         self.inner.into_mut()
156:     }
157:     /// Sets the value of the entry to `value`, and returns the entry's old value.
158:     pub fn insert(&mut self, value: V) -> V {
159:         self.inner.insert(value)
160:     }
161:     /// Remove the key, value pair stored in the map for this entry, and return the value.
162:     ///
163:     /// **NOTE:** This is equivalent to indexmap's
164:     /// [`OccupiedEntry::shift_remove`][ix::OccupiedEntry::shift_remove], and
165:     /// like [`Vec::remove`], the pair is removed by shifting all of the
166:     /// elements that follow it, preserving their relative order.
167:     /// **This perturbs the index of all of those elements!**
168:     ///
169:     /// Computes in **O(n)** time (average).
170:     pub fn remove(self) -> V {
171:         self.inner.shift_remove()
172:     }
173:     /// Remove and return the key, value pair stored in the map for this entry
174:     ///
175:     /// **NOTE:** This is equivalent to indexmap's
176:     /// [`OccupiedEntry::shift_remove_entry`][ix::OccupiedEntry::shift_remove_entry], and
177:     /// like [`Vec::remove`], the pair is removed by shifting all of the
178:     /// elements that follow it, preserving their relative order.
179:     /// **This perturbs the index of all of those elements!**
180:     ///
181:     /// Computes in **O(n)** time (average).
182:     pub fn remove_entry(self) -> (K, V) {
183:         self.inner.shift_remove_entry()
184:     }
185:     /// Remove the key, value pair stored in the map for this entry, and return the value.
186:     ///
187:     /// Like [`Vec::swap_remove`], the pair is removed by swapping it with
188:     /// the last element of the map and popping it off.
189:     /// **This perturbs the position of what used to be the last element!**
190:     ///
191:     /// Computes in **O(1)** time (average).
192:     pub fn swap_remove(self) -> V {
193:         self.inner.swap_remove()
194:     }
195:     /// Remove and return the key, value pair stored in the map for this entry
196:     ///
197:     /// Like [`Vec::swap_remove`], the pair is removed by swapping it with
198:     /// the last element of the map and popping it off.
199:     /// **This perturbs the position of what used to be the last element!**
200:     ///
201:     /// Computes in **O(1)** time (average).
202:     pub fn swap_remove_entry(self) -> (K, V) {
203:         self.inner.swap_remove_entry()
204:     }
205:     /// Moves the position of the entry to a new index
206:     /// by shifting all other entries in-between.
207:     ///
208:     /// This is equivalent to [`OrderMap::move_index`][`crate::OrderMap::move_index`]
209:     /// coming `from` the current [`.index()`][Self::index].
210:     ///
211:     /// * If `self.index() < to`, the other pairs will shift down while the targeted pair moves up.
212:     /// * If `self.index() > to`, the other pairs will shift up while the targeted pair moves down.
213:     ///
214:     /// ***Panics*** if `to` is out of bounds.
215:     ///
216:     /// Computes in **O(n)** time (average).
217:     #[track_caller]
218:     pub fn move_index(self, to: usize) {
219:         self.inner.move_index(to);
220:     }
221:     /// Swaps the position of entry with another.
222:     ///
223:     /// This is equivalent to [`OrderMap::swap_indices`][`crate::OrderMap::swap_indices`]
224:     /// with the current [`.index()`][Self::index] as one of the two being swapped.
225:     ///
226:     /// ***Panics*** if the `other` index is out of bounds.
227:     ///
228:     /// Computes in **O(1)** time (average).
229:     #[track_caller]
230:     pub fn swap_indices(self, other: usize) {
231:         self.inner.swap_indices(other);
232:     }
233: }
234: impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for OccupiedEntry<'_, K, V> {
235:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
236:         f.debug_struct("OccupiedEntry")
237:             .field("key", self.key())
238:             .field("value", self.get())
239:             .finish()
240:     }
241: }
242: impl<'a, K, V> From<IndexedEntry<'a, K, V>> for OccupiedEntry<'a, K, V> {
243:     fn from(entry: IndexedEntry<'a, K, V>) -> Self {
244:         Self {
245:             inner: entry.inner.into(),
246:         }
247:     }
248: }
249: /// A view into a vacant entry in an [`OrderMap`][crate::OrderMap].
250: /// It is part of the [`Entry`] enum.
251: pub struct VacantEntry<'a, K, V> {
252:     pub(crate) inner: ix::VacantEntry<'a, K, V>,
253: }
254: impl<'a, K, V> VacantEntry<'a, K, V> {
255:     /// Return the index where a key-value pair may be inserted.
256:     pub fn index(&self) -> usize {
257:         self.inner.index()
258:     }
259:     /// Gets a reference to the key that was used to find the entry.
260:     pub fn key(&self) -> &K {
261:         self.inner.key()
262:     }
263:     /// Takes ownership of the key, leaving the entry vacant.
264:     pub fn into_key(self) -> K {
265:         self.inner.into_key()
266:     }
267:     /// Inserts the entry's key and the given value into the map, and returns a mutable reference
268:     /// to the value.
269:     pub fn insert(self, value: V) -> &'a mut V {
270:         self.inner.insert(value)
271:     }
272:     /// Inserts the entry's key and the given value into the map, and returns an `OccupiedEntry`.
273:     ///
274:     /// Computes in **O(1)** time (amortized average).
275:     pub fn insert_entry(self, value: V) -> OccupiedEntry<'a, K, V> {
276:         OccupiedEntry {
277:             inner: self.inner.insert_entry(value),
278:         }
279:     }
280:     /// Inserts the entry's key and the given value into the map at its ordered
281:     /// position among sorted keys, and returns the new index and a mutable
282:     /// reference to the value.
283:     ///
284:     /// If the existing keys are **not** already sorted, then the insertion
285:     /// index is unspecified (like [`slice::binary_search`]), but the key-value
286:     /// pair is inserted at that position regardless.
287:     ///
288:     /// Computes in **O(n)** time (average).
289:     pub fn insert_sorted(self, value: V) -> (usize, &'a mut V)
290:     where
291:         K: Ord,
292:     {
293:         self.inner.insert_sorted(value)
294:     }
295:     /// Inserts the entry's key and the given value into the map at its ordered
296:     /// position among keys sorted by `cmp`, and returns the new index and a
297:     /// mutable reference to the value.
298:     ///
299:     /// If the existing keys are **not** already sorted, then the insertion
300:     /// index is unspecified (like [`slice::binary_search`]), but the key-value
301:     /// pair is inserted at that position regardless.
302:     ///
303:     /// Computes in **O(n)** time (average).
304:     pub fn insert_sorted_by<F>(self, value: V, cmp: F) -> (usize, &'a mut V)
305:     where
306:         F: FnMut(&K, &V, &K, &V) -> Ordering,
307:     {
308:         self.inner.insert_sorted_by(value, cmp)
309:     }
310:     /// Inserts the entry's key and the given value into the map at its ordered
311:     /// position using a sort-key extraction function, and returns the new index
312:     /// and a mutable reference to the value.
313:     ///
314:     /// If the existing keys are **not** already sorted, then the insertion
315:     /// index is unspecified (like [`slice::binary_search`]), but the key-value
316:     /// pair is inserted at that position regardless.
317:     ///
318:     /// Computes in **O(n)** time (average).
319:     pub fn insert_sorted_by_key<B, F>(self, value: V, sort_key: F) -> (usize, &'a mut V)
320:     where
321:         B: Ord,
322:         F: FnMut(&K, &V) -> B,
323:     {
324:         self.inner.insert_sorted_by_key(value, sort_key)
325:     }
326:     /// Inserts the entry's key and the given value into the map at the given index,
327:     /// shifting others to the right, and returns a mutable reference to the value.
328:     ///
329:     /// ***Panics*** if `index` is out of bounds.
330:     ///
331:     /// Computes in **O(n)** time (average).
332:     #[track_caller]
333:     pub fn shift_insert(self, index: usize, value: V) -> &'a mut V {
334:         self.inner.shift_insert(index, value)
335:     }
336:     /// Replaces the key at the given index with this entry's key, returning the
337:     /// old key and an `OccupiedEntry` for that index.
338:     ///
339:     /// ***Panics*** if `index` is out of bounds.
340:     ///
341:     /// Computes in **O(1)** time (average).
342:     #[track_caller]
343:     pub fn replace_index(self, index: usize) -> (K, OccupiedEntry<'a, K, V>) {
344:         let (old_key, inner) = self.inner.replace_index(index);
345:         (old_key, OccupiedEntry { inner })
346:     }
347: }
348: impl<K: fmt::Debug, V> fmt::Debug for VacantEntry<'_, K, V> {
349:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
350:         f.debug_tuple("VacantEntry").field(self.key()).finish()
351:     }
352: }
353: /// A view into an occupied entry in an [`OrderMap`][crate::OrderMap] obtained by index.
354: ///
355: /// This `struct` is created from the [`get_index_entry`][crate::OrderMap::get_index_entry] method.
356: pub struct IndexedEntry<'a, K, V> {
357:     pub(crate) inner: ix::IndexedEntry<'a, K, V>,
358: }
359: impl<'a, K, V> IndexedEntry<'a, K, V> {
360:     pub(super) fn new(inner: ix::IndexedEntry<'a, K, V>) -> Self {
361:         Self { inner }
362:     }
363:     /// Return the index of the key-value pair
364:     #[inline]
365:     pub fn index(&self) -> usize {
366:         self.inner.index()
367:     }
368:     /// Gets a reference to the entry's key in the map.
369:     pub fn key(&self) -> &K {
370:         self.inner.key()
371:     }
372:     /// Gets a reference to the entry's value in the map.
373:     pub fn get(&self) -> &V {
374:         self.inner.get()
375:     }
376:     /// Gets a mutable reference to the entry's value in the map.
377:     ///
378:     /// If you need a reference which may outlive the destruction of the
379:     /// `IndexedEntry` value, see [`into_mut`][Self::into_mut].
380:     pub fn get_mut(&mut self) -> &mut V {
381:         self.inner.get_mut()
382:     }
383:     /// Sets the value of the entry to `value`, and returns the entry's old value.
384:     pub fn insert(&mut self, value: V) -> V {
385:         self.inner.insert(value)
386:     }
387:     /// Converts into a mutable reference to the entry's value in the map,
388:     /// with a lifetime bound to the map itself.
389:     pub fn into_mut(self) -> &'a mut V {
390:         self.inner.into_mut()
391:     }
392:     /// Remove and return the key, value pair stored in the map for this entry
393:     ///
394:     /// **NOTE:** This is equivalent to indexmap's
395:     /// [`IndexedEntry::shift_remove_entry`][ix::IndexedEntry::shift_remove_entry], and
396:     /// like [`Vec::remove`], the pair is removed by shifting all of the
397:     /// elements that follow it, preserving their relative order.
398:     /// **This perturbs the index of all of those elements!**
399:     ///
400:     /// Computes in **O(n)** time (average).
401:     pub fn remove_entry(self) -> (K, V) {
402:         self.inner.shift_remove_entry()
403:     }
404:     /// Remove the key, value pair stored in the map for this entry, and return the value.
405:     ///
406:     /// **NOTE:** This is equivalent to indexmap's
407:     /// [`IndexedEntry::shift_remove`][ix::IndexedEntry::shift_remove], and
408:     /// like [`Vec::remove`], the pair is removed by shifting all of the
409:     /// elements that follow it, preserving their relative order.
410:     /// **This perturbs the index of all of those elements!**
411:     ///
412:     /// Computes in **O(n)** time (average).
413:     pub fn remove(self) -> V {
414:         self.inner.shift_remove()
415:     }
416:     /// Remove and return the key, value pair stored in the map for this entry
417:     ///
418:     /// Like [`Vec::swap_remove`], the pair is removed by swapping it with
419:     /// the last element of the map and popping it off.
420:     /// **This perturbs the position of what used to be the last element!**
421:     ///
422:     /// Computes in **O(1)** time (average).
423:     pub fn swap_remove_entry(self) -> (K, V) {
424:         self.inner.swap_remove_entry()
425:     }
426:     /// Remove the key, value pair stored in the map for this entry, and return the value.
427:     ///
428:     /// Like [`Vec::swap_remove`], the pair is removed by swapping it with
429:     /// the last element of the map and popping it off.
430:     /// **This perturbs the position of what used to be the last element!**
431:     ///
432:     /// Computes in **O(1)** time (average).
433:     pub fn swap_remove(self) -> V {
434:         self.inner.swap_remove()
435:     }
436:     /// Moves the position of the entry to a new index
437:     /// by shifting all other entries in-between.
438:     ///
439:     /// This is equivalent to [`OrderMap::move_index`][`crate::OrderMap::move_index`]
440:     /// coming `from` the current [`.index()`][Self::index].
441:     ///
442:     /// * If `self.index() < to`, the other pairs will shift down while the targeted pair moves up.
443:     /// * If `self.index() > to`, the other pairs will shift up while the targeted pair moves down.
444:     ///
445:     /// ***Panics*** if `to` is out of bounds.
446:     ///
447:     /// Computes in **O(n)** time (average).
448:     #[track_caller]
449:     pub fn move_index(self, to: usize) {
450:         self.inner.move_index(to)
451:     }
452:     /// Swaps the position of entry with another.
453:     ///
454:     /// This is equivalent to [`OrderMap::swap_indices`][`crate::OrderMap::swap_indices`]
455:     /// with the current [`.index()`][Self::index] as one of the two being swapped.
456:     ///
457:     /// ***Panics*** if the `other` index is out of bounds.
458:     ///
459:     /// Computes in **O(1)** time (average).
460:     #[track_caller]
461:     pub fn swap_indices(self, other: usize) {
462:         self.inner.swap_indices(other)
463:     }
464: }
465: impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for IndexedEntry<'_, K, V> {
466:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
467:         f.debug_struct("IndexedEntry")
468:             .field("index", &self.index())
469:             .field("key", self.key())
470:             .field("value", self.get())
471:             .finish()
472:     }
473: }
474: impl<'a, K, V> From<OccupiedEntry<'a, K, V>> for IndexedEntry<'a, K, V> {
475:     fn from(entry: OccupiedEntry<'a, K, V>) -> Self {
476:         Self {
477:             inner: entry.inner.into(),
478:         }
479:     }
480: }
````

## File: src/map/iter.rs
````rust
 1: use super::{IntoIter, Iter, IterMut, OrderMap};
 2: impl<'a, K, V, S> IntoIterator for &'a OrderMap<K, V, S> {
 3:     type Item = (&'a K, &'a V);
 4:     type IntoIter = Iter<'a, K, V>;
 5:     fn into_iter(self) -> Self::IntoIter {
 6:         self.inner.iter()
 7:     }
 8: }
 9: impl<'a, K, V, S> IntoIterator for &'a mut OrderMap<K, V, S> {
10:     type Item = (&'a K, &'a mut V);
11:     type IntoIter = IterMut<'a, K, V>;
12:     fn into_iter(self) -> Self::IntoIter {
13:         self.inner.iter_mut()
14:     }
15: }
16: impl<K, V, S> IntoIterator for OrderMap<K, V, S> {
17:     type Item = (K, V);
18:     type IntoIter = IntoIter<K, V>;
19:     fn into_iter(self) -> Self::IntoIter {
20:         self.inner.into_iter()
21:     }
22: }
````

## File: src/map/mutable.rs
````rust
  1: use super::{Entry, Equivalent, IndexedEntry, IterMut2, OccupiedEntry, OrderMap, VacantEntry};
  2: use core::hash::{BuildHasher, Hash};
  3: use indexmap::map::MutableEntryKey as _;
  4: use indexmap::map::MutableKeys as _;
  5: /// Opt-in mutable access to [`OrderMap`] keys.
  6: ///
  7: /// These methods expose `&mut K`, mutable references to the key as it is stored
  8: /// in the map.
  9: /// You are allowed to modify the keys in the map **if the modification
 10: /// does not change the key's hash and equality**.
 11: ///
 12: /// If keys are modified erroneously, you can no longer look them up.
 13: /// This is sound (memory safe) but a logical error hazard (just like
 14: /// implementing `PartialEq`, `Eq`, or `Hash` incorrectly would be).
 15: ///
 16: /// `use` this trait to enable its methods for `OrderMap`.
 17: ///
 18: /// This trait is sealed and cannot be implemented for types outside this crate.
 19: #[expect(private_bounds)]
 20: pub trait MutableKeys: Sealed {
 21:     type Key;
 22:     type Value;
 23:     /// Return item index, mutable reference to key and value
 24:     ///
 25:     /// Computes in **O(1)** time (average).
 26:     fn get_full_mut2<Q>(&mut self, key: &Q) -> Option<(usize, &mut Self::Key, &mut Self::Value)>
 27:     where
 28:         Q: ?Sized + Hash + Equivalent<Self::Key>;
 29:     /// Return mutable reference to key and value at an index.
 30:     ///
 31:     /// Valid indices are `0 <= index < self.len()`.
 32:     ///
 33:     /// Computes in **O(1)** time.
 34:     fn get_index_mut2(&mut self, index: usize) -> Option<(&mut Self::Key, &mut Self::Value)>;
 35:     /// Return an iterator over the key-value pairs of the map, in their order
 36:     fn iter_mut2(&mut self) -> IterMut2<'_, Self::Key, Self::Value>;
 37:     /// Scan through each key-value pair in the map and keep those where the
 38:     /// closure `keep` returns `true`.
 39:     ///
 40:     /// The elements are visited in order, and remaining elements keep their
 41:     /// order.
 42:     ///
 43:     /// Computes in **O(n)** time (average).
 44:     fn retain2<F>(&mut self, keep: F)
 45:     where
 46:         F: FnMut(&mut Self::Key, &mut Self::Value) -> bool;
 47: }
 48: /// Opt-in mutable access to [`OrderMap`] keys.
 49: ///
 50: /// See [`MutableKeys`] for more information.
 51: impl<K, V, S> MutableKeys for OrderMap<K, V, S>
 52: where
 53:     S: BuildHasher,
 54: {
 55:     type Key = K;
 56:     type Value = V;
 57:     fn get_full_mut2<Q>(&mut self, key: &Q) -> Option<(usize, &mut K, &mut V)>
 58:     where
 59:         Q: ?Sized + Hash + Equivalent<K>,
 60:     {
 61:         self.inner.get_full_mut2(key)
 62:     }
 63:     fn get_index_mut2(&mut self, index: usize) -> Option<(&mut K, &mut V)> {
 64:         self.inner.get_index_mut2(index)
 65:     }
 66:     fn iter_mut2(&mut self) -> IterMut2<'_, Self::Key, Self::Value> {
 67:         self.inner.iter_mut2()
 68:     }
 69:     fn retain2<F>(&mut self, keep: F)
 70:     where
 71:         F: FnMut(&mut K, &mut V) -> bool,
 72:     {
 73:         self.inner.retain2(keep);
 74:     }
 75: }
 76: /// Opt-in mutable access to [`Entry`] keys.
 77: ///
 78: /// These methods expose `&mut K`, mutable references to the key as it is stored
 79: /// in the map.
 80: /// You are allowed to modify the keys in the map **if the modification
 81: /// does not change the key's hash and equality**.
 82: ///
 83: /// If keys are modified erroneously, you can no longer look them up.
 84: /// This is sound (memory safe) but a logical error hazard (just like
 85: /// implementing `PartialEq`, `Eq`, or `Hash` incorrectly would be).
 86: ///
 87: /// `use` this trait to enable its methods for `Entry`.
 88: ///
 89: /// This trait is sealed and cannot be implemented for types outside this crate.
 90: #[expect(private_bounds)]
 91: pub trait MutableEntryKey: Sealed {
 92:     type Key;
 93:     /// Gets a mutable reference to the entry's key, either within the map if occupied,
 94:     /// or else the new key that was used to find the entry.
 95:     fn key_mut(&mut self) -> &mut Self::Key;
 96: }
 97: /// Opt-in mutable access to [`Entry`] keys.
 98: ///
 99: /// See [`MutableEntryKey`] for more information.
100: impl<K, V> MutableEntryKey for Entry<'_, K, V> {
101:     type Key = K;
102:     fn key_mut(&mut self) -> &mut Self::Key {
103:         match self {
104:             Entry::Occupied(e) => e.key_mut(),
105:             Entry::Vacant(e) => e.key_mut(),
106:         }
107:     }
108: }
109: /// Opt-in mutable access to [`OccupiedEntry`] keys.
110: ///
111: /// See [`MutableEntryKey`] for more information.
112: impl<K, V> MutableEntryKey for OccupiedEntry<'_, K, V> {
113:     type Key = K;
114:     fn key_mut(&mut self) -> &mut Self::Key {
115:         self.inner.key_mut()
116:     }
117: }
118: /// Opt-in mutable access to [`VacantEntry`] keys.
119: ///
120: /// See [`MutableEntryKey`] for more information.
121: impl<K, V> MutableEntryKey for VacantEntry<'_, K, V> {
122:     type Key = K;
123:     fn key_mut(&mut self) -> &mut Self::Key {
124:         self.inner.key_mut()
125:     }
126: }
127: /// Opt-in mutable access to [`IndexedEntry`] keys.
128: ///
129: /// See [`MutableEntryKey`] for more information.
130: impl<K, V> MutableEntryKey for IndexedEntry<'_, K, V> {
131:     type Key = K;
132:     fn key_mut(&mut self) -> &mut Self::Key {
133:         self.inner.key_mut()
134:     }
135: }
136: trait Sealed {}
137: impl<K, V, S> Sealed for OrderMap<K, V, S> {}
138: impl<K, V> Sealed for Entry<'_, K, V> {}
139: impl<K, V> Sealed for OccupiedEntry<'_, K, V> {}
140: impl<K, V> Sealed for VacantEntry<'_, K, V> {}
141: impl<K, V> Sealed for IndexedEntry<'_, K, V> {}
````

## File: src/map/raw_entry_v1.rs
````rust
  1: //! Opt-in access to the experimental raw entry API.
  2: //!
  3: //! This module is designed to mimic the raw entry API of [`HashMap`][std::collections::hash_map],
  4: //! matching its unstable state as of Rust 1.75. See the tracking issue
  5: //! [rust#56167](https://github.com/rust-lang/rust/issues/56167) for more details.
  6: //!
  7: //! The trait [`RawEntryApiV1`] and the `_v1` suffix on its methods are meant to insulate this for
  8: //! the future, in case later breaking changes are needed. If the standard library stabilizes its
  9: //! `hash_raw_entry` feature (or some replacement), matching *inherent* methods will be added to
 10: //! `OrderMap` without such an opt-in trait.
 11: use crate::{Equivalent, OrderMap};
 12: use core::fmt;
 13: use core::hash::{BuildHasher, Hash};
 14: use indexmap::map::RawEntryApiV1 as _;
 15: use indexmap::map::raw_entry_v1 as ix;
 16: #[cfg(doc)]
 17: use alloc::vec::Vec;
 18: /// Opt-in access to the experimental raw entry API.
 19: ///
 20: /// See the [`raw_entry_v1`][self] module documentation for more information.
 21: #[expect(private_bounds)]
 22: pub trait RawEntryApiV1<K, V, S>: Sealed {
 23:     /// Creates a raw immutable entry builder for the [`OrderMap`].
 24:     ///
 25:     /// Raw entries provide the lowest level of control for searching and
 26:     /// manipulating a map. They must be manually initialized with a hash and
 27:     /// then manually searched.
 28:     ///
 29:     /// This is useful for
 30:     /// * Hash memoization
 31:     /// * Using a search key that doesn't work with the [`Equivalent`] trait
 32:     /// * Using custom comparison logic without newtype wrappers
 33:     ///
 34:     /// Unless you are in such a situation, higher-level and more foolproof APIs like
 35:     /// [`get`][OrderMap::get] should be preferred.
 36:     ///
 37:     /// Immutable raw entries have very limited use; you might instead want
 38:     /// [`raw_entry_mut_v1`][Self::raw_entry_mut_v1].
 39:     ///
 40:     /// # Examples
 41:     ///
 42:     /// ```
 43:     /// use core::hash::BuildHasher;
 44:     /// use ordermap::map::{OrderMap, RawEntryApiV1};
 45:     ///
 46:     /// let mut map = OrderMap::new();
 47:     /// map.extend([("a", 100), ("b", 200), ("c", 300)]);
 48:     ///
 49:     /// for k in ["a", "b", "c", "d", "e", "f"] {
 50:     ///     let hash = map.hasher().hash_one(k);
 51:     ///     let i = map.get_index_of(k);
 52:     ///     let v = map.get(k);
 53:     ///     let kv = map.get_key_value(k);
 54:     ///     let ikv = map.get_full(k);
 55:     ///
 56:     ///     println!("Key: {} and value: {:?}", k, v);
 57:     ///
 58:     ///     assert_eq!(map.raw_entry_v1().from_key(k), kv);
 59:     ///     assert_eq!(map.raw_entry_v1().from_hash(hash, |q| *q == k), kv);
 60:     ///     assert_eq!(map.raw_entry_v1().from_key_hashed_nocheck(hash, k), kv);
 61:     ///     assert_eq!(map.raw_entry_v1().from_hash_full(hash, |q| *q == k), ikv);
 62:     ///     assert_eq!(map.raw_entry_v1().index_from_hash(hash, |q| *q == k), i);
 63:     /// }
 64:     /// ```
 65:     fn raw_entry_v1(&self) -> RawEntryBuilder<'_, K, V, S>;
 66:     /// Creates a raw entry builder for the [`OrderMap`].
 67:     ///
 68:     /// Raw entries provide the lowest level of control for searching and
 69:     /// manipulating a map. They must be manually initialized with a hash and
 70:     /// then manually searched. After this, insertions into a vacant entry
 71:     /// still require an owned key to be provided.
 72:     ///
 73:     /// Raw entries are useful for such exotic situations as:
 74:     ///
 75:     /// * Hash memoization
 76:     /// * Deferring the creation of an owned key until it is known to be required
 77:     /// * Using a search key that doesn't work with the [`Equivalent`] trait
 78:     /// * Using custom comparison logic without newtype wrappers
 79:     ///
 80:     /// Because raw entries provide much more low-level control, it's much easier
 81:     /// to put the `OrderMap` into an inconsistent state which, while memory-safe,
 82:     /// will cause the map to produce seemingly random results. Higher-level and more
 83:     /// foolproof APIs like [`entry`][OrderMap::entry] should be preferred when possible.
 84:     ///
 85:     /// Raw entries give mutable access to the keys. This must not be used
 86:     /// to modify how the key would compare or hash, as the map will not re-evaluate
 87:     /// where the key should go, meaning the keys may become "lost" if their
 88:     /// location does not reflect their state. For instance, if you change a key
 89:     /// so that the map now contains keys which compare equal, search may start
 90:     /// acting erratically, with two keys randomly masking each other. Implementations
 91:     /// are free to assume this doesn't happen (within the limits of memory-safety).
 92:     ///
 93:     /// # Examples
 94:     ///
 95:     /// ```
 96:     /// use core::hash::BuildHasher;
 97:     /// use ordermap::map::{OrderMap, RawEntryApiV1};
 98:     /// use ordermap::map::raw_entry_v1::RawEntryMut;
 99:     ///
100:     /// let mut map = OrderMap::new();
101:     /// map.extend([("a", 100), ("b", 200), ("c", 300)]);
102:     ///
103:     /// // Existing key (insert and update)
104:     /// match map.raw_entry_mut_v1().from_key("a") {
105:     ///     RawEntryMut::Vacant(_) => unreachable!(),
106:     ///     RawEntryMut::Occupied(mut view) => {
107:     ///         assert_eq!(view.index(), 0);
108:     ///         assert_eq!(view.get(), &100);
109:     ///         let v = view.get_mut();
110:     ///         let new_v = (*v) * 10;
111:     ///         *v = new_v;
112:     ///         assert_eq!(view.insert(1111), 1000);
113:     ///     }
114:     /// }
115:     ///
116:     /// assert_eq!(map["a"], 1111);
117:     /// assert_eq!(map.len(), 3);
118:     ///
119:     /// // Existing key (take)
120:     /// let hash = map.hasher().hash_one("c");
121:     /// match map.raw_entry_mut_v1().from_key_hashed_nocheck(hash, "c") {
122:     ///     RawEntryMut::Vacant(_) => unreachable!(),
123:     ///     RawEntryMut::Occupied(view) => {
124:     ///         assert_eq!(view.index(), 2);
125:     ///         assert_eq!(view.remove_entry(), ("c", 300));
126:     ///     }
127:     /// }
128:     /// assert_eq!(map.raw_entry_v1().from_key("c"), None);
129:     /// assert_eq!(map.len(), 2);
130:     ///
131:     /// // Nonexistent key (insert and update)
132:     /// let key = "d";
133:     /// let hash = map.hasher().hash_one(key);
134:     /// match map.raw_entry_mut_v1().from_hash(hash, |q| *q == key) {
135:     ///     RawEntryMut::Occupied(_) => unreachable!(),
136:     ///     RawEntryMut::Vacant(view) => {
137:     ///         assert_eq!(view.index(), 2);
138:     ///         let (k, value) = view.insert("d", 4000);
139:     ///         assert_eq!((*k, *value), ("d", 4000));
140:     ///         *value = 40000;
141:     ///     }
142:     /// }
143:     /// assert_eq!(map["d"], 40000);
144:     /// assert_eq!(map.len(), 3);
145:     ///
146:     /// match map.raw_entry_mut_v1().from_hash(hash, |q| *q == key) {
147:     ///     RawEntryMut::Vacant(_) => unreachable!(),
148:     ///     RawEntryMut::Occupied(view) => {
149:     ///         assert_eq!(view.index(), 2);
150:     ///         assert_eq!(view.swap_remove_entry(), ("d", 40000));
151:     ///     }
152:     /// }
153:     /// assert_eq!(map.get("d"), None);
154:     /// assert_eq!(map.len(), 2);
155:     /// ```
156:     fn raw_entry_mut_v1(&mut self) -> RawEntryBuilderMut<'_, K, V, S>;
157: }
158: impl<K, V, S> RawEntryApiV1<K, V, S> for OrderMap<K, V, S> {
159:     fn raw_entry_v1(&self) -> RawEntryBuilder<'_, K, V, S> {
160:         RawEntryBuilder {
161:             inner: self.inner.raw_entry_v1(),
162:         }
163:     }
164:     fn raw_entry_mut_v1(&mut self) -> RawEntryBuilderMut<'_, K, V, S> {
165:         RawEntryBuilderMut {
166:             inner: self.inner.raw_entry_mut_v1(),
167:         }
168:     }
169: }
170: /// A builder for computing where in an [`OrderMap`] a key-value pair would be stored.
171: ///
172: /// This `struct` is created by the [`OrderMap::raw_entry_v1`] method, provided by the
173: /// [`RawEntryApiV1`] trait. See its documentation for more.
174: pub struct RawEntryBuilder<'a, K, V, S> {
175:     inner: ix::RawEntryBuilder<'a, K, V, S>,
176: }
177: impl<K, V, S> fmt::Debug for RawEntryBuilder<'_, K, V, S> {
178:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
179:         f.debug_struct("RawEntryBuilder").finish_non_exhaustive()
180:     }
181: }
182: impl<'a, K, V, S> RawEntryBuilder<'a, K, V, S> {
183:     /// Access an entry by key.
184:     pub fn from_key<Q>(self, key: &Q) -> Option<(&'a K, &'a V)>
185:     where
186:         S: BuildHasher,
187:         Q: ?Sized + Hash + Equivalent<K>,
188:     {
189:         self.inner.from_key(key)
190:     }
191:     /// Access an entry by a key and its hash.
192:     pub fn from_key_hashed_nocheck<Q>(self, hash: u64, key: &Q) -> Option<(&'a K, &'a V)>
193:     where
194:         Q: ?Sized + Equivalent<K>,
195:     {
196:         self.inner.from_key_hashed_nocheck(hash, key)
197:     }
198:     /// Access an entry by hash.
199:     pub fn from_hash<F>(self, hash: u64, is_match: F) -> Option<(&'a K, &'a V)>
200:     where
201:         F: FnMut(&K) -> bool,
202:     {
203:         self.inner.from_hash(hash, is_match)
204:     }
205:     /// Access an entry by hash, including its index.
206:     pub fn from_hash_full<F>(self, hash: u64, is_match: F) -> Option<(usize, &'a K, &'a V)>
207:     where
208:         F: FnMut(&K) -> bool,
209:     {
210:         self.inner.from_hash_full(hash, is_match)
211:     }
212:     /// Access the index of an entry by hash.
213:     pub fn index_from_hash<F>(self, hash: u64, is_match: F) -> Option<usize>
214:     where
215:         F: FnMut(&K) -> bool,
216:     {
217:         self.inner.index_from_hash(hash, is_match)
218:     }
219: }
220: /// A builder for computing where in an [`OrderMap`] a key-value pair would be stored.
221: ///
222: /// This `struct` is created by the [`OrderMap::raw_entry_mut_v1`] method, provided by the
223: /// [`RawEntryApiV1`] trait. See its documentation for more.
224: pub struct RawEntryBuilderMut<'a, K, V, S> {
225:     inner: ix::RawEntryBuilderMut<'a, K, V, S>,
226: }
227: impl<K, V, S> fmt::Debug for RawEntryBuilderMut<'_, K, V, S> {
228:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
229:         f.debug_struct("RawEntryBuilderMut").finish_non_exhaustive()
230:     }
231: }
232: impl<'a, K, V, S> RawEntryBuilderMut<'a, K, V, S> {
233:     /// Access an entry by key.
234:     pub fn from_key<Q>(self, key: &Q) -> RawEntryMut<'a, K, V, S>
235:     where
236:         S: BuildHasher,
237:         Q: ?Sized + Hash + Equivalent<K>,
238:     {
239:         RawEntryMut::new(self.inner.from_key(key))
240:     }
241:     /// Access an entry by a key and its hash.
242:     pub fn from_key_hashed_nocheck<Q>(self, hash: u64, key: &Q) -> RawEntryMut<'a, K, V, S>
243:     where
244:         Q: ?Sized + Equivalent<K>,
245:     {
246:         RawEntryMut::new(self.inner.from_key_hashed_nocheck(hash, key))
247:     }
248:     /// Access an entry by hash.
249:     pub fn from_hash<F>(self, hash: u64, is_match: F) -> RawEntryMut<'a, K, V, S>
250:     where
251:         F: FnMut(&K) -> bool,
252:     {
253:         RawEntryMut::new(self.inner.from_hash(hash, is_match))
254:     }
255: }
256: /// Raw entry for an existing key-value pair or a vacant location to
257: /// insert one.
258: pub enum RawEntryMut<'a, K, V, S> {
259:     /// Existing slot with equivalent key.
260:     Occupied(RawOccupiedEntryMut<'a, K, V, S>),
261:     /// Vacant slot (no equivalent key in the map).
262:     Vacant(RawVacantEntryMut<'a, K, V, S>),
263: }
264: impl<K: fmt::Debug, V: fmt::Debug, S> fmt::Debug for RawEntryMut<'_, K, V, S> {
265:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
266:         let mut tuple = f.debug_tuple("RawEntryMut");
267:         match self {
268:             Self::Vacant(v) => tuple.field(v),
269:             Self::Occupied(o) => tuple.field(o),
270:         };
271:         tuple.finish()
272:     }
273: }
274: impl<'a, K, V, S> RawEntryMut<'a, K, V, S> {
275:     fn new(entry: ix::RawEntryMut<'a, K, V, S>) -> Self {
276:         match entry {
277:             ix::RawEntryMut::Occupied(inner) => Self::Occupied(RawOccupiedEntryMut { inner }),
278:             ix::RawEntryMut::Vacant(inner) => Self::Vacant(RawVacantEntryMut { inner }),
279:         }
280:     }
281:     /// Return the index where the key-value pair exists or may be inserted.
282:     #[inline]
283:     pub fn index(&self) -> usize {
284:         match self {
285:             Self::Occupied(entry) => entry.index(),
286:             Self::Vacant(entry) => entry.index(),
287:         }
288:     }
289:     /// Inserts the given default key and value in the entry if it is vacant and returns mutable
290:     /// references to them. Otherwise mutable references to an already existent pair are returned.
291:     pub fn or_insert(self, default_key: K, default_value: V) -> (&'a mut K, &'a mut V)
292:     where
293:         K: Hash,
294:         S: BuildHasher,
295:     {
296:         match self {
297:             Self::Occupied(entry) => entry.into_key_value_mut(),
298:             Self::Vacant(entry) => entry.insert(default_key, default_value),
299:         }
300:     }
301:     /// Inserts the result of the `call` function in the entry if it is vacant and returns mutable
302:     /// references to them. Otherwise mutable references to an already existent pair are returned.
303:     pub fn or_insert_with<F>(self, call: F) -> (&'a mut K, &'a mut V)
304:     where
305:         F: FnOnce() -> (K, V),
306:         K: Hash,
307:         S: BuildHasher,
308:     {
309:         match self {
310:             Self::Occupied(entry) => entry.into_key_value_mut(),
311:             Self::Vacant(entry) => {
312:                 let (key, value) = call();
313:                 entry.insert(key, value)
314:             }
315:         }
316:     }
317:     /// Modifies the entry if it is occupied.
318:     pub fn and_modify<F>(mut self, f: F) -> Self
319:     where
320:         F: FnOnce(&mut K, &mut V),
321:     {
322:         if let Self::Occupied(entry) = &mut self {
323:             let (k, v) = entry.get_key_value_mut();
324:             f(k, v);
325:         }
326:         self
327:     }
328: }
329: /// A raw view into an occupied entry in an [`OrderMap`].
330: /// It is part of the [`RawEntryMut`] enum.
331: pub struct RawOccupiedEntryMut<'a, K, V, S> {
332:     inner: ix::RawOccupiedEntryMut<'a, K, V, S>,
333: }
334: impl<K: fmt::Debug, V: fmt::Debug, S> fmt::Debug for RawOccupiedEntryMut<'_, K, V, S> {
335:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
336:         f.debug_struct("RawOccupiedEntryMut")
337:             .field("key", self.key())
338:             .field("value", self.get())
339:             .finish_non_exhaustive()
340:     }
341: }
342: impl<'a, K, V, S> RawOccupiedEntryMut<'a, K, V, S> {
343:     /// Return the index of the key-value pair
344:     #[inline]
345:     pub fn index(&self) -> usize {
346:         self.inner.index()
347:     }
348:     /// Gets a reference to the entry's key in the map.
349:     ///
350:     /// Note that this is not the key that was used to find the entry. There may be an observable
351:     /// difference if the key type has any distinguishing features outside of `Hash` and `Eq`, like
352:     /// extra fields or the memory address of an allocation.
353:     pub fn key(&self) -> &K {
354:         self.inner.key()
355:     }
356:     /// Gets a mutable reference to the entry's key in the map.
357:     ///
358:     /// Note that this is not the key that was used to find the entry. There may be an observable
359:     /// difference if the key type has any distinguishing features outside of `Hash` and `Eq`, like
360:     /// extra fields or the memory address of an allocation.
361:     pub fn key_mut(&mut self) -> &mut K {
362:         self.inner.key_mut()
363:     }
364:     /// Converts into a mutable reference to the entry's key in the map,
365:     /// with a lifetime bound to the map itself.
366:     ///
367:     /// Note that this is not the key that was used to find the entry. There may be an observable
368:     /// difference if the key type has any distinguishing features outside of `Hash` and `Eq`, like
369:     /// extra fields or the memory address of an allocation.
370:     pub fn into_key(self) -> &'a mut K {
371:         self.inner.into_key()
372:     }
373:     /// Gets a reference to the entry's value in the map.
374:     pub fn get(&self) -> &V {
375:         self.inner.get()
376:     }
377:     /// Gets a mutable reference to the entry's value in the map.
378:     ///
379:     /// If you need a reference which may outlive the destruction of the
380:     /// [`RawEntryMut`] value, see [`into_mut`][Self::into_mut].
381:     pub fn get_mut(&mut self) -> &mut V {
382:         self.inner.get_mut()
383:     }
384:     /// Converts into a mutable reference to the entry's value in the map,
385:     /// with a lifetime bound to the map itself.
386:     pub fn into_mut(self) -> &'a mut V {
387:         self.inner.into_mut()
388:     }
389:     /// Gets a reference to the entry's key and value in the map.
390:     pub fn get_key_value(&self) -> (&K, &V) {
391:         self.inner.get_key_value()
392:     }
393:     /// Gets a reference to the entry's key and value in the map.
394:     pub fn get_key_value_mut(&mut self) -> (&mut K, &mut V) {
395:         self.inner.get_key_value_mut()
396:     }
397:     /// Converts into a mutable reference to the entry's key and value in the map,
398:     /// with a lifetime bound to the map itself.
399:     pub fn into_key_value_mut(self) -> (&'a mut K, &'a mut V) {
400:         self.inner.into_key_value_mut()
401:     }
402:     /// Sets the value of the entry, and returns the entry's old value.
403:     pub fn insert(&mut self, value: V) -> V {
404:         self.inner.insert(value)
405:     }
406:     /// Sets the key of the entry, and returns the entry's old key.
407:     pub fn insert_key(&mut self, key: K) -> K {
408:         self.inner.insert_key(key)
409:     }
410:     /// Remove the key, value pair stored in the map for this entry, and return the value.
411:     ///
412:     /// **NOTE:** This is equivalent to indexmap's
413:     /// [`RawOccupiedEntryMut::shift_remove`][ix::RawOccupiedEntryMut::shift_remove], and
414:     /// like [`Vec::remove`], the pair is removed by shifting all of the
415:     /// elements that follow it, preserving their relative order.
416:     /// **This perturbs the index of all of those elements!**
417:     ///
418:     /// Computes in **O(n)** time (average).
419:     pub fn remove(self) -> V {
420:         self.inner.shift_remove()
421:     }
422:     /// Remove the key, value pair stored in the map for this entry, and return the value.
423:     ///
424:     /// Like [`Vec::swap_remove`], the pair is removed by swapping it with
425:     /// the last element of the map and popping it off.
426:     /// **This perturbs the position of what used to be the last element!**
427:     ///
428:     /// Computes in **O(1)** time (average).
429:     pub fn swap_remove(self) -> V {
430:         self.inner.swap_remove()
431:     }
432:     /// Remove and return the key, value pair stored in the map for this entry
433:     ///
434:     /// **NOTE:** This is equivalent to indexmap's
435:     /// [`RawOccupiedEntryMut::shift_remove_entry`][ix::RawOccupiedEntryMut::shift_remove_entry], and
436:     /// like [`Vec::remove`], the pair is removed by shifting all of the
437:     /// elements that follow it, preserving their relative order.
438:     /// **This perturbs the index of all of those elements!**
439:     ///
440:     /// Computes in **O(n)** time (average).
441:     pub fn remove_entry(self) -> (K, V) {
442:         self.inner.shift_remove_entry()
443:     }
444:     /// Remove and return the key, value pair stored in the map for this entry
445:     ///
446:     /// Like [`Vec::swap_remove`], the pair is removed by swapping it with
447:     /// the last element of the map and popping it off.
448:     /// **This perturbs the position of what used to be the last element!**
449:     ///
450:     /// Computes in **O(1)** time (average).
451:     pub fn swap_remove_entry(self) -> (K, V) {
452:         self.inner.swap_remove_entry()
453:     }
454:     /// Moves the position of the entry to a new index
455:     /// by shifting all other entries in-between.
456:     ///
457:     /// This is equivalent to [`OrderMap::move_index`]
458:     /// coming `from` the current [`.index()`][Self::index].
459:     ///
460:     /// * If `self.index() < to`, the other pairs will shift down while the targeted pair moves up.
461:     /// * If `self.index() > to`, the other pairs will shift up while the targeted pair moves down.
462:     ///
463:     /// ***Panics*** if `to` is out of bounds.
464:     ///
465:     /// Computes in **O(n)** time (average).
466:     #[track_caller]
467:     pub fn move_index(self, to: usize) {
468:         self.inner.move_index(to);
469:     }
470:     /// Swaps the position of entry with another.
471:     ///
472:     /// This is equivalent to [`OrderMap::swap_indices`]
473:     /// with the current [`.index()`][Self::index] as one of the two being swapped.
474:     ///
475:     /// ***Panics*** if the `other` index is out of bounds.
476:     ///
477:     /// Computes in **O(1)** time (average).
478:     #[track_caller]
479:     pub fn swap_indices(self, other: usize) {
480:         self.inner.swap_indices(other);
481:     }
482: }
483: /// A view into a vacant raw entry in an [`OrderMap`].
484: /// It is part of the [`RawEntryMut`] enum.
485: pub struct RawVacantEntryMut<'a, K, V, S> {
486:     inner: ix::RawVacantEntryMut<'a, K, V, S>,
487: }
488: impl<K, V, S> fmt::Debug for RawVacantEntryMut<'_, K, V, S> {
489:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
490:         f.debug_struct("RawVacantEntryMut").finish_non_exhaustive()
491:     }
492: }
493: impl<'a, K, V, S> RawVacantEntryMut<'a, K, V, S> {
494:     /// Return the index where a key-value pair may be inserted.
495:     pub fn index(&self) -> usize {
496:         self.inner.index()
497:     }
498:     /// Inserts the given key and value into the map,
499:     /// and returns mutable references to them.
500:     pub fn insert(self, key: K, value: V) -> (&'a mut K, &'a mut V)
501:     where
502:         K: Hash,
503:         S: BuildHasher,
504:     {
505:         self.inner.insert(key, value)
506:     }
507:     /// Inserts the given key and value into the map with the provided hash,
508:     /// and returns mutable references to them.
509:     pub fn insert_hashed_nocheck(self, hash: u64, key: K, value: V) -> (&'a mut K, &'a mut V) {
510:         self.inner.insert_hashed_nocheck(hash, key, value)
511:     }
512:     /// Inserts the given key and value into the map at the given index,
513:     /// shifting others to the right, and returns mutable references to them.
514:     ///
515:     /// ***Panics*** if `index` is out of bounds.
516:     ///
517:     /// Computes in **O(n)** time (average).
518:     #[track_caller]
519:     pub fn shift_insert(self, index: usize, key: K, value: V) -> (&'a mut K, &'a mut V)
520:     where
521:         K: Hash,
522:         S: BuildHasher,
523:     {
524:         self.inner.shift_insert(index, key, value)
525:     }
526:     /// Inserts the given key and value into the map with the provided hash
527:     /// at the given index, and returns mutable references to them.
528:     ///
529:     /// ***Panics*** if `index` is out of bounds.
530:     ///
531:     /// Computes in **O(n)** time (average).
532:     #[track_caller]
533:     pub fn shift_insert_hashed_nocheck(
534:         self,
535:         index: usize,
536:         hash: u64,
537:         key: K,
538:         value: V,
539:     ) -> (&'a mut K, &'a mut V) {
540:         self.inner
541:             .shift_insert_hashed_nocheck(index, hash, key, value)
542:     }
543: }
544: trait Sealed {}
545: impl<K, V, S> Sealed for OrderMap<K, V, S> {}
````

## File: src/map/rayon.rs
````rust
  1: //! Parallel iterator types for [`OrderMap`] with [`rayon`].
  2: //!
  3: //! You will rarely need to interact with this module directly unless you need to name one of the
  4: //! iterator types.
  5: pub use indexmap::map::rayon::{
  6:     IntoParIter, ParDrain, ParIter, ParIterMut, ParKeys, ParValues, ParValuesMut,
  7: };
  8: use super::OrderMap;
  9: use core::cmp::Ordering;
 10: use core::hash::{BuildHasher, Hash};
 11: use core::ops::RangeBounds;
 12: use rayon::prelude::*;
 13: impl<K, V, S> IntoParallelIterator for OrderMap<K, V, S>
 14: where
 15:     K: Send,
 16:     V: Send,
 17: {
 18:     type Item = (K, V);
 19:     type Iter = IntoParIter<K, V>;
 20:     fn into_par_iter(self) -> Self::Iter {
 21:         self.inner.into_par_iter()
 22:     }
 23: }
 24: impl<'a, K, V, S> IntoParallelIterator for &'a OrderMap<K, V, S>
 25: where
 26:     K: Sync,
 27:     V: Sync,
 28: {
 29:     type Item = (&'a K, &'a V);
 30:     type Iter = ParIter<'a, K, V>;
 31:     fn into_par_iter(self) -> Self::Iter {
 32:         self.inner.par_iter()
 33:     }
 34: }
 35: impl<'a, K, V, S> IntoParallelIterator for &'a mut OrderMap<K, V, S>
 36: where
 37:     K: Sync + Send,
 38:     V: Send,
 39: {
 40:     type Item = (&'a K, &'a mut V);
 41:     type Iter = ParIterMut<'a, K, V>;
 42:     fn into_par_iter(self) -> Self::Iter {
 43:         self.inner.par_iter_mut()
 44:     }
 45: }
 46: impl<'a, K, V, S> ParallelDrainRange<usize> for &'a mut OrderMap<K, V, S>
 47: where
 48:     K: Send,
 49:     V: Send,
 50: {
 51:     type Item = (K, V);
 52:     type Iter = ParDrain<'a, K, V>;
 53:     fn par_drain<R: RangeBounds<usize>>(self, range: R) -> Self::Iter {
 54:         self.inner.par_drain(range)
 55:     }
 56: }
 57: /// Parallel iterator methods and other parallel methods.
 58: ///
 59: /// The following methods **require crate feature `"rayon"`**.
 60: ///
 61: /// See also the `IntoParallelIterator` implementations.
 62: impl<K, V, S> OrderMap<K, V, S>
 63: where
 64:     K: Sync,
 65:     V: Sync,
 66: {
 67:     /// Return a parallel iterator over the keys of the map.
 68:     ///
 69:     /// While parallel iterators can process items in any order, their relative order
 70:     /// in the map is still preserved for operations like `reduce` and `collect`.
 71:     pub fn par_keys(&self) -> ParKeys<'_, K, V> {
 72:         self.inner.par_keys()
 73:     }
 74:     /// Return a parallel iterator over the values of the map.
 75:     ///
 76:     /// While parallel iterators can process items in any order, their relative order
 77:     /// in the map is still preserved for operations like `reduce` and `collect`.
 78:     pub fn par_values(&self) -> ParValues<'_, K, V> {
 79:         self.inner.par_values()
 80:     }
 81: }
 82: impl<K, V, S> OrderMap<K, V, S>
 83: where
 84:     K: PartialEq + Sync,
 85:     V: Sync,
 86: {
 87:     /// Returns `true` if `self` contains all of the same key-value pairs as `other`,
 88:     /// in the same indexed order, determined in parallel.
 89:     pub fn par_eq<S2>(&self, other: &OrderMap<K, V, S2>) -> bool
 90:     where
 91:         V: PartialEq,
 92:     {
 93:         self.len() == other.len() && self.par_iter().eq(other)
 94:     }
 95: }
 96: impl<K, V, S> OrderMap<K, V, S>
 97: where
 98:     K: Send,
 99:     V: Send,
100: {
101:     /// Return a parallel iterator over mutable references to the values of the map
102:     ///
103:     /// While parallel iterators can process items in any order, their relative order
104:     /// in the map is still preserved for operations like `reduce` and `collect`.
105:     pub fn par_values_mut(&mut self) -> ParValuesMut<'_, K, V> {
106:         self.inner.par_values_mut()
107:     }
108: }
109: impl<K, V, S> OrderMap<K, V, S>
110: where
111:     K: Send,
112:     V: Send,
113: {
114:     /// Sort the map's key-value pairs in parallel, by the default ordering of the keys.
115:     pub fn par_sort_keys(&mut self)
116:     where
117:         K: Ord,
118:     {
119:         self.inner.par_sort_keys();
120:     }
121:     /// Sort the map's key-value pairs in place and in parallel, using the comparison
122:     /// function `cmp`.
123:     ///
124:     /// The comparison function receives two key and value pairs to compare (you
125:     /// can sort by keys or values or their combination as needed).
126:     pub fn par_sort_by<F>(&mut self, cmp: F)
127:     where
128:         F: Fn(&K, &V, &K, &V) -> Ordering + Sync,
129:     {
130:         self.inner.par_sort_by(cmp);
131:     }
132:     /// Sort the key-value pairs of the map in parallel and return a by-value parallel
133:     /// iterator of the key-value pairs with the result.
134:     pub fn par_sorted_by<F>(self, cmp: F) -> IntoParIter<K, V>
135:     where
136:         F: Fn(&K, &V, &K, &V) -> Ordering + Sync,
137:     {
138:         self.inner.par_sorted_by(cmp)
139:     }
140:     /// Sort the map's key-value pairs in place and in parallel, using a sort-key extraction
141:     /// function.
142:     pub fn par_sort_by_key<T, F>(&mut self, sort_key: F)
143:     where
144:         T: Ord,
145:         F: Fn(&K, &V) -> T + Sync,
146:     {
147:         self.inner.par_sort_by_key(sort_key)
148:     }
149:     /// Sort the map's key-value pairs in parallel, by the default ordering of the keys.
150:     pub fn par_sort_unstable_keys(&mut self)
151:     where
152:         K: Ord,
153:     {
154:         self.inner.par_sort_unstable_keys();
155:     }
156:     /// Sort the map's key-value pairs in place and in parallel, using the comparison
157:     /// function `cmp`.
158:     ///
159:     /// The comparison function receives two key and value pairs to compare (you
160:     /// can sort by keys or values or their combination as needed).
161:     pub fn par_sort_unstable_by<F>(&mut self, cmp: F)
162:     where
163:         F: Fn(&K, &V, &K, &V) -> Ordering + Sync,
164:     {
165:         self.inner.par_sort_unstable_by(cmp);
166:     }
167:     /// Sort the key-value pairs of the map in parallel and return a by-value parallel
168:     /// iterator of the key-value pairs with the result.
169:     pub fn par_sorted_unstable_by<F>(self, cmp: F) -> IntoParIter<K, V>
170:     where
171:         F: Fn(&K, &V, &K, &V) -> Ordering + Sync,
172:     {
173:         self.inner.par_sorted_unstable_by(cmp)
174:     }
175:     /// Sort the map's key-value pairs in place and in parallel, using a sort-key extraction
176:     /// function.
177:     pub fn par_sort_unstable_by_key<T, F>(&mut self, sort_key: F)
178:     where
179:         T: Ord,
180:         F: Fn(&K, &V) -> T + Sync,
181:     {
182:         self.inner.par_sort_unstable_by_key(sort_key)
183:     }
184:     /// Sort the map's key-value pairs in place and in parallel, using a sort-key extraction
185:     /// function.
186:     pub fn par_sort_by_cached_key<T, F>(&mut self, sort_key: F)
187:     where
188:         T: Ord + Send,
189:         F: Fn(&K, &V) -> T + Sync,
190:     {
191:         self.inner.par_sort_by_cached_key(sort_key)
192:     }
193: }
194: impl<K, V, S> FromParallelIterator<(K, V)> for OrderMap<K, V, S>
195: where
196:     K: Eq + Hash + Send,
197:     V: Send,
198:     S: BuildHasher + Default + Send,
199: {
200:     fn from_par_iter<I>(iter: I) -> Self
201:     where
202:         I: IntoParallelIterator<Item = (K, V)>,
203:     {
204:         Self {
205:             inner: <_>::from_par_iter(iter),
206:         }
207:     }
208: }
209: impl<K, V, S> ParallelExtend<(K, V)> for OrderMap<K, V, S>
210: where
211:     K: Eq + Hash + Send,
212:     V: Send,
213:     S: BuildHasher + Send,
214: {
215:     fn par_extend<I>(&mut self, iter: I)
216:     where
217:         I: IntoParallelIterator<Item = (K, V)>,
218:     {
219:         self.inner.par_extend(iter);
220:     }
221: }
222: impl<'a, K: 'a, V: 'a, S> ParallelExtend<(&'a K, &'a V)> for OrderMap<K, V, S>
223: where
224:     K: Copy + Eq + Hash + Send + Sync,
225:     V: Copy + Send + Sync,
226:     S: BuildHasher + Send,
227: {
228:     fn par_extend<I>(&mut self, iter: I)
229:     where
230:         I: IntoParallelIterator<Item = (&'a K, &'a V)>,
231:     {
232:         self.inner.par_extend(iter);
233:     }
234: }
235: #[cfg(test)]
236: mod tests {
237:     use super::*;
238:     use std::vec::Vec;
239:     #[test]
240:     fn insert_order() {
241:         let insert = [0, 4, 2, 12, 8, 7, 11, 5, 3, 17, 19, 22, 23];
242:         let mut map = OrderMap::new();
243:         for &elt in &insert {
244:             map.insert(elt, ());
245:         }
246:         assert_eq!(map.par_keys().count(), map.len());
247:         assert_eq!(map.par_keys().count(), insert.len());
248:         insert.par_iter().zip(map.par_keys()).for_each(|(a, b)| {
249:             assert_eq!(a, b);
250:         });
251:         (0..insert.len())
252:             .into_par_iter()
253:             .zip(map.par_keys())
254:             .for_each(|(i, k)| {
255:                 assert_eq!(map.get_index(i).unwrap().0, k);
256:             });
257:     }
258:     #[test]
259:     fn partial_eq_and_eq() {
260:         let mut map_a = OrderMap::new();
261:         map_a.insert(1, "1");
262:         map_a.insert(2, "2");
263:         let mut map_b = map_a.clone();
264:         assert!(map_a.par_eq(&map_b));
265:         map_b.swap_remove(&1);
266:         assert!(!map_a.par_eq(&map_b));
267:         map_b.insert(3, "3");
268:         assert!(!map_a.par_eq(&map_b));
269:     }
270:     #[test]
271:     fn extend() {
272:         let mut map = OrderMap::new();
273:         map.par_extend(vec![(&1, &2), (&3, &4)]);
274:         map.par_extend(vec![(5, 6)]);
275:         assert_eq!(
276:             map.into_par_iter().collect::<Vec<_>>(),
277:             vec![(1, 2), (3, 4), (5, 6)]
278:         );
279:     }
280:     #[test]
281:     fn keys() {
282:         let vec = vec![(1, 'a'), (2, 'b'), (3, 'c')];
283:         let map: OrderMap<_, _> = vec.into_par_iter().collect();
284:         let keys: Vec<_> = map.par_keys().copied().collect();
285:         assert_eq!(keys.len(), 3);
286:         assert!(keys.contains(&1));
287:         assert!(keys.contains(&2));
288:         assert!(keys.contains(&3));
289:     }
290:     #[test]
291:     fn values() {
292:         let vec = vec![(1, 'a'), (2, 'b'), (3, 'c')];
293:         let map: OrderMap<_, _> = vec.into_par_iter().collect();
294:         let values: Vec<_> = map.par_values().copied().collect();
295:         assert_eq!(values.len(), 3);
296:         assert!(values.contains(&'a'));
297:         assert!(values.contains(&'b'));
298:         assert!(values.contains(&'c'));
299:     }
300:     #[test]
301:     fn values_mut() {
302:         let vec = vec![(1, 1), (2, 2), (3, 3)];
303:         let mut map: OrderMap<_, _> = vec.into_par_iter().collect();
304:         map.par_values_mut().for_each(|value| *value *= 2);
305:         let values: Vec<_> = map.par_values().copied().collect();
306:         assert_eq!(values.len(), 3);
307:         assert!(values.contains(&2));
308:         assert!(values.contains(&4));
309:         assert!(values.contains(&6));
310:     }
311: }
````

## File: src/map/serde_seq.rs
````rust
 1: //! Functions to serialize and deserialize an [`OrderMap`] as an ordered sequence.
 2: //!
 3: //! The default `serde` implementation serializes `OrderMap` as a normal map,
 4: //! but there is no guarantee that serialization formats will preserve the order
 5: //! of the key-value pairs. This module serializes `OrderMap` as a sequence of
 6: //! `(key, value)` elements instead, in order.
 7: //!
 8: //! This module may be used in a field attribute for derived implementations:
 9: //!
10: //! ```
11: //! # use ordermap::OrderMap;
12: //! # use serde::{Deserialize, Serialize};
13: //! #[derive(Deserialize, Serialize)]
14: //! struct Data {
15: //!     #[serde(with = "ordermap::map::serde_seq")]
16: //!     map: OrderMap<i32, u64>,
17: //!     // ...
18: //! }
19: //! ```
20: use crate::OrderMap;
21: use core::hash::{BuildHasher, Hash};
22: use indexmap::map::serde_seq as ix;
23: use serde_core::de::{Deserialize, Deserializer};
24: use serde_core::ser::{Serialize, Serializer};
25: /// Serializes an [`OrderMap`] as an ordered sequence.
26: ///
27: /// This function may be used in a field attribute for deriving [`Serialize`]:
28: ///
29: /// ```
30: /// # use ordermap::OrderMap;
31: /// # use serde::Serialize;
32: /// #[derive(Serialize)]
33: /// struct Data {
34: ///     #[serde(serialize_with = "ordermap::map::serde_seq::serialize")]
35: ///     map: OrderMap<i32, u64>,
36: ///     // ...
37: /// }
38: /// ```
39: pub fn serialize<K, V, S, T>(map: &OrderMap<K, V, S>, serializer: T) -> Result<T::Ok, T::Error>
40: where
41:     K: Serialize,
42:     V: Serialize,
43:     T: Serializer,
44: {
45:     ix::serialize(&map.inner, serializer)
46: }
47: /// Deserializes an [`OrderMap`] from an ordered sequence.
48: ///
49: /// This function may be used in a field attribute for deriving [`Deserialize`]:
50: ///
51: /// ```
52: /// # use ordermap::OrderMap;
53: /// # use serde::Deserialize;
54: /// #[derive(Deserialize)]
55: /// struct Data {
56: ///     #[serde(deserialize_with = "ordermap::map::serde_seq::deserialize")]
57: ///     map: OrderMap<i32, u64>,
58: ///     // ...
59: /// }
60: /// ```
61: pub fn deserialize<'de, D, K, V, S>(deserializer: D) -> Result<OrderMap<K, V, S>, D::Error>
62: where
63:     D: Deserializer<'de>,
64:     K: Deserialize<'de> + Eq + Hash,
65:     V: Deserialize<'de>,
66:     S: Default + BuildHasher,
67: {
68:     Ok(OrderMap {
69:         inner: ix::deserialize(deserializer)?,
70:     })
71: }
````

## File: src/map/slice.rs
````rust
  1: use super::OrderMap;
  2: use core::ops::{self, Bound, Index, IndexMut};
  3: use indexmap::map::Slice;
  4: // We can't have `impl<I: RangeBounds<usize>> Index<I>` because that conflicts
  5: // both upstream with `Index<usize>` and downstream with `Index<&Q>`.
  6: // Instead, we repeat the implementations for all the core range types.
  7: macro_rules! impl_index {
  8:     ($($range:ty),*) => {$(
  9:         impl<K, V, S> Index<$range> for OrderMap<K, V, S> {
 10:             type Output = Slice<K, V>;
 11:             fn index(&self, range: $range) -> &Self::Output {
 12:                 &self.inner[range]
 13:             }
 14:         }
 15:         impl<K, V, S> IndexMut<$range> for OrderMap<K, V, S> {
 16:             fn index_mut(&mut self, range: $range) -> &mut Self::Output {
 17:                 &mut self.inner[range]
 18:             }
 19:         }
 20:     )*}
 21: }
 22: impl_index!(
 23:     ops::Range<usize>,
 24:     ops::RangeFrom<usize>,
 25:     ops::RangeFull,
 26:     ops::RangeInclusive<usize>,
 27:     ops::RangeTo<usize>,
 28:     ops::RangeToInclusive<usize>,
 29:     (Bound<usize>, Bound<usize>)
 30: );
 31: #[cfg(test)]
 32: mod tests {
 33:     use super::*;
 34:     use alloc::vec::Vec;
 35:     #[test]
 36:     fn slice_index() {
 37:         fn check(
 38:             vec_slice: &[(i32, i32)],
 39:             map_slice: &Slice<i32, i32>,
 40:             sub_slice: &Slice<i32, i32>,
 41:         ) {
 42:             assert_eq!(map_slice as *const _, sub_slice as *const _);
 43:             itertools::assert_equal(
 44:                 vec_slice.iter().copied(),
 45:                 map_slice.iter().map(|(&k, &v)| (k, v)),
 46:             );
 47:             itertools::assert_equal(vec_slice.iter().map(|(k, _)| k), map_slice.keys());
 48:             itertools::assert_equal(vec_slice.iter().map(|(_, v)| v), map_slice.values());
 49:         }
 50:         let vec: Vec<(i32, i32)> = (0..10).map(|i| (i, i * i)).collect();
 51:         let map: OrderMap<i32, i32> = vec.iter().cloned().collect();
 52:         let slice = map.as_slice();
 53:         // RangeFull
 54:         #[expect(clippy::redundant_slicing)]
 55:         check(&vec[..], &map[..], &slice[..]);
 56:         for i in 0usize..10 {
 57:             // Index
 58:             assert_eq!(vec[i].1, map[i]);
 59:             assert_eq!(vec[i].1, slice[i]);
 60:             assert_eq!(map[&(i as i32)], map[i]);
 61:             assert_eq!(map[&(i as i32)], slice[i]);
 62:             // RangeFrom
 63:             check(&vec[i..], &map[i..], &slice[i..]);
 64:             // RangeTo
 65:             check(&vec[..i], &map[..i], &slice[..i]);
 66:             // RangeToInclusive
 67:             check(&vec[..=i], &map[..=i], &slice[..=i]);
 68:             // (Bound<usize>, Bound<usize>)
 69:             let bounds = (Bound::Excluded(i), Bound::Unbounded);
 70:             check(&vec[i + 1..], &map[bounds], &slice[bounds]);
 71:             for j in i..=10 {
 72:                 // Range
 73:                 check(&vec[i..j], &map[i..j], &slice[i..j]);
 74:             }
 75:             for j in i..10 {
 76:                 // RangeInclusive
 77:                 check(&vec[i..=j], &map[i..=j], &slice[i..=j]);
 78:             }
 79:         }
 80:     }
 81:     #[test]
 82:     fn slice_index_mut() {
 83:         fn check_mut(
 84:             vec_slice: &[(i32, i32)],
 85:             map_slice: &mut Slice<i32, i32>,
 86:             sub_slice: &mut Slice<i32, i32>,
 87:         ) {
 88:             assert_eq!(map_slice, sub_slice);
 89:             itertools::assert_equal(
 90:                 vec_slice.iter().copied(),
 91:                 map_slice.iter_mut().map(|(&k, &mut v)| (k, v)),
 92:             );
 93:             itertools::assert_equal(
 94:                 vec_slice.iter().map(|&(_, v)| v),
 95:                 map_slice.values_mut().map(|&mut v| v),
 96:             );
 97:         }
 98:         let vec: Vec<(i32, i32)> = (0..10).map(|i| (i, i * i)).collect();
 99:         let mut map: OrderMap<i32, i32> = vec.iter().cloned().collect();
100:         let mut map2 = map.clone();
101:         let slice = map2.as_mut_slice();
102:         // RangeFull
103:         check_mut(&vec[..], &mut map[..], &mut slice[..]);
104:         for i in 0usize..10 {
105:             // IndexMut
106:             assert_eq!(&mut map[i], &mut slice[i]);
107:             // RangeFrom
108:             check_mut(&vec[i..], &mut map[i..], &mut slice[i..]);
109:             // RangeTo
110:             check_mut(&vec[..i], &mut map[..i], &mut slice[..i]);
111:             // RangeToInclusive
112:             check_mut(&vec[..=i], &mut map[..=i], &mut slice[..=i]);
113:             // (Bound<usize>, Bound<usize>)
114:             let bounds = (Bound::Excluded(i), Bound::Unbounded);
115:             check_mut(&vec[i + 1..], &mut map[bounds], &mut slice[bounds]);
116:             for j in i..=10 {
117:                 // Range
118:                 check_mut(&vec[i..j], &mut map[i..j], &mut slice[i..j]);
119:             }
120:             for j in i..10 {
121:                 // RangeInclusive
122:                 check_mut(&vec[i..=j], &mut map[i..=j], &mut slice[i..=j]);
123:             }
124:         }
125:     }
126:     #[test]
127:     fn slice_new() {
128:         let slice: &Slice<i32, i32> = Slice::new();
129:         assert!(slice.is_empty());
130:         assert_eq!(slice.len(), 0);
131:     }
132:     #[test]
133:     fn slice_new_mut() {
134:         let slice: &mut Slice<i32, i32> = Slice::new_mut();
135:         assert!(slice.is_empty());
136:         assert_eq!(slice.len(), 0);
137:     }
138:     #[test]
139:     fn slice_get_index_mut() {
140:         let mut map: OrderMap<i32, i32> = (0..10).map(|i| (i, i * i)).collect();
141:         let slice: &mut Slice<i32, i32> = map.as_mut_slice();
142:         {
143:             let (key, value) = slice.get_index_mut(0).unwrap();
144:             assert_eq!(*key, 0);
145:             assert_eq!(*value, 0);
146:             *value = 11;
147:         }
148:         assert_eq!(slice[0], 11);
149:         {
150:             let result = slice.get_index_mut(11);
151:             assert!(result.is_none());
152:         }
153:     }
154:     #[test]
155:     fn slice_split_first() {
156:         let slice: &mut Slice<i32, i32> = Slice::new_mut();
157:         let result = slice.split_first();
158:         assert!(result.is_none());
159:         let mut map: OrderMap<i32, i32> = (0..10).map(|i| (i, i * i)).collect();
160:         let slice: &mut Slice<i32, i32> = map.as_mut_slice();
161:         {
162:             let (first, rest) = slice.split_first().unwrap();
163:             assert_eq!(first, (&0, &0));
164:             assert_eq!(rest.len(), 9);
165:         }
166:         assert_eq!(slice.len(), 10);
167:     }
168:     #[test]
169:     fn slice_split_first_mut() {
170:         let slice: &mut Slice<i32, i32> = Slice::new_mut();
171:         let result = slice.split_first_mut();
172:         assert!(result.is_none());
173:         let mut map: OrderMap<i32, i32> = (0..10).map(|i| (i, i * i)).collect();
174:         let slice: &mut Slice<i32, i32> = map.as_mut_slice();
175:         {
176:             let (first, rest) = slice.split_first_mut().unwrap();
177:             assert_eq!(first, (&0, &mut 0));
178:             assert_eq!(rest.len(), 9);
179:             *first.1 = 11;
180:         }
181:         assert_eq!(slice.len(), 10);
182:         assert_eq!(slice[0], 11);
183:     }
184:     #[test]
185:     fn slice_split_last() {
186:         let slice: &mut Slice<i32, i32> = Slice::new_mut();
187:         let result = slice.split_last();
188:         assert!(result.is_none());
189:         let mut map: OrderMap<i32, i32> = (0..10).map(|i| (i, i * i)).collect();
190:         let slice: &mut Slice<i32, i32> = map.as_mut_slice();
191:         {
192:             let (last, rest) = slice.split_last().unwrap();
193:             assert_eq!(last, (&9, &81));
194:             assert_eq!(rest.len(), 9);
195:         }
196:         assert_eq!(slice.len(), 10);
197:     }
198:     #[test]
199:     fn slice_split_last_mut() {
200:         let slice: &mut Slice<i32, i32> = Slice::new_mut();
201:         let result = slice.split_last_mut();
202:         assert!(result.is_none());
203:         let mut map: OrderMap<i32, i32> = (0..10).map(|i| (i, i * i)).collect();
204:         let slice: &mut Slice<i32, i32> = map.as_mut_slice();
205:         {
206:             let (last, rest) = slice.split_last_mut().unwrap();
207:             assert_eq!(last, (&9, &mut 81));
208:             assert_eq!(rest.len(), 9);
209:             *last.1 = 100;
210:         }
211:         assert_eq!(slice.len(), 10);
212:         assert_eq!(slice[slice.len() - 1], 100);
213:     }
214:     #[test]
215:     fn slice_get_range() {
216:         let mut map: OrderMap<i32, i32> = (0..10).map(|i| (i, i * i)).collect();
217:         let slice: &mut Slice<i32, i32> = map.as_mut_slice();
218:         let subslice = slice.get_range(3..6).unwrap();
219:         assert_eq!(subslice.len(), 3);
220:         assert_eq!(subslice, &[(3, 9), (4, 16), (5, 25)]);
221:     }
222: }
````

## File: src/map/tests.rs
````rust
   1: use super::*;
   2: use std::string::String;
   3: use std::vec::Vec;
   4: #[test]
   5: fn it_works() {
   6:     let mut map = OrderMap::new();
   7:     assert_eq!(map.is_empty(), true);
   8:     map.insert(1, ());
   9:     map.insert(1, ());
  10:     assert_eq!(map.len(), 1);
  11:     assert!(map.get(&1).is_some());
  12:     assert_eq!(map.is_empty(), false);
  13: }
  14: #[test]
  15: fn new() {
  16:     let map = OrderMap::<String, String>::new();
  17:     println!("{:?}", map);
  18:     assert_eq!(map.capacity(), 0);
  19:     assert_eq!(map.len(), 0);
  20:     assert_eq!(map.is_empty(), true);
  21: }
  22: #[test]
  23: fn insert() {
  24:     let insert = [0, 4, 2, 12, 8, 7, 11, 5];
  25:     let not_present = [1, 3, 6, 9, 10];
  26:     let mut map = OrderMap::with_capacity(insert.len());
  27:     for (i, &elt) in insert.iter().enumerate() {
  28:         assert_eq!(map.len(), i);
  29:         map.insert(elt, elt);
  30:         assert_eq!(map.len(), i + 1);
  31:         assert_eq!(map.get(&elt), Some(&elt));
  32:         assert_eq!(map[&elt], elt);
  33:     }
  34:     println!("{:?}", map);
  35:     for &elt in &not_present {
  36:         assert!(map.get(&elt).is_none());
  37:     }
  38: }
  39: #[test]
  40: fn insert_full() {
  41:     let insert = [9, 2, 7, 1, 4, 6, 13];
  42:     let present = [1, 6, 2];
  43:     let mut map = OrderMap::with_capacity(insert.len());
  44:     for (i, &elt) in insert.iter().enumerate() {
  45:         assert_eq!(map.len(), i);
  46:         let (index, existing) = map.insert_full(elt, elt);
  47:         assert_eq!(existing, None);
  48:         assert_eq!(Some(index), map.get_full(&elt).map(|x| x.0));
  49:         assert_eq!(map.len(), i + 1);
  50:     }
  51:     let len = map.len();
  52:     for &elt in &present {
  53:         let (index, existing) = map.insert_full(elt, elt);
  54:         assert_eq!(existing, Some(elt));
  55:         assert_eq!(Some(index), map.get_full(&elt).map(|x| x.0));
  56:         assert_eq!(map.len(), len);
  57:     }
  58: }
  59: #[test]
  60: fn insert_2() {
  61:     let mut map = OrderMap::with_capacity(16);
  62:     let mut keys = vec![];
  63:     keys.extend(0..16);
  64:     keys.extend(if cfg!(miri) { 32..64 } else { 128..267 });
  65:     for &i in &keys {
  66:         let old_map = map.clone();
  67:         map.insert(i, ());
  68:         for key in old_map.keys() {
  69:             if map.get(key).is_none() {
  70:                 println!("old_map: {:?}", old_map);
  71:                 println!("map: {:?}", map);
  72:                 panic!("did not find {} in map", key);
  73:             }
  74:         }
  75:     }
  76:     for &i in &keys {
  77:         assert!(map.get(&i).is_some(), "did not find {}", i);
  78:     }
  79: }
  80: #[test]
  81: fn insert_order() {
  82:     let insert = [0, 4, 2, 12, 8, 7, 11, 5, 3, 17, 19, 22, 23];
  83:     let mut map = OrderMap::new();
  84:     for &elt in &insert {
  85:         map.insert(elt, ());
  86:     }
  87:     assert_eq!(map.keys().count(), map.len());
  88:     assert_eq!(map.keys().count(), insert.len());
  89:     for (a, b) in insert.iter().zip(map.keys()) {
  90:         assert_eq!(a, b);
  91:     }
  92:     for (i, k) in (0..insert.len()).zip(map.keys()) {
  93:         assert_eq!(map.get_index(i).unwrap().0, k);
  94:     }
  95: }
  96: #[test]
  97: fn shift_insert() {
  98:     let insert = [0, 4, 2, 12, 8, 7, 11, 5, 3, 17, 19, 22, 23];
  99:     let mut map = OrderMap::new();
 100:     for &elt in &insert {
 101:         map.shift_insert(0, elt, ());
 102:     }
 103:     assert_eq!(map.keys().count(), map.len());
 104:     assert_eq!(map.keys().count(), insert.len());
 105:     for (a, b) in insert.iter().rev().zip(map.keys()) {
 106:         assert_eq!(a, b);
 107:     }
 108:     for (i, k) in (0..insert.len()).zip(map.keys()) {
 109:         assert_eq!(map.get_index(i).unwrap().0, k);
 110:     }
 111:     // "insert" that moves an existing entry
 112:     map.shift_insert(0, insert[0], ());
 113:     assert_eq!(map.keys().count(), insert.len());
 114:     assert_eq!(insert[0], map.keys()[0]);
 115:     for (a, b) in insert[1..].iter().rev().zip(map.keys().skip(1)) {
 116:         assert_eq!(a, b);
 117:     }
 118: }
 119: #[test]
 120: fn insert_sorted_bad() {
 121:     let mut map = OrderMap::new();
 122:     map.insert(10, ());
 123:     for i in 0..10 {
 124:         map.insert(i, ());
 125:     }
 126:     // The binary search will want to insert this at the end (index == len()),
 127:     // but that's only possible for *new* inserts. It should still be handled
 128:     // without panicking though, and in this case it's simple enough that we
 129:     // know the exact result. (But don't read this as an API guarantee!)
 130:     assert_eq!(map.first(), Some((&10, &())));
 131:     map.insert_sorted(10, ());
 132:     assert_eq!(map.last(), Some((&10, &())));
 133:     assert!(map.keys().copied().eq(0..=10));
 134:     // Other out-of-order entries can also "insert" to a binary-searched
 135:     // position, moving in either direction.
 136:     map.move_index(5, 0);
 137:     map.move_index(6, 10);
 138:     assert_eq!(map.first(), Some((&5, &())));
 139:     assert_eq!(map.last(), Some((&6, &())));
 140:     map.insert_sorted(5, ()); // moves back up
 141:     map.insert_sorted(6, ()); // moves back down
 142:     assert!(map.keys().copied().eq(0..=10));
 143: }
 144: #[test]
 145: fn grow() {
 146:     let insert = [0, 4, 2, 12, 8, 7, 11];
 147:     let not_present = [1, 3, 6, 9, 10];
 148:     let mut map = OrderMap::with_capacity(insert.len());
 149:     for (i, &elt) in insert.iter().enumerate() {
 150:         assert_eq!(map.len(), i);
 151:         map.insert(elt, elt);
 152:         assert_eq!(map.len(), i + 1);
 153:         assert_eq!(map.get(&elt), Some(&elt));
 154:         assert_eq!(map[&elt], elt);
 155:     }
 156:     println!("{:?}", map);
 157:     for &elt in &insert {
 158:         map.insert(elt * 10, elt);
 159:     }
 160:     for &elt in &insert {
 161:         map.insert(elt * 100, elt);
 162:     }
 163:     for (i, &elt) in insert.iter().cycle().enumerate().take(100) {
 164:         map.insert(elt * 100 + i as i32, elt);
 165:     }
 166:     println!("{:?}", map);
 167:     for &elt in &not_present {
 168:         assert!(map.get(&elt).is_none());
 169:     }
 170: }
 171: #[test]
 172: fn reserve() {
 173:     let mut map = OrderMap::<usize, usize>::new();
 174:     assert_eq!(map.capacity(), 0);
 175:     map.reserve(100);
 176:     let capacity = map.capacity();
 177:     assert!(capacity >= 100);
 178:     for i in 0..capacity {
 179:         assert_eq!(map.len(), i);
 180:         map.insert(i, i * i);
 181:         assert_eq!(map.len(), i + 1);
 182:         assert_eq!(map.capacity(), capacity);
 183:         assert_eq!(map.get(&i), Some(&(i * i)));
 184:     }
 185:     map.insert(capacity, usize::MAX);
 186:     assert_eq!(map.len(), capacity + 1);
 187:     assert!(map.capacity() > capacity);
 188:     assert_eq!(map.get(&capacity), Some(&usize::MAX));
 189: }
 190: #[test]
 191: fn try_reserve() {
 192:     let mut map = OrderMap::<usize, usize>::new();
 193:     assert_eq!(map.capacity(), 0);
 194:     assert_eq!(map.try_reserve(100), Ok(()));
 195:     assert!(map.capacity() >= 100);
 196:     assert!(map.try_reserve(usize::MAX).is_err());
 197: }
 198: #[test]
 199: fn shrink_to_fit() {
 200:     let mut map = OrderMap::<usize, usize>::new();
 201:     assert_eq!(map.capacity(), 0);
 202:     for i in 0..100 {
 203:         assert_eq!(map.len(), i);
 204:         map.insert(i, i * i);
 205:         assert_eq!(map.len(), i + 1);
 206:         assert!(map.capacity() > i);
 207:         assert_eq!(map.get(&i), Some(&(i * i)));
 208:         map.shrink_to_fit();
 209:         assert_eq!(map.len(), i + 1);
 210:         assert_eq!(map.capacity(), i + 1);
 211:         assert_eq!(map.get(&i), Some(&(i * i)));
 212:     }
 213: }
 214: #[test]
 215: fn remove() {
 216:     let insert = [0, 4, 2, 12, 8, 7, 11, 5, 3, 17, 19, 22, 23];
 217:     let mut map = OrderMap::new();
 218:     for &elt in &insert {
 219:         map.insert(elt, elt);
 220:     }
 221:     assert_eq!(map.keys().count(), map.len());
 222:     assert_eq!(map.keys().count(), insert.len());
 223:     for (a, b) in insert.iter().zip(map.keys()) {
 224:         assert_eq!(a, b);
 225:     }
 226:     let remove_fail = [99, 77];
 227:     let remove = [4, 12, 8, 7];
 228:     for &key in &remove_fail {
 229:         assert!(map.swap_remove_full(&key).is_none());
 230:     }
 231:     println!("{:?}", map);
 232:     for &key in &remove {
 233:         //println!("{:?}", map);
 234:         let index = map.get_full(&key).unwrap().0;
 235:         assert_eq!(map.swap_remove_full(&key), Some((index, key, key)));
 236:     }
 237:     println!("{:?}", map);
 238:     for key in &insert {
 239:         assert_eq!(map.get(key).is_some(), !remove.contains(key));
 240:     }
 241:     assert_eq!(map.len(), insert.len() - remove.len());
 242:     assert_eq!(map.keys().count(), insert.len() - remove.len());
 243: }
 244: #[test]
 245: fn remove_to_empty() {
 246:     let mut map = ordermap! { 0 => 0, 4 => 4, 5 => 5 };
 247:     map.swap_remove(&5).unwrap();
 248:     map.swap_remove(&4).unwrap();
 249:     map.swap_remove(&0).unwrap();
 250:     assert!(map.is_empty());
 251: }
 252: #[test]
 253: fn swap_remove_index() {
 254:     let insert = [0, 4, 2, 12, 8, 7, 11, 5, 3, 17, 19, 22, 23];
 255:     let mut map = OrderMap::new();
 256:     for &elt in &insert {
 257:         map.insert(elt, elt * 2);
 258:     }
 259:     let mut vector = insert.to_vec();
 260:     let remove_sequence = &[3, 3, 10, 4, 5, 4, 3, 0, 1];
 261:     // check that the same swap remove sequence on vec and map
 262:     // have the same result.
 263:     for &rm in remove_sequence {
 264:         let out_vec = vector.swap_remove(rm);
 265:         let (out_map, _) = map.swap_remove_index(rm).unwrap();
 266:         assert_eq!(out_vec, out_map);
 267:     }
 268:     assert_eq!(vector.len(), map.len());
 269:     for (a, b) in vector.iter().zip(map.keys()) {
 270:         assert_eq!(a, b);
 271:     }
 272: }
 273: #[test]
 274: fn partial_eq_and_eq() {
 275:     let mut map_a = OrderMap::new();
 276:     map_a.insert(1, "1");
 277:     map_a.insert(2, "2");
 278:     let mut map_b = map_a.clone();
 279:     assert_eq!(map_a, map_b);
 280:     map_b.swap_remove(&1);
 281:     assert_ne!(map_a, map_b);
 282: }
 283: #[test]
 284: fn extend() {
 285:     let mut map = OrderMap::new();
 286:     map.extend([(&1, &2), (&3, &4)]);
 287:     map.extend([(5, 6)]);
 288:     assert_eq!(
 289:         map.into_iter().collect::<Vec<_>>(),
 290:         [(1, 2), (3, 4), (5, 6)]
 291:     );
 292: }
 293: #[test]
 294: fn entry() {
 295:     let mut map = OrderMap::new();
 296:     map.insert(1, "1");
 297:     map.insert(2, "2");
 298:     {
 299:         let e = map.entry(3);
 300:         assert_eq!(e.index(), 2);
 301:         let e = e.or_insert("3");
 302:         assert_eq!(e, &"3");
 303:     }
 304:     let e = map.entry(2);
 305:     assert_eq!(e.index(), 1);
 306:     assert_eq!(e.key(), &2);
 307:     match e {
 308:         Entry::Occupied(ref e) => assert_eq!(e.get(), &"2"),
 309:         Entry::Vacant(_) => panic!(),
 310:     }
 311:     assert_eq!(e.or_insert("4"), &"2");
 312: }
 313: #[test]
 314: fn entry_and_modify() {
 315:     let mut map = OrderMap::new();
 316:     map.insert(1, "1");
 317:     map.entry(1).and_modify(|x| *x = "2");
 318:     assert_eq!(Some(&"2"), map.get(&1));
 319:     map.entry(2).and_modify(|x| *x = "doesn't exist");
 320:     assert_eq!(None, map.get(&2));
 321: }
 322: #[test]
 323: fn entry_or_default() {
 324:     let mut map = OrderMap::new();
 325:     #[derive(Debug, Default, PartialEq)]
 326:     enum TestEnum {
 327:         #[default]
 328:         DefaultValue,
 329:         NonDefaultValue,
 330:     }
 331:     map.insert(1, TestEnum::NonDefaultValue);
 332:     assert_eq!(&mut TestEnum::NonDefaultValue, map.entry(1).or_default());
 333:     assert_eq!(&mut TestEnum::DefaultValue, map.entry(2).or_default());
 334: }
 335: #[test]
 336: fn occupied_entry_key() {
 337:     // These keys match hash and equality, but their addresses are distinct.
 338:     let (k1, k2) = (&mut 1, &mut 1);
 339:     let k1_ptr = k1 as *const i32;
 340:     let k2_ptr = k2 as *const i32;
 341:     assert_ne!(k1_ptr, k2_ptr);
 342:     let mut map = OrderMap::new();
 343:     map.insert(k1, "value");
 344:     match map.entry(k2) {
 345:         Entry::Occupied(ref e) => {
 346:             // `OccupiedEntry::key` should reference the key in the map,
 347:             // not the key that was used to find the entry.
 348:             let ptr = *e.key() as *const i32;
 349:             assert_eq!(ptr, k1_ptr);
 350:             assert_ne!(ptr, k2_ptr);
 351:         }
 352:         Entry::Vacant(_) => panic!(),
 353:     }
 354: }
 355: #[test]
 356: fn get_index_entry() {
 357:     let mut map = OrderMap::new();
 358:     assert!(map.get_index_entry(0).is_none());
 359:     assert!(map.first_entry().is_none());
 360:     assert!(map.last_entry().is_none());
 361:     map.insert(0, "0");
 362:     map.insert(1, "1");
 363:     map.insert(2, "2");
 364:     map.insert(3, "3");
 365:     assert!(map.get_index_entry(4).is_none());
 366:     {
 367:         let e = map.get_index_entry(1).unwrap();
 368:         assert_eq!(*e.key(), 1);
 369:         assert_eq!(*e.get(), "1");
 370:         assert_eq!(e.swap_remove(), "1");
 371:     }
 372:     {
 373:         let mut e = map.get_index_entry(1).unwrap();
 374:         assert_eq!(*e.key(), 3);
 375:         assert_eq!(*e.get(), "3");
 376:         assert_eq!(e.insert("4"), "3");
 377:     }
 378:     assert_eq!(*map.get(&3).unwrap(), "4");
 379:     {
 380:         let e = map.first_entry().unwrap();
 381:         assert_eq!(*e.key(), 0);
 382:         assert_eq!(*e.get(), "0");
 383:     }
 384:     {
 385:         let e = map.last_entry().unwrap();
 386:         assert_eq!(*e.key(), 2);
 387:         assert_eq!(*e.get(), "2");
 388:     }
 389: }
 390: #[test]
 391: fn from_entries() {
 392:     let mut map = OrderMap::from([(1, "1"), (2, "2"), (3, "3")]);
 393:     {
 394:         let e = match map.entry(1) {
 395:             Entry::Occupied(e) => IndexedEntry::from(e),
 396:             Entry::Vacant(_) => panic!(),
 397:         };
 398:         assert_eq!(e.index(), 0);
 399:         assert_eq!(*e.key(), 1);
 400:         assert_eq!(*e.get(), "1");
 401:     }
 402:     {
 403:         let e = match map.get_index_entry(1) {
 404:             Some(e) => OccupiedEntry::from(e),
 405:             None => panic!(),
 406:         };
 407:         assert_eq!(e.index(), 1);
 408:         assert_eq!(*e.key(), 2);
 409:         assert_eq!(*e.get(), "2");
 410:     }
 411: }
 412: #[test]
 413: fn keys() {
 414:     let map = OrderMap::<_, _>::from_iter([(1, 'a'), (2, 'b'), (3, 'c')]);
 415:     let keys: Vec<_> = map.keys().copied().collect();
 416:     assert_eq!(keys.len(), 3);
 417:     assert!(keys.contains(&1));
 418:     assert!(keys.contains(&2));
 419:     assert!(keys.contains(&3));
 420: }
 421: #[test]
 422: fn into_keys() {
 423:     let map = OrderMap::<_, _>::from_iter([(1, 'a'), (2, 'b'), (3, 'c')]);
 424:     let keys: Vec<i32> = map.into_keys().collect();
 425:     assert_eq!(keys.len(), 3);
 426:     assert!(keys.contains(&1));
 427:     assert!(keys.contains(&2));
 428:     assert!(keys.contains(&3));
 429: }
 430: #[test]
 431: fn values() {
 432:     let map = OrderMap::<_, _>::from_iter([(1, 'a'), (2, 'b'), (3, 'c')]);
 433:     let values: Vec<_> = map.values().copied().collect();
 434:     assert_eq!(values.len(), 3);
 435:     assert!(values.contains(&'a'));
 436:     assert!(values.contains(&'b'));
 437:     assert!(values.contains(&'c'));
 438: }
 439: #[test]
 440: fn values_mut() {
 441:     let mut map = OrderMap::<_, _>::from_iter([(1, 1), (2, 2), (3, 3)]);
 442:     for value in map.values_mut() {
 443:         *value *= 2
 444:     }
 445:     let values: Vec<_> = map.values().copied().collect();
 446:     assert_eq!(values.len(), 3);
 447:     assert!(values.contains(&2));
 448:     assert!(values.contains(&4));
 449:     assert!(values.contains(&6));
 450: }
 451: #[test]
 452: fn into_values() {
 453:     let map = OrderMap::<_, _>::from_iter([(1, 'a'), (2, 'b'), (3, 'c')]);
 454:     let values: Vec<char> = map.into_values().collect();
 455:     assert_eq!(values.len(), 3);
 456:     assert!(values.contains(&'a'));
 457:     assert!(values.contains(&'b'));
 458:     assert!(values.contains(&'c'));
 459: }
 460: #[test]
 461: fn drain_range() {
 462:     // Test the various heuristics of `erase_indices`
 463:     for range in [
 464:         0..0,   // nothing erased
 465:         10..90, // reinsert the few kept (..10 and 90..)
 466:         80..90, // update the few to adjust (80..)
 467:         20..30, // sweep everything
 468:     ] {
 469:         let mut vec = Vec::from_iter(0..100);
 470:         let mut map: OrderMap<i32, ()> = (0..100).map(|i| (i, ())).collect();
 471:         drop(vec.drain(range.clone()));
 472:         drop(map.drain(range));
 473:         assert!(vec.iter().eq(map.keys()));
 474:         for (i, x) in vec.iter().enumerate() {
 475:             assert_eq!(map.get_index_of(x), Some(i));
 476:         }
 477:     }
 478: }
 479: #[test]
 480: #[cfg(feature = "std")]
 481: fn from_array() {
 482:     let map = OrderMap::from([(1, 2), (3, 4)]);
 483:     let mut expected = OrderMap::new();
 484:     expected.insert(1, 2);
 485:     expected.insert(3, 4);
 486:     assert_eq!(map, expected)
 487: }
 488: #[test]
 489: fn iter_default() {
 490:     struct K;
 491:     struct V;
 492:     fn assert_default<T>()
 493:     where
 494:         T: Default + Iterator,
 495:     {
 496:         assert!(T::default().next().is_none());
 497:     }
 498:     assert_default::<Iter<'static, K, V>>();
 499:     assert_default::<IterMut<'static, K, V>>();
 500:     assert_default::<IterMut2<'static, K, V>>();
 501:     assert_default::<IntoIter<K, V>>();
 502:     assert_default::<Keys<'static, K, V>>();
 503:     assert_default::<IntoKeys<K, V>>();
 504:     assert_default::<Values<'static, K, V>>();
 505:     assert_default::<ValuesMut<'static, K, V>>();
 506:     assert_default::<IntoValues<K, V>>();
 507: }
 508: #[test]
 509: fn get_index_mut2() {
 510:     let mut map: OrderMap<i32, i32> = OrderMap::new();
 511:     map.insert(1, 2);
 512:     map.insert(3, 4);
 513:     map.insert(5, 6);
 514:     {
 515:         let (key, value) = map.get_index_mut2(0).unwrap();
 516:         assert_eq!(*key, 1);
 517:         assert_eq!(*value, 2);
 518:         *value = 7;
 519:     }
 520:     assert_eq!(map[0], 7);
 521:     {
 522:         let (key, _) = map.get_index_mut2(0).unwrap();
 523:         *key = 8;
 524:     }
 525:     assert_eq!(map.get_index(0).unwrap().0, &8);
 526: }
 527: #[test]
 528: fn shift_shift_remove_index() {
 529:     let mut map: OrderMap<i32, i32> = OrderMap::new();
 530:     map.insert(1, 2);
 531:     map.insert(3, 4);
 532:     map.insert(5, 6);
 533:     map.insert(7, 8);
 534:     map.insert(9, 10);
 535:     let result = map.remove_index(1);
 536:     assert_eq!(result, Some((3, 4)));
 537:     assert_eq!(map.len(), 4);
 538:     assert_eq!(map.as_slice(), &[(1, 2), (5, 6), (7, 8), (9, 10)]);
 539:     let result = map.remove_index(1);
 540:     assert_eq!(result, Some((5, 6)));
 541:     assert_eq!(map.len(), 3);
 542:     assert_eq!(map.as_slice(), &[(1, 2), (7, 8), (9, 10)]);
 543:     let result = map.remove_index(2);
 544:     assert_eq!(result, Some((9, 10)));
 545:     assert_eq!(map.len(), 2);
 546:     assert_eq!(map.as_slice(), &[(1, 2), (7, 8)]);
 547:     let result = map.remove_index(2);
 548:     assert_eq!(result, None);
 549:     assert_eq!(map.len(), 2);
 550:     assert_eq!(map.as_slice(), &[(1, 2), (7, 8)]);
 551: }
 552: #[test]
 553: fn shift_remove_entry() {
 554:     let mut map: OrderMap<i32, i32> = OrderMap::new();
 555:     map.insert(1, 2);
 556:     map.insert(3, 4);
 557:     map.insert(5, 6);
 558:     map.insert(7, 8);
 559:     map.insert(9, 10);
 560:     let result = map.remove_entry(&3);
 561:     assert_eq!(result, Some((3, 4)));
 562:     assert_eq!(map.len(), 4);
 563:     assert_eq!(map.as_slice(), &[(1, 2), (5, 6), (7, 8), (9, 10)]);
 564:     let result = map.remove_entry(&9);
 565:     assert_eq!(result, Some((9, 10)));
 566:     assert_eq!(map.len(), 3);
 567:     assert_eq!(map.as_slice(), &[(1, 2), (5, 6), (7, 8)]);
 568:     let result = map.remove_entry(&9);
 569:     assert_eq!(result, None);
 570:     assert_eq!(map.len(), 3);
 571:     assert_eq!(map.as_slice(), &[(1, 2), (5, 6), (7, 8)]);
 572: }
 573: #[test]
 574: fn shift_remove_full() {
 575:     let mut map: OrderMap<i32, i32> = OrderMap::new();
 576:     map.insert(1, 2);
 577:     map.insert(3, 4);
 578:     map.insert(5, 6);
 579:     map.insert(7, 8);
 580:     map.insert(9, 10);
 581:     let result = map.remove_full(&3);
 582:     assert_eq!(result, Some((1, 3, 4)));
 583:     assert_eq!(map.len(), 4);
 584:     assert_eq!(map.as_slice(), &[(1, 2), (5, 6), (7, 8), (9, 10)]);
 585:     let result = map.remove_full(&9);
 586:     assert_eq!(result, Some((3, 9, 10)));
 587:     assert_eq!(map.len(), 3);
 588:     assert_eq!(map.as_slice(), &[(1, 2), (5, 6), (7, 8)]);
 589:     let result = map.remove_full(&9);
 590:     assert_eq!(result, None);
 591:     assert_eq!(map.len(), 3);
 592:     assert_eq!(map.as_slice(), &[(1, 2), (5, 6), (7, 8)]);
 593: }
 594: #[test]
 595: fn sorted_unstable_by() {
 596:     let map = OrderMap::<i32, i32>::from_iter([(1, 10), (2, 20), (3, 30), (4, 40), (5, 50)]);
 597:     let sorted = map.sorted_unstable_by(|_a, b, _c, d| d.cmp(&b));
 598:     assert_eq!(
 599:         sorted.as_slice(),
 600:         &[(5, 50), (4, 40), (3, 30), (2, 20), (1, 10)]
 601:     );
 602: }
 603: #[test]
 604: fn into_boxed_slice() {
 605:     let mut map: OrderMap<i32, i32> = OrderMap::new();
 606:     for i in 0..5 {
 607:         map.insert(i, i * 10);
 608:     }
 609:     let boxed_slice: Box<Slice<i32, i32>> = map.into_boxed_slice();
 610:     assert_eq!(boxed_slice.len(), 5);
 611:     assert_eq!(
 612:         boxed_slice.as_ref(),
 613:         &[(0, 0), (1, 10), (2, 20), (3, 30), (4, 40)]
 614:     );
 615: }
 616: #[test]
 617: fn last_mut() {
 618:     let mut map: OrderMap<&str, i32> = OrderMap::new();
 619:     let last_entry = map.last_mut();
 620:     assert_eq!(last_entry, None);
 621:     map.insert("key1", 1);
 622:     map.insert("key2", 2);
 623:     map.insert("key3", 3);
 624:     let last_entry = map.last_mut();
 625:     assert_eq!(last_entry, Some((&"key3", &mut 3)));
 626:     *last_entry.unwrap().1 = 4;
 627:     assert_eq!(map.get("key3"), Some(&4));
 628: }
 629: #[test]
 630: #[should_panic = "index out of bounds"]
 631: fn insert_before_oob() {
 632:     let mut map: OrderMap<char, ()> = OrderMap::new();
 633:     let _ = map.insert_before(0, 'a', ());
 634:     let _ = map.insert_before(1, 'b', ());
 635:     map.insert_before(3, 'd', ());
 636: }
 637: #[test]
 638: fn clear() {
 639:     let mut map = OrderMap::<i32, i32>::from_iter([(1, 10), (2, 20), (3, 30), (4, 40), (5, 50)]);
 640:     assert_ne!(map.len(), 0);
 641:     map.clear();
 642:     assert_eq!(map.len(), 0);
 643: }
 644: #[test]
 645: fn get_range() {
 646:     let map = OrderMap::<i32, i32>::from_iter([(1, 10), (2, 20), (3, 30), (4, 40), (5, 50)]);
 647:     let result = map.get_range(2..2);
 648:     assert!(result.unwrap().is_empty());
 649:     #[expect(clippy::reversed_empty_ranges)]
 650:     let result = map.get_range(4..2);
 651:     assert!(result.is_none());
 652:     let result = map.get_range(2..4);
 653:     let slice: &Slice<i32, i32> = result.unwrap();
 654:     assert_eq!(slice.len(), 2);
 655:     assert_eq!(slice, &[(3, 30), (4, 40)]);
 656: }
 657: #[test]
 658: fn get_range_mut() {
 659:     let mut index_map: OrderMap<i32, i32> = OrderMap::new();
 660:     index_map.insert(1, 10);
 661:     index_map.insert(2, 20);
 662:     index_map.insert(3, 30);
 663:     index_map.insert(4, 40);
 664:     index_map.insert(5, 50);
 665:     let result = index_map.get_range_mut(2..2);
 666:     assert!(result.unwrap().is_empty());
 667:     #[expect(clippy::reversed_empty_ranges)]
 668:     let result = index_map.get_range_mut(4..2);
 669:     assert!(result.is_none());
 670:     let result = index_map.get_range_mut(2..4);
 671:     let slice: &mut Slice<i32, i32> = result.unwrap();
 672:     assert_eq!(slice.len(), 2);
 673:     assert_eq!(slice, &mut [(3, 30), (4, 40)]);
 674:     for i in 0..slice.len() {
 675:         slice[i] += 1;
 676:     }
 677:     assert_eq!(slice, &mut [(3, 31), (4, 41)]);
 678: }
 679: #[test]
 680: #[should_panic = "index out of bounds"]
 681: fn shift_insert_oob() {
 682:     let mut map: OrderMap<u32, u32> = OrderMap::new();
 683:     map.shift_insert(0, 1, 10);
 684:     map.shift_insert(1, 2, 20);
 685:     map.shift_insert(2, 3, 30);
 686:     map.shift_insert(5, 4, 40);
 687: }
 688: #[test]
 689: fn test_binary_search_by() {
 690:     // adapted from std's test for binary_search
 691:     let b: OrderMap<_, i32> = []
 692:         .into_iter()
 693:         .enumerate()
 694:         .map(|(i, x)| (i + 100, x))
 695:         .collect();
 696:     assert_eq!(b.binary_search_by(|_, x| x.cmp(&5)), Err(0));
 697:     let b: OrderMap<_, i32> = [4]
 698:         .into_iter()
 699:         .enumerate()
 700:         .map(|(i, x)| (i + 100, x))
 701:         .collect();
 702:     assert_eq!(b.binary_search_by(|_, x| x.cmp(&3)), Err(0));
 703:     assert_eq!(b.binary_search_by(|_, x| x.cmp(&4)), Ok(0));
 704:     assert_eq!(b.binary_search_by(|_, x| x.cmp(&5)), Err(1));
 705:     let b: OrderMap<_, i32> = [1, 2, 4, 6, 8, 9]
 706:         .into_iter()
 707:         .enumerate()
 708:         .map(|(i, x)| (i + 100, x))
 709:         .collect();
 710:     assert_eq!(b.binary_search_by(|_, x| x.cmp(&5)), Err(3));
 711:     assert_eq!(b.binary_search_by(|_, x| x.cmp(&6)), Ok(3));
 712:     assert_eq!(b.binary_search_by(|_, x| x.cmp(&7)), Err(4));
 713:     assert_eq!(b.binary_search_by(|_, x| x.cmp(&8)), Ok(4));
 714:     let b: OrderMap<_, i32> = [1, 2, 4, 5, 6, 8]
 715:         .into_iter()
 716:         .enumerate()
 717:         .map(|(i, x)| (i + 100, x))
 718:         .collect();
 719:     assert_eq!(b.binary_search_by(|_, x| x.cmp(&9)), Err(6));
 720:     let b: OrderMap<_, i32> = [1, 2, 4, 6, 7, 8, 9]
 721:         .into_iter()
 722:         .enumerate()
 723:         .map(|(i, x)| (i + 100, x))
 724:         .collect();
 725:     assert_eq!(b.binary_search_by(|_, x| x.cmp(&6)), Ok(3));
 726:     assert_eq!(b.binary_search_by(|_, x| x.cmp(&5)), Err(3));
 727:     assert_eq!(b.binary_search_by(|_, x| x.cmp(&8)), Ok(5));
 728:     let b: OrderMap<_, i32> = [1, 2, 4, 5, 6, 8, 9]
 729:         .into_iter()
 730:         .enumerate()
 731:         .map(|(i, x)| (i + 100, x))
 732:         .collect();
 733:     assert_eq!(b.binary_search_by(|_, x| x.cmp(&7)), Err(5));
 734:     assert_eq!(b.binary_search_by(|_, x| x.cmp(&0)), Err(0));
 735:     let b: OrderMap<_, i32> = [1, 3, 3, 3, 7]
 736:         .into_iter()
 737:         .enumerate()
 738:         .map(|(i, x)| (i + 100, x))
 739:         .collect();
 740:     assert_eq!(b.binary_search_by(|_, x| x.cmp(&0)), Err(0));
 741:     assert_eq!(b.binary_search_by(|_, x| x.cmp(&1)), Ok(0));
 742:     assert_eq!(b.binary_search_by(|_, x| x.cmp(&2)), Err(1));
 743:     assert!(match b.binary_search_by(|_, x| x.cmp(&3)) {
 744:         Ok(1..=3) => true,
 745:         _ => false,
 746:     });
 747:     assert!(match b.binary_search_by(|_, x| x.cmp(&3)) {
 748:         Ok(1..=3) => true,
 749:         _ => false,
 750:     });
 751:     assert_eq!(b.binary_search_by(|_, x| x.cmp(&4)), Err(4));
 752:     assert_eq!(b.binary_search_by(|_, x| x.cmp(&5)), Err(4));
 753:     assert_eq!(b.binary_search_by(|_, x| x.cmp(&6)), Err(4));
 754:     assert_eq!(b.binary_search_by(|_, x| x.cmp(&7)), Ok(4));
 755:     assert_eq!(b.binary_search_by(|_, x| x.cmp(&8)), Err(5));
 756: }
 757: #[test]
 758: fn test_binary_search_by_key() {
 759:     // adapted from std's test for binary_search
 760:     let b: OrderMap<_, i32> = []
 761:         .into_iter()
 762:         .enumerate()
 763:         .map(|(i, x)| (i + 100, x))
 764:         .collect();
 765:     assert_eq!(b.binary_search_by_key(&5, |_, &x| x), Err(0));
 766:     let b: OrderMap<_, i32> = [4]
 767:         .into_iter()
 768:         .enumerate()
 769:         .map(|(i, x)| (i + 100, x))
 770:         .collect();
 771:     assert_eq!(b.binary_search_by_key(&3, |_, &x| x), Err(0));
 772:     assert_eq!(b.binary_search_by_key(&4, |_, &x| x), Ok(0));
 773:     assert_eq!(b.binary_search_by_key(&5, |_, &x| x), Err(1));
 774:     let b: OrderMap<_, i32> = [1, 2, 4, 6, 8, 9]
 775:         .into_iter()
 776:         .enumerate()
 777:         .map(|(i, x)| (i + 100, x))
 778:         .collect();
 779:     assert_eq!(b.binary_search_by_key(&5, |_, &x| x), Err(3));
 780:     assert_eq!(b.binary_search_by_key(&6, |_, &x| x), Ok(3));
 781:     assert_eq!(b.binary_search_by_key(&7, |_, &x| x), Err(4));
 782:     assert_eq!(b.binary_search_by_key(&8, |_, &x| x), Ok(4));
 783:     let b: OrderMap<_, i32> = [1, 2, 4, 5, 6, 8]
 784:         .into_iter()
 785:         .enumerate()
 786:         .map(|(i, x)| (i + 100, x))
 787:         .collect();
 788:     assert_eq!(b.binary_search_by_key(&9, |_, &x| x), Err(6));
 789:     let b: OrderMap<_, i32> = [1, 2, 4, 6, 7, 8, 9]
 790:         .into_iter()
 791:         .enumerate()
 792:         .map(|(i, x)| (i + 100, x))
 793:         .collect();
 794:     assert_eq!(b.binary_search_by_key(&6, |_, &x| x), Ok(3));
 795:     assert_eq!(b.binary_search_by_key(&5, |_, &x| x), Err(3));
 796:     assert_eq!(b.binary_search_by_key(&8, |_, &x| x), Ok(5));
 797:     let b: OrderMap<_, i32> = [1, 2, 4, 5, 6, 8, 9]
 798:         .into_iter()
 799:         .enumerate()
 800:         .map(|(i, x)| (i + 100, x))
 801:         .collect();
 802:     assert_eq!(b.binary_search_by_key(&7, |_, &x| x), Err(5));
 803:     assert_eq!(b.binary_search_by_key(&0, |_, &x| x), Err(0));
 804:     let b: OrderMap<_, i32> = [1, 3, 3, 3, 7]
 805:         .into_iter()
 806:         .enumerate()
 807:         .map(|(i, x)| (i + 100, x))
 808:         .collect();
 809:     assert_eq!(b.binary_search_by_key(&0, |_, &x| x), Err(0));
 810:     assert_eq!(b.binary_search_by_key(&1, |_, &x| x), Ok(0));
 811:     assert_eq!(b.binary_search_by_key(&2, |_, &x| x), Err(1));
 812:     assert!(match b.binary_search_by_key(&3, |_, &x| x) {
 813:         Ok(1..=3) => true,
 814:         _ => false,
 815:     });
 816:     assert!(match b.binary_search_by_key(&3, |_, &x| x) {
 817:         Ok(1..=3) => true,
 818:         _ => false,
 819:     });
 820:     assert_eq!(b.binary_search_by_key(&4, |_, &x| x), Err(4));
 821:     assert_eq!(b.binary_search_by_key(&5, |_, &x| x), Err(4));
 822:     assert_eq!(b.binary_search_by_key(&6, |_, &x| x), Err(4));
 823:     assert_eq!(b.binary_search_by_key(&7, |_, &x| x), Ok(4));
 824:     assert_eq!(b.binary_search_by_key(&8, |_, &x| x), Err(5));
 825: }
 826: #[test]
 827: fn test_partition_point() {
 828:     // adapted from std's test for partition_point
 829:     let b: OrderMap<_, i32> = []
 830:         .into_iter()
 831:         .enumerate()
 832:         .map(|(i, x)| (i + 100, x))
 833:         .collect();
 834:     assert_eq!(b.partition_point(|_, &x| x < 5), 0);
 835:     let b: OrderMap<_, i32> = [4]
 836:         .into_iter()
 837:         .enumerate()
 838:         .map(|(i, x)| (i + 100, x))
 839:         .collect();
 840:     assert_eq!(b.partition_point(|_, &x| x < 3), 0);
 841:     assert_eq!(b.partition_point(|_, &x| x < 4), 0);
 842:     assert_eq!(b.partition_point(|_, &x| x < 5), 1);
 843:     let b: OrderMap<_, i32> = [1, 2, 4, 6, 8, 9]
 844:         .into_iter()
 845:         .enumerate()
 846:         .map(|(i, x)| (i + 100, x))
 847:         .collect();
 848:     assert_eq!(b.partition_point(|_, &x| x < 5), 3);
 849:     assert_eq!(b.partition_point(|_, &x| x < 6), 3);
 850:     assert_eq!(b.partition_point(|_, &x| x < 7), 4);
 851:     assert_eq!(b.partition_point(|_, &x| x < 8), 4);
 852:     let b: OrderMap<_, i32> = [1, 2, 4, 5, 6, 8]
 853:         .into_iter()
 854:         .enumerate()
 855:         .map(|(i, x)| (i + 100, x))
 856:         .collect();
 857:     assert_eq!(b.partition_point(|_, &x| x < 9), 6);
 858:     let b: OrderMap<_, i32> = [1, 2, 4, 6, 7, 8, 9]
 859:         .into_iter()
 860:         .enumerate()
 861:         .map(|(i, x)| (i + 100, x))
 862:         .collect();
 863:     assert_eq!(b.partition_point(|_, &x| x < 6), 3);
 864:     assert_eq!(b.partition_point(|_, &x| x < 5), 3);
 865:     assert_eq!(b.partition_point(|_, &x| x < 8), 5);
 866:     let b: OrderMap<_, i32> = [1, 2, 4, 5, 6, 8, 9]
 867:         .into_iter()
 868:         .enumerate()
 869:         .map(|(i, x)| (i + 100, x))
 870:         .collect();
 871:     assert_eq!(b.partition_point(|_, &x| x < 7), 5);
 872:     assert_eq!(b.partition_point(|_, &x| x < 0), 0);
 873:     let b: OrderMap<_, i32> = [1, 3, 3, 3, 7]
 874:         .into_iter()
 875:         .enumerate()
 876:         .map(|(i, x)| (i + 100, x))
 877:         .collect();
 878:     assert_eq!(b.partition_point(|_, &x| x < 0), 0);
 879:     assert_eq!(b.partition_point(|_, &x| x < 1), 0);
 880:     assert_eq!(b.partition_point(|_, &x| x < 2), 1);
 881:     assert_eq!(b.partition_point(|_, &x| x < 3), 1);
 882:     assert_eq!(b.partition_point(|_, &x| x < 4), 4);
 883:     assert_eq!(b.partition_point(|_, &x| x < 5), 4);
 884:     assert_eq!(b.partition_point(|_, &x| x < 6), 4);
 885:     assert_eq!(b.partition_point(|_, &x| x < 7), 4);
 886:     assert_eq!(b.partition_point(|_, &x| x < 8), 5);
 887: }
 888: #[test]
 889: fn disjoint_mut_empty_map() {
 890:     let mut map: OrderMap<u32, u32> = OrderMap::default();
 891:     assert_eq!(
 892:         map.get_disjoint_mut([&0, &1, &2, &3]),
 893:         [None, None, None, None]
 894:     );
 895: }
 896: #[test]
 897: fn disjoint_mut_empty_param() {
 898:     let mut map: OrderMap<u32, u32> = OrderMap::default();
 899:     map.insert(1, 10);
 900:     assert_eq!(map.get_disjoint_mut([] as [&u32; 0]), []);
 901: }
 902: #[test]
 903: fn disjoint_mut_single_fail() {
 904:     let mut map: OrderMap<u32, u32> = OrderMap::default();
 905:     map.insert(1, 10);
 906:     assert_eq!(map.get_disjoint_mut([&0]), [None]);
 907: }
 908: #[test]
 909: fn disjoint_mut_single_success() {
 910:     let mut map: OrderMap<u32, u32> = OrderMap::default();
 911:     map.insert(1, 10);
 912:     assert_eq!(map.get_disjoint_mut([&1]), [Some(&mut 10)]);
 913: }
 914: #[test]
 915: fn disjoint_mut_multi_success() {
 916:     let mut map: OrderMap<u32, u32> = OrderMap::default();
 917:     map.insert(1, 100);
 918:     map.insert(2, 200);
 919:     map.insert(3, 300);
 920:     map.insert(4, 400);
 921:     assert_eq!(
 922:         map.get_disjoint_mut([&1, &2]),
 923:         [Some(&mut 100), Some(&mut 200)]
 924:     );
 925:     assert_eq!(
 926:         map.get_disjoint_mut([&1, &3]),
 927:         [Some(&mut 100), Some(&mut 300)]
 928:     );
 929:     assert_eq!(
 930:         map.get_disjoint_mut([&3, &1, &4, &2]),
 931:         [
 932:             Some(&mut 300),
 933:             Some(&mut 100),
 934:             Some(&mut 400),
 935:             Some(&mut 200)
 936:         ]
 937:     );
 938: }
 939: #[test]
 940: fn disjoint_mut_multi_success_unsized_key() {
 941:     let mut map: OrderMap<&'static str, u32> = OrderMap::default();
 942:     map.insert("1", 100);
 943:     map.insert("2", 200);
 944:     map.insert("3", 300);
 945:     map.insert("4", 400);
 946:     assert_eq!(
 947:         map.get_disjoint_mut(["1", "2"]),
 948:         [Some(&mut 100), Some(&mut 200)]
 949:     );
 950:     assert_eq!(
 951:         map.get_disjoint_mut(["1", "3"]),
 952:         [Some(&mut 100), Some(&mut 300)]
 953:     );
 954:     assert_eq!(
 955:         map.get_disjoint_mut(["3", "1", "4", "2"]),
 956:         [
 957:             Some(&mut 300),
 958:             Some(&mut 100),
 959:             Some(&mut 400),
 960:             Some(&mut 200)
 961:         ]
 962:     );
 963: }
 964: #[test]
 965: fn disjoint_mut_multi_success_borrow_key() {
 966:     let mut map: OrderMap<String, u32> = OrderMap::default();
 967:     map.insert("1".into(), 100);
 968:     map.insert("2".into(), 200);
 969:     map.insert("3".into(), 300);
 970:     map.insert("4".into(), 400);
 971:     assert_eq!(
 972:         map.get_disjoint_mut(["1", "2"]),
 973:         [Some(&mut 100), Some(&mut 200)]
 974:     );
 975:     assert_eq!(
 976:         map.get_disjoint_mut(["1", "3"]),
 977:         [Some(&mut 100), Some(&mut 300)]
 978:     );
 979:     assert_eq!(
 980:         map.get_disjoint_mut(["3", "1", "4", "2"]),
 981:         [
 982:             Some(&mut 300),
 983:             Some(&mut 100),
 984:             Some(&mut 400),
 985:             Some(&mut 200)
 986:         ]
 987:     );
 988: }
 989: #[test]
 990: fn disjoint_mut_multi_fail_missing() {
 991:     let mut map: OrderMap<u32, u32> = OrderMap::default();
 992:     map.insert(1, 100);
 993:     map.insert(2, 200);
 994:     map.insert(3, 300);
 995:     map.insert(4, 400);
 996:     assert_eq!(map.get_disjoint_mut([&1, &5]), [Some(&mut 100), None]);
 997:     assert_eq!(map.get_disjoint_mut([&5, &6]), [None, None]);
 998:     assert_eq!(
 999:         map.get_disjoint_mut([&1, &5, &4]),
1000:         [Some(&mut 100), None, Some(&mut 400)]
1001:     );
1002: }
1003: #[test]
1004: #[should_panic]
1005: fn disjoint_mut_multi_fail_duplicate_panic() {
1006:     let mut map: OrderMap<u32, u32> = OrderMap::default();
1007:     map.insert(1, 100);
1008:     map.get_disjoint_mut([&1, &2, &1]);
1009: }
1010: #[test]
1011: fn disjoint_indices_mut_fail_oob() {
1012:     let mut map: OrderMap<u32, u32> = OrderMap::default();
1013:     map.insert(1, 10);
1014:     map.insert(321, 20);
1015:     assert_eq!(
1016:         map.get_disjoint_indices_mut([1, 3]),
1017:         Err(crate::GetDisjointMutError::IndexOutOfBounds)
1018:     );
1019: }
1020: #[test]
1021: fn disjoint_indices_mut_empty() {
1022:     let mut map: OrderMap<u32, u32> = OrderMap::default();
1023:     map.insert(1, 10);
1024:     map.insert(321, 20);
1025:     assert_eq!(map.get_disjoint_indices_mut([]), Ok([]));
1026: }
1027: #[test]
1028: fn disjoint_indices_mut_success() {
1029:     let mut map: OrderMap<u32, u32> = OrderMap::default();
1030:     map.insert(1, 10);
1031:     map.insert(321, 20);
1032:     assert_eq!(map.get_disjoint_indices_mut([0]), Ok([(&1, &mut 10)]));
1033:     assert_eq!(map.get_disjoint_indices_mut([1]), Ok([(&321, &mut 20)]));
1034:     assert_eq!(
1035:         map.get_disjoint_indices_mut([0, 1]),
1036:         Ok([(&1, &mut 10), (&321, &mut 20)])
1037:     );
1038: }
1039: #[test]
1040: fn disjoint_indices_mut_fail_duplicate() {
1041:     let mut map: OrderMap<u32, u32> = OrderMap::default();
1042:     map.insert(1, 10);
1043:     map.insert(321, 20);
1044:     assert_eq!(
1045:         map.get_disjoint_indices_mut([1, 0, 1]),
1046:         Err(crate::GetDisjointMutError::OverlappingIndices)
1047:     );
1048: }
1049: #[test]
1050: fn insert_sorted_by_key() {
1051:     let mut values = [(-1, 8), (3, 18), (-27, 2), (-2, 5)];
1052:     let mut map: OrderMap<i32, i32> = OrderMap::new();
1053:     for (key, value) in values {
1054:         let (_, old) = map.insert_sorted_by_key(key, value, |k, _| k.abs());
1055:         assert_eq!(old, None);
1056:     }
1057:     values.sort_by_key(|(key, _)| key.abs());
1058:     assert_eq!(values, *map.as_slice());
1059:     for (key, value) in &mut values {
1060:         let (_, old) = map.insert_sorted_by_key(*key, -*value, |k, _| k.abs());
1061:         assert_eq!(old, Some(*value));
1062:         *value = -*value;
1063:     }
1064:     assert_eq!(values, *map.as_slice());
1065: }
1066: #[test]
1067: fn insert_sorted_by() {
1068:     let mut values = [(1, 1), (2, 2), (3, 3), (4, 4), (5, 5)];
1069:     let mut map: OrderMap<i32, i32> = OrderMap::new();
1070:     for (key, value) in values {
1071:         let (_, old) = map.insert_sorted_by(key, value, |key1, _, key2, _| key2.cmp(key1));
1072:         assert_eq!(old, None);
1073:     }
1074:     values.reverse();
1075:     assert_eq!(values, *map.as_slice());
1076:     for (key, value) in &mut values {
1077:         let (_, old) = map.insert_sorted_by(*key, -*value, |key1, _, key2, _| key2.cmp(key1));
1078:         assert_eq!(old, Some(*value));
1079:         *value = -*value;
1080:     }
1081:     assert_eq!(values, *map.as_slice());
1082: }
1083: #[test]
1084: fn is_sorted() {
1085:     fn expect(map: &OrderMap<i32, i32>, e: [bool; 7]) {
1086:         assert_eq!(e[0], map.is_sorted());
1087:         assert_eq!(e[1], map.is_sorted_by(|k1, _, k2, _| k1 < k2));
1088:         assert_eq!(e[2], map.is_sorted_by(|k1, _, k2, _| k1 > k2));
1089:         assert_eq!(e[3], map.is_sorted_by(|_, v1, _, v2| v1 < v2));
1090:         assert_eq!(e[4], map.is_sorted_by(|_, v1, _, v2| v1 > v2));
1091:         assert_eq!(e[5], map.is_sorted_by_key(|k, _| k));
1092:         assert_eq!(e[6], map.is_sorted_by_key(|_, v| v));
1093:     }
1094:     let mut map = OrderMap::from_iter((0..10).map(|i| (i, i * i)));
1095:     expect(&map, [true, true, false, true, false, true, true]);
1096:     map[5] = -1;
1097:     expect(&map, [true, true, false, false, false, true, false]);
1098:     map[5] = 25;
1099:     map.replace_index(5, -1).unwrap();
1100:     expect(&map, [false, false, false, true, false, false, true]);
1101: }
1102: #[test]
1103: fn is_sorted_trivial() {
1104:     fn expect(map: &OrderMap<i32, i32>, e: [bool; 5]) {
1105:         assert_eq!(e[0], map.is_sorted());
1106:         assert_eq!(e[1], map.is_sorted_by(|_, _, _, _| true));
1107:         assert_eq!(e[2], map.is_sorted_by(|_, _, _, _| false));
1108:         assert_eq!(e[3], map.is_sorted_by_key(|_, _| 0f64));
1109:         assert_eq!(e[4], map.is_sorted_by_key(|_, _| f64::NAN));
1110:     }
1111:     let mut map = OrderMap::new();
1112:     expect(&map, [true, true, true, true, true]);
1113:     map.insert(0, 0);
1114:     expect(&map, [true, true, true, true, true]);
1115:     map.insert(1, 1);
1116:     expect(&map, [true, true, false, true, false]);
1117:     map.reverse();
1118:     expect(&map, [false, true, false, true, false]);
1119: }
````

## File: src/set/iter.rs
````rust
 1: use super::OrderSet;
 2: use indexmap::set::{IntoIter, Iter};
 3: impl<'a, T, S> IntoIterator for &'a OrderSet<T, S> {
 4:     type Item = &'a T;
 5:     type IntoIter = Iter<'a, T>;
 6:     fn into_iter(self) -> Self::IntoIter {
 7:         self.inner.iter()
 8:     }
 9: }
10: impl<T, S> IntoIterator for OrderSet<T, S> {
11:     type Item = T;
12:     type IntoIter = IntoIter<T>;
13:     fn into_iter(self) -> Self::IntoIter {
14:         self.inner.into_iter()
15:     }
16: }
````

## File: src/set/mutable.rs
````rust
 1: use super::{Equivalent, OrderSet};
 2: use core::hash::{BuildHasher, Hash};
 3: use indexmap::set::MutableValues as _;
 4: /// Opt-in mutable access to [`OrderSet`] values.
 5: ///
 6: /// These methods expose `&mut T`, mutable references to the value as it is stored
 7: /// in the set.
 8: /// You are allowed to modify the values in the set **if the modification
 9: /// does not change the value's hash and equality**.
10: ///
11: /// If values are modified erroneously, you can no longer look them up.
12: /// This is sound (memory safe) but a logical error hazard (just like
13: /// implementing `PartialEq`, `Eq`, or `Hash` incorrectly would be).
14: ///
15: /// `use` this trait to enable its methods for `OrderSet`.
16: ///
17: /// This trait is sealed and cannot be implemented for types outside this crate.
18: #[expect(private_bounds)]
19: pub trait MutableValues: Sealed {
20:     type Value;
21:     /// Return item index and mutable reference to the value
22:     ///
23:     /// Computes in **O(1)** time (average).
24:     fn get_full_mut2<Q>(&mut self, value: &Q) -> Option<(usize, &mut Self::Value)>
25:     where
26:         Q: ?Sized + Hash + Equivalent<Self::Value>;
27:     /// Return mutable reference to the value at an index.
28:     ///
29:     /// Valid indices are `0 <= index < self.len()`.
30:     ///
31:     /// Computes in **O(1)** time.
32:     fn get_index_mut2(&mut self, index: usize) -> Option<&mut Self::Value>;
33:     /// Scan through each value in the set and keep those where the
34:     /// closure `keep` returns `true`.
35:     ///
36:     /// The values are visited in order, and remaining values keep their order.
37:     ///
38:     /// Computes in **O(n)** time (average).
39:     fn retain2<F>(&mut self, keep: F)
40:     where
41:         F: FnMut(&mut Self::Value) -> bool;
42: }
43: /// Opt-in mutable access to [`OrderSet`] values.
44: ///
45: /// See [`MutableValues`] for more information.
46: impl<T, S> MutableValues for OrderSet<T, S>
47: where
48:     S: BuildHasher,
49: {
50:     type Value = T;
51:     fn get_full_mut2<Q>(&mut self, value: &Q) -> Option<(usize, &mut T)>
52:     where
53:         Q: ?Sized + Hash + Equivalent<T>,
54:     {
55:         self.inner.get_full_mut2(value)
56:     }
57:     fn get_index_mut2(&mut self, index: usize) -> Option<&mut T> {
58:         self.inner.get_index_mut2(index)
59:     }
60:     fn retain2<F>(&mut self, keep: F)
61:     where
62:         F: FnMut(&mut T) -> bool,
63:     {
64:         self.inner.retain2(keep);
65:     }
66: }
67: trait Sealed {}
68: impl<T, S> Sealed for OrderSet<T, S> {}
````

## File: src/set/rayon.rs
````rust
  1: //! Parallel iterator types for [`OrderSet`] with [rayon].
  2: //!
  3: //! You will rarely need to interact with this module directly unless you need to name one of the
  4: //! iterator types.
  5: pub use indexmap::set::rayon::{
  6:     IntoParIter, ParDifference, ParDrain, ParIntersection, ParIter, ParSymmetricDifference,
  7:     ParUnion,
  8: };
  9: use super::OrderSet;
 10: use core::cmp::Ordering;
 11: use core::hash::{BuildHasher, Hash};
 12: use core::ops::RangeBounds;
 13: use rayon::prelude::*;
 14: impl<T, S> IntoParallelIterator for OrderSet<T, S>
 15: where
 16:     T: Send,
 17: {
 18:     type Item = T;
 19:     type Iter = IntoParIter<T>;
 20:     fn into_par_iter(self) -> Self::Iter {
 21:         self.inner.into_par_iter()
 22:     }
 23: }
 24: impl<'a, T, S> IntoParallelIterator for &'a OrderSet<T, S>
 25: where
 26:     T: Sync,
 27: {
 28:     type Item = &'a T;
 29:     type Iter = ParIter<'a, T>;
 30:     fn into_par_iter(self) -> Self::Iter {
 31:         self.inner.par_iter()
 32:     }
 33: }
 34: impl<'a, T, S> ParallelDrainRange<usize> for &'a mut OrderSet<T, S>
 35: where
 36:     T: Send,
 37: {
 38:     type Item = T;
 39:     type Iter = ParDrain<'a, T>;
 40:     fn par_drain<R: RangeBounds<usize>>(self, range: R) -> Self::Iter {
 41:         self.inner.par_drain(range)
 42:     }
 43: }
 44: impl<T, S> OrderSet<T, S>
 45: where
 46:     T: PartialEq + Sync,
 47: {
 48:     /// Returns `true` if `self` contains all of the same values as `other`,
 49:     /// in the same indexed order, determined in parallel.
 50:     pub fn par_eq<S2>(&self, other: &OrderSet<T, S2>) -> bool
 51:     where
 52:         S2: BuildHasher + Sync,
 53:     {
 54:         self.len() == other.len() && self.par_iter().eq(other)
 55:     }
 56: }
 57: /// Parallel iterator methods and other parallel methods.
 58: ///
 59: /// The following methods **require crate feature `"rayon"`**.
 60: ///
 61: /// See also the `IntoParallelIterator` implementations.
 62: impl<T, S> OrderSet<T, S>
 63: where
 64:     T: Hash + Eq + Sync,
 65:     S: BuildHasher + Sync,
 66: {
 67:     /// Return a parallel iterator over the values that are in `self` but not `other`.
 68:     ///
 69:     /// While parallel iterators can process items in any order, their relative order
 70:     /// in the `self` set is still preserved for operations like `reduce` and `collect`.
 71:     pub fn par_difference<'a, S2>(
 72:         &'a self,
 73:         other: &'a OrderSet<T, S2>,
 74:     ) -> ParDifference<'a, T, S, S2>
 75:     where
 76:         S2: BuildHasher + Sync,
 77:     {
 78:         self.inner.par_difference(&other.inner)
 79:     }
 80:     /// Return a parallel iterator over the values that are in `self` or `other`,
 81:     /// but not in both.
 82:     ///
 83:     /// While parallel iterators can process items in any order, their relative order
 84:     /// in the sets is still preserved for operations like `reduce` and `collect`.
 85:     /// Values from `self` are produced in their original order, followed by
 86:     /// values from `other` in their original order.
 87:     pub fn par_symmetric_difference<'a, S2>(
 88:         &'a self,
 89:         other: &'a OrderSet<T, S2>,
 90:     ) -> ParSymmetricDifference<'a, T, S, S2>
 91:     where
 92:         S2: BuildHasher + Sync,
 93:     {
 94:         self.inner.par_symmetric_difference(&other.inner)
 95:     }
 96:     /// Return a parallel iterator over the values that are in both `self` and `other`.
 97:     ///
 98:     /// While parallel iterators can process items in any order, their relative order
 99:     /// in the `self` set is still preserved for operations like `reduce` and `collect`.
100:     pub fn par_intersection<'a, S2>(
101:         &'a self,
102:         other: &'a OrderSet<T, S2>,
103:     ) -> ParIntersection<'a, T, S, S2>
104:     where
105:         S2: BuildHasher + Sync,
106:     {
107:         self.inner.par_intersection(&other.inner)
108:     }
109:     /// Return a parallel iterator over all values that are in `self` or `other`.
110:     ///
111:     /// While parallel iterators can process items in any order, their relative order
112:     /// in the sets is still preserved for operations like `reduce` and `collect`.
113:     /// Values from `self` are produced in their original order, followed by
114:     /// values that are unique to `other` in their original order.
115:     pub fn par_union<'a, S2>(&'a self, other: &'a OrderSet<T, S2>) -> ParUnion<'a, T, S, S2>
116:     where
117:         S2: BuildHasher + Sync,
118:     {
119:         self.inner.par_union(&other.inner)
120:     }
121:     /// Returns `true` if `self` has no elements in common with `other`,
122:     /// determined in parallel.
123:     pub fn par_is_disjoint<S2>(&self, other: &OrderSet<T, S2>) -> bool
124:     where
125:         S2: BuildHasher + Sync,
126:     {
127:         self.inner.par_is_disjoint(&other.inner)
128:     }
129:     /// Returns `true` if all elements of `other` are contained in `self`,
130:     /// determined in parallel.
131:     pub fn par_is_superset<S2>(&self, other: &OrderSet<T, S2>) -> bool
132:     where
133:         S2: BuildHasher + Sync,
134:     {
135:         self.inner.par_is_superset(&other.inner)
136:     }
137:     /// Returns `true` if all elements of `self` are contained in `other`,
138:     /// determined in parallel.
139:     pub fn par_is_subset<S2>(&self, other: &OrderSet<T, S2>) -> bool
140:     where
141:         S2: BuildHasher + Sync,
142:     {
143:         self.inner.par_is_subset(&other.inner)
144:     }
145:     /// Returns `true` if `self` contains all of the same values as `other`,
146:     /// regardless of each set's indexed order, determined in parallel.
147:     pub fn par_set_eq<S2>(&self, other: &OrderSet<T, S2>) -> bool
148:     where
149:         S2: BuildHasher + Sync,
150:     {
151:         self.inner.par_eq(&other.inner)
152:     }
153: }
154: /// Parallel sorting methods.
155: ///
156: /// The following methods **require crate feature `"rayon"`**.
157: impl<T, S> OrderSet<T, S>
158: where
159:     T: Send,
160: {
161:     /// Sort the set's values in parallel by their default ordering.
162:     pub fn par_sort(&mut self)
163:     where
164:         T: Ord,
165:     {
166:         self.inner.par_sort();
167:     }
168:     /// Sort the set's values in place and in parallel, using the comparison function `cmp`.
169:     pub fn par_sort_by<F>(&mut self, cmp: F)
170:     where
171:         F: Fn(&T, &T) -> Ordering + Sync,
172:     {
173:         self.inner.par_sort_by(cmp);
174:     }
175:     /// Sort the values of the set in parallel and return a by-value parallel iterator of
176:     /// the values with the result.
177:     pub fn par_sorted_by<F>(self, cmp: F) -> IntoParIter<T>
178:     where
179:         F: Fn(&T, &T) -> Ordering + Sync,
180:     {
181:         self.inner.par_sorted_by(cmp)
182:     }
183:     /// Sort the set's values in place and in parallel, using a key extraction function.
184:     pub fn par_sort_by_key<K, F>(&mut self, sort_key: F)
185:     where
186:         K: Ord,
187:         F: Fn(&T) -> K + Sync,
188:     {
189:         self.inner.par_sort_by_key(sort_key)
190:     }
191:     /// Sort the set's values in parallel by their default ordering.
192:     pub fn par_sort_unstable(&mut self)
193:     where
194:         T: Ord,
195:     {
196:         self.inner.par_sort_unstable();
197:     }
198:     /// Sort the set's values in place and in parallel, using the comparison function `cmp`.
199:     pub fn par_sort_unstable_by<F>(&mut self, cmp: F)
200:     where
201:         F: Fn(&T, &T) -> Ordering + Sync,
202:     {
203:         self.inner.par_sort_unstable_by(cmp);
204:     }
205:     /// Sort the values of the set in parallel and return a by-value parallel iterator of
206:     /// the values with the result.
207:     pub fn par_sorted_unstable_by<F>(self, cmp: F) -> IntoParIter<T>
208:     where
209:         F: Fn(&T, &T) -> Ordering + Sync,
210:     {
211:         self.inner.par_sorted_unstable_by(cmp)
212:     }
213:     /// Sort the set's values in place and in parallel, using a key extraction function.
214:     pub fn par_sort_unstable_by_key<K, F>(&mut self, sort_key: F)
215:     where
216:         K: Ord,
217:         F: Fn(&T) -> K + Sync,
218:     {
219:         self.inner.par_sort_unstable_by_key(sort_key)
220:     }
221:     /// Sort the set's values in place and in parallel, using a key extraction function.
222:     pub fn par_sort_by_cached_key<K, F>(&mut self, sort_key: F)
223:     where
224:         K: Ord + Send,
225:         F: Fn(&T) -> K + Sync,
226:     {
227:         self.inner.par_sort_by_cached_key(sort_key);
228:     }
229: }
230: impl<T, S> FromParallelIterator<T> for OrderSet<T, S>
231: where
232:     T: Eq + Hash + Send,
233:     S: BuildHasher + Default + Send,
234: {
235:     fn from_par_iter<I>(iter: I) -> Self
236:     where
237:         I: IntoParallelIterator<Item = T>,
238:     {
239:         Self {
240:             inner: <_>::from_par_iter(iter),
241:         }
242:     }
243: }
244: impl<T, S> ParallelExtend<T> for OrderSet<T, S>
245: where
246:     T: Eq + Hash + Send,
247:     S: BuildHasher + Send,
248: {
249:     fn par_extend<I>(&mut self, iter: I)
250:     where
251:         I: IntoParallelIterator<Item = T>,
252:     {
253:         self.inner.par_extend(iter);
254:     }
255: }
256: impl<'a, T: 'a, S> ParallelExtend<&'a T> for OrderSet<T, S>
257: where
258:     T: Copy + Eq + Hash + Send + Sync,
259:     S: BuildHasher + Send,
260: {
261:     fn par_extend<I>(&mut self, iter: I)
262:     where
263:         I: IntoParallelIterator<Item = &'a T>,
264:     {
265:         self.inner.par_extend(iter);
266:     }
267: }
268: #[cfg(test)]
269: mod tests {
270:     use super::*;
271:     use std::vec::Vec;
272:     #[test]
273:     fn insert_order() {
274:         let insert = [0, 4, 2, 12, 8, 7, 11, 5, 3, 17, 19, 22, 23];
275:         let mut set = OrderSet::new();
276:         for &elt in &insert {
277:             set.insert(elt);
278:         }
279:         assert_eq!(set.par_iter().count(), set.len());
280:         assert_eq!(set.par_iter().count(), insert.len());
281:         insert.par_iter().zip(&set).for_each(|(a, b)| {
282:             assert_eq!(a, b);
283:         });
284:         (0..insert.len())
285:             .into_par_iter()
286:             .zip(&set)
287:             .for_each(|(i, v)| {
288:                 assert_eq!(set.get_index(i).unwrap(), v);
289:             });
290:     }
291:     #[test]
292:     fn partial_eq_and_eq() {
293:         let mut set_a = OrderSet::new();
294:         set_a.insert(1);
295:         set_a.insert(2);
296:         let mut set_b = set_a.clone();
297:         assert!(set_a.par_eq(&set_b));
298:         set_b.swap_remove(&1);
299:         assert!(!set_a.par_eq(&set_b));
300:         set_b.insert(3);
301:         assert!(!set_a.par_eq(&set_b));
302:         let set_c: OrderSet<_> = set_b.into_par_iter().collect();
303:         assert!(!set_a.par_eq(&set_c));
304:         assert!(!set_c.par_eq(&set_a));
305:     }
306:     #[test]
307:     fn extend() {
308:         let mut set = OrderSet::new();
309:         set.par_extend(vec![&1, &2, &3, &4]);
310:         set.par_extend(vec![5, 6]);
311:         assert_eq!(
312:             set.into_par_iter().collect::<Vec<_>>(),
313:             vec![1, 2, 3, 4, 5, 6]
314:         );
315:     }
316:     #[test]
317:     fn comparisons() {
318:         let set_a: OrderSet<_> = (0..3).collect();
319:         let set_b: OrderSet<_> = (3..6).collect();
320:         let set_c: OrderSet<_> = (0..6).collect();
321:         let set_d: OrderSet<_> = (3..9).collect();
322:         assert!(!set_a.par_is_disjoint(&set_a));
323:         assert!(set_a.par_is_subset(&set_a));
324:         assert!(set_a.par_is_superset(&set_a));
325:         assert!(set_a.par_is_disjoint(&set_b));
326:         assert!(set_b.par_is_disjoint(&set_a));
327:         assert!(!set_a.par_is_subset(&set_b));
328:         assert!(!set_b.par_is_subset(&set_a));
329:         assert!(!set_a.par_is_superset(&set_b));
330:         assert!(!set_b.par_is_superset(&set_a));
331:         assert!(!set_a.par_is_disjoint(&set_c));
332:         assert!(!set_c.par_is_disjoint(&set_a));
333:         assert!(set_a.par_is_subset(&set_c));
334:         assert!(!set_c.par_is_subset(&set_a));
335:         assert!(!set_a.par_is_superset(&set_c));
336:         assert!(set_c.par_is_superset(&set_a));
337:         assert!(!set_c.par_is_disjoint(&set_d));
338:         assert!(!set_d.par_is_disjoint(&set_c));
339:         assert!(!set_c.par_is_subset(&set_d));
340:         assert!(!set_d.par_is_subset(&set_c));
341:         assert!(!set_c.par_is_superset(&set_d));
342:         assert!(!set_d.par_is_superset(&set_c));
343:     }
344:     #[test]
345:     fn iter_comparisons() {
346:         use std::iter::empty;
347:         fn check<'a, I1, I2>(iter1: I1, iter2: I2)
348:         where
349:             I1: ParallelIterator<Item = &'a i32>,
350:             I2: Iterator<Item = i32>,
351:         {
352:             let v1: Vec<_> = iter1.copied().collect();
353:             let v2: Vec<_> = iter2.collect();
354:             assert_eq!(v1, v2);
355:         }
356:         let set_a: OrderSet<_> = (0..3).collect();
357:         let set_b: OrderSet<_> = (3..6).collect();
358:         let set_c: OrderSet<_> = (0..6).collect();
359:         let set_d: OrderSet<_> = (3..9).rev().collect();
360:         check(set_a.par_difference(&set_a), empty());
361:         check(set_a.par_symmetric_difference(&set_a), empty());
362:         check(set_a.par_intersection(&set_a), 0..3);
363:         check(set_a.par_union(&set_a), 0..3);
364:         check(set_a.par_difference(&set_b), 0..3);
365:         check(set_b.par_difference(&set_a), 3..6);
366:         check(set_a.par_symmetric_difference(&set_b), 0..6);
367:         check(set_b.par_symmetric_difference(&set_a), (3..6).chain(0..3));
368:         check(set_a.par_intersection(&set_b), empty());
369:         check(set_b.par_intersection(&set_a), empty());
370:         check(set_a.par_union(&set_b), 0..6);
371:         check(set_b.par_union(&set_a), (3..6).chain(0..3));
372:         check(set_a.par_difference(&set_c), empty());
373:         check(set_c.par_difference(&set_a), 3..6);
374:         check(set_a.par_symmetric_difference(&set_c), 3..6);
375:         check(set_c.par_symmetric_difference(&set_a), 3..6);
376:         check(set_a.par_intersection(&set_c), 0..3);
377:         check(set_c.par_intersection(&set_a), 0..3);
378:         check(set_a.par_union(&set_c), 0..6);
379:         check(set_c.par_union(&set_a), 0..6);
380:         check(set_c.par_difference(&set_d), 0..3);
381:         check(set_d.par_difference(&set_c), (6..9).rev());
382:         check(
383:             set_c.par_symmetric_difference(&set_d),
384:             (0..3).chain((6..9).rev()),
385:         );
386:         check(
387:             set_d.par_symmetric_difference(&set_c),
388:             (6..9).rev().chain(0..3),
389:         );
390:         check(set_c.par_intersection(&set_d), 3..6);
391:         check(set_d.par_intersection(&set_c), (3..6).rev());
392:         check(set_c.par_union(&set_d), (0..6).chain((6..9).rev()));
393:         check(set_d.par_union(&set_c), (3..9).rev().chain(0..3));
394:     }
395: }
````

## File: src/set/slice.rs
````rust
 1: use super::OrderSet;
 2: use core::ops::{self, Bound, Index};
 3: use indexmap::set::Slice;
 4: // We can't have `impl<I: RangeBounds<usize>> Index<I>` because that conflicts with `Index<usize>`.
 5: // Instead, we repeat the implementations for all the core range types.
 6: macro_rules! impl_index {
 7:     ($($range:ty),*) => {$(
 8:         impl<T, S> Index<$range> for OrderSet<T, S> {
 9:             type Output = Slice<T>;
10:             fn index(&self, range: $range) -> &Self::Output {
11:                 &self.inner[range]
12:             }
13:         }
14:     )*}
15: }
16: impl_index!(
17:     ops::Range<usize>,
18:     ops::RangeFrom<usize>,
19:     ops::RangeFull,
20:     ops::RangeInclusive<usize>,
21:     ops::RangeTo<usize>,
22:     ops::RangeToInclusive<usize>,
23:     (Bound<usize>, Bound<usize>)
24: );
25: #[cfg(test)]
26: mod tests {
27:     use super::*;
28:     use alloc::vec::Vec;
29:     #[test]
30:     fn slice_index() {
31:         fn check(vec_slice: &[i32], set_slice: &Slice<i32>, sub_slice: &Slice<i32>) {
32:             assert_eq!(set_slice as *const _, sub_slice as *const _);
33:             itertools::assert_equal(vec_slice, set_slice);
34:         }
35:         let vec: Vec<i32> = (0..10).map(|i| i * i).collect();
36:         let set: OrderSet<i32> = vec.iter().cloned().collect();
37:         let slice = set.as_slice();
38:         // RangeFull
39:         #[expect(clippy::redundant_slicing)]
40:         check(&vec[..], &set[..], &slice[..]);
41:         for i in 0usize..10 {
42:             // Index
43:             assert_eq!(vec[i], set[i]);
44:             assert_eq!(vec[i], slice[i]);
45:             // RangeFrom
46:             check(&vec[i..], &set[i..], &slice[i..]);
47:             // RangeTo
48:             check(&vec[..i], &set[..i], &slice[..i]);
49:             // RangeToInclusive
50:             check(&vec[..=i], &set[..=i], &slice[..=i]);
51:             // (Bound<usize>, Bound<usize>)
52:             let bounds = (Bound::Excluded(i), Bound::Unbounded);
53:             check(&vec[i + 1..], &set[bounds], &slice[bounds]);
54:             for j in i..=10 {
55:                 // Range
56:                 check(&vec[i..j], &set[i..j], &slice[i..j]);
57:             }
58:             for j in i..10 {
59:                 // RangeInclusive
60:                 check(&vec[i..=j], &set[i..=j], &slice[i..=j]);
61:             }
62:         }
63:     }
64: }
````

## File: src/set/tests.rs
````rust
  1: use super::*;
  2: use std::string::String;
  3: use std::vec::Vec;
  4: #[test]
  5: fn it_works() {
  6:     let mut set = OrderSet::new();
  7:     assert_eq!(set.is_empty(), true);
  8:     set.insert(1);
  9:     set.insert(1);
 10:     assert_eq!(set.len(), 1);
 11:     assert!(set.get(&1).is_some());
 12:     assert_eq!(set.is_empty(), false);
 13: }
 14: #[test]
 15: fn new() {
 16:     let set = OrderSet::<String>::new();
 17:     println!("{:?}", set);
 18:     assert_eq!(set.capacity(), 0);
 19:     assert_eq!(set.len(), 0);
 20:     assert_eq!(set.is_empty(), true);
 21: }
 22: #[test]
 23: fn insert() {
 24:     let insert = [0, 4, 2, 12, 8, 7, 11, 5];
 25:     let not_present = [1, 3, 6, 9, 10];
 26:     let mut set = OrderSet::with_capacity(insert.len());
 27:     for (i, &elt) in insert.iter().enumerate() {
 28:         assert_eq!(set.len(), i);
 29:         set.insert(elt);
 30:         assert_eq!(set.len(), i + 1);
 31:         assert_eq!(set.get(&elt), Some(&elt));
 32:     }
 33:     println!("{:?}", set);
 34:     for &elt in &not_present {
 35:         assert!(set.get(&elt).is_none());
 36:     }
 37: }
 38: #[test]
 39: fn insert_full() {
 40:     let insert = [9, 2, 7, 1, 4, 6, 13];
 41:     let present = [1, 6, 2];
 42:     let mut set = OrderSet::with_capacity(insert.len());
 43:     for (i, &elt) in insert.iter().enumerate() {
 44:         assert_eq!(set.len(), i);
 45:         let (index, success) = set.insert_full(elt);
 46:         assert!(success);
 47:         assert_eq!(Some(index), set.get_full(&elt).map(|x| x.0));
 48:         assert_eq!(set.len(), i + 1);
 49:     }
 50:     let len = set.len();
 51:     for &elt in &present {
 52:         let (index, success) = set.insert_full(elt);
 53:         assert!(!success);
 54:         assert_eq!(Some(index), set.get_full(&elt).map(|x| x.0));
 55:         assert_eq!(set.len(), len);
 56:     }
 57: }
 58: #[test]
 59: fn insert_2() {
 60:     let mut set = OrderSet::with_capacity(16);
 61:     let mut values = vec![];
 62:     values.extend(0..16);
 63:     values.extend(if cfg!(miri) { 32..64 } else { 128..267 });
 64:     for &i in &values {
 65:         let old_set = set.clone();
 66:         set.insert(i);
 67:         for value in old_set.iter() {
 68:             if set.get(value).is_none() {
 69:                 println!("old_set: {:?}", old_set);
 70:                 println!("set: {:?}", set);
 71:                 panic!("did not find {} in set", value);
 72:             }
 73:         }
 74:     }
 75:     for &i in &values {
 76:         assert!(set.get(&i).is_some(), "did not find {}", i);
 77:     }
 78: }
 79: #[test]
 80: fn insert_dup() {
 81:     let mut set = OrderSet::<u8>::from_iter([0, 2, 4, 6, 8]);
 82:     {
 83:         let (i, v) = set.get_full(&0).unwrap();
 84:         assert_eq!(set.len(), 5);
 85:         assert_eq!(i, 0);
 86:         assert_eq!(*v, 0);
 87:     }
 88:     {
 89:         let inserted = set.insert(0);
 90:         let (i, v) = set.get_full(&0).unwrap();
 91:         assert_eq!(set.len(), 5);
 92:         assert_eq!(inserted, false);
 93:         assert_eq!(i, 0);
 94:         assert_eq!(*v, 0);
 95:     }
 96: }
 97: #[test]
 98: fn insert_order() {
 99:     let insert = [0, 4, 2, 12, 8, 7, 11, 5, 3, 17, 19, 22, 23];
100:     let mut set = OrderSet::new();
101:     for &elt in &insert {
102:         set.insert(elt);
103:     }
104:     assert_eq!(set.iter().count(), set.len());
105:     assert_eq!(set.iter().count(), insert.len());
106:     for (a, b) in insert.iter().zip(set.iter()) {
107:         assert_eq!(a, b);
108:     }
109:     for (i, v) in (0..insert.len()).zip(set.iter()) {
110:         assert_eq!(set.get_index(i).unwrap(), v);
111:     }
112: }
113: #[test]
114: fn shift_insert() {
115:     let insert = [0, 4, 2, 12, 8, 7, 11, 5, 3, 17, 19, 22, 23];
116:     let mut set = OrderSet::new();
117:     for &elt in &insert {
118:         set.shift_insert(0, elt);
119:     }
120:     assert_eq!(set.iter().count(), set.len());
121:     assert_eq!(set.iter().count(), insert.len());
122:     for (a, b) in insert.iter().rev().zip(set.iter()) {
123:         assert_eq!(a, b);
124:     }
125:     for (i, v) in (0..insert.len()).zip(set.iter()) {
126:         assert_eq!(set.get_index(i).unwrap(), v);
127:     }
128:     // "insert" that moves an existing entry
129:     set.shift_insert(0, insert[0]);
130:     assert_eq!(set.iter().count(), insert.len());
131:     assert_eq!(insert[0], set[0]);
132:     for (a, b) in insert[1..].iter().rev().zip(set.iter().skip(1)) {
133:         assert_eq!(a, b);
134:     }
135: }
136: #[test]
137: fn replace() {
138:     let replace = [0, 4, 2, 12, 8, 7, 11, 5];
139:     let not_present = [1, 3, 6, 9, 10];
140:     let mut set = OrderSet::with_capacity(replace.len());
141:     for (i, &elt) in replace.iter().enumerate() {
142:         assert_eq!(set.len(), i);
143:         set.replace(elt);
144:         assert_eq!(set.len(), i + 1);
145:         assert_eq!(set.get(&elt), Some(&elt));
146:     }
147:     println!("{:?}", set);
148:     for &elt in &not_present {
149:         assert!(set.get(&elt).is_none());
150:     }
151: }
152: #[test]
153: fn replace_full() {
154:     let replace = [9, 2, 7, 1, 4, 6, 13];
155:     let present = [1, 6, 2];
156:     let mut set = OrderSet::with_capacity(replace.len());
157:     for (i, &elt) in replace.iter().enumerate() {
158:         assert_eq!(set.len(), i);
159:         let (index, replaced) = set.replace_full(elt);
160:         assert!(replaced.is_none());
161:         assert_eq!(Some(index), set.get_full(&elt).map(|x| x.0));
162:         assert_eq!(set.len(), i + 1);
163:     }
164:     let len = set.len();
165:     for &elt in &present {
166:         let (index, replaced) = set.replace_full(elt);
167:         assert_eq!(Some(elt), replaced);
168:         assert_eq!(Some(index), set.get_full(&elt).map(|x| x.0));
169:         assert_eq!(set.len(), len);
170:     }
171: }
172: #[test]
173: fn replace_2() {
174:     let mut set = OrderSet::with_capacity(16);
175:     let mut values = vec![];
176:     values.extend(0..16);
177:     values.extend(if cfg!(miri) { 32..64 } else { 128..267 });
178:     for &i in &values {
179:         let old_set = set.clone();
180:         set.replace(i);
181:         for value in old_set.iter() {
182:             if set.get(value).is_none() {
183:                 println!("old_set: {:?}", old_set);
184:                 println!("set: {:?}", set);
185:                 panic!("did not find {} in set", value);
186:             }
187:         }
188:     }
189:     for &i in &values {
190:         assert!(set.get(&i).is_some(), "did not find {}", i);
191:     }
192: }
193: #[test]
194: fn replace_dup() {
195:     let mut set = OrderSet::<u8>::from_iter([0, 2, 4, 6, 8]);
196:     {
197:         let (i, v) = set.get_full(&0).unwrap();
198:         assert_eq!(set.len(), 5);
199:         assert_eq!(i, 0);
200:         assert_eq!(*v, 0);
201:     }
202:     {
203:         let replaced = set.replace(0);
204:         let (i, v) = set.get_full(&0).unwrap();
205:         assert_eq!(set.len(), 5);
206:         assert_eq!(replaced, Some(0));
207:         assert_eq!(i, 0);
208:         assert_eq!(*v, 0);
209:     }
210: }
211: #[test]
212: fn replace_order() {
213:     let replace = [0, 4, 2, 12, 8, 7, 11, 5, 3, 17, 19, 22, 23];
214:     let mut set = OrderSet::new();
215:     for &elt in &replace {
216:         set.replace(elt);
217:     }
218:     assert_eq!(set.iter().count(), set.len());
219:     assert_eq!(set.iter().count(), replace.len());
220:     for (a, b) in replace.iter().zip(set.iter()) {
221:         assert_eq!(a, b);
222:     }
223:     for (i, v) in (0..replace.len()).zip(set.iter()) {
224:         assert_eq!(set.get_index(i).unwrap(), v);
225:     }
226: }
227: #[test]
228: fn replace_change() {
229:     // Check pointers to make sure it really changes
230:     let mut set = orderset!(vec![42]);
231:     let old_ptr = set[0].as_ptr();
232:     let new = set[0].clone();
233:     let new_ptr = new.as_ptr();
234:     assert_ne!(old_ptr, new_ptr);
235:     let replaced = set.replace(new).unwrap();
236:     assert_eq!(replaced.as_ptr(), old_ptr);
237: }
238: #[test]
239: fn grow() {
240:     let insert = [0, 4, 2, 12, 8, 7, 11];
241:     let not_present = [1, 3, 6, 9, 10];
242:     let mut set = OrderSet::with_capacity(insert.len());
243:     for (i, &elt) in insert.iter().enumerate() {
244:         assert_eq!(set.len(), i);
245:         set.insert(elt);
246:         assert_eq!(set.len(), i + 1);
247:         assert_eq!(set.get(&elt), Some(&elt));
248:     }
249:     println!("{:?}", set);
250:     for &elt in &insert {
251:         set.insert(elt * 10);
252:     }
253:     for &elt in &insert {
254:         set.insert(elt * 100);
255:     }
256:     for (i, &elt) in insert.iter().cycle().enumerate().take(100) {
257:         set.insert(elt * 100 + i as i32);
258:     }
259:     println!("{:?}", set);
260:     for &elt in &not_present {
261:         assert!(set.get(&elt).is_none());
262:     }
263: }
264: #[test]
265: fn reserve() {
266:     let mut set = OrderSet::<usize>::new();
267:     assert_eq!(set.capacity(), 0);
268:     set.reserve(100);
269:     let capacity = set.capacity();
270:     assert!(capacity >= 100);
271:     for i in 0..capacity {
272:         assert_eq!(set.len(), i);
273:         set.insert(i);
274:         assert_eq!(set.len(), i + 1);
275:         assert_eq!(set.capacity(), capacity);
276:         assert_eq!(set.get(&i), Some(&i));
277:     }
278:     set.insert(capacity);
279:     assert_eq!(set.len(), capacity + 1);
280:     assert!(set.capacity() > capacity);
281:     assert_eq!(set.get(&capacity), Some(&capacity));
282: }
283: #[test]
284: fn try_reserve() {
285:     let mut set = OrderSet::<usize>::new();
286:     assert_eq!(set.capacity(), 0);
287:     assert_eq!(set.try_reserve(100), Ok(()));
288:     assert!(set.capacity() >= 100);
289:     assert!(set.try_reserve(usize::MAX).is_err());
290: }
291: #[test]
292: fn shrink_to_fit() {
293:     let mut set = OrderSet::<usize>::new();
294:     assert_eq!(set.capacity(), 0);
295:     for i in 0..100 {
296:         assert_eq!(set.len(), i);
297:         set.insert(i);
298:         assert_eq!(set.len(), i + 1);
299:         assert!(set.capacity() > i);
300:         assert_eq!(set.get(&i), Some(&i));
301:         set.shrink_to_fit();
302:         assert_eq!(set.len(), i + 1);
303:         assert_eq!(set.capacity(), i + 1);
304:         assert_eq!(set.get(&i), Some(&i));
305:     }
306: }
307: #[test]
308: fn remove() {
309:     let insert = [0, 4, 2, 12, 8, 7, 11, 5, 3, 17, 19, 22, 23];
310:     let mut set = OrderSet::new();
311:     for &elt in &insert {
312:         set.insert(elt);
313:     }
314:     assert_eq!(set.iter().count(), set.len());
315:     assert_eq!(set.iter().count(), insert.len());
316:     for (a, b) in insert.iter().zip(set.iter()) {
317:         assert_eq!(a, b);
318:     }
319:     let remove_fail = [99, 77];
320:     let remove = [4, 12, 8, 7];
321:     for &value in &remove_fail {
322:         assert!(set.swap_remove_full(&value).is_none());
323:     }
324:     println!("{:?}", set);
325:     for &value in &remove {
326:         //println!("{:?}", set);
327:         let index = set.get_full(&value).unwrap().0;
328:         assert_eq!(set.swap_remove_full(&value), Some((index, value)));
329:     }
330:     println!("{:?}", set);
331:     for value in &insert {
332:         assert_eq!(set.get(value).is_some(), !remove.contains(value));
333:     }
334:     assert_eq!(set.len(), insert.len() - remove.len());
335:     assert_eq!(set.iter().count(), insert.len() - remove.len());
336: }
337: #[test]
338: fn swap_remove_index() {
339:     let insert = [0, 4, 2, 12, 8, 7, 11, 5, 3, 17, 19, 22, 23];
340:     let mut set = OrderSet::new();
341:     for &elt in &insert {
342:         set.insert(elt);
343:     }
344:     let mut vector = insert.to_vec();
345:     let remove_sequence = &[3, 3, 10, 4, 5, 4, 3, 0, 1];
346:     // check that the same swap remove sequence on vec and set
347:     // have the same result.
348:     for &rm in remove_sequence {
349:         let out_vec = vector.swap_remove(rm);
350:         let out_set = set.swap_remove_index(rm).unwrap();
351:         assert_eq!(out_vec, out_set);
352:     }
353:     assert_eq!(vector.len(), set.len());
354:     for (a, b) in vector.iter().zip(set.iter()) {
355:         assert_eq!(a, b);
356:     }
357: }
358: #[test]
359: fn partial_eq_and_eq() {
360:     let mut set_a = OrderSet::new();
361:     set_a.insert(1);
362:     set_a.insert(2);
363:     let mut set_b = set_a.clone();
364:     assert_eq!(set_a, set_b);
365:     set_b.swap_remove(&1);
366:     assert_ne!(set_a, set_b);
367:     let set_c: OrderSet<_> = set_b.into_iter().collect();
368:     assert_ne!(set_a, set_c);
369:     assert_ne!(set_c, set_a);
370: }
371: #[test]
372: fn extend() {
373:     let mut set = OrderSet::new();
374:     set.extend([&1, &2, &3, &4]);
375:     set.extend([5, 6]);
376:     assert_eq!(set.into_iter().collect::<Vec<_>>(), [1, 2, 3, 4, 5, 6]);
377: }
378: #[test]
379: fn comparisons() {
380:     let set_a: OrderSet<_> = (0..3).collect();
381:     let set_b: OrderSet<_> = (3..6).collect();
382:     let set_c: OrderSet<_> = (0..6).collect();
383:     let set_d: OrderSet<_> = (3..9).collect();
384:     assert!(!set_a.is_disjoint(&set_a));
385:     assert!(set_a.is_subset(&set_a));
386:     assert!(set_a.is_superset(&set_a));
387:     assert!(set_a.is_disjoint(&set_b));
388:     assert!(set_b.is_disjoint(&set_a));
389:     assert!(!set_a.is_subset(&set_b));
390:     assert!(!set_b.is_subset(&set_a));
391:     assert!(!set_a.is_superset(&set_b));
392:     assert!(!set_b.is_superset(&set_a));
393:     assert!(!set_a.is_disjoint(&set_c));
394:     assert!(!set_c.is_disjoint(&set_a));
395:     assert!(set_a.is_subset(&set_c));
396:     assert!(!set_c.is_subset(&set_a));
397:     assert!(!set_a.is_superset(&set_c));
398:     assert!(set_c.is_superset(&set_a));
399:     assert!(!set_c.is_disjoint(&set_d));
400:     assert!(!set_d.is_disjoint(&set_c));
401:     assert!(!set_c.is_subset(&set_d));
402:     assert!(!set_d.is_subset(&set_c));
403:     assert!(!set_c.is_superset(&set_d));
404:     assert!(!set_d.is_superset(&set_c));
405: }
406: #[test]
407: fn iter_comparisons() {
408:     use std::iter::empty;
409:     fn check<'a, I1, I2>(iter1: I1, iter2: I2)
410:     where
411:         I1: Iterator<Item = &'a i32>,
412:         I2: Iterator<Item = i32>,
413:     {
414:         assert!(iter1.copied().eq(iter2));
415:     }
416:     let set_a: OrderSet<_> = (0..3).collect();
417:     let set_b: OrderSet<_> = (3..6).collect();
418:     let set_c: OrderSet<_> = (0..6).collect();
419:     let set_d: OrderSet<_> = (3..9).rev().collect();
420:     check(set_a.difference(&set_a), empty());
421:     check(set_a.symmetric_difference(&set_a), empty());
422:     check(set_a.intersection(&set_a), 0..3);
423:     check(set_a.union(&set_a), 0..3);
424:     check(set_a.difference(&set_b), 0..3);
425:     check(set_b.difference(&set_a), 3..6);
426:     check(set_a.symmetric_difference(&set_b), 0..6);
427:     check(set_b.symmetric_difference(&set_a), (3..6).chain(0..3));
428:     check(set_a.intersection(&set_b), empty());
429:     check(set_b.intersection(&set_a), empty());
430:     check(set_a.union(&set_b), 0..6);
431:     check(set_b.union(&set_a), (3..6).chain(0..3));
432:     check(set_a.difference(&set_c), empty());
433:     check(set_c.difference(&set_a), 3..6);
434:     check(set_a.symmetric_difference(&set_c), 3..6);
435:     check(set_c.symmetric_difference(&set_a), 3..6);
436:     check(set_a.intersection(&set_c), 0..3);
437:     check(set_c.intersection(&set_a), 0..3);
438:     check(set_a.union(&set_c), 0..6);
439:     check(set_c.union(&set_a), 0..6);
440:     check(set_c.difference(&set_d), 0..3);
441:     check(set_d.difference(&set_c), (6..9).rev());
442:     check(
443:         set_c.symmetric_difference(&set_d),
444:         (0..3).chain((6..9).rev()),
445:     );
446:     check(set_d.symmetric_difference(&set_c), (6..9).rev().chain(0..3));
447:     check(set_c.intersection(&set_d), 3..6);
448:     check(set_d.intersection(&set_c), (3..6).rev());
449:     check(set_c.union(&set_d), (0..6).chain((6..9).rev()));
450:     check(set_d.union(&set_c), (3..9).rev().chain(0..3));
451: }
452: #[test]
453: fn ops() {
454:     let empty = OrderSet::<i32>::new();
455:     let set_a: OrderSet<_> = (0..3).collect();
456:     let set_b: OrderSet<_> = (3..6).collect();
457:     let set_c: OrderSet<_> = (0..6).collect();
458:     let set_d: OrderSet<_> = (3..9).rev().collect();
459:     #[allow(clippy::eq_op)]
460:     {
461:         assert_eq!(&set_a & &set_a, set_a);
462:         assert_eq!(&set_a | &set_a, set_a);
463:         assert_eq!(&set_a ^ &set_a, empty);
464:         assert_eq!(&set_a - &set_a, empty);
465:     }
466:     assert_eq!(&set_a & &set_b, empty);
467:     assert_eq!(&set_b & &set_a, empty);
468:     assert_eq!(&set_a | &set_b, set_c);
469:     assert_ne!(&set_b | &set_a, set_c);
470:     assert!((&set_b | &set_a).set_eq(&set_c));
471:     assert_eq!(&set_a ^ &set_b, set_c);
472:     assert_ne!(&set_b ^ &set_a, set_c);
473:     assert!((&set_b ^ &set_a).set_eq(&set_c));
474:     assert_eq!(&set_a - &set_b, set_a);
475:     assert_eq!(&set_b - &set_a, set_b);
476:     assert_eq!(&set_a & &set_c, set_a);
477:     assert_eq!(&set_c & &set_a, set_a);
478:     assert_eq!(&set_a | &set_c, set_c);
479:     assert_eq!(&set_c | &set_a, set_c);
480:     assert_eq!(&set_a ^ &set_c, set_b);
481:     assert_eq!(&set_c ^ &set_a, set_b);
482:     assert_eq!(&set_a - &set_c, empty);
483:     assert_eq!(&set_c - &set_a, set_b);
484:     assert_eq!(&set_c & &set_d, set_b);
485:     assert_ne!(&set_d & &set_c, set_b);
486:     assert!((&set_d & &set_c).set_eq(&set_b));
487:     assert_ne!(&set_c | &set_d, &set_a | &set_d);
488:     assert!((&set_c | &set_d).set_eq(&(&set_a | &set_d)));
489:     assert_ne!(&set_d | &set_c, &set_a | &set_d);
490:     assert!((&set_d | &set_c).set_eq(&(&set_a | &set_d)));
491:     assert_eq!(&set_c ^ &set_d, &set_a | &(&set_d - &set_b));
492:     assert_ne!(&set_d ^ &set_c, &set_a | &(&set_d - &set_b));
493:     assert!((&set_d ^ &set_c).set_eq(&(&set_a | &(&set_d - &set_b))));
494:     assert_eq!(&set_c - &set_d, set_a);
495:     assert_eq!(&set_d - &set_c, &set_d - &set_b);
496: }
497: #[test]
498: #[cfg(feature = "std")]
499: fn from_array() {
500:     let set1 = OrderSet::from([1, 2, 3, 4]);
501:     let set2: OrderSet<_> = [1, 2, 3, 4].into();
502:     assert_eq!(set1, set2);
503: }
504: #[test]
505: fn iter_default() {
506:     struct Item;
507:     fn assert_default<T>()
508:     where
509:         T: Default + Iterator,
510:     {
511:         assert!(T::default().next().is_none());
512:     }
513:     assert_default::<Iter<'static, Item>>();
514:     assert_default::<IntoIter<Item>>();
515: }
516: #[test]
517: #[allow(deprecated)]
518: fn take() {
519:     let mut index_set: OrderSet<i32> = OrderSet::new();
520:     index_set.insert(10);
521:     assert_eq!(index_set.len(), 1);
522:     let result = index_set.take(&10);
523:     assert_eq!(result, Some(10));
524:     assert_eq!(index_set.len(), 0);
525:     let result = index_set.take(&20);
526:     assert_eq!(result, None);
527: }
528: #[test]
529: fn swap_take() {
530:     let mut index_set: OrderSet<i32> = OrderSet::new();
531:     index_set.insert(10);
532:     index_set.insert(20);
533:     index_set.insert(30);
534:     index_set.insert(40);
535:     assert_eq!(index_set.len(), 4);
536:     let result = index_set.swap_take(&20);
537:     assert_eq!(result, Some(20));
538:     assert_eq!(index_set.len(), 3);
539:     assert_eq!(index_set.as_slice(), &[10, 40, 30]);
540:     let result = index_set.swap_take(&50);
541:     assert_eq!(result, None);
542: }
543: #[test]
544: fn sort_unstable() {
545:     let mut index_set: OrderSet<i32> = OrderSet::new();
546:     index_set.insert(30);
547:     index_set.insert(20);
548:     index_set.insert(10);
549:     index_set.sort_unstable();
550:     assert_eq!(index_set.as_slice(), &[10, 20, 30]);
551: }
552: #[test]
553: fn try_reserve_exact() {
554:     let mut index_set: OrderSet<i32> = OrderSet::new();
555:     index_set.insert(10);
556:     index_set.insert(20);
557:     index_set.insert(30);
558:     index_set.shrink_to_fit();
559:     assert_eq!(index_set.capacity(), 3);
560:     index_set.try_reserve_exact(2).unwrap();
561:     assert_eq!(index_set.capacity(), 5);
562: }
563: #[test]
564: fn shift_remove_full() {
565:     let mut set: OrderSet<i32> = OrderSet::new();
566:     set.insert(10);
567:     set.insert(20);
568:     set.insert(30);
569:     set.insert(40);
570:     set.insert(50);
571:     let result = set.remove_full(&20);
572:     assert_eq!(result, Some((1, 20)));
573:     assert_eq!(set.len(), 4);
574:     assert_eq!(set.as_slice(), &[10, 30, 40, 50]);
575:     let result = set.remove_full(&50);
576:     assert_eq!(result, Some((3, 50)));
577:     assert_eq!(set.len(), 3);
578:     assert_eq!(set.as_slice(), &[10, 30, 40]);
579:     let result = set.remove_full(&60);
580:     assert_eq!(result, None);
581:     assert_eq!(set.len(), 3);
582:     assert_eq!(set.as_slice(), &[10, 30, 40]);
583: }
584: #[test]
585: fn shift_remove_index() {
586:     let mut set: OrderSet<i32> = OrderSet::new();
587:     set.insert(10);
588:     set.insert(20);
589:     set.insert(30);
590:     set.insert(40);
591:     set.insert(50);
592:     let result = set.remove_index(1);
593:     assert_eq!(result, Some(20));
594:     assert_eq!(set.len(), 4);
595:     assert_eq!(set.as_slice(), &[10, 30, 40, 50]);
596:     let result = set.remove_index(1);
597:     assert_eq!(result, Some(30));
598:     assert_eq!(set.len(), 3);
599:     assert_eq!(set.as_slice(), &[10, 40, 50]);
600:     let result = set.remove_index(3);
601:     assert_eq!(result, None);
602:     assert_eq!(set.len(), 3);
603:     assert_eq!(set.as_slice(), &[10, 40, 50]);
604: }
605: #[test]
606: fn sort_unstable_by() {
607:     let mut set: OrderSet<i32> = OrderSet::from([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
608:     set.sort_unstable_by(|a, b| b.cmp(a));
609:     assert_eq!(set.as_slice(), &[10, 9, 8, 7, 6, 5, 4, 3, 2, 1]);
610: }
611: #[test]
612: fn sort_by() {
613:     let mut set: OrderSet<i32> = OrderSet::new();
614:     set.insert(3);
615:     set.insert(1);
616:     set.insert(2);
617:     set.sort_by(|a, b| a.cmp(b));
618:     assert_eq!(set.as_slice(), &[1, 2, 3]);
619: }
620: #[test]
621: fn drain() {
622:     let mut set: OrderSet<i32> = OrderSet::new();
623:     set.insert(1);
624:     set.insert(2);
625:     set.insert(3);
626:     {
627:         let drain = set.drain(0..2);
628:         assert_eq!(drain.as_slice(), &[1, 2]);
629:     }
630:     assert_eq!(set.len(), 1);
631:     assert_eq!(set.as_slice(), &[3]);
632: }
633: #[test]
634: fn split_off() {
635:     let mut set: OrderSet<i32> = OrderSet::from([1, 2, 3, 4, 5]);
636:     let split_set: OrderSet<i32> = set.split_off(3);
637:     assert_eq!(split_set.len(), 2);
638:     assert_eq!(split_set.as_slice(), &[4, 5]);
639:     assert_eq!(set.len(), 3);
640:     assert_eq!(set.as_slice(), &[1, 2, 3]);
641: }
642: #[test]
643: fn retain() {
644:     let mut set: OrderSet<i32> = OrderSet::from([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
645:     set.retain(|&x| x > 4);
646:     assert_eq!(set.len(), 6);
647:     assert_eq!(set.as_slice(), &[5, 6, 7, 8, 9, 10]);
648:     set.retain(|_| false);
649:     assert_eq!(set.len(), 0);
650: }
651: #[test]
652: fn first() {
653:     let mut index_set: OrderSet<i32> = OrderSet::new();
654:     index_set.insert(10);
655:     index_set.insert(20);
656:     index_set.insert(30);
657:     let result = index_set.first();
658:     assert_eq!(*result.unwrap(), 10);
659:     index_set.clear();
660:     let result = index_set.first();
661:     assert!(result.is_none());
662: }
663: #[test]
664: fn sort_by_key() {
665:     let mut index_set: OrderSet<i32> = OrderSet::new();
666:     index_set.insert(3);
667:     index_set.insert(1);
668:     index_set.insert(2);
669:     index_set.insert(0);
670:     index_set.sort_by_key(|&x| -x);
671:     assert_eq!(index_set.as_slice(), &[3, 2, 1, 0]);
672: }
673: #[test]
674: fn sort_unstable_by_key() {
675:     let mut index_set: OrderSet<i32> = OrderSet::new();
676:     index_set.insert(3);
677:     index_set.insert(1);
678:     index_set.insert(2);
679:     index_set.insert(0);
680:     index_set.sort_unstable_by_key(|&x| -x);
681:     assert_eq!(index_set.as_slice(), &[3, 2, 1, 0]);
682: }
683: #[test]
684: fn sort_by_cached_key() {
685:     let mut index_set: OrderSet<i32> = OrderSet::new();
686:     index_set.insert(3);
687:     index_set.insert(1);
688:     index_set.insert(2);
689:     index_set.insert(0);
690:     index_set.sort_by_cached_key(|&x| -x);
691:     assert_eq!(index_set.as_slice(), &[3, 2, 1, 0]);
692: }
693: #[test]
694: fn insert_sorted() {
695:     let mut set: OrderSet<i32> = OrderSet::<i32>::new();
696:     set.insert_sorted(1);
697:     set.insert_sorted(3);
698:     assert_eq!(set.insert_sorted(2), (1, true));
699: }
700: #[test]
701: fn binary_search() {
702:     let mut set: OrderSet<i32> = OrderSet::new();
703:     set.insert(100);
704:     set.insert(300);
705:     set.insert(200);
706:     set.insert(400);
707:     let result = set.binary_search(&200);
708:     assert_eq!(result, Ok(2));
709:     let result = set.binary_search(&500);
710:     assert_eq!(result, Err(4));
711: }
712: #[test]
713: fn sorted_unstable_by() {
714:     let mut set: OrderSet<i32> = OrderSet::from([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
715:     set.sort_unstable_by(|a, b| b.cmp(a));
716:     assert_eq!(set.as_slice(), &[10, 9, 8, 7, 6, 5, 4, 3, 2, 1]);
717: }
718: #[test]
719: fn last() {
720:     let mut set: OrderSet<i32> = OrderSet::new();
721:     set.insert(1);
722:     set.insert(2);
723:     set.insert(3);
724:     set.insert(4);
725:     set.insert(5);
726:     set.insert(6);
727:     assert_eq!(set.last(), Some(&6));
728:     set.pop();
729:     assert_eq!(set.last(), Some(&5));
730:     set.clear();
731:     assert_eq!(set.last(), None);
732: }
733: #[test]
734: fn get_range() {
735:     let set: OrderSet<i32> = OrderSet::from([1, 2, 3, 4, 5]);
736:     let result = set.get_range(0..3);
737:     let slice: &Slice<i32> = result.unwrap();
738:     assert_eq!(slice, &[1, 2, 3]);
739:     let result = set.get_range(0..0);
740:     assert_eq!(result.unwrap().len(), 0);
741:     #[expect(clippy::reversed_empty_ranges)]
742:     let result = set.get_range(2..1);
743:     assert!(result.is_none());
744: }
745: #[test]
746: fn shift_take() {
747:     let mut set: OrderSet<i32> = OrderSet::new();
748:     set.insert(1);
749:     set.insert(2);
750:     set.insert(3);
751:     set.insert(4);
752:     set.insert(5);
753:     let result = set.take(&2);
754:     assert_eq!(result, Some(2));
755:     assert_eq!(set.len(), 4);
756:     assert_eq!(set.as_slice(), &[1, 3, 4, 5]);
757:     let result = set.take(&5);
758:     assert_eq!(result, Some(5));
759:     assert_eq!(set.len(), 3);
760:     assert_eq!(set.as_slice(), &[1, 3, 4]);
761:     let result = set.take(&5);
762:     assert_eq!(result, None);
763:     assert_eq!(set.len(), 3);
764:     assert_eq!(set.as_slice(), &[1, 3, 4]);
765: }
766: #[test]
767: fn test_binary_search_by() {
768:     // adapted from std's test for binary_search
769:     let b: OrderSet<i32> = [].into();
770:     assert_eq!(b.binary_search_by(|x| x.cmp(&5)), Err(0));
771:     let b: OrderSet<i32> = [4].into();
772:     assert_eq!(b.binary_search_by(|x| x.cmp(&3)), Err(0));
773:     assert_eq!(b.binary_search_by(|x| x.cmp(&4)), Ok(0));
774:     assert_eq!(b.binary_search_by(|x| x.cmp(&5)), Err(1));
775:     let b: OrderSet<i32> = [1, 2, 4, 6, 8, 9].into();
776:     assert_eq!(b.binary_search_by(|x| x.cmp(&5)), Err(3));
777:     assert_eq!(b.binary_search_by(|x| x.cmp(&6)), Ok(3));
778:     assert_eq!(b.binary_search_by(|x| x.cmp(&7)), Err(4));
779:     assert_eq!(b.binary_search_by(|x| x.cmp(&8)), Ok(4));
780:     let b: OrderSet<i32> = [1, 2, 4, 5, 6, 8].into();
781:     assert_eq!(b.binary_search_by(|x| x.cmp(&9)), Err(6));
782:     let b: OrderSet<i32> = [1, 2, 4, 6, 7, 8, 9].into();
783:     assert_eq!(b.binary_search_by(|x| x.cmp(&6)), Ok(3));
784:     assert_eq!(b.binary_search_by(|x| x.cmp(&5)), Err(3));
785:     assert_eq!(b.binary_search_by(|x| x.cmp(&8)), Ok(5));
786:     let b: OrderSet<i32> = [1, 2, 4, 5, 6, 8, 9].into();
787:     assert_eq!(b.binary_search_by(|x| x.cmp(&7)), Err(5));
788:     assert_eq!(b.binary_search_by(|x| x.cmp(&0)), Err(0));
789:     let b: OrderSet<i32> = [1, 3, 3, 3, 7].into();
790:     assert_eq!(b.binary_search_by(|x| x.cmp(&0)), Err(0));
791:     assert_eq!(b.binary_search_by(|x| x.cmp(&1)), Ok(0));
792:     assert_eq!(b.binary_search_by(|x| x.cmp(&2)), Err(1));
793:     // diff from std as set merges the duplicate keys
794:     assert!(match b.binary_search_by(|x| x.cmp(&3)) {
795:         Ok(1..=2) => true,
796:         _ => false,
797:     });
798:     assert!(match b.binary_search_by(|x| x.cmp(&3)) {
799:         Ok(1..=2) => true,
800:         _ => false,
801:     });
802:     assert_eq!(b.binary_search_by(|x| x.cmp(&4)), Err(2));
803:     assert_eq!(b.binary_search_by(|x| x.cmp(&5)), Err(2));
804:     assert_eq!(b.binary_search_by(|x| x.cmp(&6)), Err(2));
805:     assert_eq!(b.binary_search_by(|x| x.cmp(&7)), Ok(2));
806:     assert_eq!(b.binary_search_by(|x| x.cmp(&8)), Err(3));
807: }
808: #[test]
809: fn test_binary_search_by_key() {
810:     // adapted from std's test for binary_search
811:     let b: OrderSet<i32> = [].into();
812:     assert_eq!(b.binary_search_by_key(&5, |&x| x), Err(0));
813:     let b: OrderSet<i32> = [4].into();
814:     assert_eq!(b.binary_search_by_key(&3, |&x| x), Err(0));
815:     assert_eq!(b.binary_search_by_key(&4, |&x| x), Ok(0));
816:     assert_eq!(b.binary_search_by_key(&5, |&x| x), Err(1));
817:     let b: OrderSet<i32> = [1, 2, 4, 6, 8, 9].into();
818:     assert_eq!(b.binary_search_by_key(&5, |&x| x), Err(3));
819:     assert_eq!(b.binary_search_by_key(&6, |&x| x), Ok(3));
820:     assert_eq!(b.binary_search_by_key(&7, |&x| x), Err(4));
821:     assert_eq!(b.binary_search_by_key(&8, |&x| x), Ok(4));
822:     let b: OrderSet<i32> = [1, 2, 4, 5, 6, 8].into();
823:     assert_eq!(b.binary_search_by_key(&9, |&x| x), Err(6));
824:     let b: OrderSet<i32> = [1, 2, 4, 6, 7, 8, 9].into();
825:     assert_eq!(b.binary_search_by_key(&6, |&x| x), Ok(3));
826:     assert_eq!(b.binary_search_by_key(&5, |&x| x), Err(3));
827:     assert_eq!(b.binary_search_by_key(&8, |&x| x), Ok(5));
828:     let b: OrderSet<i32> = [1, 2, 4, 5, 6, 8, 9].into();
829:     assert_eq!(b.binary_search_by_key(&7, |&x| x), Err(5));
830:     assert_eq!(b.binary_search_by_key(&0, |&x| x), Err(0));
831:     let b: OrderSet<i32> = [1, 3, 3, 3, 7].into();
832:     assert_eq!(b.binary_search_by_key(&0, |&x| x), Err(0));
833:     assert_eq!(b.binary_search_by_key(&1, |&x| x), Ok(0));
834:     assert_eq!(b.binary_search_by_key(&2, |&x| x), Err(1));
835:     // diff from std as set merges the duplicate keys
836:     assert!(match b.binary_search_by_key(&3, |&x| x) {
837:         Ok(1..=2) => true,
838:         _ => false,
839:     });
840:     assert!(match b.binary_search_by_key(&3, |&x| x) {
841:         Ok(1..=2) => true,
842:         _ => false,
843:     });
844:     assert_eq!(b.binary_search_by_key(&4, |&x| x), Err(2));
845:     assert_eq!(b.binary_search_by_key(&5, |&x| x), Err(2));
846:     assert_eq!(b.binary_search_by_key(&6, |&x| x), Err(2));
847:     assert_eq!(b.binary_search_by_key(&7, |&x| x), Ok(2));
848:     assert_eq!(b.binary_search_by_key(&8, |&x| x), Err(3));
849: }
850: #[test]
851: fn test_partition_point() {
852:     // adapted from std's test for partition_point
853:     let b: OrderSet<i32> = [].into();
854:     assert_eq!(b.partition_point(|&x| x < 5), 0);
855:     let b: OrderSet<_> = [4].into();
856:     assert_eq!(b.partition_point(|&x| x < 3), 0);
857:     assert_eq!(b.partition_point(|&x| x < 4), 0);
858:     assert_eq!(b.partition_point(|&x| x < 5), 1);
859:     let b: OrderSet<_> = [1, 2, 4, 6, 8, 9].into();
860:     assert_eq!(b.partition_point(|&x| x < 5), 3);
861:     assert_eq!(b.partition_point(|&x| x < 6), 3);
862:     assert_eq!(b.partition_point(|&x| x < 7), 4);
863:     assert_eq!(b.partition_point(|&x| x < 8), 4);
864:     let b: OrderSet<_> = [1, 2, 4, 5, 6, 8].into();
865:     assert_eq!(b.partition_point(|&x| x < 9), 6);
866:     let b: OrderSet<_> = [1, 2, 4, 6, 7, 8, 9].into();
867:     assert_eq!(b.partition_point(|&x| x < 6), 3);
868:     assert_eq!(b.partition_point(|&x| x < 5), 3);
869:     assert_eq!(b.partition_point(|&x| x < 8), 5);
870:     let b: OrderSet<_> = [1, 2, 4, 5, 6, 8, 9].into();
871:     assert_eq!(b.partition_point(|&x| x < 7), 5);
872:     assert_eq!(b.partition_point(|&x| x < 0), 0);
873:     let b: OrderSet<_> = [1, 3, 3, 3, 7].into();
874:     assert_eq!(b.partition_point(|&x| x < 0), 0);
875:     assert_eq!(b.partition_point(|&x| x < 1), 0);
876:     assert_eq!(b.partition_point(|&x| x < 2), 1);
877:     assert_eq!(b.partition_point(|&x| x < 3), 1);
878:     assert_eq!(b.partition_point(|&x| x < 4), 2); // diff from std as set merges the duplicate keys
879:     assert_eq!(b.partition_point(|&x| x < 5), 2);
880:     assert_eq!(b.partition_point(|&x| x < 6), 2);
881:     assert_eq!(b.partition_point(|&x| x < 7), 2);
882:     assert_eq!(b.partition_point(|&x| x < 8), 3);
883: }
884: #[test]
885: fn is_sorted() {
886:     fn expect(set: &OrderSet<i32>, e: [bool; 4]) {
887:         assert_eq!(e[0], set.is_sorted());
888:         assert_eq!(e[1], set.is_sorted_by(|v1, v2| v1 < v2));
889:         assert_eq!(e[2], set.is_sorted_by(|v1, v2| v1 > v2));
890:         assert_eq!(e[3], set.is_sorted_by_key(|v| v));
891:     }
892:     let mut set = OrderSet::<i32>::from_iter(0..10);
893:     expect(&set, [true, true, false, true]);
894:     set.replace_index(5, -1).unwrap();
895:     expect(&set, [false, false, false, false]);
896: }
897: #[test]
898: fn is_sorted_trivial() {
899:     fn expect(set: &OrderSet<i32>, e: [bool; 5]) {
900:         assert_eq!(e[0], set.is_sorted());
901:         assert_eq!(e[1], set.is_sorted_by(|_, _| true));
902:         assert_eq!(e[2], set.is_sorted_by(|_, _| false));
903:         assert_eq!(e[3], set.is_sorted_by_key(|_| 0f64));
904:         assert_eq!(e[4], set.is_sorted_by_key(|_| f64::NAN));
905:     }
906:     let mut set = OrderSet::<i32>::default();
907:     expect(&set, [true, true, true, true, true]);
908:     set.insert(0);
909:     expect(&set, [true, true, true, true, true]);
910:     set.insert(1);
911:     expect(&set, [true, true, false, true, false]);
912:     set.reverse();
913:     expect(&set, [false, true, false, true, false]);
914: }
````

## File: src/arbitrary.rs
````rust
 1: #[cfg(feature = "arbitrary")]
 2: #[cfg_attr(docsrs, doc(cfg(feature = "arbitrary")))]
 3: mod impl_arbitrary {
 4:     use crate::{OrderMap, OrderSet};
 5:     use arbitrary::{Arbitrary, Result, Unstructured};
 6:     use core::hash::{BuildHasher, Hash};
 7:     use indexmap::{IndexMap, IndexSet};
 8:     impl<'a, K, V, S> Arbitrary<'a> for OrderMap<K, V, S>
 9:     where
10:         K: Arbitrary<'a> + Hash + Eq,
11:         V: Arbitrary<'a>,
12:         S: BuildHasher + Default,
13:     {
14:         fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
15:             IndexMap::arbitrary(u).map(|inner| Self { inner })
16:         }
17:         fn arbitrary_take_rest(u: Unstructured<'a>) -> Result<Self> {
18:             IndexMap::arbitrary_take_rest(u).map(|inner| Self { inner })
19:         }
20:     }
21:     impl<'a, T, S> Arbitrary<'a> for OrderSet<T, S>
22:     where
23:         T: Arbitrary<'a> + Hash + Eq,
24:         S: BuildHasher + Default,
25:     {
26:         fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
27:             IndexSet::arbitrary(u).map(|inner| Self { inner })
28:         }
29:         fn arbitrary_take_rest(u: Unstructured<'a>) -> Result<Self> {
30:             IndexSet::arbitrary_take_rest(u).map(|inner| Self { inner })
31:         }
32:     }
33: }
34: #[cfg(feature = "quickcheck")]
35: #[cfg_attr(docsrs, doc(cfg(feature = "quickcheck")))]
36: mod impl_quickcheck {
37:     use crate::{OrderMap, OrderSet};
38:     use alloc::boxed::Box;
39:     use core::hash::{BuildHasher, Hash};
40:     use indexmap::{IndexMap, IndexSet};
41:     use quickcheck::{Arbitrary, Gen};
42:     impl<K, V, S> Arbitrary for OrderMap<K, V, S>
43:     where
44:         K: Arbitrary + Hash + Eq,
45:         V: Arbitrary,
46:         S: BuildHasher + Default + Clone + 'static,
47:     {
48:         fn arbitrary(g: &mut Gen) -> Self {
49:             Self {
50:                 inner: IndexMap::arbitrary(g),
51:             }
52:         }
53:         fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
54:             Box::new(self.inner.shrink().map(|inner| Self { inner }))
55:         }
56:     }
57:     impl<T, S> Arbitrary for OrderSet<T, S>
58:     where
59:         T: Arbitrary + Hash + Eq,
60:         S: BuildHasher + Default + Clone + 'static,
61:     {
62:         fn arbitrary(g: &mut Gen) -> Self {
63:             Self {
64:                 inner: IndexSet::arbitrary(g),
65:             }
66:         }
67:         fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
68:             Box::new(self.inner.shrink().map(|inner| Self { inner }))
69:         }
70:     }
71: }
````

## File: src/borsh.rs
````rust
 1: #![cfg_attr(docsrs, doc(cfg(feature = "borsh")))]
 2: use crate::{OrderMap, OrderSet};
 3: use borsh::io::{Read, Result, Write};
 4: use borsh::{BorshDeserialize, BorshSerialize};
 5: use core::hash::BuildHasher;
 6: use core::hash::Hash;
 7: impl<K, V, S> BorshSerialize for OrderMap<K, V, S>
 8: where
 9:     K: BorshSerialize,
10:     V: BorshSerialize,
11: {
12:     #[inline]
13:     fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
14:         self.inner.serialize(writer)
15:     }
16: }
17: impl<K, V, S> BorshDeserialize for OrderMap<K, V, S>
18: where
19:     K: BorshDeserialize + Eq + Hash,
20:     V: BorshDeserialize,
21:     S: BuildHasher + Default,
22: {
23:     #[inline]
24:     fn deserialize_reader<R: Read>(reader: &mut R) -> Result<Self> {
25:         Ok(Self {
26:             inner: <_>::deserialize_reader(reader)?,
27:         })
28:     }
29: }
30: impl<T, S> BorshSerialize for OrderSet<T, S>
31: where
32:     T: BorshSerialize,
33: {
34:     #[inline]
35:     fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
36:         self.inner.serialize(writer)
37:     }
38: }
39: impl<T, S> BorshDeserialize for OrderSet<T, S>
40: where
41:     T: BorshDeserialize + Eq + Hash,
42:     S: BuildHasher + Default,
43: {
44:     #[inline]
45:     fn deserialize_reader<R: Read>(reader: &mut R) -> Result<Self> {
46:         Ok(Self {
47:             inner: <_>::deserialize_reader(reader)?,
48:         })
49:     }
50: }
51: #[cfg(test)]
52: mod borsh_tests {
53:     use super::*;
54:     #[test]
55:     fn map_borsh_roundtrip() {
56:         let original_map: OrderMap<i32, i32> = {
57:             let mut map = OrderMap::new();
58:             map.insert(1, 2);
59:             map.insert(3, 4);
60:             map.insert(5, 6);
61:             map
62:         };
63:         let serialized_map = borsh::to_vec(&original_map).unwrap();
64:         let deserialized_map: OrderMap<i32, i32> =
65:             BorshDeserialize::try_from_slice(&serialized_map).unwrap();
66:         assert_eq!(original_map, deserialized_map);
67:     }
68:     #[test]
69:     fn set_borsh_roundtrip() {
70:         let original_map: OrderSet<i32> = [1, 2, 3, 4, 5, 6].into_iter().collect();
71:         let serialized_map = borsh::to_vec(&original_map).unwrap();
72:         let deserialized_map: OrderSet<i32> =
73:             BorshDeserialize::try_from_slice(&serialized_map).unwrap();
74:         assert_eq!(original_map, deserialized_map);
75:     }
76: }
````

## File: src/lib.rs
````rust
  1: #![no_std]
  2: //! [`OrderMap`] is a hash table where the iteration order of the key-value
  3: //! pairs is independent of the hash values of the keys.
  4: //!
  5: //! [`OrderSet`] is a corresponding hash set using the same implementation and
  6: //! with similar properties.
  7: //!
  8: //! ### Highlights
  9: //!
 10: //! [`OrderMap`] and [`OrderSet`] are drop-in compatible with the std `HashMap`
 11: //! and `HashSet`, but they also have some features of note:
 12: //!
 13: //! - The ordering semantics (see their documentation for details)
 14: //! - Sorting methods and the [`.pop()`][OrderMap::pop] methods.
 15: //! - The [`Equivalent`] trait, which offers more flexible equality definitions
 16: //!   between borrowed and owned versions of keys.
 17: //! - The [`MutableKeys`][map::MutableKeys] trait, which gives opt-in mutable
 18: //!   access to map keys, and [`MutableValues`][set::MutableValues] for sets.
 19: //!
 20: //! ### Feature Flags
 21: //!
 22: //! To reduce the amount of compiled code in the crate by default, certain
 23: //! features are gated behind [feature flags]. These allow you to opt in to (or
 24: //! out of) functionality. Below is a list of the features available in this
 25: //! crate.
 26: //!
 27: //! * `std`: Enables features which require the Rust standard library. For more
 28: //!   information see the section on [`no_std`].
 29: //! * `rayon`: Enables parallel iteration and other parallel methods.
 30: //! * `serde`: Adds implementations for [`Serialize`] and [`Deserialize`]
 31: //!   to [`OrderMap`] and [`OrderSet`]. Alternative implementations for
 32: //!   (de)serializing [`OrderMap`] as an ordered sequence are available in the
 33: //!   [`map::serde_seq`] module.
 34: //! * `borsh`: Adds implementations for [`BorshSerialize`] and [`BorshDeserialize`]
 35: //!   to [`OrderMap`] and [`OrderSet`].
 36: //! * `arbitrary`: Adds implementations for the [`arbitrary::Arbitrary`] trait
 37: //!   to [`OrderMap`] and [`OrderSet`].
 38: //! * `quickcheck`: Adds implementations for the [`quickcheck::Arbitrary`] trait
 39: //!   to [`OrderMap`] and [`OrderSet`].
 40: //!
 41: //! _Note: only the `std` feature is enabled by default._
 42: //!
 43: //! [feature flags]: https://doc.rust-lang.org/cargo/reference/manifest.html#the-features-section
 44: //! [`no_std`]: #no-standard-library-targets
 45: //! [`Serialize`]: `::serde_core::Serialize`
 46: //! [`Deserialize`]: `::serde_core::Deserialize`
 47: //! [`BorshSerialize`]: `::borsh::BorshSerialize`
 48: //! [`BorshDeserialize`]: `::borsh::BorshDeserialize`
 49: //! [`arbitrary::Arbitrary`]: `::arbitrary::Arbitrary`
 50: //! [`quickcheck::Arbitrary`]: `::quickcheck::Arbitrary`
 51: //!
 52: //! ### Alternate Hashers
 53: //!
 54: //! [`OrderMap`] and [`OrderSet`] have a default hasher type
 55: //! [`S = RandomState`][std::hash::RandomState],
 56: //! just like the standard `HashMap` and `HashSet`, which is resistant to
 57: //! HashDoS attacks but not the most performant. Type aliases can make it easier
 58: //! to use alternate hashers:
 59: //!
 60: //! ```
 61: //! use fnv::FnvBuildHasher;
 62: //! use ordermap::{OrderMap, OrderSet};
 63: //!
 64: //! type FnvOrderMap<K, V> = OrderMap<K, V, FnvBuildHasher>;
 65: //! type FnvOrderSet<T> = OrderSet<T, FnvBuildHasher>;
 66: //!
 67: //! let std: OrderSet<i32> = (0..100).collect();
 68: //! let fnv: FnvOrderSet<i32> = (0..100).collect();
 69: //! assert_eq!(std, fnv);
 70: //! ```
 71: //!
 72: //! ### Rust Version
 73: //!
 74: //! This version of ordermap requires Rust 1.85 or later.
 75: //!
 76: //! The ordermap 1.x release series will use a carefully considered version
 77: //! upgrade policy, where in a later 1.x version, we will raise the minimum
 78: //! required Rust version.
 79: //!
 80: //! ## No Standard Library Targets
 81: //!
 82: //! This crate supports being built without `std`, requiring `alloc` instead.
 83: //! This is chosen by disabling the default "std" cargo feature, by adding
 84: //! `default-features = false` to your dependency specification.
 85: //!
 86: //! - Creating maps and sets using [`new`][OrderMap::new] and
 87: //!   [`with_capacity`][OrderMap::with_capacity] is unavailable without `std`.
 88: //!   Use methods [`OrderMap::default`], [`with_hasher`][OrderMap::with_hasher],
 89: //!   [`with_capacity_and_hasher`][OrderMap::with_capacity_and_hasher] instead.
 90: //!   A no-std compatible hasher will be needed as well, for example
 91: //!   from the crate `twox-hash`.
 92: //! - Macros [`ordermap!`] and [`orderset!`] are unavailable without `std`. Use
 93: //!   the macros [`ordermap_with_default!`] and [`orderset_with_default!`] instead.
 94: #![cfg_attr(docsrs, feature(doc_cfg))]
 95: extern crate alloc;
 96: #[cfg(feature = "std")]
 97: #[cfg_attr(test, macro_use)]
 98: extern crate std;
 99: mod arbitrary;
100: #[macro_use]
101: mod macros;
102: #[cfg(feature = "borsh")]
103: mod borsh;
104: #[cfg(feature = "serde")]
105: mod serde;
106: #[cfg(feature = "sval")]
107: mod sval;
108: pub mod map;
109: pub mod set;
110: pub use crate::map::OrderMap;
111: pub use crate::set::OrderSet;
112: pub use indexmap::{Equivalent, GetDisjointMutError, TryReserveError};
````

## File: src/macros.rs
````rust
  1: /// Create an [`OrderMap`][crate::OrderMap] from a list of key-value pairs
  2: /// and a [`BuildHasherDefault`][core::hash::BuildHasherDefault]-wrapped custom hasher.
  3: ///
  4: /// ## Example
  5: ///
  6: /// ```
  7: /// use ordermap::ordermap_with_default;
  8: /// use fnv::FnvHasher;
  9: ///
 10: /// let map = ordermap_with_default!{
 11: ///     FnvHasher;
 12: ///     "a" => 1,
 13: ///     "b" => 2,
 14: /// };
 15: /// assert_eq!(map["a"], 1);
 16: /// assert_eq!(map["b"], 2);
 17: /// assert_eq!(map.get("c"), None);
 18: ///
 19: /// // "a" is the first key
 20: /// assert_eq!(map.keys().next(), Some(&"a"));
 21: /// ```
 22: ///
 23: /// This can also be initialized in `const` contexts:
 24: ///
 25: /// ```
 26: /// use ordermap::{OrderMap, ordermap_with_default};
 27: /// use fnv::FnvBuildHasher; // = BuildHasherDefault<FnvHasher>
 28: /// use std::sync::Mutex;
 29: ///
 30: /// static GLOBAL: Mutex<OrderMap<String, i32, FnvBuildHasher>> =
 31: ///     Mutex::new(ordermap_with_default!());
 32: ///
 33: /// if let Ok(mut map) = GLOBAL.lock() {
 34: ///     map.insert("a".into(), 1);
 35: ///     map.insert("b".into(), 2);
 36: /// }
 37: ///
 38: /// assert_eq!(GLOBAL.lock().unwrap()["a"], 1);
 39: /// assert_eq!(GLOBAL.lock().unwrap()["b"], 2);
 40: /// ```
 41: #[macro_export]
 42: macro_rules! ordermap_with_default {
 43:     () => { const {
 44:         $crate::OrderMap::with_hasher(
 45:             // Let type inference figure out the hasher:
 46:             ::core::hash::BuildHasherDefault::new(),
 47:         )
 48:     }};
 49:     ($H:ty $(;)?) => { const {
 50:         $crate::OrderMap::with_hasher(
 51:             // Specify your custom `H` (must implement Default + Hasher) as the hasher:
 52:             ::core::hash::BuildHasherDefault::<$H>::new(),
 53:         )
 54:     }};
 55:     ($H:ty; $($key:expr => $value:expr),+ $(,)?) => {{
 56:         let mut map = $crate::OrderMap::with_capacity_and_hasher(
 57:             // Note: `stringify!($key)` is just here to consume the repetition,
 58:             // but we throw away that string literal during constant evaluation.
 59:             const { <[()]>::len(&[$({ stringify!($key); }),*]) },
 60:             // Specify your custom `H` (must implement Default + Hasher) as the hasher:
 61:             ::core::hash::BuildHasherDefault::<$H>::new(),
 62:         );
 63:         $(
 64:             map.insert($key, $value);
 65:         )+
 66:         map
 67:     }};
 68: }
 69: #[cfg(feature = "std")]
 70: #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
 71: #[macro_export]
 72: /// Create an [`OrderMap`][crate::OrderMap] from a list of key-value pairs
 73: ///
 74: /// ## Example
 75: ///
 76: /// ```
 77: /// use ordermap::ordermap;
 78: ///
 79: /// let map = ordermap!{
 80: ///     "a" => 1,
 81: ///     "b" => 2,
 82: /// };
 83: /// assert_eq!(map["a"], 1);
 84: /// assert_eq!(map["b"], 2);
 85: /// assert_eq!(map.get("c"), None);
 86: ///
 87: /// // "a" is the first key
 88: /// assert_eq!(map.keys().next(), Some(&"a"));
 89: /// ```
 90: macro_rules! ordermap {
 91:     () => { $crate::OrderMap::new() };
 92:     ($($key:expr => $value:expr),+ $(,)?) => {{
 93:         let mut map = $crate::OrderMap::with_capacity(
 94:             // Note: `stringify!($key)` is just here to consume the repetition,
 95:             // but we throw away that string literal during constant evaluation.
 96:             const { <[()]>::len(&[$({ stringify!($key); }),*]) },
 97:         );
 98:         $(
 99:             map.insert($key, $value);
100:         )+
101:         map
102:     }};
103: }
104: /// Create an [`OrderSet`][crate::OrderSet] from a list of values
105: /// and a [`BuildHasherDefault`][core::hash::BuildHasherDefault]-wrapped custom hasher.
106: ///
107: /// ## Example
108: ///
109: /// ```
110: /// use ordermap::orderset_with_default;
111: /// use fnv::FnvHasher;
112: ///
113: /// let set = orderset_with_default!{
114: ///     FnvHasher;
115: ///     "a",
116: ///     "b",
117: /// };
118: /// assert!(set.contains("a"));
119: /// assert!(set.contains("b"));
120: /// assert!(!set.contains("c"));
121: ///
122: /// // "a" is the first value
123: /// assert_eq!(set.iter().next(), Some(&"a"));
124: /// ```
125: ///
126: /// This can also be initialized in `const` contexts:
127: ///
128: /// ```
129: /// use ordermap::{OrderSet, orderset_with_default};
130: /// use fnv::FnvBuildHasher; // = BuildHasherDefault<FnvHasher>
131: /// use std::sync::Mutex;
132: ///
133: /// static INTERN: Mutex<OrderSet<String, FnvBuildHasher>> =
134: ///     Mutex::new(orderset_with_default!());
135: ///
136: /// if let Ok(mut set) = INTERN.lock() {
137: ///     set.insert("a".into());
138: ///     set.insert("b".into());
139: ///     set.insert("c".into());
140: /// }
141: ///
142: /// assert!(INTERN.lock().unwrap().contains("a"));
143: /// assert!(INTERN.lock().unwrap().contains("b"));
144: /// assert!(INTERN.lock().unwrap().contains("c"));
145: /// ```
146: #[macro_export]
147: macro_rules! orderset_with_default {
148:     () => { const {
149:         $crate::OrderSet::with_hasher(
150:             // Let type inference figure out the hasher:
151:             ::core::hash::BuildHasherDefault::new(),
152:         )
153:     }};
154:     ($H:ty $(;)?) => { const {
155:         $crate::OrderSet::with_hasher(
156:             // Specify your custom `H` (must implement Default + Hasher) as the hasher:
157:             ::core::hash::BuildHasherDefault::<$H>::new(),
158:         )
159:     }};
160:     ($H:ty; $($value:expr),+ $(,)?) => {{
161:         let mut set = $crate::OrderSet::with_capacity_and_hasher(
162:             // Note: `stringify!($value)` is just here to consume the repetition,
163:             // but we throw away that string literal during constant evaluation.
164:             const { <[()]>::len(&[$({ stringify!($value); }),*]) },
165:             // Specify your custom `H` (must implement Default + Hasher) as the hasher:
166:             ::core::hash::BuildHasherDefault::<$H>::new(),
167:         );
168:         $(
169:             set.insert($value);
170:         )+
171:         set
172:     }};
173: }
174: #[cfg(feature = "std")]
175: #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
176: #[macro_export]
177: /// Create an [`OrderSet`][crate::OrderSet] from a list of values
178: ///
179: /// ## Example
180: ///
181: /// ```
182: /// use ordermap::orderset;
183: ///
184: /// let set = orderset!{
185: ///     "a",
186: ///     "b",
187: /// };
188: /// assert!(set.contains("a"));
189: /// assert!(set.contains("b"));
190: /// assert!(!set.contains("c"));
191: ///
192: /// // "a" is the first value
193: /// assert_eq!(set.iter().next(), Some(&"a"));
194: /// ```
195: macro_rules! orderset {
196:     () => { $crate::OrderSet::new() };
197:     ($($value:expr),+ $(,)?) => {{
198:         let mut set = $crate::OrderSet::with_capacity(
199:             // Note: `stringify!($value)` is just here to consume the repetition,
200:             // but we throw away that string literal during constant evaluation.
201:             const { <[()]>::len(&[$({ stringify!($value); }),*]) },
202:         );
203:         $(
204:             set.insert($value);
205:         )+
206:         set
207:     }};
208: }
````

## File: src/map.rs
````rust
   1: //! [`OrderMap`] is a hash table where the iteration order of the key-value
   2: //! pairs is independent of the hash values of the keys.
   3: //!
   4: //! It is based on [`IndexMap`], and even shares many of the auxiliary types
   5: //! like [`Slice`] and all of the iterators.
   6: //!
   7: //! **Unlike** `IndexMap`, `OrderMap` does consider the order for [`PartialEq`]
   8: //! and [`Eq`], and it also implements [`PartialOrd`], [`Ord`], and [`Hash`].
   9: //! Methods like [`OrderMap::remove`] use `IndexMap`'s "shift" semantics, so
  10: //! they preserve the relative order of remaining entries.
  11: mod entry;
  12: mod iter;
  13: mod mutable;
  14: mod slice;
  15: pub mod raw_entry_v1;
  16: #[cfg(feature = "serde")]
  17: #[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
  18: pub mod serde_seq;
  19: #[cfg(test)]
  20: mod tests;
  21: pub use self::entry::{Entry, IndexedEntry, OccupiedEntry, VacantEntry};
  22: pub use self::mutable::MutableEntryKey;
  23: pub use self::mutable::MutableKeys;
  24: pub use self::raw_entry_v1::RawEntryApiV1;
  25: pub use indexmap::map::{
  26:     Drain, ExtractIf, IntoIter, IntoKeys, IntoValues, Iter, IterMut, IterMut2, Keys, Slice, Splice,
  27:     Values, ValuesMut,
  28: };
  29: #[cfg(feature = "rayon")]
  30: #[cfg_attr(docsrs, doc(cfg(feature = "rayon")))]
  31: pub mod rayon;
  32: use alloc::boxed::Box;
  33: use core::cmp::Ordering;
  34: use core::fmt;
  35: use core::hash::{BuildHasher, Hash, Hasher};
  36: use core::ops::{Index, IndexMut, RangeBounds};
  37: use indexmap::IndexMap;
  38: #[cfg(doc)]
  39: use alloc::vec::Vec;
  40: #[cfg(feature = "std")]
  41: use std::hash::RandomState;
  42: use crate::{Equivalent, GetDisjointMutError, TryReserveError};
  43: /// A hash table where the iteration order of the key-value pairs is independent
  44: /// of the hash values of the keys.
  45: ///
  46: /// The interface is closely compatible with the standard
  47: /// [`HashMap`][std::collections::HashMap],
  48: /// but also has additional features.
  49: ///
  50: /// # Order
  51: ///
  52: /// The key-value pairs have a consistent order that is determined by
  53: /// the sequence of insertion and removal calls on the map. The order does
  54: /// not depend on the keys or the hash function at all.
  55: ///
  56: /// All iterators traverse the map in *the order*.
  57: ///
  58: /// The insertion order is preserved, with **notable exceptions** like the
  59: /// [`.swap_remove()`][Self::swap_remove] method.
  60: /// Methods such as [`.sort_by()`][Self::sort_by] of
  61: /// course result in a new order, depending on the sorting order.
  62: ///
  63: /// # Indices
  64: ///
  65: /// The key-value pairs are indexed in a compact range without holes in the
  66: /// range `0..self.len()`. For example, the method `.get_full` looks up the
  67: /// index for a key, and the method `.get_index` looks up the key-value pair by
  68: /// index.
  69: ///
  70: /// # Complexity
  71: ///
  72: /// Internally, `OrderMap<K, V, S>` just holds an [`IndexMap<K, V, S>`](IndexMap).
  73: /// Thus the complexity of the two are the same for most methods.
  74: ///
  75: /// # Examples
  76: ///
  77: /// ```
  78: /// use ordermap::OrderMap;
  79: ///
  80: /// // count the frequency of each letter in a sentence.
  81: /// let mut letters = OrderMap::new();
  82: /// for ch in "a short treatise on fungi".chars() {
  83: ///     *letters.entry(ch).or_insert(0) += 1;
  84: /// }
  85: ///
  86: /// assert_eq!(letters[&'s'], 2);
  87: /// assert_eq!(letters[&'t'], 3);
  88: /// assert_eq!(letters[&'u'], 1);
  89: /// assert_eq!(letters.get(&'y'), None);
  90: /// ```
  91: #[cfg(feature = "std")]
  92: pub struct OrderMap<K, V, S = RandomState> {
  93:     pub(crate) inner: IndexMap<K, V, S>,
  94: }
  95: #[cfg(not(feature = "std"))]
  96: pub struct OrderMap<K, V, S> {
  97:     pub(crate) inner: IndexMap<K, V, S>,
  98: }
  99: impl<K, V, S> Clone for OrderMap<K, V, S>
 100: where
 101:     K: Clone,
 102:     V: Clone,
 103:     S: Clone,
 104: {
 105:     fn clone(&self) -> Self {
 106:         Self {
 107:             inner: self.inner.clone(),
 108:         }
 109:     }
 110:     fn clone_from(&mut self, other: &Self) {
 111:         self.inner.clone_from(&other.inner);
 112:     }
 113: }
 114: impl<K, V, S> fmt::Debug for OrderMap<K, V, S>
 115: where
 116:     K: fmt::Debug,
 117:     V: fmt::Debug,
 118: {
 119:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
 120:         f.debug_map().entries(self.iter()).finish()
 121:     }
 122: }
 123: #[cfg(feature = "std")]
 124: #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
 125: impl<K, V> OrderMap<K, V> {
 126:     /// Create a new map. (Does not allocate.)
 127:     #[inline]
 128:     pub fn new() -> Self {
 129:         Self {
 130:             inner: IndexMap::new(),
 131:         }
 132:     }
 133:     /// Create a new map with capacity for `n` key-value pairs. (Does not
 134:     /// allocate if `n` is zero.)
 135:     ///
 136:     /// Computes in **O(n)** time.
 137:     #[inline]
 138:     pub fn with_capacity(n: usize) -> Self {
 139:         Self {
 140:             inner: IndexMap::with_capacity(n),
 141:         }
 142:     }
 143: }
 144: impl<K, V, S> OrderMap<K, V, S> {
 145:     /// Create a new map with capacity for `n` key-value pairs. (Does not
 146:     /// allocate if `n` is zero.)
 147:     ///
 148:     /// Computes in **O(n)** time.
 149:     #[inline]
 150:     pub fn with_capacity_and_hasher(n: usize, hash_builder: S) -> Self {
 151:         Self {
 152:             inner: IndexMap::with_capacity_and_hasher(n, hash_builder),
 153:         }
 154:     }
 155:     /// Create a new map with `hash_builder`.
 156:     ///
 157:     /// This function is `const`, so it
 158:     /// can be called in `static` contexts.
 159:     pub const fn with_hasher(hash_builder: S) -> Self {
 160:         Self {
 161:             inner: IndexMap::with_hasher(hash_builder),
 162:         }
 163:     }
 164:     /// Return the number of elements the map can hold without reallocating.
 165:     ///
 166:     /// This number is a lower bound; the map might be able to hold more,
 167:     /// but is guaranteed to be able to hold at least this many.
 168:     ///
 169:     /// Computes in **O(1)** time.
 170:     pub fn capacity(&self) -> usize {
 171:         self.inner.capacity()
 172:     }
 173:     /// Return a reference to the map's `BuildHasher`.
 174:     pub fn hasher(&self) -> &S {
 175:         self.inner.hasher()
 176:     }
 177:     /// Return the number of key-value pairs in the map.
 178:     ///
 179:     /// Computes in **O(1)** time.
 180:     #[inline]
 181:     pub fn len(&self) -> usize {
 182:         self.inner.len()
 183:     }
 184:     /// Returns true if the map contains no elements.
 185:     ///
 186:     /// Computes in **O(1)** time.
 187:     #[inline]
 188:     pub fn is_empty(&self) -> bool {
 189:         self.inner.is_empty()
 190:     }
 191:     /// Return an iterator over the key-value pairs of the map, in their order
 192:     pub fn iter(&self) -> Iter<'_, K, V> {
 193:         self.inner.iter()
 194:     }
 195:     /// Return an iterator over the key-value pairs of the map, in their order
 196:     pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
 197:         self.inner.iter_mut()
 198:     }
 199:     /// Return an iterator over the keys of the map, in their order
 200:     pub fn keys(&self) -> Keys<'_, K, V> {
 201:         self.inner.keys()
 202:     }
 203:     /// Return an owning iterator over the keys of the map, in their order
 204:     pub fn into_keys(self) -> IntoKeys<K, V> {
 205:         self.inner.into_keys()
 206:     }
 207:     /// Return an iterator over the values of the map, in their order
 208:     pub fn values(&self) -> Values<'_, K, V> {
 209:         self.inner.values()
 210:     }
 211:     /// Return an iterator over mutable references to the values of the map,
 212:     /// in their order
 213:     pub fn values_mut(&mut self) -> ValuesMut<'_, K, V> {
 214:         self.inner.values_mut()
 215:     }
 216:     /// Return an owning iterator over the values of the map, in their order
 217:     pub fn into_values(self) -> IntoValues<K, V> {
 218:         self.inner.into_values()
 219:     }
 220:     /// Remove all key-value pairs in the map, while preserving its capacity.
 221:     ///
 222:     /// Computes in **O(n)** time.
 223:     pub fn clear(&mut self) {
 224:         self.inner.clear();
 225:     }
 226:     /// Shortens the map, keeping the first `len` elements and dropping the rest.
 227:     ///
 228:     /// If `len` is greater than the map's current length, this has no effect.
 229:     pub fn truncate(&mut self, len: usize) {
 230:         self.inner.truncate(len);
 231:     }
 232:     /// Clears the `OrderMap` in the given index range, returning those
 233:     /// key-value pairs as a drain iterator.
 234:     ///
 235:     /// The range may be any type that implements [`RangeBounds<usize>`],
 236:     /// including all of the `std::ops::Range*` types, or even a tuple pair of
 237:     /// `Bound` start and end values. To drain the map entirely, use `RangeFull`
 238:     /// like `map.drain(..)`.
 239:     ///
 240:     /// This shifts down all entries following the drained range to fill the
 241:     /// gap, and keeps the allocated memory for reuse.
 242:     ///
 243:     /// ***Panics*** if the starting point is greater than the end point or if
 244:     /// the end point is greater than the length of the map.
 245:     #[track_caller]
 246:     pub fn drain<R>(&mut self, range: R) -> Drain<'_, K, V>
 247:     where
 248:         R: RangeBounds<usize>,
 249:     {
 250:         self.inner.drain(range)
 251:     }
 252:     /// Creates an iterator which uses a closure to determine if an element should be removed,
 253:     /// for all elements in the given range.
 254:     ///
 255:     /// If the closure returns true, the element is removed from the map and yielded.
 256:     /// If the closure returns false, or panics, the element remains in the map and will not be
 257:     /// yielded.
 258:     ///
 259:     /// Note that `extract_if` lets you mutate every value in the filter closure, regardless of
 260:     /// whether you choose to keep or remove it.
 261:     ///
 262:     /// The range may be any type that implements [`RangeBounds<usize>`],
 263:     /// including all of the `std::ops::Range*` types, or even a tuple pair of
 264:     /// `Bound` start and end values. To check the entire map, use `RangeFull`
 265:     /// like `map.extract_if(.., predicate)`.
 266:     ///
 267:     /// If the returned `ExtractIf` is not exhausted, e.g. because it is dropped without iterating
 268:     /// or the iteration short-circuits, then the remaining elements will be retained.
 269:     /// Use [`retain`] with a negated predicate if you do not need the returned iterator.
 270:     ///
 271:     /// [`retain`]: OrderMap::retain
 272:     ///
 273:     /// ***Panics*** if the starting point is greater than the end point or if
 274:     /// the end point is greater than the length of the map.
 275:     ///
 276:     /// # Examples
 277:     ///
 278:     /// Splitting a map into even and odd keys, reusing the original map:
 279:     ///
 280:     /// ```
 281:     /// use ordermap::OrderMap;
 282:     ///
 283:     /// let mut map: OrderMap<i32, i32> = (0..8).map(|x| (x, x)).collect();
 284:     /// let extracted: OrderMap<i32, i32> = map.extract_if(.., |k, _v| k % 2 == 0).collect();
 285:     ///
 286:     /// let evens = extracted.keys().copied().collect::<Vec<_>>();
 287:     /// let odds = map.keys().copied().collect::<Vec<_>>();
 288:     ///
 289:     /// assert_eq!(evens, vec![0, 2, 4, 6]);
 290:     /// assert_eq!(odds, vec![1, 3, 5, 7]);
 291:     /// ```
 292:     #[track_caller]
 293:     pub fn extract_if<F, R>(&mut self, range: R, pred: F) -> ExtractIf<'_, K, V, F>
 294:     where
 295:         F: FnMut(&K, &mut V) -> bool,
 296:         R: RangeBounds<usize>,
 297:     {
 298:         self.inner.extract_if(range, pred)
 299:     }
 300:     /// Splits the collection into two at the given index.
 301:     ///
 302:     /// Returns a newly allocated map containing the elements in the range
 303:     /// `[at, len)`. After the call, the original map will be left containing
 304:     /// the elements `[0, at)` with its previous capacity unchanged.
 305:     ///
 306:     /// ***Panics*** if `at > len`.
 307:     #[track_caller]
 308:     pub fn split_off(&mut self, at: usize) -> Self
 309:     where
 310:         S: Clone,
 311:     {
 312:         Self {
 313:             inner: self.inner.split_off(at),
 314:         }
 315:     }
 316:     /// Reserve capacity for `additional` more key-value pairs.
 317:     ///
 318:     /// Computes in **O(n)** time.
 319:     pub fn reserve(&mut self, additional: usize) {
 320:         self.inner.reserve(additional);
 321:     }
 322:     /// Reserve capacity for `additional` more key-value pairs, without over-allocating.
 323:     ///
 324:     /// Unlike `reserve`, this does not deliberately over-allocate the entry capacity to avoid
 325:     /// frequent re-allocations. However, the underlying data structures may still have internal
 326:     /// capacity requirements, and the allocator itself may give more space than requested, so this
 327:     /// cannot be relied upon to be precisely minimal.
 328:     ///
 329:     /// Computes in **O(n)** time.
 330:     pub fn reserve_exact(&mut self, additional: usize) {
 331:         self.inner.reserve_exact(additional);
 332:     }
 333:     /// Try to reserve capacity for `additional` more key-value pairs.
 334:     ///
 335:     /// Computes in **O(n)** time.
 336:     pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
 337:         self.inner.try_reserve(additional)
 338:     }
 339:     /// Try to reserve capacity for `additional` more key-value pairs, without over-allocating.
 340:     ///
 341:     /// Unlike `try_reserve`, this does not deliberately over-allocate the entry capacity to avoid
 342:     /// frequent re-allocations. However, the underlying data structures may still have internal
 343:     /// capacity requirements, and the allocator itself may give more space than requested, so this
 344:     /// cannot be relied upon to be precisely minimal.
 345:     ///
 346:     /// Computes in **O(n)** time.
 347:     pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
 348:         self.inner.try_reserve_exact(additional)
 349:     }
 350:     /// Shrink the capacity of the map as much as possible.
 351:     ///
 352:     /// Computes in **O(n)** time.
 353:     pub fn shrink_to_fit(&mut self) {
 354:         self.inner.shrink_to(0);
 355:     }
 356:     /// Shrink the capacity of the map with a lower limit.
 357:     ///
 358:     /// Computes in **O(n)** time.
 359:     pub fn shrink_to(&mut self, min_capacity: usize) {
 360:         self.inner.shrink_to(min_capacity);
 361:     }
 362: }
 363: impl<K, V, S> OrderMap<K, V, S>
 364: where
 365:     K: Hash + Eq,
 366:     S: BuildHasher,
 367: {
 368:     /// Insert a key-value pair in the map.
 369:     ///
 370:     /// If an equivalent key already exists in the map: the key remains and
 371:     /// retains in its place in the order, its corresponding value is updated
 372:     /// with `value`, and the older value is returned inside `Some(_)`.
 373:     ///
 374:     /// If no equivalent key existed in the map: the new key-value pair is
 375:     /// inserted, last in order, and `None` is returned.
 376:     ///
 377:     /// Computes in **O(1)** time (amortized average).
 378:     ///
 379:     /// See also [`entry`][Self::entry] if you want to insert *or* modify,
 380:     /// or [`insert_full`][Self::insert_full] if you need to get the index of
 381:     /// the corresponding key-value pair.
 382:     pub fn insert(&mut self, key: K, value: V) -> Option<V> {
 383:         self.inner.insert(key, value)
 384:     }
 385:     /// Insert a key-value pair in the map, and get their index.
 386:     ///
 387:     /// If an equivalent key already exists in the map: the key remains and
 388:     /// retains in its place in the order, its corresponding value is updated
 389:     /// with `value`, and the older value is returned inside `(index, Some(_))`.
 390:     ///
 391:     /// If no equivalent key existed in the map: the new key-value pair is
 392:     /// inserted, last in order, and `(index, None)` is returned.
 393:     ///
 394:     /// Computes in **O(1)** time (amortized average).
 395:     ///
 396:     /// See also [`entry`][Self::entry] if you want to insert *or* modify.
 397:     pub fn insert_full(&mut self, key: K, value: V) -> (usize, Option<V>) {
 398:         self.inner.insert_full(key, value)
 399:     }
 400:     /// Insert a key-value pair in the map at its ordered position among sorted keys.
 401:     ///
 402:     /// This is equivalent to finding the position with
 403:     /// [`binary_search_keys`][Self::binary_search_keys], then either updating
 404:     /// it or calling [`insert_before`][Self::insert_before] for a new key.
 405:     ///
 406:     /// If the sorted key is found in the map, its corresponding value is
 407:     /// updated with `value`, and the older value is returned inside
 408:     /// `(index, Some(_))`. Otherwise, the new key-value pair is inserted at
 409:     /// the sorted position, and `(index, None)` is returned.
 410:     ///
 411:     /// If the existing keys are **not** already sorted, then the insertion
 412:     /// index is unspecified (like [`slice::binary_search`]), but the key-value
 413:     /// pair is moved to or inserted at that position regardless.
 414:     ///
 415:     /// Computes in **O(n)** time (average). Instead of repeating calls to
 416:     /// `insert_sorted`, it may be faster to call batched [`insert`][Self::insert]
 417:     /// or [`extend`][Self::extend] and only call [`sort_keys`][Self::sort_keys]
 418:     /// or [`sort_unstable_keys`][Self::sort_unstable_keys] once.
 419:     pub fn insert_sorted(&mut self, key: K, value: V) -> (usize, Option<V>)
 420:     where
 421:         K: Ord,
 422:     {
 423:         self.inner.insert_sorted(key, value)
 424:     }
 425:     /// Insert a key-value pair in the map at its ordered position among keys
 426:     /// sorted by `cmp`.
 427:     ///
 428:     /// This is equivalent to finding the position with
 429:     /// [`binary_search_by`][Self::binary_search_by], then calling
 430:     /// [`insert_before`][Self::insert_before] with the given key and value.
 431:     ///
 432:     /// If the existing keys are **not** already sorted, then the insertion
 433:     /// index is unspecified (like [`slice::binary_search`]), but the key-value
 434:     /// pair is moved to or inserted at that position regardless.
 435:     ///
 436:     /// Computes in **O(n)** time (average).
 437:     pub fn insert_sorted_by<F>(&mut self, key: K, value: V, cmp: F) -> (usize, Option<V>)
 438:     where
 439:         F: FnMut(&K, &V, &K, &V) -> Ordering,
 440:     {
 441:         self.inner.insert_sorted_by(key, value, cmp)
 442:     }
 443:     /// Insert a key-value pair in the map at its ordered position
 444:     /// using a sort-key extraction function.
 445:     ///
 446:     /// This is equivalent to finding the position with
 447:     /// [`binary_search_by_key`][Self::binary_search_by_key] with `sort_key(key)`, then
 448:     /// calling [`insert_before`][Self::insert_before] with the given key and value.
 449:     ///
 450:     /// If the existing keys are **not** already sorted, then the insertion
 451:     /// index is unspecified (like [`slice::binary_search`]), but the key-value
 452:     /// pair is moved to or inserted at that position regardless.
 453:     ///
 454:     /// Computes in **O(n)** time (average).
 455:     pub fn insert_sorted_by_key<B, F>(
 456:         &mut self,
 457:         key: K,
 458:         value: V,
 459:         sort_key: F,
 460:     ) -> (usize, Option<V>)
 461:     where
 462:         B: Ord,
 463:         F: FnMut(&K, &V) -> B,
 464:     {
 465:         self.inner.insert_sorted_by_key(key, value, sort_key)
 466:     }
 467:     /// Insert a key-value pair in the map before the entry at the given index, or at the end.
 468:     ///
 469:     /// If an equivalent key already exists in the map: the key remains and
 470:     /// is moved to the new position in the map, its corresponding value is updated
 471:     /// with `value`, and the older value is returned inside `Some(_)`. The returned index
 472:     /// will either be the given index or one less, depending on how the entry moved.
 473:     /// (See [`shift_insert`](Self::shift_insert) for different behavior here.)
 474:     ///
 475:     /// If no equivalent key existed in the map: the new key-value pair is
 476:     /// inserted exactly at the given index, and `None` is returned.
 477:     ///
 478:     /// ***Panics*** if `index` is out of bounds.
 479:     /// Valid indices are `0..=map.len()` (inclusive).
 480:     ///
 481:     /// Computes in **O(n)** time (average).
 482:     ///
 483:     /// See also [`entry`][Self::entry] if you want to insert *or* modify,
 484:     /// perhaps only using the index for new entries with [`VacantEntry::shift_insert`].
 485:     ///
 486:     /// # Examples
 487:     ///
 488:     /// ```
 489:     /// use ordermap::OrderMap;
 490:     /// let mut map: OrderMap<char, ()> = ('a'..='z').map(|c| (c, ())).collect();
 491:     ///
 492:     /// // The new key '*' goes exactly at the given index.
 493:     /// assert_eq!(map.get_index_of(&'*'), None);
 494:     /// assert_eq!(map.insert_before(10, '*', ()), (10, None));
 495:     /// assert_eq!(map.get_index_of(&'*'), Some(10));
 496:     ///
 497:     /// // Moving the key 'a' up will shift others down, so this moves *before* 10 to index 9.
 498:     /// assert_eq!(map.insert_before(10, 'a', ()), (9, Some(())));
 499:     /// assert_eq!(map.get_index_of(&'a'), Some(9));
 500:     /// assert_eq!(map.get_index_of(&'*'), Some(10));
 501:     ///
 502:     /// // Moving the key 'z' down will shift others up, so this moves to exactly 10.
 503:     /// assert_eq!(map.insert_before(10, 'z', ()), (10, Some(())));
 504:     /// assert_eq!(map.get_index_of(&'z'), Some(10));
 505:     /// assert_eq!(map.get_index_of(&'*'), Some(11));
 506:     ///
 507:     /// // Moving or inserting before the endpoint is also valid.
 508:     /// assert_eq!(map.len(), 27);
 509:     /// assert_eq!(map.insert_before(map.len(), '*', ()), (26, Some(())));
 510:     /// assert_eq!(map.get_index_of(&'*'), Some(26));
 511:     /// assert_eq!(map.insert_before(map.len(), '+', ()), (27, None));
 512:     /// assert_eq!(map.get_index_of(&'+'), Some(27));
 513:     /// assert_eq!(map.len(), 28);
 514:     /// ```
 515:     #[track_caller]
 516:     pub fn insert_before(&mut self, index: usize, key: K, value: V) -> (usize, Option<V>) {
 517:         self.inner.insert_before(index, key, value)
 518:     }
 519:     /// Insert a key-value pair in the map at the given index.
 520:     ///
 521:     /// If an equivalent key already exists in the map: the key remains and
 522:     /// is moved to the given index in the map, its corresponding value is updated
 523:     /// with `value`, and the older value is returned inside `Some(_)`.
 524:     /// Note that existing entries **cannot** be moved to `index == map.len()`!
 525:     /// (See [`insert_before`](Self::insert_before) for different behavior here.)
 526:     ///
 527:     /// If no equivalent key existed in the map: the new key-value pair is
 528:     /// inserted at the given index, and `None` is returned.
 529:     ///
 530:     /// ***Panics*** if `index` is out of bounds.
 531:     /// Valid indices are `0..map.len()` (exclusive) when moving an existing entry, or
 532:     /// `0..=map.len()` (inclusive) when inserting a new key.
 533:     ///
 534:     /// Computes in **O(n)** time (average).
 535:     ///
 536:     /// See also [`entry`][Self::entry] if you want to insert *or* modify,
 537:     /// perhaps only using the index for new entries with [`VacantEntry::shift_insert`].
 538:     ///
 539:     /// # Examples
 540:     ///
 541:     /// ```
 542:     /// use ordermap::OrderMap;
 543:     /// let mut map: OrderMap<char, ()> = ('a'..='z').map(|c| (c, ())).collect();
 544:     ///
 545:     /// // The new key '*' goes exactly at the given index.
 546:     /// assert_eq!(map.get_index_of(&'*'), None);
 547:     /// assert_eq!(map.shift_insert(10, '*', ()), None);
 548:     /// assert_eq!(map.get_index_of(&'*'), Some(10));
 549:     ///
 550:     /// // Moving the key 'a' up to 10 will shift others down, including the '*' that was at 10.
 551:     /// assert_eq!(map.shift_insert(10, 'a', ()), Some(()));
 552:     /// assert_eq!(map.get_index_of(&'a'), Some(10));
 553:     /// assert_eq!(map.get_index_of(&'*'), Some(9));
 554:     ///
 555:     /// // Moving the key 'z' down to 9 will shift others up, including the '*' that was at 9.
 556:     /// assert_eq!(map.shift_insert(9, 'z', ()), Some(()));
 557:     /// assert_eq!(map.get_index_of(&'z'), Some(9));
 558:     /// assert_eq!(map.get_index_of(&'*'), Some(10));
 559:     ///
 560:     /// // Existing keys can move to len-1 at most, but new keys can insert at the endpoint.
 561:     /// assert_eq!(map.len(), 27);
 562:     /// assert_eq!(map.shift_insert(map.len() - 1, '*', ()), Some(()));
 563:     /// assert_eq!(map.get_index_of(&'*'), Some(26));
 564:     /// assert_eq!(map.shift_insert(map.len(), '+', ()), None);
 565:     /// assert_eq!(map.get_index_of(&'+'), Some(27));
 566:     /// assert_eq!(map.len(), 28);
 567:     /// ```
 568:     ///
 569:     /// ```should_panic
 570:     /// use ordermap::OrderMap;
 571:     /// let mut map: OrderMap<char, ()> = ('a'..='z').map(|c| (c, ())).collect();
 572:     ///
 573:     /// // This is an invalid index for moving an existing key!
 574:     /// map.shift_insert(map.len(), 'a', ());
 575:     /// ```
 576:     #[track_caller]
 577:     pub fn shift_insert(&mut self, index: usize, key: K, value: V) -> Option<V> {
 578:         self.inner.shift_insert(index, key, value)
 579:     }
 580:     /// Replaces the key at the given index. The new key does not need to be
 581:     /// equivalent to the one it is replacing, but it must be unique to the rest
 582:     /// of the map.
 583:     ///
 584:     /// Returns `Ok(old_key)` if successful, or `Err((other_index, key))` if an
 585:     /// equivalent key already exists at a different index. The map will be
 586:     /// unchanged in the error case.
 587:     ///
 588:     /// Direct indexing can be used to change the corresponding value: simply
 589:     /// `map[index] = value`, or `mem::replace(&mut map[index], value)` to
 590:     /// retrieve the old value as well.
 591:     ///
 592:     /// ***Panics*** if `index` is out of bounds.
 593:     ///
 594:     /// Computes in **O(1)** time (average).
 595:     #[track_caller]
 596:     pub fn replace_index(&mut self, index: usize, key: K) -> Result<K, (usize, K)> {
 597:         self.inner.replace_index(index, key)
 598:     }
 599:     /// Get the given key's corresponding entry in the map for insertion and/or
 600:     /// in-place manipulation.
 601:     ///
 602:     /// Computes in **O(1)** time (amortized average).
 603:     pub fn entry(&mut self, key: K) -> Entry<'_, K, V> {
 604:         Entry::new(self.inner.entry(key))
 605:     }
 606:     /// Creates a splicing iterator that replaces the specified range in the map
 607:     /// with the given `replace_with` key-value iterator and yields the removed
 608:     /// items. `replace_with` does not need to be the same length as `range`.
 609:     ///
 610:     /// The `range` is removed even if the iterator is not consumed until the
 611:     /// end. It is unspecified how many elements are removed from the map if the
 612:     /// `Splice` value is leaked.
 613:     ///
 614:     /// The input iterator `replace_with` is only consumed when the `Splice`
 615:     /// value is dropped. If a key from the iterator matches an existing entry
 616:     /// in the map (outside of `range`), then the value will be updated in that
 617:     /// position. Otherwise, the new key-value pair will be inserted in the
 618:     /// replaced `range`.
 619:     ///
 620:     /// ***Panics*** if the starting point is greater than the end point or if
 621:     /// the end point is greater than the length of the map.
 622:     ///
 623:     /// # Examples
 624:     ///
 625:     /// ```
 626:     /// use ordermap::OrderMap;
 627:     ///
 628:     /// let mut map = OrderMap::from([(0, '_'), (1, 'a'), (2, 'b'), (3, 'c'), (4, 'd')]);
 629:     /// let new = [(5, 'E'), (4, 'D'), (3, 'C'), (2, 'B'), (1, 'A')];
 630:     /// let removed: Vec<_> = map.splice(2..4, new).collect();
 631:     ///
 632:     /// // 1 and 4 got new values, while 5, 3, and 2 were newly inserted.
 633:     /// assert!(map.into_iter().eq([(0, '_'), (1, 'A'), (5, 'E'), (3, 'C'), (2, 'B'), (4, 'D')]));
 634:     /// assert_eq!(removed, &[(2, 'b'), (3, 'c')]);
 635:     /// ```
 636:     #[track_caller]
 637:     pub fn splice<R, I>(&mut self, range: R, replace_with: I) -> Splice<'_, I::IntoIter, K, V, S>
 638:     where
 639:         R: RangeBounds<usize>,
 640:         I: IntoIterator<Item = (K, V)>,
 641:     {
 642:         self.inner.splice(range, replace_with)
 643:     }
 644:     /// Moves all key-value pairs from `other` into `self`, leaving `other` empty.
 645:     ///
 646:     /// This is equivalent to calling [`insert`][Self::insert] for each
 647:     /// key-value pair from `other` in order, which means that for keys that
 648:     /// already exist in `self`, their value is updated in the current position.
 649:     ///
 650:     /// # Examples
 651:     ///
 652:     /// ```
 653:     /// use ordermap::OrderMap;
 654:     ///
 655:     /// // Note: Key (3) is present in both maps.
 656:     /// let mut a = OrderMap::from([(3, "c"), (2, "b"), (1, "a")]);
 657:     /// let mut b = OrderMap::from([(3, "d"), (4, "e"), (5, "f")]);
 658:     /// let old_capacity = b.capacity();
 659:     ///
 660:     /// a.append(&mut b);
 661:     ///
 662:     /// assert_eq!(a.len(), 5);
 663:     /// assert_eq!(b.len(), 0);
 664:     /// assert_eq!(b.capacity(), old_capacity);
 665:     ///
 666:     /// assert!(a.keys().eq(&[3, 2, 1, 4, 5]));
 667:     /// assert_eq!(a[&3], "d"); // "c" was overwritten.
 668:     /// ```
 669:     pub fn append<S2>(&mut self, other: &mut OrderMap<K, V, S2>) {
 670:         self.inner.append(&mut other.inner);
 671:     }
 672: }
 673: impl<K, V, S> OrderMap<K, V, S>
 674: where
 675:     S: BuildHasher,
 676: {
 677:     /// Return `true` if an equivalent to `key` exists in the map.
 678:     ///
 679:     /// Computes in **O(1)** time (average).
 680:     pub fn contains_key<Q>(&self, key: &Q) -> bool
 681:     where
 682:         Q: ?Sized + Hash + Equivalent<K>,
 683:     {
 684:         self.inner.contains_key(key)
 685:     }
 686:     /// Return a reference to the stored value for `key`, if it is present,
 687:     /// else `None`.
 688:     ///
 689:     /// Computes in **O(1)** time (average).
 690:     pub fn get<Q>(&self, key: &Q) -> Option<&V>
 691:     where
 692:         Q: ?Sized + Hash + Equivalent<K>,
 693:     {
 694:         self.inner.get(key)
 695:     }
 696:     /// Return references to the stored key-value pair for the lookup `key`,
 697:     /// if it is present, else `None`.
 698:     ///
 699:     /// Computes in **O(1)** time (average).
 700:     pub fn get_key_value<Q>(&self, key: &Q) -> Option<(&K, &V)>
 701:     where
 702:         Q: ?Sized + Hash + Equivalent<K>,
 703:     {
 704:         self.inner.get_key_value(key)
 705:     }
 706:     /// Return the index with references to the stored key-value pair for the
 707:     /// lookup `key`, if it is present, else `None`.
 708:     ///
 709:     /// Computes in **O(1)** time (average).
 710:     pub fn get_full<Q>(&self, key: &Q) -> Option<(usize, &K, &V)>
 711:     where
 712:         Q: ?Sized + Hash + Equivalent<K>,
 713:     {
 714:         self.inner.get_full(key)
 715:     }
 716:     /// Return the item index for `key`, if it is present, else `None`.
 717:     ///
 718:     /// Computes in **O(1)** time (average).
 719:     pub fn get_index_of<Q>(&self, key: &Q) -> Option<usize>
 720:     where
 721:         Q: ?Sized + Hash + Equivalent<K>,
 722:     {
 723:         self.inner.get_index_of(key)
 724:     }
 725:     /// Return a mutable reference to the stored value for `key`,
 726:     /// if it is present, else `None`.
 727:     ///
 728:     /// Computes in **O(1)** time (average).
 729:     pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
 730:     where
 731:         Q: ?Sized + Hash + Equivalent<K>,
 732:     {
 733:         self.inner.get_mut(key)
 734:     }
 735:     /// Return a reference and mutable references to the stored key-value pair
 736:     /// for the lookup `key`, if it is present, else `None`.
 737:     ///
 738:     /// Computes in **O(1)** time (average).
 739:     pub fn get_key_value_mut<Q>(&mut self, key: &Q) -> Option<(&K, &mut V)>
 740:     where
 741:         Q: ?Sized + Hash + Equivalent<K>,
 742:     {
 743:         self.inner.get_key_value_mut(key)
 744:     }
 745:     /// Return the index with a reference and mutable reference to the stored
 746:     /// key-value pair for the lookup `key`, if it is present, else `None`.
 747:     ///
 748:     /// Computes in **O(1)** time (average).
 749:     pub fn get_full_mut<Q>(&mut self, key: &Q) -> Option<(usize, &K, &mut V)>
 750:     where
 751:         Q: ?Sized + Hash + Equivalent<K>,
 752:     {
 753:         self.inner.get_full_mut(key)
 754:     }
 755:     /// Return the values for `N` keys.
 756:     ///
 757:     /// ***Panics*** if any key is duplicated.
 758:     ///
 759:     /// # Examples
 760:     ///
 761:     /// ```
 762:     /// let mut map = ordermap::OrderMap::from([(1, 'a'), (3, 'b'), (2, 'c')]);
 763:     /// assert_eq!(
 764:     ///   map.get_disjoint_mut([&2, &1, &0]),
 765:     ///   [Some(&mut 'c'), Some(&mut 'a'), None],
 766:     /// );
 767:     /// ```
 768:     #[track_caller]
 769:     pub fn get_disjoint_mut<Q, const N: usize>(&mut self, keys: [&Q; N]) -> [Option<&mut V>; N]
 770:     where
 771:         Q: ?Sized + Hash + Equivalent<K>,
 772:     {
 773:         self.inner.get_disjoint_mut(keys)
 774:     }
 775:     /// Remove the key-value pair equivalent to `key` and return its value.
 776:     ///
 777:     /// **NOTE:** This is equivalent to [`IndexMap::shift_remove`], and
 778:     /// like [`Vec::remove`], the pair is removed by shifting all of the
 779:     /// elements that follow it, preserving their relative order.
 780:     /// **This perturbs the index of all of those elements!**
 781:     ///
 782:     /// Return `None` if `key` is not in map.
 783:     ///
 784:     /// Computes in **O(n)** time (average).
 785:     pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
 786:     where
 787:         Q: ?Sized + Hash + Equivalent<K>,
 788:     {
 789:         self.inner.shift_remove(key)
 790:     }
 791:     /// Remove and return the key-value pair equivalent to `key`.
 792:     ///
 793:     /// **NOTE:** This is equivalent to [`IndexMap::shift_remove_entry`], and
 794:     /// like [`Vec::remove`], the pair is removed by shifting all of the
 795:     /// elements that follow it, preserving their relative order.
 796:     /// **This perturbs the index of all of those elements!**
 797:     ///
 798:     /// Return `None` if `key` is not in map.
 799:     ///
 800:     /// Computes in **O(n)** time (average).
 801:     pub fn remove_entry<Q>(&mut self, key: &Q) -> Option<(K, V)>
 802:     where
 803:         Q: ?Sized + Hash + Equivalent<K>,
 804:     {
 805:         self.inner.shift_remove_entry(key)
 806:     }
 807:     /// Remove the key-value pair equivalent to `key` and return it and
 808:     /// the index it had.
 809:     ///
 810:     /// **NOTE:** This is equivalent to [`IndexMap::shift_remove_full`], and
 811:     /// like [`Vec::remove`], the pair is removed by shifting all of the
 812:     /// elements that follow it, preserving their relative order.
 813:     /// **This perturbs the index of all of those elements!**
 814:     ///
 815:     /// Return `None` if `key` is not in map.
 816:     ///
 817:     /// Computes in **O(n)** time (average).
 818:     pub fn remove_full<Q>(&mut self, key: &Q) -> Option<(usize, K, V)>
 819:     where
 820:         Q: ?Sized + Hash + Equivalent<K>,
 821:     {
 822:         self.inner.shift_remove_full(key)
 823:     }
 824:     /// Remove the key-value pair equivalent to `key` and return
 825:     /// its value.
 826:     ///
 827:     /// Like [`Vec::swap_remove`], the pair is removed by swapping it with the
 828:     /// last element of the map and popping it off. **This perturbs
 829:     /// the position of what used to be the last element!**
 830:     ///
 831:     /// Return `None` if `key` is not in map.
 832:     ///
 833:     /// Computes in **O(1)** time (average).
 834:     pub fn swap_remove<Q>(&mut self, key: &Q) -> Option<V>
 835:     where
 836:         Q: ?Sized + Hash + Equivalent<K>,
 837:     {
 838:         self.inner.swap_remove(key)
 839:     }
 840:     /// Remove and return the key-value pair equivalent to `key`.
 841:     ///
 842:     /// Like [`Vec::swap_remove`], the pair is removed by swapping it with the
 843:     /// last element of the map and popping it off. **This perturbs
 844:     /// the position of what used to be the last element!**
 845:     ///
 846:     /// Return `None` if `key` is not in map.
 847:     ///
 848:     /// Computes in **O(1)** time (average).
 849:     pub fn swap_remove_entry<Q>(&mut self, key: &Q) -> Option<(K, V)>
 850:     where
 851:         Q: ?Sized + Hash + Equivalent<K>,
 852:     {
 853:         self.inner.swap_remove_entry(key)
 854:     }
 855:     /// Remove the key-value pair equivalent to `key` and return it and
 856:     /// the index it had.
 857:     ///
 858:     /// Like [`Vec::swap_remove`], the pair is removed by swapping it with the
 859:     /// last element of the map and popping it off. **This perturbs
 860:     /// the position of what used to be the last element!**
 861:     ///
 862:     /// Return `None` if `key` is not in map.
 863:     ///
 864:     /// Computes in **O(1)** time (average).
 865:     pub fn swap_remove_full<Q>(&mut self, key: &Q) -> Option<(usize, K, V)>
 866:     where
 867:         Q: ?Sized + Hash + Equivalent<K>,
 868:     {
 869:         self.inner.swap_remove_full(key)
 870:     }
 871: }
 872: impl<K, V, S> OrderMap<K, V, S> {
 873:     /// Remove the last key-value pair
 874:     ///
 875:     /// This preserves the order of the remaining elements.
 876:     ///
 877:     /// Computes in **O(1)** time (average).
 878:     #[doc(alias = "pop_last")] // like `BTreeMap`
 879:     pub fn pop(&mut self) -> Option<(K, V)> {
 880:         self.inner.pop()
 881:     }
 882:     /// Removes and returns the last key-value pair from a map if the predicate
 883:     /// returns `true`, or [`None`] if the predicate returns false or the map
 884:     /// is empty (the predicate will not be called in that case).
 885:     ///
 886:     /// This preserves the order of the remaining elements.
 887:     ///
 888:     /// Computes in **O(1)** time (average).
 889:     ///
 890:     /// # Examples
 891:     ///
 892:     /// ```
 893:     /// use ordermap::OrderMap;
 894:     ///
 895:     /// let init = [(1, 'a'), (2, 'b'), (3, 'c'), (4, 'd')];
 896:     /// let mut map = OrderMap::from(init);
 897:     /// let pred = |key: &i32, _value: &mut char| *key % 2 == 0;
 898:     ///
 899:     /// assert_eq!(map.pop_if(pred), Some((4, 'd')));
 900:     /// assert_eq!(map.as_slice(), &init[..3]);
 901:     /// assert_eq!(map.pop_if(pred), None);
 902:     /// ```
 903:     pub fn pop_if(&mut self, predicate: impl FnOnce(&K, &mut V) -> bool) -> Option<(K, V)> {
 904:         self.inner.pop_if(predicate)
 905:     }
 906:     /// Scan through each key-value pair in the map and keep those where the
 907:     /// closure `keep` returns `true`.
 908:     ///
 909:     /// The elements are visited in order, and remaining elements keep their
 910:     /// order.
 911:     ///
 912:     /// Computes in **O(n)** time (average).
 913:     pub fn retain<F>(&mut self, keep: F)
 914:     where
 915:         F: FnMut(&K, &mut V) -> bool,
 916:     {
 917:         self.inner.retain(keep);
 918:     }
 919:     /// Sort the map's key-value pairs by the default ordering of the keys.
 920:     ///
 921:     /// This is a stable sort -- but equivalent keys should not normally coexist in
 922:     /// a map at all, so [`sort_unstable_keys`][Self::sort_unstable_keys] is preferred
 923:     /// because it is generally faster and doesn't allocate auxiliary memory.
 924:     ///
 925:     /// See [`sort_by`](Self::sort_by) for details.
 926:     pub fn sort_keys(&mut self)
 927:     where
 928:         K: Ord,
 929:     {
 930:         self.inner.sort_keys();
 931:     }
 932:     /// Sort the map's key-value pairs in place using the comparison
 933:     /// function `cmp`.
 934:     ///
 935:     /// The comparison function receives two key and value pairs to compare (you
 936:     /// can sort by keys or values or their combination as needed).
 937:     ///
 938:     /// Computes in **O(n log n + c)** time and **O(n)** space where *n* is
 939:     /// the length of the map and *c* the capacity. The sort is stable.
 940:     pub fn sort_by<F>(&mut self, cmp: F)
 941:     where
 942:         F: FnMut(&K, &V, &K, &V) -> Ordering,
 943:     {
 944:         self.inner.sort_by(cmp);
 945:     }
 946:     /// Sort the key-value pairs of the map and return a by-value iterator of
 947:     /// the key-value pairs with the result.
 948:     ///
 949:     /// The sort is stable.
 950:     pub fn sorted_by<F>(self, cmp: F) -> IntoIter<K, V>
 951:     where
 952:         F: FnMut(&K, &V, &K, &V) -> Ordering,
 953:     {
 954:         self.inner.sorted_by(cmp)
 955:     }
 956:     /// Sort the map's key-value pairs in place using a sort-key extraction function.
 957:     ///
 958:     /// Computes in **O(n log n + c)** time and **O(n)** space where *n* is
 959:     /// the length of the map and *c* the capacity. The sort is stable.
 960:     pub fn sort_by_key<T, F>(&mut self, sort_key: F)
 961:     where
 962:         T: Ord,
 963:         F: FnMut(&K, &V) -> T,
 964:     {
 965:         self.inner.sort_by_key(sort_key)
 966:     }
 967:     /// Sort the map's key-value pairs by the default ordering of the keys, but
 968:     /// may not preserve the order of equal elements.
 969:     ///
 970:     /// See [`sort_unstable_by`](Self::sort_unstable_by) for details.
 971:     pub fn sort_unstable_keys(&mut self)
 972:     where
 973:         K: Ord,
 974:     {
 975:         self.inner.sort_unstable_keys();
 976:     }
 977:     /// Sort the map's key-value pairs in place using the comparison function `cmp`, but
 978:     /// may not preserve the order of equal elements.
 979:     ///
 980:     /// The comparison function receives two key and value pairs to compare (you
 981:     /// can sort by keys or values or their combination as needed).
 982:     ///
 983:     /// Computes in **O(n log n + c)** time where *n* is
 984:     /// the length of the map and *c* is the capacity. The sort is unstable.
 985:     pub fn sort_unstable_by<F>(&mut self, cmp: F)
 986:     where
 987:         F: FnMut(&K, &V, &K, &V) -> Ordering,
 988:     {
 989:         self.inner.sort_unstable_by(cmp);
 990:     }
 991:     /// Sort the key-value pairs of the map and return a by-value iterator of
 992:     /// the key-value pairs with the result.
 993:     ///
 994:     /// The sort is unstable.
 995:     #[inline]
 996:     pub fn sorted_unstable_by<F>(self, cmp: F) -> IntoIter<K, V>
 997:     where
 998:         F: FnMut(&K, &V, &K, &V) -> Ordering,
 999:     {
1000:         self.inner.sorted_unstable_by(cmp)
1001:     }
1002:     /// Sort the map's key-value pairs in place using a sort-key extraction function.
1003:     ///
1004:     /// Computes in **O(n log n + c)** time where *n* is
1005:     /// the length of the map and *c* is the capacity. The sort is unstable.
1006:     pub fn sort_unstable_by_key<T, F>(&mut self, sort_key: F)
1007:     where
1008:         T: Ord,
1009:         F: FnMut(&K, &V) -> T,
1010:     {
1011:         self.inner.sort_unstable_by_key(sort_key)
1012:     }
1013:     /// Sort the map's key-value pairs in place using a sort-key extraction function.
1014:     ///
1015:     /// During sorting, the function is called at most once per entry, by using temporary storage
1016:     /// to remember the results of its evaluation. The order of calls to the function is
1017:     /// unspecified and may change between versions of `ordermap` or the standard library.
1018:     ///
1019:     /// Computes in **O(m n + n log n + c)** time () and **O(n)** space, where the function is
1020:     /// **O(m)**, *n* is the length of the map, and *c* the capacity. The sort is stable.
1021:     pub fn sort_by_cached_key<T, F>(&mut self, sort_key: F)
1022:     where
1023:         T: Ord,
1024:         F: FnMut(&K, &V) -> T,
1025:     {
1026:         self.inner.sort_by_cached_key(sort_key);
1027:     }
1028:     /// Search over a sorted map for a key.
1029:     ///
1030:     /// Returns the position where that key is present, or the position where it can be inserted to
1031:     /// maintain the sort. See [`slice::binary_search`] for more details.
1032:     ///
1033:     /// Computes in **O(log(n))** time, which is notably less scalable than looking the key up
1034:     /// using [`get_index_of`][OrderMap::get_index_of], but this can also position missing keys.
1035:     pub fn binary_search_keys(&self, x: &K) -> Result<usize, usize>
1036:     where
1037:         K: Ord,
1038:     {
1039:         self.inner.binary_search_keys(x)
1040:     }
1041:     /// Search over a sorted map with a comparator function.
1042:     ///
1043:     /// Returns the position where that value is present, or the position where it can be inserted
1044:     /// to maintain the sort. See [`slice::binary_search_by`] for more details.
1045:     ///
1046:     /// Computes in **O(log(n))** time.
1047:     #[inline]
1048:     pub fn binary_search_by<'a, F>(&'a self, f: F) -> Result<usize, usize>
1049:     where
1050:         F: FnMut(&'a K, &'a V) -> Ordering,
1051:     {
1052:         self.inner.binary_search_by(f)
1053:     }
1054:     /// Search over a sorted map with an extraction function.
1055:     ///
1056:     /// Returns the position where that value is present, or the position where it can be inserted
1057:     /// to maintain the sort. See [`slice::binary_search_by_key`] for more details.
1058:     ///
1059:     /// Computes in **O(log(n))** time.
1060:     #[inline]
1061:     pub fn binary_search_by_key<'a, B, F>(&'a self, b: &B, f: F) -> Result<usize, usize>
1062:     where
1063:         F: FnMut(&'a K, &'a V) -> B,
1064:         B: Ord,
1065:     {
1066:         self.inner.binary_search_by_key(b, f)
1067:     }
1068:     /// Checks if the keys of this map are sorted.
1069:     #[inline]
1070:     pub fn is_sorted(&self) -> bool
1071:     where
1072:         K: PartialOrd,
1073:     {
1074:         self.inner.is_sorted()
1075:     }
1076:     /// Checks if this map is sorted using the given comparator function.
1077:     #[inline]
1078:     pub fn is_sorted_by<'a, F>(&'a self, cmp: F) -> bool
1079:     where
1080:         F: FnMut(&'a K, &'a V, &'a K, &'a V) -> bool,
1081:     {
1082:         self.inner.is_sorted_by(cmp)
1083:     }
1084:     /// Checks if this map is sorted using the given sort-key function.
1085:     #[inline]
1086:     pub fn is_sorted_by_key<'a, F, T>(&'a self, sort_key: F) -> bool
1087:     where
1088:         F: FnMut(&'a K, &'a V) -> T,
1089:         T: PartialOrd,
1090:     {
1091:         self.inner.is_sorted_by_key(sort_key)
1092:     }
1093:     /// Returns the index of the partition point of a sorted map according to the given predicate
1094:     /// (the index of the first element of the second partition).
1095:     ///
1096:     /// See [`slice::partition_point`] for more details.
1097:     ///
1098:     /// Computes in **O(log(n))** time.
1099:     #[must_use]
1100:     pub fn partition_point<P>(&self, pred: P) -> usize
1101:     where
1102:         P: FnMut(&K, &V) -> bool,
1103:     {
1104:         self.inner.partition_point(pred)
1105:     }
1106:     /// Reverses the order of the map's key-value pairs in place.
1107:     ///
1108:     /// Computes in **O(n)** time and **O(1)** space.
1109:     pub fn reverse(&mut self) {
1110:         self.inner.reverse()
1111:     }
1112:     /// Returns a slice of all the key-value pairs in the map.
1113:     ///
1114:     /// Computes in **O(1)** time.
1115:     pub fn as_slice(&self) -> &Slice<K, V> {
1116:         self.inner.as_slice()
1117:     }
1118:     /// Returns a mutable slice of all the key-value pairs in the map.
1119:     ///
1120:     /// Computes in **O(1)** time.
1121:     pub fn as_mut_slice(&mut self) -> &mut Slice<K, V> {
1122:         self.inner.as_mut_slice()
1123:     }
1124:     /// Converts into a boxed slice of all the key-value pairs in the map.
1125:     ///
1126:     /// Note that this will drop the inner hash table and any excess capacity.
1127:     pub fn into_boxed_slice(self) -> Box<Slice<K, V>> {
1128:         self.inner.into_boxed_slice()
1129:     }
1130:     /// Get a key-value pair by index
1131:     ///
1132:     /// Valid indices are `0 <= index < self.len()`.
1133:     ///
1134:     /// Computes in **O(1)** time.
1135:     pub fn get_index(&self, index: usize) -> Option<(&K, &V)> {
1136:         self.inner.get_index(index)
1137:     }
1138:     /// Get a key-value pair by index
1139:     ///
1140:     /// Valid indices are `0 <= index < self.len()`.
1141:     ///
1142:     /// Computes in **O(1)** time.
1143:     pub fn get_index_mut(&mut self, index: usize) -> Option<(&K, &mut V)> {
1144:         self.inner.get_index_mut(index)
1145:     }
1146:     /// Get an entry in the map by index for in-place manipulation.
1147:     ///
1148:     /// Valid indices are `0 <= index < self.len()`.
1149:     ///
1150:     /// Computes in **O(1)** time.
1151:     pub fn get_index_entry(&mut self, index: usize) -> Option<IndexedEntry<'_, K, V>> {
1152:         self.inner.get_index_entry(index).map(IndexedEntry::new)
1153:     }
1154:     /// Get an array of `N` key-value pairs by `N` indices
1155:     ///
1156:     /// Valid indices are *0 <= index < self.len()* and each index needs to be unique.
1157:     ///
1158:     /// # Examples
1159:     ///
1160:     /// ```
1161:     /// let mut map = ordermap::OrderMap::from([(1, 'a'), (3, 'b'), (2, 'c')]);
1162:     /// assert_eq!(map.get_disjoint_indices_mut([2, 0]), Ok([(&2, &mut 'c'), (&1, &mut 'a')]));
1163:     /// ```
1164:     pub fn get_disjoint_indices_mut<const N: usize>(
1165:         &mut self,
1166:         indices: [usize; N],
1167:     ) -> Result<[(&K, &mut V); N], GetDisjointMutError> {
1168:         self.as_mut_slice().get_disjoint_mut(indices)
1169:     }
1170:     /// Returns a slice of key-value pairs in the given range of indices.
1171:     ///
1172:     /// Valid indices are `0 <= index < self.len()`.
1173:     ///
1174:     /// Computes in **O(1)** time.
1175:     pub fn get_range<R: RangeBounds<usize>>(&self, range: R) -> Option<&Slice<K, V>> {
1176:         self.inner.get_range(range)
1177:     }
1178:     /// Returns a mutable slice of key-value pairs in the given range of indices.
1179:     ///
1180:     /// Valid indices are `0 <= index < self.len()`.
1181:     ///
1182:     /// Computes in **O(1)** time.
1183:     pub fn get_range_mut<R: RangeBounds<usize>>(&mut self, range: R) -> Option<&mut Slice<K, V>> {
1184:         self.inner.get_range_mut(range)
1185:     }
1186:     /// Get the first key-value pair
1187:     ///
1188:     /// Computes in **O(1)** time.
1189:     #[doc(alias = "first_key_value")] // like `BTreeMap`
1190:     pub fn first(&self) -> Option<(&K, &V)> {
1191:         self.inner.first()
1192:     }
1193:     /// Get the first key-value pair, with mutable access to the value
1194:     ///
1195:     /// Computes in **O(1)** time.
1196:     pub fn first_mut(&mut self) -> Option<(&K, &mut V)> {
1197:         self.inner.first_mut()
1198:     }
1199:     /// Get the first entry in the map for in-place manipulation.
1200:     ///
1201:     /// Computes in **O(1)** time.
1202:     pub fn first_entry(&mut self) -> Option<IndexedEntry<'_, K, V>> {
1203:         self.inner.first_entry().map(IndexedEntry::new)
1204:     }
1205:     /// Get the last key-value pair
1206:     ///
1207:     /// Computes in **O(1)** time.
1208:     #[doc(alias = "last_key_value")] // like `BTreeMap`
1209:     pub fn last(&self) -> Option<(&K, &V)> {
1210:         self.inner.last()
1211:     }
1212:     /// Get the last key-value pair, with mutable access to the value
1213:     ///
1214:     /// Computes in **O(1)** time.
1215:     pub fn last_mut(&mut self) -> Option<(&K, &mut V)> {
1216:         self.inner.last_mut()
1217:     }
1218:     /// Get the last entry in the map for in-place manipulation.
1219:     ///
1220:     /// Computes in **O(1)** time.
1221:     pub fn last_entry(&mut self) -> Option<IndexedEntry<'_, K, V>> {
1222:         self.inner.last_entry().map(IndexedEntry::new)
1223:     }
1224:     /// Remove the key-value pair by index
1225:     ///
1226:     /// Valid indices are `0 <= index < self.len()`
1227:     ///
1228:     /// **NOTE:** This is equivalent to [`IndexMap::shift_remove_index`], and
1229:     /// like [`Vec::remove`], the pair is removed by shifting all of the
1230:     /// elements that follow it, preserving their relative order.
1231:     /// **This perturbs the index of all of those elements!**
1232:     ///
1233:     /// Computes in **O(n)** time (average).
1234:     pub fn remove_index(&mut self, index: usize) -> Option<(K, V)> {
1235:         self.inner.shift_remove_index(index)
1236:     }
1237:     /// Remove the key-value pair by index
1238:     ///
1239:     /// Valid indices are `0 <= index < self.len()`.
1240:     ///
1241:     /// Like [`Vec::swap_remove`], the pair is removed by swapping it with the
1242:     /// last element of the map and popping it off. **This perturbs
1243:     /// the position of what used to be the last element!**
1244:     ///
1245:     /// Computes in **O(1)** time (average).
1246:     pub fn swap_remove_index(&mut self, index: usize) -> Option<(K, V)> {
1247:         self.inner.swap_remove_index(index)
1248:     }
1249:     /// Moves the position of a key-value pair from one index to another
1250:     /// by shifting all other pairs in-between.
1251:     ///
1252:     /// * If `from < to`, the other pairs will shift down while the targeted pair moves up.
1253:     /// * If `from > to`, the other pairs will shift up while the targeted pair moves down.
1254:     ///
1255:     /// ***Panics*** if `from` or `to` are out of bounds.
1256:     ///
1257:     /// Computes in **O(n)** time (average).
1258:     #[track_caller]
1259:     pub fn move_index(&mut self, from: usize, to: usize) {
1260:         self.inner.move_index(from, to)
1261:     }
1262:     /// Swaps the position of two key-value pairs in the map.
1263:     ///
1264:     /// ***Panics*** if `a` or `b` are out of bounds.
1265:     ///
1266:     /// Computes in **O(1)** time (average).
1267:     #[track_caller]
1268:     pub fn swap_indices(&mut self, a: usize, b: usize) {
1269:         self.inner.swap_indices(a, b)
1270:     }
1271: }
1272: /// Access [`OrderMap`] values corresponding to a key.
1273: ///
1274: /// # Examples
1275: ///
1276: /// ```
1277: /// use ordermap::OrderMap;
1278: ///
1279: /// let mut map = OrderMap::new();
1280: /// for word in "Lorem ipsum dolor sit amet".split_whitespace() {
1281: ///     map.insert(word.to_lowercase(), word.to_uppercase());
1282: /// }
1283: /// assert_eq!(map["lorem"], "LOREM");
1284: /// assert_eq!(map["ipsum"], "IPSUM");
1285: /// ```
1286: ///
1287: /// ```should_panic
1288: /// use ordermap::OrderMap;
1289: ///
1290: /// let mut map = OrderMap::new();
1291: /// map.insert("foo", 1);
1292: /// println!("{:?}", map["bar"]); // panics!
1293: /// ```
1294: impl<K, V, Q: ?Sized, S> Index<&Q> for OrderMap<K, V, S>
1295: where
1296:     Q: Hash + Equivalent<K>,
1297:     S: BuildHasher,
1298: {
1299:     type Output = V;
1300:     /// Returns a reference to the value corresponding to the supplied `key`.
1301:     ///
1302:     /// ***Panics*** if `key` is not present in the map.
1303:     fn index(&self, key: &Q) -> &V {
1304:         self.get(key).expect("no entry found for key")
1305:     }
1306: }
1307: /// Access [`OrderMap`] values corresponding to a key.
1308: ///
1309: /// Mutable indexing allows changing / updating values of key-value
1310: /// pairs that are already present.
1311: ///
1312: /// You can **not** insert new pairs with index syntax, use `.insert()`.
1313: ///
1314: /// # Examples
1315: ///
1316: /// ```
1317: /// use ordermap::OrderMap;
1318: ///
1319: /// let mut map = OrderMap::new();
1320: /// for word in "Lorem ipsum dolor sit amet".split_whitespace() {
1321: ///     map.insert(word.to_lowercase(), word.to_string());
1322: /// }
1323: /// let lorem = &mut map["lorem"];
1324: /// assert_eq!(lorem, "Lorem");
1325: /// lorem.retain(char::is_lowercase);
1326: /// assert_eq!(map["lorem"], "orem");
1327: /// ```
1328: ///
1329: /// ```should_panic
1330: /// use ordermap::OrderMap;
1331: ///
1332: /// let mut map = OrderMap::new();
1333: /// map.insert("foo", 1);
1334: /// map["bar"] = 1; // panics!
1335: /// ```
1336: impl<K, V, Q: ?Sized, S> IndexMut<&Q> for OrderMap<K, V, S>
1337: where
1338:     Q: Hash + Equivalent<K>,
1339:     S: BuildHasher,
1340: {
1341:     /// Returns a mutable reference to the value corresponding to the supplied `key`.
1342:     ///
1343:     /// ***Panics*** if `key` is not present in the map.
1344:     fn index_mut(&mut self, key: &Q) -> &mut V {
1345:         self.get_mut(key).expect("no entry found for key")
1346:     }
1347: }
1348: /// Access [`OrderMap`] values at indexed positions.
1349: ///
1350: /// See [`Index<usize> for Keys`][keys] to access a map's keys instead.
1351: ///
1352: /// [keys]: Keys#impl-Index<usize>-for-Keys<'a,+K,+V>
1353: ///
1354: /// # Examples
1355: ///
1356: /// ```
1357: /// use ordermap::OrderMap;
1358: ///
1359: /// let mut map = OrderMap::new();
1360: /// for word in "Lorem ipsum dolor sit amet".split_whitespace() {
1361: ///     map.insert(word.to_lowercase(), word.to_uppercase());
1362: /// }
1363: /// assert_eq!(map[0], "LOREM");
1364: /// assert_eq!(map[1], "IPSUM");
1365: /// map.reverse();
1366: /// assert_eq!(map[0], "AMET");
1367: /// assert_eq!(map[1], "SIT");
1368: /// map.sort_keys();
1369: /// assert_eq!(map[0], "AMET");
1370: /// assert_eq!(map[1], "DOLOR");
1371: /// ```
1372: ///
1373: /// ```should_panic
1374: /// use ordermap::OrderMap;
1375: ///
1376: /// let mut map = OrderMap::new();
1377: /// map.insert("foo", 1);
1378: /// println!("{:?}", map[10]); // panics!
1379: /// ```
1380: impl<K, V, S> Index<usize> for OrderMap<K, V, S> {
1381:     type Output = V;
1382:     /// Returns a reference to the value at the supplied `index`.
1383:     ///
1384:     /// ***Panics*** if `index` is out of bounds.
1385:     fn index(&self, index: usize) -> &V {
1386:         &self.inner[index]
1387:     }
1388: }
1389: /// Access [`OrderMap`] values at indexed positions.
1390: ///
1391: /// Mutable indexing allows changing / updating indexed values
1392: /// that are already present.
1393: ///
1394: /// You can **not** insert new values with index syntax -- use [`.insert()`][OrderMap::insert].
1395: ///
1396: /// # Examples
1397: ///
1398: /// ```
1399: /// use ordermap::OrderMap;
1400: ///
1401: /// let mut map = OrderMap::new();
1402: /// for word in "Lorem ipsum dolor sit amet".split_whitespace() {
1403: ///     map.insert(word.to_lowercase(), word.to_string());
1404: /// }
1405: /// let lorem = &mut map[0];
1406: /// assert_eq!(lorem, "Lorem");
1407: /// lorem.retain(char::is_lowercase);
1408: /// assert_eq!(map["lorem"], "orem");
1409: /// ```
1410: ///
1411: /// ```should_panic
1412: /// use ordermap::OrderMap;
1413: ///
1414: /// let mut map = OrderMap::new();
1415: /// map.insert("foo", 1);
1416: /// map[10] = 1; // panics!
1417: /// ```
1418: impl<K, V, S> IndexMut<usize> for OrderMap<K, V, S> {
1419:     /// Returns a mutable reference to the value at the supplied `index`.
1420:     ///
1421:     /// ***Panics*** if `index` is out of bounds.
1422:     fn index_mut(&mut self, index: usize) -> &mut V {
1423:         &mut self.inner[index]
1424:     }
1425: }
1426: impl<K, V, S> FromIterator<(K, V)> for OrderMap<K, V, S>
1427: where
1428:     K: Hash + Eq,
1429:     S: BuildHasher + Default,
1430: {
1431:     /// Create an `OrderMap` from the sequence of key-value pairs in the
1432:     /// iterable.
1433:     ///
1434:     /// `from_iter` uses the same logic as `extend`. See
1435:     /// [`extend`][OrderMap::extend] for more details.
1436:     fn from_iter<I: IntoIterator<Item = (K, V)>>(iterable: I) -> Self {
1437:         Self {
1438:             inner: IndexMap::from_iter(iterable),
1439:         }
1440:     }
1441: }
1442: #[cfg(feature = "std")]
1443: #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
1444: impl<K, V, const N: usize> From<[(K, V); N]> for OrderMap<K, V, RandomState>
1445: where
1446:     K: Hash + Eq,
1447: {
1448:     /// # Examples
1449:     ///
1450:     /// ```
1451:     /// use ordermap::OrderMap;
1452:     ///
1453:     /// let map1 = OrderMap::from([(1, 2), (3, 4)]);
1454:     /// let map2: OrderMap<_, _> = [(1, 2), (3, 4)].into();
1455:     /// assert_eq!(map1, map2);
1456:     /// ```
1457:     fn from(arr: [(K, V); N]) -> Self {
1458:         Self::from_iter(arr)
1459:     }
1460: }
1461: impl<K, V, S> Extend<(K, V)> for OrderMap<K, V, S>
1462: where
1463:     K: Hash + Eq,
1464:     S: BuildHasher,
1465: {
1466:     /// Extend the map with all key-value pairs in the iterable.
1467:     ///
1468:     /// This is equivalent to calling [`insert`][OrderMap::insert] for each of
1469:     /// them in order, which means that for keys that already existed
1470:     /// in the map, their value is updated but it keeps the existing order.
1471:     ///
1472:     /// New keys are inserted in the order they appear in the sequence. If
1473:     /// equivalents of a key occur more than once, the last corresponding value
1474:     /// prevails.
1475:     fn extend<I: IntoIterator<Item = (K, V)>>(&mut self, iterable: I) {
1476:         self.inner.extend(iterable);
1477:     }
1478: }
1479: impl<'a, K, V, S> Extend<(&'a K, &'a V)> for OrderMap<K, V, S>
1480: where
1481:     K: Hash + Eq + Copy,
1482:     V: Copy,
1483:     S: BuildHasher,
1484: {
1485:     /// Extend the map with all key-value pairs in the iterable.
1486:     ///
1487:     /// See the first extend method for more details.
1488:     fn extend<I: IntoIterator<Item = (&'a K, &'a V)>>(&mut self, iterable: I) {
1489:         self.inner.extend(iterable);
1490:     }
1491: }
1492: impl<K, V, S> Default for OrderMap<K, V, S>
1493: where
1494:     S: Default,
1495: {
1496:     /// Return an empty [`OrderMap`]
1497:     fn default() -> Self {
1498:         Self::with_capacity_and_hasher(0, S::default())
1499:     }
1500: }
1501: impl<K, V, S1, S2> PartialEq<OrderMap<K, V, S2>> for OrderMap<K, V, S1>
1502: where
1503:     K: PartialEq,
1504:     V: PartialEq,
1505: {
1506:     fn eq(&self, other: &OrderMap<K, V, S2>) -> bool {
1507:         self.len() == other.len() && self.iter().eq(other)
1508:     }
1509: }
1510: impl<K, V, S> Eq for OrderMap<K, V, S>
1511: where
1512:     K: Eq,
1513:     V: Eq,
1514: {
1515: }
1516: impl<K, V, S1, S2> PartialOrd<OrderMap<K, V, S2>> for OrderMap<K, V, S1>
1517: where
1518:     K: PartialOrd,
1519:     V: PartialOrd,
1520: {
1521:     fn partial_cmp(&self, other: &OrderMap<K, V, S2>) -> Option<Ordering> {
1522:         self.iter().partial_cmp(other)
1523:     }
1524: }
1525: impl<K, V, S> Ord for OrderMap<K, V, S>
1526: where
1527:     K: Ord,
1528:     V: Ord,
1529: {
1530:     fn cmp(&self, other: &Self) -> Ordering {
1531:         self.iter().cmp(other)
1532:     }
1533: }
1534: impl<K, V, S> Hash for OrderMap<K, V, S>
1535: where
1536:     K: Hash,
1537:     V: Hash,
1538: {
1539:     fn hash<H: Hasher>(&self, state: &mut H) {
1540:         self.len().hash(state);
1541:         for (key, value) in self {
1542:             key.hash(state);
1543:             value.hash(state);
1544:         }
1545:     }
1546: }
````

## File: src/serde.rs
````rust
 1: #![cfg_attr(docsrs, doc(cfg(feature = "serde")))]
 2: use crate::{OrderMap, OrderSet};
 3: use core::hash::{BuildHasher, Hash};
 4: use serde_core::de::value::{MapDeserializer, SeqDeserializer};
 5: use serde_core::de::{Deserialize, Deserializer, Error, IntoDeserializer};
 6: use serde_core::ser::{Serialize, Serializer};
 7: impl<K, V, S> Serialize for OrderMap<K, V, S>
 8: where
 9:     K: Serialize,
10:     V: Serialize,
11: {
12:     fn serialize<T>(&self, serializer: T) -> Result<T::Ok, T::Error>
13:     where
14:         T: Serializer,
15:     {
16:         self.inner.serialize(serializer)
17:     }
18: }
19: impl<'de, K, V, S> Deserialize<'de> for OrderMap<K, V, S>
20: where
21:     K: Deserialize<'de> + Eq + Hash,
22:     V: Deserialize<'de>,
23:     S: Default + BuildHasher,
24: {
25:     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
26:     where
27:         D: Deserializer<'de>,
28:     {
29:         Ok(Self {
30:             inner: <_>::deserialize(deserializer)?,
31:         })
32:     }
33: }
34: impl<'de, K, V, S, E> IntoDeserializer<'de, E> for OrderMap<K, V, S>
35: where
36:     K: IntoDeserializer<'de, E> + Eq + Hash,
37:     V: IntoDeserializer<'de, E>,
38:     S: BuildHasher,
39:     E: Error,
40: {
41:     type Deserializer = MapDeserializer<'de, <Self as IntoIterator>::IntoIter, E>;
42:     fn into_deserializer(self) -> Self::Deserializer {
43:         self.inner.into_deserializer()
44:     }
45: }
46: impl<T, S> Serialize for OrderSet<T, S>
47: where
48:     T: Serialize,
49: {
50:     fn serialize<Se>(&self, serializer: Se) -> Result<Se::Ok, Se::Error>
51:     where
52:         Se: Serializer,
53:     {
54:         self.inner.serialize(serializer)
55:     }
56: }
57: impl<'de, T, S> Deserialize<'de> for OrderSet<T, S>
58: where
59:     T: Deserialize<'de> + Eq + Hash,
60:     S: Default + BuildHasher,
61: {
62:     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
63:     where
64:         D: Deserializer<'de>,
65:     {
66:         Ok(Self {
67:             inner: <_>::deserialize(deserializer)?,
68:         })
69:     }
70: }
71: impl<'de, T, S, E> IntoDeserializer<'de, E> for OrderSet<T, S>
72: where
73:     T: IntoDeserializer<'de, E> + Eq + Hash,
74:     S: BuildHasher,
75:     E: Error,
76: {
77:     type Deserializer = SeqDeserializer<<Self as IntoIterator>::IntoIter, E>;
78:     fn into_deserializer(self) -> Self::Deserializer {
79:         self.inner.into_deserializer()
80:     }
81: }
````

## File: src/set.rs
````rust
   1: //! [`OrderSet`] is a hash table set where the iteration order of the items
   2: //! is independent of the hash values of the items.
   3: //!
   4: //! It is based on [`IndexSet`], and even shares many of the auxiliary types
   5: //! like [`Slice`] and all of the iterators.
   6: //!
   7: //! **Unlike** `IndexSet`, `OrderSet` does consider the order for [`PartialEq`]
   8: //! and [`Eq`], and it also implements [`PartialOrd`], [`Ord`], and [`Hash`].
   9: //! Methods like [`OrderSet::remove`] use `IndexSet`'s "shift" semantics, so
  10: //! they preserve the relative order of remaining entries.
  11: mod iter;
  12: mod mutable;
  13: mod slice;
  14: #[cfg(test)]
  15: mod tests;
  16: pub use self::mutable::MutableValues;
  17: pub use indexmap::set::{
  18:     Difference, Drain, ExtractIf, Intersection, IntoIter, Iter, Slice, Splice, SymmetricDifference,
  19:     Union,
  20: };
  21: #[cfg(feature = "rayon")]
  22: #[cfg_attr(docsrs, doc(cfg(feature = "rayon")))]
  23: pub mod rayon;
  24: use alloc::boxed::Box;
  25: use core::cmp::Ordering;
  26: use core::fmt;
  27: use core::hash::{BuildHasher, Hash, Hasher};
  28: use core::ops::{BitAnd, BitOr, BitXor, Index, RangeBounds, Sub};
  29: use indexmap::IndexSet;
  30: #[cfg(doc)]
  31: use alloc::vec::Vec;
  32: #[cfg(feature = "std")]
  33: use std::hash::RandomState;
  34: use crate::{Equivalent, TryReserveError};
  35: /// A hash set where the iteration order of the values is independent of their
  36: /// hash values.
  37: ///
  38: /// The interface is closely compatible with the standard
  39: /// [`HashSet`][std::collections::HashSet],
  40: /// but also has additional features.
  41: ///
  42: /// # Order
  43: ///
  44: /// The values have a consistent order that is determined by the sequence of
  45: /// insertion and removal calls on the set. The order does not depend on the
  46: /// values or the hash function at all. Note that insertion order and value
  47: /// are not affected if a re-insertion is attempted once an element is
  48: /// already present.
  49: ///
  50: /// All iterators traverse the set *in order*.  Set operation iterators like
  51: /// [`OrderSet::union`] produce a concatenated order, as do their matching "bitwise"
  52: /// operators.  See their documentation for specifics.
  53: ///
  54: /// The insertion order is preserved, with **notable exceptions** like the
  55: /// [`.swap_remove()`][Self::swap_remove] method.
  56: /// Methods such as [`.sort_by()`][Self::sort_by] of
  57: /// course result in a new order, depending on the sorting order.
  58: ///
  59: /// # Indices
  60: ///
  61: /// The values are indexed in a compact range without holes in the range
  62: /// `0..self.len()`. For example, the method `.get_full` looks up the index for
  63: /// a value, and the method `.get_index` looks up the value by index.
  64: ///
  65: /// # Complexity
  66: ///
  67: /// Internally, `OrderSet<T, S>` just holds an [`IndexSet<T, S>`](IndexSet).
  68: /// Thus the complexity of the two are the same for most methods.
  69: ///
  70: /// # Examples
  71: ///
  72: /// ```
  73: /// use ordermap::OrderSet;
  74: ///
  75: /// // Collects which letters appear in a sentence.
  76: /// let letters: OrderSet<_> = "a short treatise on fungi".chars().collect();
  77: ///
  78: /// assert!(letters.contains(&'s'));
  79: /// assert!(letters.contains(&'t'));
  80: /// assert!(letters.contains(&'u'));
  81: /// assert!(!letters.contains(&'y'));
  82: /// ```
  83: #[cfg(feature = "std")]
  84: pub struct OrderSet<T, S = RandomState> {
  85:     pub(crate) inner: IndexSet<T, S>,
  86: }
  87: #[cfg(not(feature = "std"))]
  88: pub struct OrderSet<T, S> {
  89:     pub(crate) inner: IndexSet<T, S>,
  90: }
  91: impl<T, S> Clone for OrderSet<T, S>
  92: where
  93:     T: Clone,
  94:     S: Clone,
  95: {
  96:     fn clone(&self) -> Self {
  97:         Self {
  98:             inner: self.inner.clone(),
  99:         }
 100:     }
 101:     fn clone_from(&mut self, other: &Self) {
 102:         self.inner.clone_from(&other.inner);
 103:     }
 104: }
 105: impl<T, S> fmt::Debug for OrderSet<T, S>
 106: where
 107:     T: fmt::Debug,
 108: {
 109:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
 110:         f.debug_set().entries(self.iter()).finish()
 111:     }
 112: }
 113: #[cfg(feature = "std")]
 114: #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
 115: impl<T> OrderSet<T> {
 116:     /// Create a new set. (Does not allocate.)
 117:     pub fn new() -> Self {
 118:         Self {
 119:             inner: IndexSet::new(),
 120:         }
 121:     }
 122:     /// Create a new set with capacity for `n` elements.
 123:     /// (Does not allocate if `n` is zero.)
 124:     ///
 125:     /// Computes in **O(n)** time.
 126:     pub fn with_capacity(n: usize) -> Self {
 127:         Self {
 128:             inner: IndexSet::with_capacity(n),
 129:         }
 130:     }
 131: }
 132: impl<T, S> OrderSet<T, S> {
 133:     /// Create a new set with capacity for `n` elements.
 134:     /// (Does not allocate if `n` is zero.)
 135:     ///
 136:     /// Computes in **O(n)** time.
 137:     pub fn with_capacity_and_hasher(n: usize, hash_builder: S) -> Self {
 138:         Self {
 139:             inner: IndexSet::with_capacity_and_hasher(n, hash_builder),
 140:         }
 141:     }
 142:     /// Create a new set with `hash_builder`.
 143:     ///
 144:     /// This function is `const`, so it
 145:     /// can be called in `static` contexts.
 146:     pub const fn with_hasher(hash_builder: S) -> Self {
 147:         Self {
 148:             inner: IndexSet::with_hasher(hash_builder),
 149:         }
 150:     }
 151:     /// Return the number of elements the set can hold without reallocating.
 152:     ///
 153:     /// This number is a lower bound; the set might be able to hold more,
 154:     /// but is guaranteed to be able to hold at least this many.
 155:     ///
 156:     /// Computes in **O(1)** time.
 157:     pub fn capacity(&self) -> usize {
 158:         self.inner.capacity()
 159:     }
 160:     /// Return a reference to the set's `BuildHasher`.
 161:     pub fn hasher(&self) -> &S {
 162:         self.inner.hasher()
 163:     }
 164:     /// Return the number of elements in the set.
 165:     ///
 166:     /// Computes in **O(1)** time.
 167:     pub fn len(&self) -> usize {
 168:         self.inner.len()
 169:     }
 170:     /// Returns true if the set contains no elements.
 171:     ///
 172:     /// Computes in **O(1)** time.
 173:     pub fn is_empty(&self) -> bool {
 174:         self.inner.is_empty()
 175:     }
 176:     /// Return an iterator over the values of the set, in their order
 177:     pub fn iter(&self) -> Iter<'_, T> {
 178:         self.inner.iter()
 179:     }
 180:     /// Remove all elements in the set, while preserving its capacity.
 181:     ///
 182:     /// Computes in **O(n)** time.
 183:     pub fn clear(&mut self) {
 184:         self.inner.clear();
 185:     }
 186:     /// Shortens the set, keeping the first `len` elements and dropping the rest.
 187:     ///
 188:     /// If `len` is greater than the set's current length, this has no effect.
 189:     pub fn truncate(&mut self, len: usize) {
 190:         self.inner.truncate(len);
 191:     }
 192:     /// Clears the `OrderSet` in the given index range, returning those values
 193:     /// as a drain iterator.
 194:     ///
 195:     /// The range may be any type that implements [`RangeBounds<usize>`],
 196:     /// including all of the `std::ops::Range*` types, or even a tuple pair of
 197:     /// `Bound` start and end values. To drain the set entirely, use `RangeFull`
 198:     /// like `set.drain(..)`.
 199:     ///
 200:     /// This shifts down all entries following the drained range to fill the
 201:     /// gap, and keeps the allocated memory for reuse.
 202:     ///
 203:     /// ***Panics*** if the starting point is greater than the end point or if
 204:     /// the end point is greater than the length of the set.
 205:     #[track_caller]
 206:     pub fn drain<R>(&mut self, range: R) -> Drain<'_, T>
 207:     where
 208:         R: RangeBounds<usize>,
 209:     {
 210:         self.inner.drain(range)
 211:     }
 212:     /// Creates an iterator which uses a closure to determine if a value should be removed,
 213:     /// for all values in the given range.
 214:     ///
 215:     /// If the closure returns true, then the value is removed and yielded.
 216:     /// If the closure returns false, the value will remain in the list and will not be yielded
 217:     /// by the iterator.
 218:     ///
 219:     /// The range may be any type that implements [`RangeBounds<usize>`],
 220:     /// including all of the `std::ops::Range*` types, or even a tuple pair of
 221:     /// `Bound` start and end values. To check the entire set, use `RangeFull`
 222:     /// like `set.extract_if(.., predicate)`.
 223:     ///
 224:     /// If the returned `ExtractIf` is not exhausted, e.g. because it is dropped without iterating
 225:     /// or the iteration short-circuits, then the remaining elements will be retained.
 226:     /// Use [`retain`] with a negated predicate if you do not need the returned iterator.
 227:     ///
 228:     /// [`retain`]: OrderSet::retain
 229:     ///
 230:     /// ***Panics*** if the starting point is greater than the end point or if
 231:     /// the end point is greater than the length of the set.
 232:     ///
 233:     /// # Examples
 234:     ///
 235:     /// Splitting a set into even and odd values, reusing the original set:
 236:     ///
 237:     /// ```
 238:     /// use ordermap::OrderSet;
 239:     ///
 240:     /// let mut set: OrderSet<i32> = (0..8).collect();
 241:     /// let extracted: OrderSet<i32> = set.extract_if(.., |v| v % 2 == 0).collect();
 242:     ///
 243:     /// let evens = extracted.into_iter().collect::<Vec<_>>();
 244:     /// let odds = set.into_iter().collect::<Vec<_>>();
 245:     ///
 246:     /// assert_eq!(evens, vec![0, 2, 4, 6]);
 247:     /// assert_eq!(odds, vec![1, 3, 5, 7]);
 248:     /// ```
 249:     #[track_caller]
 250:     pub fn extract_if<F, R>(&mut self, range: R, pred: F) -> ExtractIf<'_, T, F>
 251:     where
 252:         F: FnMut(&T) -> bool,
 253:         R: RangeBounds<usize>,
 254:     {
 255:         self.inner.extract_if(range, pred)
 256:     }
 257:     /// Splits the collection into two at the given index.
 258:     ///
 259:     /// Returns a newly allocated set containing the elements in the range
 260:     /// `[at, len)`. After the call, the original set will be left containing
 261:     /// the elements `[0, at)` with its previous capacity unchanged.
 262:     ///
 263:     /// ***Panics*** if `at > len`.
 264:     #[track_caller]
 265:     pub fn split_off(&mut self, at: usize) -> Self
 266:     where
 267:         S: Clone,
 268:     {
 269:         Self {
 270:             inner: self.inner.split_off(at),
 271:         }
 272:     }
 273:     /// Reserve capacity for `additional` more values.
 274:     ///
 275:     /// Computes in **O(n)** time.
 276:     pub fn reserve(&mut self, additional: usize) {
 277:         self.inner.reserve(additional);
 278:     }
 279:     /// Reserve capacity for `additional` more values, without over-allocating.
 280:     ///
 281:     /// Unlike `reserve`, this does not deliberately over-allocate the entry capacity to avoid
 282:     /// frequent re-allocations. However, the underlying data structures may still have internal
 283:     /// capacity requirements, and the allocator itself may give more space than requested, so this
 284:     /// cannot be relied upon to be precisely minimal.
 285:     ///
 286:     /// Computes in **O(n)** time.
 287:     pub fn reserve_exact(&mut self, additional: usize) {
 288:         self.inner.reserve_exact(additional);
 289:     }
 290:     /// Try to reserve capacity for `additional` more values.
 291:     ///
 292:     /// Computes in **O(n)** time.
 293:     pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
 294:         self.inner.try_reserve(additional)
 295:     }
 296:     /// Try to reserve capacity for `additional` more values, without over-allocating.
 297:     ///
 298:     /// Unlike `try_reserve`, this does not deliberately over-allocate the entry capacity to avoid
 299:     /// frequent re-allocations. However, the underlying data structures may still have internal
 300:     /// capacity requirements, and the allocator itself may give more space than requested, so this
 301:     /// cannot be relied upon to be precisely minimal.
 302:     ///
 303:     /// Computes in **O(n)** time.
 304:     pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
 305:         self.inner.try_reserve_exact(additional)
 306:     }
 307:     /// Shrink the capacity of the set as much as possible.
 308:     ///
 309:     /// Computes in **O(n)** time.
 310:     pub fn shrink_to_fit(&mut self) {
 311:         self.inner.shrink_to_fit();
 312:     }
 313:     /// Shrink the capacity of the set with a lower limit.
 314:     ///
 315:     /// Computes in **O(n)** time.
 316:     pub fn shrink_to(&mut self, min_capacity: usize) {
 317:         self.inner.shrink_to(min_capacity);
 318:     }
 319: }
 320: impl<T, S> OrderSet<T, S>
 321: where
 322:     T: Hash + Eq,
 323:     S: BuildHasher,
 324: {
 325:     /// Insert the value into the set.
 326:     ///
 327:     /// If an equivalent item already exists in the set, it returns
 328:     /// `false` leaving the original value in the set and without
 329:     /// altering its insertion order. Otherwise, it inserts the new
 330:     /// item and returns `true`.
 331:     ///
 332:     /// Computes in **O(1)** time (amortized average).
 333:     pub fn insert(&mut self, value: T) -> bool {
 334:         self.inner.insert(value)
 335:     }
 336:     /// Insert the value into the set, and get its index.
 337:     ///
 338:     /// If an equivalent item already exists in the set, it returns
 339:     /// the index of the existing item and `false`, leaving the
 340:     /// original value in the set and without altering its insertion
 341:     /// order. Otherwise, it inserts the new item and returns the index
 342:     /// of the inserted item and `true`.
 343:     ///
 344:     /// Computes in **O(1)** time (amortized average).
 345:     pub fn insert_full(&mut self, value: T) -> (usize, bool) {
 346:         self.inner.insert_full(value)
 347:     }
 348:     /// Insert the value into the set at its ordered position among sorted values.
 349:     ///
 350:     /// This is equivalent to finding the position with
 351:     /// [`binary_search`][Self::binary_search], and if needed calling
 352:     /// [`insert_before`][Self::insert_before] for a new value.
 353:     ///
 354:     /// If the sorted item is found in the set, it returns the index of that
 355:     /// existing item and `false`, without any change. Otherwise, it inserts the
 356:     /// new item and returns its sorted index and `true`.
 357:     ///
 358:     /// If the existing items are **not** already sorted, then the insertion
 359:     /// index is unspecified (like [`slice::binary_search`]), but the value
 360:     /// is moved to or inserted at that position regardless.
 361:     ///
 362:     /// Computes in **O(n)** time (average). Instead of repeating calls to
 363:     /// `insert_sorted`, it may be faster to call batched [`insert`][Self::insert]
 364:     /// or [`extend`][Self::extend] and only call [`sort`][Self::sort] or
 365:     /// [`sort_unstable`][Self::sort_unstable] once.
 366:     pub fn insert_sorted(&mut self, value: T) -> (usize, bool)
 367:     where
 368:         T: Ord,
 369:     {
 370:         self.inner.insert_sorted(value)
 371:     }
 372:     /// Insert the value into the set at its ordered position among values
 373:     /// sorted by `cmp`.
 374:     ///
 375:     /// This is equivalent to finding the position with
 376:     /// [`binary_search_by`][Self::binary_search_by], then calling
 377:     /// [`insert_before`][Self::insert_before].
 378:     ///
 379:     /// If the existing items are **not** already sorted, then the insertion
 380:     /// index is unspecified (like [`slice::binary_search`]), but the value
 381:     /// is moved to or inserted at that position regardless.
 382:     ///
 383:     /// Computes in **O(n)** time (average).
 384:     pub fn insert_sorted_by<F>(&mut self, value: T, cmp: F) -> (usize, bool)
 385:     where
 386:         F: FnMut(&T, &T) -> Ordering,
 387:     {
 388:         self.inner.insert_sorted_by(value, cmp)
 389:     }
 390:     /// Insert the value into the set at its ordered position among values
 391:     /// using a sort-key extraction function.
 392:     ///
 393:     /// This is equivalent to finding the position with
 394:     /// [`binary_search_by_key`][Self::binary_search_by_key] with `sort_key(key)`,
 395:     /// then calling [`insert_before`][Self::insert_before].
 396:     ///
 397:     /// If the existing items are **not** already sorted, then the insertion
 398:     /// index is unspecified (like [`slice::binary_search`]), but the value
 399:     /// is moved to or inserted at that position regardless.
 400:     ///
 401:     /// Computes in **O(n)** time (average).
 402:     pub fn insert_sorted_by_key<B, F>(&mut self, value: T, sort_key: F) -> (usize, bool)
 403:     where
 404:         B: Ord,
 405:         F: FnMut(&T) -> B,
 406:     {
 407:         self.inner.insert_sorted_by_key(value, sort_key)
 408:     }
 409:     /// Insert the value into the set before the value at the given index, or at the end.
 410:     ///
 411:     /// If an equivalent item already exists in the set, it returns `false` leaving the
 412:     /// original value in the set, but moved to the new position. The returned index
 413:     /// will either be the given index or one less, depending on how the value moved.
 414:     /// (See [`shift_insert`](Self::shift_insert) for different behavior here.)
 415:     ///
 416:     /// Otherwise, it inserts the new value exactly at the given index and returns `true`.
 417:     ///
 418:     /// ***Panics*** if `index` is out of bounds.
 419:     /// Valid indices are `0..=set.len()` (inclusive).
 420:     ///
 421:     /// Computes in **O(n)** time (average).
 422:     ///
 423:     /// # Examples
 424:     ///
 425:     /// ```
 426:     /// use ordermap::OrderSet;
 427:     /// let mut set: OrderSet<char> = ('a'..='z').collect();
 428:     ///
 429:     /// // The new value '*' goes exactly at the given index.
 430:     /// assert_eq!(set.get_index_of(&'*'), None);
 431:     /// assert_eq!(set.insert_before(10, '*'), (10, true));
 432:     /// assert_eq!(set.get_index_of(&'*'), Some(10));
 433:     ///
 434:     /// // Moving the value 'a' up will shift others down, so this moves *before* 10 to index 9.
 435:     /// assert_eq!(set.insert_before(10, 'a'), (9, false));
 436:     /// assert_eq!(set.get_index_of(&'a'), Some(9));
 437:     /// assert_eq!(set.get_index_of(&'*'), Some(10));
 438:     ///
 439:     /// // Moving the value 'z' down will shift others up, so this moves to exactly 10.
 440:     /// assert_eq!(set.insert_before(10, 'z'), (10, false));
 441:     /// assert_eq!(set.get_index_of(&'z'), Some(10));
 442:     /// assert_eq!(set.get_index_of(&'*'), Some(11));
 443:     ///
 444:     /// // Moving or inserting before the endpoint is also valid.
 445:     /// assert_eq!(set.len(), 27);
 446:     /// assert_eq!(set.insert_before(set.len(), '*'), (26, false));
 447:     /// assert_eq!(set.get_index_of(&'*'), Some(26));
 448:     /// assert_eq!(set.insert_before(set.len(), '+'), (27, true));
 449:     /// assert_eq!(set.get_index_of(&'+'), Some(27));
 450:     /// assert_eq!(set.len(), 28);
 451:     /// ```
 452:     #[track_caller]
 453:     pub fn insert_before(&mut self, index: usize, value: T) -> (usize, bool) {
 454:         self.inner.insert_before(index, value)
 455:     }
 456:     /// Insert the value into the set at the given index.
 457:     ///
 458:     /// If an equivalent item already exists in the set, it returns `false` leaving
 459:     /// the original value in the set, but moved to the given index.
 460:     /// Note that existing values **cannot** be moved to `index == set.len()`!
 461:     /// (See [`insert_before`](Self::insert_before) for different behavior here.)
 462:     ///
 463:     /// Otherwise, it inserts the new value at the given index and returns `true`.
 464:     ///
 465:     /// ***Panics*** if `index` is out of bounds.
 466:     /// Valid indices are `0..set.len()` (exclusive) when moving an existing value, or
 467:     /// `0..=set.len()` (inclusive) when inserting a new value.
 468:     ///
 469:     /// Computes in **O(n)** time (average).
 470:     ///
 471:     /// # Examples
 472:     ///
 473:     /// ```
 474:     /// use ordermap::OrderSet;
 475:     /// let mut set: OrderSet<char> = ('a'..='z').collect();
 476:     ///
 477:     /// // The new value '*' goes exactly at the given index.
 478:     /// assert_eq!(set.get_index_of(&'*'), None);
 479:     /// assert_eq!(set.shift_insert(10, '*'), true);
 480:     /// assert_eq!(set.get_index_of(&'*'), Some(10));
 481:     ///
 482:     /// // Moving the value 'a' up to 10 will shift others down, including the '*' that was at 10.
 483:     /// assert_eq!(set.shift_insert(10, 'a'), false);
 484:     /// assert_eq!(set.get_index_of(&'a'), Some(10));
 485:     /// assert_eq!(set.get_index_of(&'*'), Some(9));
 486:     ///
 487:     /// // Moving the value 'z' down to 9 will shift others up, including the '*' that was at 9.
 488:     /// assert_eq!(set.shift_insert(9, 'z'), false);
 489:     /// assert_eq!(set.get_index_of(&'z'), Some(9));
 490:     /// assert_eq!(set.get_index_of(&'*'), Some(10));
 491:     ///
 492:     /// // Existing values can move to len-1 at most, but new values can insert at the endpoint.
 493:     /// assert_eq!(set.len(), 27);
 494:     /// assert_eq!(set.shift_insert(set.len() - 1, '*'), false);
 495:     /// assert_eq!(set.get_index_of(&'*'), Some(26));
 496:     /// assert_eq!(set.shift_insert(set.len(), '+'), true);
 497:     /// assert_eq!(set.get_index_of(&'+'), Some(27));
 498:     /// assert_eq!(set.len(), 28);
 499:     /// ```
 500:     ///
 501:     /// ```should_panic
 502:     /// use ordermap::OrderSet;
 503:     /// let mut set: OrderSet<char> = ('a'..='z').collect();
 504:     ///
 505:     /// // This is an invalid index for moving an existing value!
 506:     /// set.shift_insert(set.len(), 'a');
 507:     /// ```
 508:     #[track_caller]
 509:     pub fn shift_insert(&mut self, index: usize, value: T) -> bool {
 510:         self.inner.shift_insert(index, value)
 511:     }
 512:     /// Adds a value to the set, replacing the existing value, if any, that is
 513:     /// equal to the given one, without altering its insertion order. Returns
 514:     /// the replaced value.
 515:     ///
 516:     /// Computes in **O(1)** time (average).
 517:     pub fn replace(&mut self, value: T) -> Option<T> {
 518:         self.inner.replace(value)
 519:     }
 520:     /// Adds a value to the set, replacing the existing value, if any, that is
 521:     /// equal to the given one, without altering its insertion order. Returns
 522:     /// the index of the item and its replaced value.
 523:     ///
 524:     /// Computes in **O(1)** time (average).
 525:     pub fn replace_full(&mut self, value: T) -> (usize, Option<T>) {
 526:         self.inner.replace_full(value)
 527:     }
 528:     /// Replaces the value at the given index. The new value does not need to be
 529:     /// equivalent to the one it is replacing, but it must be unique to the rest
 530:     /// of the set.
 531:     ///
 532:     /// Returns `Ok(old_value)` if successful, or `Err((other_index, value))` if
 533:     /// an equivalent value already exists at a different index. The set will be
 534:     /// unchanged in the error case.
 535:     ///
 536:     /// ***Panics*** if `index` is out of bounds.
 537:     ///
 538:     /// Computes in **O(1)** time (average).
 539:     #[track_caller]
 540:     pub fn replace_index(&mut self, index: usize, value: T) -> Result<T, (usize, T)> {
 541:         self.inner.replace_index(index, value)
 542:     }
 543:     /// Return an iterator over the values that are in `self` but not `other`.
 544:     ///
 545:     /// Values are produced in the same order that they appear in `self`.
 546:     pub fn difference<'a, S2>(&'a self, other: &'a OrderSet<T, S2>) -> Difference<'a, T, S2>
 547:     where
 548:         S2: BuildHasher,
 549:     {
 550:         self.inner.difference(&other.inner)
 551:     }
 552:     /// Return an iterator over the values that are in `self` or `other`,
 553:     /// but not in both.
 554:     ///
 555:     /// Values from `self` are produced in their original order, followed by
 556:     /// values from `other` in their original order.
 557:     pub fn symmetric_difference<'a, S2>(
 558:         &'a self,
 559:         other: &'a OrderSet<T, S2>,
 560:     ) -> SymmetricDifference<'a, T, S, S2>
 561:     where
 562:         S2: BuildHasher,
 563:     {
 564:         self.inner.symmetric_difference(&other.inner)
 565:     }
 566:     /// Return an iterator over the values that are in both `self` and `other`.
 567:     ///
 568:     /// Values are produced in the same order that they appear in `self`.
 569:     pub fn intersection<'a, S2>(&'a self, other: &'a OrderSet<T, S2>) -> Intersection<'a, T, S2>
 570:     where
 571:         S2: BuildHasher,
 572:     {
 573:         self.inner.intersection(&other.inner)
 574:     }
 575:     /// Return an iterator over all values that are in `self` or `other`.
 576:     ///
 577:     /// Values from `self` are produced in their original order, followed by
 578:     /// values that are unique to `other` in their original order.
 579:     pub fn union<'a, S2>(&'a self, other: &'a OrderSet<T, S2>) -> Union<'a, T, S>
 580:     where
 581:         S2: BuildHasher,
 582:     {
 583:         self.inner.union(&other.inner)
 584:     }
 585:     /// Creates a splicing iterator that replaces the specified range in the set
 586:     /// with the given `replace_with` iterator and yields the removed items.
 587:     /// `replace_with` does not need to be the same length as `range`.
 588:     ///
 589:     /// The `range` is removed even if the iterator is not consumed until the
 590:     /// end. It is unspecified how many elements are removed from the set if the
 591:     /// `Splice` value is leaked.
 592:     ///
 593:     /// The input iterator `replace_with` is only consumed when the `Splice`
 594:     /// value is dropped. If a value from the iterator matches an existing entry
 595:     /// in the set (outside of `range`), then the original will be unchanged.
 596:     /// Otherwise, the new value will be inserted in the replaced `range`.
 597:     ///
 598:     /// ***Panics*** if the starting point is greater than the end point or if
 599:     /// the end point is greater than the length of the set.
 600:     ///
 601:     /// # Examples
 602:     ///
 603:     /// ```
 604:     /// use ordermap::OrderSet;
 605:     ///
 606:     /// let mut set = OrderSet::from([0, 1, 2, 3, 4]);
 607:     /// let new = [5, 4, 3, 2, 1];
 608:     /// let removed: Vec<_> = set.splice(2..4, new).collect();
 609:     ///
 610:     /// // 1 and 4 kept their positions, while 5, 3, and 2 were newly inserted.
 611:     /// assert!(set.into_iter().eq([0, 1, 5, 3, 2, 4]));
 612:     /// assert_eq!(removed, &[2, 3]);
 613:     /// ```
 614:     #[track_caller]
 615:     pub fn splice<R, I>(&mut self, range: R, replace_with: I) -> Splice<'_, I::IntoIter, T, S>
 616:     where
 617:         R: RangeBounds<usize>,
 618:         I: IntoIterator<Item = T>,
 619:     {
 620:         self.inner.splice(range, replace_with)
 621:     }
 622:     /// Moves all values from `other` into `self`, leaving `other` empty.
 623:     ///
 624:     /// This is equivalent to calling [`insert`][Self::insert] for each value
 625:     /// from `other` in order, which means that values that already exist
 626:     /// in `self` are unchanged in their current position.
 627:     ///
 628:     /// See also [`union`][Self::union] to iterate the combined values by
 629:     /// reference, without modifying `self` or `other`.
 630:     ///
 631:     /// # Examples
 632:     ///
 633:     /// ```
 634:     /// use ordermap::OrderSet;
 635:     ///
 636:     /// let mut a = OrderSet::from([3, 2, 1]);
 637:     /// let mut b = OrderSet::from([3, 4, 5]);
 638:     /// let old_capacity = b.capacity();
 639:     ///
 640:     /// a.append(&mut b);
 641:     ///
 642:     /// assert_eq!(a.len(), 5);
 643:     /// assert_eq!(b.len(), 0);
 644:     /// assert_eq!(b.capacity(), old_capacity);
 645:     ///
 646:     /// assert!(a.iter().eq(&[3, 2, 1, 4, 5]));
 647:     /// ```
 648:     pub fn append<S2>(&mut self, other: &mut OrderSet<T, S2>) {
 649:         self.inner.append(&mut other.inner);
 650:     }
 651: }
 652: impl<T, S> OrderSet<T, S>
 653: where
 654:     S: BuildHasher,
 655: {
 656:     /// Return `true` if an equivalent to `value` exists in the set.
 657:     ///
 658:     /// Computes in **O(1)** time (average).
 659:     pub fn contains<Q>(&self, value: &Q) -> bool
 660:     where
 661:         Q: ?Sized + Hash + Equivalent<T>,
 662:     {
 663:         self.inner.contains(value)
 664:     }
 665:     /// Return a reference to the value stored in the set, if it is present,
 666:     /// else `None`.
 667:     ///
 668:     /// Computes in **O(1)** time (average).
 669:     pub fn get<Q>(&self, value: &Q) -> Option<&T>
 670:     where
 671:         Q: ?Sized + Hash + Equivalent<T>,
 672:     {
 673:         self.inner.get(value)
 674:     }
 675:     /// Return item index and value
 676:     pub fn get_full<Q>(&self, value: &Q) -> Option<(usize, &T)>
 677:     where
 678:         Q: ?Sized + Hash + Equivalent<T>,
 679:     {
 680:         self.inner.get_full(value)
 681:     }
 682:     /// Return item index, if it exists in the set
 683:     ///
 684:     /// Computes in **O(1)** time (average).
 685:     pub fn get_index_of<Q>(&self, value: &Q) -> Option<usize>
 686:     where
 687:         Q: ?Sized + Hash + Equivalent<T>,
 688:     {
 689:         self.inner.get_index_of(value)
 690:     }
 691:     /// Remove the value from the set, and return `true` if it was present.
 692:     ///
 693:     /// **NOTE:** This is equivalent to [`IndexSet::shift_remove`], and
 694:     /// like [`Vec::remove`], the value is removed by shifting all of the
 695:     /// elements that follow it, preserving their relative order.
 696:     /// **This perturbs the index of all of those elements!**
 697:     ///
 698:     /// Return `false` if `value` was not in the set.
 699:     ///
 700:     /// Computes in **O(n)** time (average).
 701:     pub fn remove<Q>(&mut self, value: &Q) -> bool
 702:     where
 703:         Q: ?Sized + Hash + Equivalent<T>,
 704:     {
 705:         self.inner.shift_remove(value)
 706:     }
 707:     /// Remove the value from the set, and return `true` if it was present.
 708:     ///
 709:     /// Like [`Vec::swap_remove`], the value is removed by swapping it with the
 710:     /// last element of the set and popping it off. **This perturbs
 711:     /// the position of what used to be the last element!**
 712:     ///
 713:     /// Return `false` if `value` was not in the set.
 714:     ///
 715:     /// Computes in **O(1)** time (average).
 716:     pub fn swap_remove<Q>(&mut self, value: &Q) -> bool
 717:     where
 718:         Q: ?Sized + Hash + Equivalent<T>,
 719:     {
 720:         self.inner.swap_remove(value)
 721:     }
 722:     /// Removes and returns the value in the set, if any, that is equal to the
 723:     /// given one.
 724:     ///
 725:     /// **NOTE:** This is equivalent to [`IndexSet::shift_take`], and
 726:     /// like [`Vec::remove`], the value is removed by shifting all of the
 727:     /// elements that follow it, preserving their relative order.
 728:     /// **This perturbs the index of all of those elements!**
 729:     ///
 730:     /// Return `None` if `value` was not in the set.
 731:     ///
 732:     /// Computes in **O(n)** time (average).
 733:     pub fn take<Q>(&mut self, value: &Q) -> Option<T>
 734:     where
 735:         Q: ?Sized + Hash + Equivalent<T>,
 736:     {
 737:         self.inner.shift_take(value)
 738:     }
 739:     /// Removes and returns the value in the set, if any, that is equal to the
 740:     /// given one.
 741:     ///
 742:     /// Like [`Vec::swap_remove`], the value is removed by swapping it with the
 743:     /// last element of the set and popping it off. **This perturbs
 744:     /// the position of what used to be the last element!**
 745:     ///
 746:     /// Return `None` if `value` was not in the set.
 747:     ///
 748:     /// Computes in **O(1)** time (average).
 749:     pub fn swap_take<Q>(&mut self, value: &Q) -> Option<T>
 750:     where
 751:         Q: ?Sized + Hash + Equivalent<T>,
 752:     {
 753:         self.inner.swap_take(value)
 754:     }
 755:     /// Remove the value from the set return it and the index it had.
 756:     ///
 757:     /// **NOTE:** This is equivalent to [`IndexSet::shift_remove_full`], and
 758:     /// like [`Vec::remove`], the value is removed by shifting all of the
 759:     /// elements that follow it, preserving their relative order.
 760:     /// **This perturbs the index of all of those elements!**
 761:     ///
 762:     /// Return `None` if `value` was not in the set.
 763:     pub fn remove_full<Q>(&mut self, value: &Q) -> Option<(usize, T)>
 764:     where
 765:         Q: ?Sized + Hash + Equivalent<T>,
 766:     {
 767:         self.inner.shift_remove_full(value)
 768:     }
 769:     /// Remove the value from the set return it and the index it had.
 770:     ///
 771:     /// Like [`Vec::swap_remove`], the value is removed by swapping it with the
 772:     /// last element of the set and popping it off. **This perturbs
 773:     /// the position of what used to be the last element!**
 774:     ///
 775:     /// Return `None` if `value` was not in the set.
 776:     pub fn swap_remove_full<Q>(&mut self, value: &Q) -> Option<(usize, T)>
 777:     where
 778:         Q: ?Sized + Hash + Equivalent<T>,
 779:     {
 780:         self.inner.swap_remove_full(value)
 781:     }
 782: }
 783: impl<T, S> OrderSet<T, S> {
 784:     /// Remove the last value
 785:     ///
 786:     /// This preserves the order of the remaining elements.
 787:     ///
 788:     /// Computes in **O(1)** time (average).
 789:     #[doc(alias = "pop_last")] // like `BTreeSet`
 790:     pub fn pop(&mut self) -> Option<T> {
 791:         self.inner.pop()
 792:     }
 793:     /// Removes and returns the last value from a set if the predicate
 794:     /// returns `true`, or [`None`] if the predicate returns false or the set
 795:     /// is empty (the predicate will not be called in that case).
 796:     ///
 797:     /// This preserves the order of the remaining elements.
 798:     ///
 799:     /// Computes in **O(1)** time (average).
 800:     ///
 801:     /// # Examples
 802:     ///
 803:     /// ```
 804:     /// use ordermap::OrderSet;
 805:     ///
 806:     /// let mut set = OrderSet::from([1, 2, 3, 4]);
 807:     /// let pred = |x: &i32| *x % 2 == 0;
 808:     ///
 809:     /// assert_eq!(set.pop_if(pred), Some(4));
 810:     /// assert_eq!(set.as_slice(), &[1, 2, 3]);
 811:     /// assert_eq!(set.pop_if(pred), None);
 812:     /// ```
 813:     pub fn pop_if(&mut self, predicate: impl FnOnce(&T) -> bool) -> Option<T> {
 814:         self.inner.pop_if(predicate)
 815:     }
 816:     /// Scan through each value in the set and keep those where the
 817:     /// closure `keep` returns `true`.
 818:     ///
 819:     /// The elements are visited in order, and remaining elements keep their
 820:     /// order.
 821:     ///
 822:     /// Computes in **O(n)** time (average).
 823:     pub fn retain<F>(&mut self, keep: F)
 824:     where
 825:         F: FnMut(&T) -> bool,
 826:     {
 827:         self.inner.retain(keep)
 828:     }
 829:     /// Sort the set's values by their default ordering.
 830:     ///
 831:     /// This is a stable sort -- but equivalent values should not normally coexist in
 832:     /// a set at all, so [`sort_unstable`][Self::sort_unstable] is preferred
 833:     /// because it is generally faster and doesn't allocate auxiliary memory.
 834:     ///
 835:     /// See [`sort_by`](Self::sort_by) for details.
 836:     pub fn sort(&mut self)
 837:     where
 838:         T: Ord,
 839:     {
 840:         self.inner.sort()
 841:     }
 842:     /// Sort the set's values in place using the comparison function `cmp`.
 843:     ///
 844:     /// Computes in **O(n log n)** time and **O(n)** space. The sort is stable.
 845:     pub fn sort_by<F>(&mut self, cmp: F)
 846:     where
 847:         F: FnMut(&T, &T) -> Ordering,
 848:     {
 849:         self.inner.sort_by(cmp);
 850:     }
 851:     /// Sort the values of the set and return a by-value iterator of
 852:     /// the values with the result.
 853:     ///
 854:     /// The sort is stable.
 855:     pub fn sorted_by<F>(self, cmp: F) -> IntoIter<T>
 856:     where
 857:         F: FnMut(&T, &T) -> Ordering,
 858:     {
 859:         self.inner.sorted_by(cmp)
 860:     }
 861:     /// Sort the set's values in place using a key extraction function.
 862:     ///
 863:     /// Computes in **O(n log n)** time and **O(n)** space. The sort is stable.
 864:     pub fn sort_by_key<K, F>(&mut self, sort_key: F)
 865:     where
 866:         K: Ord,
 867:         F: FnMut(&T) -> K,
 868:     {
 869:         self.inner.sort_by_key(sort_key)
 870:     }
 871:     /// Sort the set's values by their default ordering.
 872:     ///
 873:     /// See [`sort_unstable_by`](Self::sort_unstable_by) for details.
 874:     pub fn sort_unstable(&mut self)
 875:     where
 876:         T: Ord,
 877:     {
 878:         self.inner.sort_unstable()
 879:     }
 880:     /// Sort the set's values in place using the comparison function `cmp`.
 881:     ///
 882:     /// Computes in **O(n log n)** time. The sort is unstable.
 883:     pub fn sort_unstable_by<F>(&mut self, cmp: F)
 884:     where
 885:         F: FnMut(&T, &T) -> Ordering,
 886:     {
 887:         self.inner.sort_unstable_by(cmp)
 888:     }
 889:     /// Sort the values of the set and return a by-value iterator of
 890:     /// the values with the result.
 891:     pub fn sorted_unstable_by<F>(self, cmp: F) -> IntoIter<T>
 892:     where
 893:         F: FnMut(&T, &T) -> Ordering,
 894:     {
 895:         self.inner.sorted_unstable_by(cmp)
 896:     }
 897:     /// Sort the set's values in place using a key extraction function.
 898:     ///
 899:     /// Computes in **O(n log n)** time. The sort is unstable.
 900:     pub fn sort_unstable_by_key<K, F>(&mut self, sort_key: F)
 901:     where
 902:         K: Ord,
 903:         F: FnMut(&T) -> K,
 904:     {
 905:         self.inner.sort_unstable_by_key(sort_key)
 906:     }
 907:     /// Sort the set's values in place using a key extraction function.
 908:     ///
 909:     /// During sorting, the function is called at most once per entry, by using temporary storage
 910:     /// to remember the results of its evaluation. The order of calls to the function is
 911:     /// unspecified and may change between versions of `ordermap` or the standard library.
 912:     ///
 913:     /// Computes in **O(m n + n log n + c)** time () and **O(n)** space, where the function is
 914:     /// **O(m)**, *n* is the length of the map, and *c* the capacity. The sort is stable.
 915:     pub fn sort_by_cached_key<K, F>(&mut self, sort_key: F)
 916:     where
 917:         K: Ord,
 918:         F: FnMut(&T) -> K,
 919:     {
 920:         self.inner.sort_by_cached_key(sort_key)
 921:     }
 922:     /// Search over a sorted set for a value.
 923:     ///
 924:     /// Returns the position where that value is present, or the position where it can be inserted
 925:     /// to maintain the sort. See [`slice::binary_search`] for more details.
 926:     ///
 927:     /// Computes in **O(log(n))** time, which is notably less scalable than looking the value up
 928:     /// using [`get_index_of`][OrderSet::get_index_of], but this can also position missing values.
 929:     pub fn binary_search(&self, x: &T) -> Result<usize, usize>
 930:     where
 931:         T: Ord,
 932:     {
 933:         self.inner.binary_search(x)
 934:     }
 935:     /// Search over a sorted set with a comparator function.
 936:     ///
 937:     /// Returns the position where that value is present, or the position where it can be inserted
 938:     /// to maintain the sort. See [`slice::binary_search_by`] for more details.
 939:     ///
 940:     /// Computes in **O(log(n))** time.
 941:     #[inline]
 942:     pub fn binary_search_by<'a, F>(&'a self, f: F) -> Result<usize, usize>
 943:     where
 944:         F: FnMut(&'a T) -> Ordering,
 945:     {
 946:         self.inner.binary_search_by(f)
 947:     }
 948:     /// Search over a sorted set with an extraction function.
 949:     ///
 950:     /// Returns the position where that value is present, or the position where it can be inserted
 951:     /// to maintain the sort. See [`slice::binary_search_by_key`] for more details.
 952:     ///
 953:     /// Computes in **O(log(n))** time.
 954:     #[inline]
 955:     pub fn binary_search_by_key<'a, B, F>(&'a self, b: &B, f: F) -> Result<usize, usize>
 956:     where
 957:         F: FnMut(&'a T) -> B,
 958:         B: Ord,
 959:     {
 960:         self.inner.binary_search_by_key(b, f)
 961:     }
 962:     /// Checks if the values of this set are sorted.
 963:     #[inline]
 964:     pub fn is_sorted(&self) -> bool
 965:     where
 966:         T: PartialOrd,
 967:     {
 968:         self.inner.is_sorted()
 969:     }
 970:     /// Checks if this set is sorted using the given comparator function.
 971:     #[inline]
 972:     pub fn is_sorted_by<'a, F>(&'a self, cmp: F) -> bool
 973:     where
 974:         F: FnMut(&'a T, &'a T) -> bool,
 975:     {
 976:         self.inner.is_sorted_by(cmp)
 977:     }
 978:     /// Checks if this set is sorted using the given sort-key function.
 979:     #[inline]
 980:     pub fn is_sorted_by_key<'a, F, K>(&'a self, sort_key: F) -> bool
 981:     where
 982:         F: FnMut(&'a T) -> K,
 983:         K: PartialOrd,
 984:     {
 985:         self.inner.is_sorted_by_key(sort_key)
 986:     }
 987:     /// Returns the index of the partition point of a sorted set according to the given predicate
 988:     /// (the index of the first element of the second partition).
 989:     ///
 990:     /// See [`slice::partition_point`] for more details.
 991:     ///
 992:     /// Computes in **O(log(n))** time.
 993:     #[must_use]
 994:     pub fn partition_point<P>(&self, pred: P) -> usize
 995:     where
 996:         P: FnMut(&T) -> bool,
 997:     {
 998:         self.inner.partition_point(pred)
 999:     }
1000:     /// Reverses the order of the set's values in place.
1001:     ///
1002:     /// Computes in **O(n)** time and **O(1)** space.
1003:     pub fn reverse(&mut self) {
1004:         self.inner.reverse()
1005:     }
1006:     /// Returns a slice of all the values in the set.
1007:     ///
1008:     /// Computes in **O(1)** time.
1009:     pub fn as_slice(&self) -> &Slice<T> {
1010:         self.inner.as_slice()
1011:     }
1012:     /// Converts into a boxed slice of all the values in the set.
1013:     ///
1014:     /// Note that this will drop the inner hash table and any excess capacity.
1015:     pub fn into_boxed_slice(self) -> Box<Slice<T>> {
1016:         self.inner.into_boxed_slice()
1017:     }
1018:     /// Get a value by index
1019:     ///
1020:     /// Valid indices are `0 <= index < self.len()`.
1021:     ///
1022:     /// Computes in **O(1)** time.
1023:     pub fn get_index(&self, index: usize) -> Option<&T> {
1024:         self.inner.get_index(index)
1025:     }
1026:     /// Returns a slice of values in the given range of indices.
1027:     ///
1028:     /// Valid indices are `0 <= index < self.len()`.
1029:     ///
1030:     /// Computes in **O(1)** time.
1031:     pub fn get_range<R: RangeBounds<usize>>(&self, range: R) -> Option<&Slice<T>> {
1032:         self.inner.get_range(range)
1033:     }
1034:     /// Get the first value
1035:     ///
1036:     /// Computes in **O(1)** time.
1037:     pub fn first(&self) -> Option<&T> {
1038:         self.inner.first()
1039:     }
1040:     /// Get the last value
1041:     ///
1042:     /// Computes in **O(1)** time.
1043:     pub fn last(&self) -> Option<&T> {
1044:         self.inner.last()
1045:     }
1046:     /// Remove the value by index
1047:     ///
1048:     /// Valid indices are `0 <= index < self.len()`
1049:     ///
1050:     /// **NOTE:** This is equivalent to [`IndexSet::shift_remove_index`], and
1051:     /// like [`Vec::remove`], the value is removed by shifting all of the
1052:     /// elements that follow it, preserving their relative order.
1053:     /// **This perturbs the index of all of those elements!**
1054:     ///
1055:     /// Computes in **O(n)** time (average).
1056:     pub fn remove_index(&mut self, index: usize) -> Option<T> {
1057:         self.inner.shift_remove_index(index)
1058:     }
1059:     /// Remove the value by index
1060:     ///
1061:     /// Valid indices are `0 <= index < self.len()`.
1062:     ///
1063:     /// Like [`Vec::swap_remove`], the value is removed by swapping it with the
1064:     /// last element of the set and popping it off. **This perturbs
1065:     /// the position of what used to be the last element!**
1066:     ///
1067:     /// Computes in **O(1)** time (average).
1068:     pub fn swap_remove_index(&mut self, index: usize) -> Option<T> {
1069:         self.inner.swap_remove_index(index)
1070:     }
1071:     /// Moves the position of a value from one index to another
1072:     /// by shifting all other values in-between.
1073:     ///
1074:     /// * If `from < to`, the other values will shift down while the targeted value moves up.
1075:     /// * If `from > to`, the other values will shift up while the targeted value moves down.
1076:     ///
1077:     /// ***Panics*** if `from` or `to` are out of bounds.
1078:     ///
1079:     /// Computes in **O(n)** time (average).
1080:     #[track_caller]
1081:     pub fn move_index(&mut self, from: usize, to: usize) {
1082:         self.inner.move_index(from, to)
1083:     }
1084:     /// Swaps the position of two values in the set.
1085:     ///
1086:     /// ***Panics*** if `a` or `b` are out of bounds.
1087:     ///
1088:     /// Computes in **O(1)** time (average).
1089:     #[track_caller]
1090:     pub fn swap_indices(&mut self, a: usize, b: usize) {
1091:         self.inner.swap_indices(a, b)
1092:     }
1093: }
1094: /// Access [`OrderSet`] values at indexed positions.
1095: ///
1096: /// # Examples
1097: ///
1098: /// ```
1099: /// use ordermap::OrderSet;
1100: ///
1101: /// let mut set = OrderSet::new();
1102: /// for word in "Lorem ipsum dolor sit amet".split_whitespace() {
1103: ///     set.insert(word.to_string());
1104: /// }
1105: /// assert_eq!(set[0], "Lorem");
1106: /// assert_eq!(set[1], "ipsum");
1107: /// set.reverse();
1108: /// assert_eq!(set[0], "amet");
1109: /// assert_eq!(set[1], "sit");
1110: /// set.sort();
1111: /// assert_eq!(set[0], "Lorem");
1112: /// assert_eq!(set[1], "amet");
1113: /// ```
1114: ///
1115: /// ```should_panic
1116: /// use ordermap::OrderSet;
1117: ///
1118: /// let mut set = OrderSet::new();
1119: /// set.insert("foo");
1120: /// println!("{:?}", set[10]); // panics!
1121: /// ```
1122: impl<T, S> Index<usize> for OrderSet<T, S> {
1123:     type Output = T;
1124:     /// Returns a reference to the value at the supplied `index`.
1125:     ///
1126:     /// ***Panics*** if `index` is out of bounds.
1127:     fn index(&self, index: usize) -> &T {
1128:         &self.inner[index]
1129:     }
1130: }
1131: impl<T, S> FromIterator<T> for OrderSet<T, S>
1132: where
1133:     T: Hash + Eq,
1134:     S: BuildHasher + Default,
1135: {
1136:     fn from_iter<I: IntoIterator<Item = T>>(iterable: I) -> Self {
1137:         Self {
1138:             inner: IndexSet::from_iter(iterable),
1139:         }
1140:     }
1141: }
1142: #[cfg(feature = "std")]
1143: #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
1144: impl<T, const N: usize> From<[T; N]> for OrderSet<T, RandomState>
1145: where
1146:     T: Eq + Hash,
1147: {
1148:     /// # Examples
1149:     ///
1150:     /// ```
1151:     /// use ordermap::OrderSet;
1152:     ///
1153:     /// let set1 = OrderSet::from([1, 2, 3, 4]);
1154:     /// let set2: OrderSet<_> = [1, 2, 3, 4].into();
1155:     /// assert_eq!(set1, set2);
1156:     /// ```
1157:     fn from(arr: [T; N]) -> Self {
1158:         Self::from_iter(arr)
1159:     }
1160: }
1161: impl<T, S> Extend<T> for OrderSet<T, S>
1162: where
1163:     T: Hash + Eq,
1164:     S: BuildHasher,
1165: {
1166:     fn extend<I: IntoIterator<Item = T>>(&mut self, iterable: I) {
1167:         self.inner.extend(iterable);
1168:     }
1169: }
1170: impl<'a, T, S> Extend<&'a T> for OrderSet<T, S>
1171: where
1172:     T: Hash + Eq + Copy + 'a,
1173:     S: BuildHasher,
1174: {
1175:     fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iterable: I) {
1176:         self.inner.extend(iterable);
1177:     }
1178: }
1179: impl<T, S> Default for OrderSet<T, S>
1180: where
1181:     S: Default,
1182: {
1183:     /// Return an empty [`OrderSet`]
1184:     fn default() -> Self {
1185:         OrderSet {
1186:             inner: IndexSet::default(),
1187:         }
1188:     }
1189: }
1190: impl<T, S1, S2> PartialEq<OrderSet<T, S2>> for OrderSet<T, S1>
1191: where
1192:     T: PartialEq,
1193: {
1194:     fn eq(&self, other: &OrderSet<T, S2>) -> bool {
1195:         self.len() == other.len() && self.iter().eq(other)
1196:     }
1197: }
1198: impl<T, S> Eq for OrderSet<T, S> where T: Eq {}
1199: impl<T, S1, S2> PartialOrd<OrderSet<T, S2>> for OrderSet<T, S1>
1200: where
1201:     T: PartialOrd,
1202: {
1203:     fn partial_cmp(&self, other: &OrderSet<T, S2>) -> Option<Ordering> {
1204:         self.iter().partial_cmp(other)
1205:     }
1206: }
1207: impl<T, S> Ord for OrderSet<T, S>
1208: where
1209:     T: Ord,
1210: {
1211:     fn cmp(&self, other: &Self) -> Ordering {
1212:         self.iter().cmp(other)
1213:     }
1214: }
1215: impl<T, S> Hash for OrderSet<T, S>
1216: where
1217:     T: Hash,
1218: {
1219:     fn hash<H: Hasher>(&self, state: &mut H) {
1220:         self.len().hash(state);
1221:         for value in self {
1222:             value.hash(state);
1223:         }
1224:     }
1225: }
1226: impl<T, S> OrderSet<T, S>
1227: where
1228:     T: Eq + Hash,
1229:     S: BuildHasher,
1230: {
1231:     /// Returns `true` if `self` has no elements in common with `other`.
1232:     pub fn is_disjoint<S2>(&self, other: &OrderSet<T, S2>) -> bool
1233:     where
1234:         S2: BuildHasher,
1235:     {
1236:         self.inner.is_disjoint(&other.inner)
1237:     }
1238:     /// Returns `true` if all elements of `self` are contained in `other`.
1239:     pub fn is_subset<S2>(&self, other: &OrderSet<T, S2>) -> bool
1240:     where
1241:         S2: BuildHasher,
1242:     {
1243:         self.inner.is_subset(&other.inner)
1244:     }
1245:     /// Returns `true` if all elements of `other` are contained in `self`.
1246:     pub fn is_superset<S2>(&self, other: &OrderSet<T, S2>) -> bool
1247:     where
1248:         S2: BuildHasher,
1249:     {
1250:         self.inner.is_superset(&other.inner)
1251:     }
1252:     /// Returns `true` if `self` and `other` contain exactly the same elements,
1253:     /// even if they are not in the same order.
1254:     ///
1255:     /// (Note that `PartialEq for OrderSet` **does** consider the order.)
1256:     pub fn set_eq<S2>(&self, other: &OrderSet<T, S2>) -> bool
1257:     where
1258:         S2: BuildHasher,
1259:     {
1260:         self.inner == other.inner
1261:     }
1262: }
1263: impl<T, S1, S2> BitAnd<&OrderSet<T, S2>> for &OrderSet<T, S1>
1264: where
1265:     T: Eq + Hash + Clone,
1266:     S1: BuildHasher + Default,
1267:     S2: BuildHasher,
1268: {
1269:     type Output = OrderSet<T, S1>;
1270:     /// Returns the set intersection, cloned into a new set.
1271:     ///
1272:     /// Values are collected in the same order that they appear in `self`.
1273:     fn bitand(self, other: &OrderSet<T, S2>) -> Self::Output {
1274:         OrderSet {
1275:             inner: &self.inner & &other.inner,
1276:         }
1277:     }
1278: }
1279: impl<T, S1, S2> BitOr<&OrderSet<T, S2>> for &OrderSet<T, S1>
1280: where
1281:     T: Eq + Hash + Clone,
1282:     S1: BuildHasher + Default,
1283:     S2: BuildHasher,
1284: {
1285:     type Output = OrderSet<T, S1>;
1286:     /// Returns the set union, cloned into a new set.
1287:     ///
1288:     /// Values from `self` are collected in their original order, followed by
1289:     /// values that are unique to `other` in their original order.
1290:     fn bitor(self, other: &OrderSet<T, S2>) -> Self::Output {
1291:         OrderSet {
1292:             inner: &self.inner | &other.inner,
1293:         }
1294:     }
1295: }
1296: impl<T, S1, S2> BitXor<&OrderSet<T, S2>> for &OrderSet<T, S1>
1297: where
1298:     T: Eq + Hash + Clone,
1299:     S1: BuildHasher + Default,
1300:     S2: BuildHasher,
1301: {
1302:     type Output = OrderSet<T, S1>;
1303:     /// Returns the set symmetric-difference, cloned into a new set.
1304:     ///
1305:     /// Values from `self` are collected in their original order, followed by
1306:     /// values from `other` in their original order.
1307:     fn bitxor(self, other: &OrderSet<T, S2>) -> Self::Output {
1308:         OrderSet {
1309:             inner: &self.inner ^ &other.inner,
1310:         }
1311:     }
1312: }
1313: impl<T, S1, S2> Sub<&OrderSet<T, S2>> for &OrderSet<T, S1>
1314: where
1315:     T: Eq + Hash + Clone,
1316:     S1: BuildHasher + Default,
1317:     S2: BuildHasher,
1318: {
1319:     type Output = OrderSet<T, S1>;
1320:     /// Returns the set difference, cloned into a new set.
1321:     ///
1322:     /// Values are collected in the same order that they appear in `self`.
1323:     fn sub(self, other: &OrderSet<T, S2>) -> Self::Output {
1324:         OrderSet {
1325:             inner: &self.inner - &other.inner,
1326:         }
1327:     }
1328: }
````

## File: src/sval.rs
````rust
 1: #![cfg_attr(docsrs, doc(cfg(feature = "sval")))]
 2: use crate::{OrderMap, OrderSet};
 3: use sval::{Stream, Value};
 4: impl<K: Value, V: Value, S> Value for OrderMap<K, V, S> {
 5:     fn stream<'sval, ST: Stream<'sval> + ?Sized>(&'sval self, stream: &mut ST) -> sval::Result {
 6:         self.inner.stream(stream)
 7:     }
 8: }
 9: impl<K: Value, S> Value for OrderSet<K, S> {
10:     fn stream<'sval, ST: Stream<'sval> + ?Sized>(&'sval self, stream: &mut ST) -> sval::Result {
11:         self.inner.stream(stream)
12:     }
13: }
````
