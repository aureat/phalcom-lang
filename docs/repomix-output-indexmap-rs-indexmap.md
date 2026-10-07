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
  inner/
    entry.rs
    extract.rs
  map/
    disjoint.rs
    entry.rs
    iter.rs
    mutable.rs
    raw_entry_v1.rs
    serde_seq.rs
    slice.rs
    tests.rs
  rayon/
    map.rs
    mod.rs
    set.rs
  set/
    iter.rs
    mutable.rs
    slice.rs
    tests.rs
  arbitrary.rs
  borsh.rs
  inner.rs
  lib.rs
  macros.rs
  map.rs
  serde.rs
  set.rs
  sval.rs
  util.rs
````

# Files

## File: src/inner/entry.rs
````rust
  1: use super::{Bucket, Core, equal, get_hash};
  2: use crate::HashValue;
  3: use crate::map::{Entry, IndexedEntry};
  4: use crate::util::assert_index_lt;
  5: use core::cmp::Ordering;
  6: use core::mem;
  7: impl<'a, K, V> Entry<'a, K, V> {
  8:     pub(crate) fn new(map: &'a mut Core<K, V>, hash: HashValue, key: K) -> Self
  9:     where
 10:         K: Eq,
 11:     {
 12:         let eq = equal(&key, &map.entries);
 13:         match map.indices.find_entry(hash.get(), eq) {
 14:             Ok(entry) => Entry::Occupied(OccupiedEntry {
 15:                 bucket: entry.bucket_index(),
 16:                 index: *entry.get(),
 17:                 map,
 18:             }),
 19:             Err(_) => Entry::Vacant(VacantEntry { map, hash, key }),
 20:         }
 21:     }
 22: }
 23: /// A view into an occupied entry in an [`IndexMap`][crate::IndexMap].
 24: /// It is part of the [`Entry`] enum.
 25: pub struct OccupiedEntry<'a, K, V> {
 26:     map: &'a mut Core<K, V>,
 27:     // We have a mutable reference to the map, which keeps these two
 28:     // indices valid and pointing to the correct entry.
 29:     index: usize,
 30:     bucket: usize,
 31: }
 32: impl<'a, K, V> OccupiedEntry<'a, K, V> {
 33:     /// Constructor for `RawEntryMut::from_hash`
 34:     pub(crate) fn from_hash<F>(
 35:         map: &'a mut Core<K, V>,
 36:         hash: HashValue,
 37:         mut is_match: F,
 38:     ) -> Result<Self, &'a mut Core<K, V>>
 39:     where
 40:         F: FnMut(&K) -> bool,
 41:     {
 42:         let entries = &*map.entries;
 43:         let eq = move |&i: &usize| is_match(&entries[i].key);
 44:         match map.indices.find_entry(hash.get(), eq) {
 45:             Ok(entry) => Ok(OccupiedEntry {
 46:                 bucket: entry.bucket_index(),
 47:                 index: *entry.get(),
 48:                 map,
 49:             }),
 50:             Err(_) => Err(map),
 51:         }
 52:     }
 53:     pub(crate) fn into_core(self) -> &'a mut Core<K, V> {
 54:         self.map
 55:     }
 56:     pub(crate) fn get_bucket(&self) -> &Bucket<K, V> {
 57:         &self.map.entries[self.index]
 58:     }
 59:     pub(crate) fn get_bucket_mut(&mut self) -> &mut Bucket<K, V> {
 60:         &mut self.map.entries[self.index]
 61:     }
 62:     pub(crate) fn into_bucket(self) -> &'a mut Bucket<K, V> {
 63:         &mut self.map.entries[self.index]
 64:     }
 65:     /// Return the index of the key-value pair
 66:     #[inline]
 67:     pub fn index(&self) -> usize {
 68:         self.index
 69:     }
 70:     /// Gets a reference to the entry's key in the map.
 71:     ///
 72:     /// Note that this is not the key that was used to find the entry. There may be an observable
 73:     /// difference if the key type has any distinguishing features outside of `Hash` and `Eq`, like
 74:     /// extra fields or the memory address of an allocation.
 75:     pub fn key(&self) -> &K {
 76:         &self.get_bucket().key
 77:     }
 78:     /// Gets a reference to the entry's value in the map.
 79:     pub fn get(&self) -> &V {
 80:         &self.get_bucket().value
 81:     }
 82:     /// Gets a mutable reference to the entry's value in the map.
 83:     ///
 84:     /// If you need a reference which may outlive the destruction of the
 85:     /// [`Entry`] value, see [`into_mut`][Self::into_mut].
 86:     pub fn get_mut(&mut self) -> &mut V {
 87:         &mut self.get_bucket_mut().value
 88:     }
 89:     /// Converts into a mutable reference to the entry's value in the map,
 90:     /// with a lifetime bound to the map itself.
 91:     pub fn into_mut(self) -> &'a mut V {
 92:         &mut self.into_bucket().value
 93:     }
 94:     /// Sets the value of the entry to `value`, and returns the entry's old value.
 95:     pub fn insert(&mut self, value: V) -> V {
 96:         mem::replace(self.get_mut(), value)
 97:     }
 98:     /// Remove the key, value pair stored in the map for this entry, and return the value.
 99:     ///
100:     /// **NOTE:** This is equivalent to [`.swap_remove()`][Self::swap_remove], replacing this
101:     /// entry's position with the last element, and it is deprecated in favor of calling that
102:     /// explicitly. If you need to preserve the relative order of the keys in the map, use
103:     /// [`.shift_remove()`][Self::shift_remove] instead.
104:     #[deprecated(note = "`remove` disrupts the map order -- \
105:         use `swap_remove` or `shift_remove` for explicit behavior.")]
106:     pub fn remove(self) -> V {
107:         self.swap_remove()
108:     }
109:     /// Remove the key, value pair stored in the map for this entry, and return the value.
110:     ///
111:     /// Like [`Vec::swap_remove`][alloc::vec::Vec::swap_remove], the pair is removed by swapping it
112:     /// with the last element of the map and popping it off.
113:     /// **This perturbs the position of what used to be the last element!**
114:     ///
115:     /// Computes in **O(1)** time (average).
116:     pub fn swap_remove(self) -> V {
117:         self.swap_remove_entry().1
118:     }
119:     /// Remove the key, value pair stored in the map for this entry, and return the value.
120:     ///
121:     /// Like [`Vec::remove`][alloc::vec::Vec::remove], the pair is removed by shifting all of the
122:     /// elements that follow it, preserving their relative order.
123:     /// **This perturbs the index of all of those elements!**
124:     ///
125:     /// Computes in **O(n)** time (average).
126:     pub fn shift_remove(self) -> V {
127:         self.shift_remove_entry().1
128:     }
129:     /// Remove and return the key, value pair stored in the map for this entry
130:     ///
131:     /// **NOTE:** This is equivalent to [`.swap_remove_entry()`][Self::swap_remove_entry],
132:     /// replacing this entry's position with the last element, and it is deprecated in favor of
133:     /// calling that explicitly. If you need to preserve the relative order of the keys in the map,
134:     /// use [`.shift_remove_entry()`][Self::shift_remove_entry] instead.
135:     #[deprecated(note = "`remove_entry` disrupts the map order -- \
136:         use `swap_remove_entry` or `shift_remove_entry` for explicit behavior.")]
137:     pub fn remove_entry(self) -> (K, V) {
138:         self.swap_remove_entry()
139:     }
140:     /// Remove and return the key, value pair stored in the map for this entry
141:     ///
142:     /// Like [`Vec::swap_remove`][alloc::vec::Vec::swap_remove], the pair is removed by swapping it
143:     /// with the last element of the map and popping it off.
144:     /// **This perturbs the position of what used to be the last element!**
145:     ///
146:     /// Computes in **O(1)** time (average).
147:     pub fn swap_remove_entry(mut self) -> (K, V) {
148:         self.remove_index();
149:         self.map.swap_remove_finish(self.index)
150:     }
151:     /// Remove and return the key, value pair stored in the map for this entry
152:     ///
153:     /// Like [`Vec::remove`][alloc::vec::Vec::remove], the pair is removed by shifting all of the
154:     /// elements that follow it, preserving their relative order.
155:     /// **This perturbs the index of all of those elements!**
156:     ///
157:     /// Computes in **O(n)** time (average).
158:     pub fn shift_remove_entry(mut self) -> (K, V) {
159:         self.remove_index();
160:         self.map.shift_remove_finish(self.index)
161:     }
162:     fn remove_index(&mut self) {
163:         let entry = self.map.indices.get_bucket_entry(self.bucket).unwrap();
164:         debug_assert_eq!(*entry.get(), self.index);
165:         entry.remove();
166:     }
167:     /// Moves the position of the entry to a new index
168:     /// by shifting all other entries in-between.
169:     ///
170:     /// This is equivalent to [`IndexMap::move_index`][`crate::IndexMap::move_index`]
171:     /// coming `from` the current [`.index()`][Self::index].
172:     ///
173:     /// * If `self.index() < to`, the other pairs will shift down while the targeted pair moves up.
174:     /// * If `self.index() > to`, the other pairs will shift up while the targeted pair moves down.
175:     ///
176:     /// ***Panics*** if `to` is out of bounds.
177:     ///
178:     /// Computes in **O(n)** time (average).
179:     #[track_caller]
180:     pub fn move_index(self, to: usize) {
181:         if self.index != to {
182:             assert_index_lt(to, self.map.len());
183:             self.map.move_index_inner(self.index, to);
184:             self.update_index(to);
185:         }
186:     }
187:     /// Swaps the position of entry with another.
188:     ///
189:     /// This is equivalent to [`IndexMap::swap_indices`][`crate::IndexMap::swap_indices`]
190:     /// with the current [`.index()`][Self::index] as one of the two being swapped.
191:     ///
192:     /// ***Panics*** if the `other` index is out of bounds.
193:     ///
194:     /// Computes in **O(1)** time (average).
195:     #[track_caller]
196:     pub fn swap_indices(self, other: usize) {
197:         if self.index != other {
198:             assert_index_lt(other, self.map.len());
199:             // Since we already know where our bucket is, we only need to find the other.
200:             let hash = self.map.entries[other].hash;
201:             let other_mut = self.map.indices.find_mut(hash.get(), move |&i| i == other);
202:             *other_mut.expect("index not found") = self.index;
203:             self.map.entries.swap(self.index, other);
204:             self.update_index(other);
205:         }
206:     }
207:     fn update_index(self, to: usize) {
208:         let index = self.map.indices.get_bucket_mut(self.bucket).unwrap();
209:         debug_assert_eq!(*index, self.index);
210:         *index = to;
211:     }
212: }
213: impl<'a, K, V> From<IndexedEntry<'a, K, V>> for OccupiedEntry<'a, K, V> {
214:     fn from(other: IndexedEntry<'a, K, V>) -> Self {
215:         let index = other.index();
216:         let map = other.into_core();
217:         let hash = map.entries[index].hash;
218:         let bucket = map
219:             .indices
220:             .find_bucket_index(hash.get(), move |&i| i == index)
221:             .expect("index not found");
222:         Self { map, index, bucket }
223:     }
224: }
225: /// A view into a vacant entry in an [`IndexMap`][crate::IndexMap].
226: /// It is part of the [`Entry`] enum.
227: pub struct VacantEntry<'a, K, V> {
228:     map: &'a mut Core<K, V>,
229:     hash: HashValue,
230:     key: K,
231: }
232: impl<'a, K, V> VacantEntry<'a, K, V> {
233:     /// Return the index where a key-value pair may be inserted.
234:     pub fn index(&self) -> usize {
235:         self.map.indices.len()
236:     }
237:     /// Gets a reference to the key that was used to find the entry.
238:     pub fn key(&self) -> &K {
239:         &self.key
240:     }
241:     pub(crate) fn key_mut(&mut self) -> &mut K {
242:         &mut self.key
243:     }
244:     /// Takes ownership of the key, leaving the entry vacant.
245:     pub fn into_key(self) -> K {
246:         self.key
247:     }
248:     /// Inserts the entry's key and the given value into the map, and returns a mutable reference
249:     /// to the value.
250:     ///
251:     /// Computes in **O(1)** time (amortized average).
252:     pub fn insert(self, value: V) -> &'a mut V {
253:         let Self { map, hash, key } = self;
254:         map.insert_unique(hash, key, value).value_mut()
255:     }
256:     /// Inserts the entry's key and the given value into the map, and returns an `OccupiedEntry`.
257:     ///
258:     /// Computes in **O(1)** time (amortized average).
259:     pub fn insert_entry(self, value: V) -> OccupiedEntry<'a, K, V> {
260:         let Self { map, hash, key } = self;
261:         let index = map.indices.len();
262:         debug_assert_eq!(index, map.entries.len());
263:         let bucket = map
264:             .indices
265:             .insert_unique(hash.get(), index, get_hash(&map.entries))
266:             .bucket_index();
267:         map.push_entry(hash, key, value);
268:         OccupiedEntry { map, index, bucket }
269:     }
270:     /// Inserts the entry's key and the given value into the map at its ordered
271:     /// position among sorted keys, and returns the new index and a mutable
272:     /// reference to the value.
273:     ///
274:     /// If the existing keys are **not** already sorted, then the insertion
275:     /// index is unspecified (like [`slice::binary_search`]), but the key-value
276:     /// pair is inserted at that position regardless.
277:     ///
278:     /// Computes in **O(n)** time (average).
279:     pub fn insert_sorted(self, value: V) -> (usize, &'a mut V)
280:     where
281:         K: Ord,
282:     {
283:         let slice = crate::map::Slice::from_slice(&self.map.entries);
284:         let i = slice.binary_search_keys(&self.key).unwrap_err();
285:         (i, self.shift_insert(i, value))
286:     }
287:     /// Inserts the entry's key and the given value into the map at its ordered
288:     /// position among keys sorted by `cmp`, and returns the new index and a
289:     /// mutable reference to the value.
290:     ///
291:     /// If the existing keys are **not** already sorted, then the insertion
292:     /// index is unspecified (like [`slice::binary_search`]), but the key-value
293:     /// pair is inserted at that position regardless.
294:     ///
295:     /// Computes in **O(n)** time (average).
296:     pub fn insert_sorted_by<F>(self, value: V, mut cmp: F) -> (usize, &'a mut V)
297:     where
298:         F: FnMut(&K, &V, &K, &V) -> Ordering,
299:     {
300:         let slice = crate::map::Slice::from_slice(&self.map.entries);
301:         let (Ok(i) | Err(i)) = slice.binary_search_by(|k, v| cmp(k, v, &self.key, &value));
302:         (i, self.shift_insert(i, value))
303:     }
304:     /// Inserts the entry's key and the given value into the map at its ordered
305:     /// position using a sort-key extraction function, and returns the new index
306:     /// and a mutable reference to the value.
307:     ///
308:     /// If the existing keys are **not** already sorted, then the insertion
309:     /// index is unspecified (like [`slice::binary_search`]), but the key-value
310:     /// pair is inserted at that position regardless.
311:     ///
312:     /// Computes in **O(n)** time (average).
313:     pub fn insert_sorted_by_key<B, F>(self, value: V, mut sort_key: F) -> (usize, &'a mut V)
314:     where
315:         B: Ord,
316:         F: FnMut(&K, &V) -> B,
317:     {
318:         let search_key = sort_key(&self.key, &value);
319:         let slice = crate::map::Slice::from_slice(&self.map.entries);
320:         let (Ok(i) | Err(i)) = slice.binary_search_by_key(&search_key, sort_key);
321:         (i, self.shift_insert(i, value))
322:     }
323:     /// Inserts the entry's key and the given value into the map at the given index,
324:     /// shifting others to the right, and returns a mutable reference to the value.
325:     ///
326:     /// ***Panics*** if `index` is out of bounds.
327:     ///
328:     /// Computes in **O(n)** time (average).
329:     #[track_caller]
330:     pub fn shift_insert(self, index: usize, value: V) -> &'a mut V {
331:         self.map
332:             .shift_insert_unique(index, self.hash, self.key, value)
333:             .value_mut()
334:     }
335:     /// Replaces the key at the given index with this entry's key, returning the
336:     /// old key and an `OccupiedEntry` for that index.
337:     ///
338:     /// ***Panics*** if `index` is out of bounds.
339:     ///
340:     /// Computes in **O(1)** time (average).
341:     #[track_caller]
342:     pub fn replace_index(self, index: usize) -> (K, OccupiedEntry<'a, K, V>) {
343:         let Self { map, hash, key } = self;
344:         assert_index_lt(index, map.len());
345:         // NB: This removal and insertion isn't "no grow" (with unreachable hasher)
346:         // because hashbrown's tombstones might force a resize anyway.
347:         let old_hash = map.entries[index].hash;
348:         map.indices
349:             .find_entry(old_hash.get(), move |&i| i == index)
350:             .expect("index not found")
351:             .remove();
352:         let bucket = map
353:             .indices
354:             .insert_unique(hash.get(), index, get_hash(&map.entries))
355:             .bucket_index();
356:         let entry = &mut map.entries[index];
357:         entry.hash = hash;
358:         let old_key = mem::replace(&mut entry.key, key);
359:         (old_key, OccupiedEntry { map, index, bucket })
360:     }
361: }
````

## File: src/inner/extract.rs
````rust
 1: #![allow(unsafe_code)]
 2: use super::{Bucket, Core};
 3: use crate::util::simplify_range;
 4: use core::ops::RangeBounds;
 5: impl<K, V> Core<K, V> {
 6:     #[track_caller]
 7:     pub(crate) fn extract<R>(&mut self, range: R) -> ExtractCore<'_, K, V>
 8:     where
 9:         R: RangeBounds<usize>,
10:     {
11:         let range = simplify_range(range, self.entries.len());
12:         // SAFETY: We must have consistent lengths to start, so that's a hard assertion.
13:         // Then the worst `set_len` can do is leak items if `ExtractCore` doesn't drop.
14:         assert_eq!(self.entries.len(), self.indices.len());
15:         unsafe {
16:             self.entries.set_len(range.start);
17:         }
18:         ExtractCore {
19:             map: self,
20:             new_len: range.start,
21:             current: range.start,
22:             end: range.end,
23:         }
24:     }
25: }
26: pub(crate) struct ExtractCore<'a, K, V> {
27:     map: &'a mut Core<K, V>,
28:     new_len: usize,
29:     current: usize,
30:     end: usize,
31: }
32: impl<K, V> Drop for ExtractCore<'_, K, V> {
33:     fn drop(&mut self) {
34:         let old_len = self.map.indices.len();
35:         let mut new_len = self.new_len;
36:         debug_assert!(new_len <= self.current);
37:         debug_assert!(self.current <= self.end);
38:         debug_assert!(self.current <= old_len);
39:         debug_assert!(old_len <= self.map.entries.capacity());
40:         // SAFETY: We assume `new_len` and `current` were correctly maintained by the iterator.
41:         // So `entries[new_len..current]` were extracted, but the rest before and after are valid.
42:         unsafe {
43:             if new_len == self.current {
44:                 // Nothing was extracted, so any remaining items can be left in place.
45:                 new_len = old_len;
46:             } else if self.current < old_len {
47:                 // Need to shift the remaining items down.
48:                 let tail_len = old_len - self.current;
49:                 let base = self.map.entries.as_mut_ptr();
50:                 let src = base.add(self.current);
51:                 let dest = base.add(new_len);
52:                 src.copy_to(dest, tail_len);
53:                 new_len += tail_len;
54:             }
55:             self.map.entries.set_len(new_len);
56:         }
57:         if new_len != old_len {
58:             // We don't keep track of *which* items were extracted, so reindex everything.
59:             self.map.rebuild_hash_table();
60:         }
61:     }
62: }
63: impl<K, V> ExtractCore<'_, K, V> {
64:     pub(crate) fn extract_if<F>(&mut self, mut pred: F) -> Option<Bucket<K, V>>
65:     where
66:         F: FnMut(&mut Bucket<K, V>) -> bool,
67:     {
68:         debug_assert!(self.end <= self.map.entries.capacity());
69:         let base = self.map.entries.as_mut_ptr();
70:         while self.current < self.end {
71:             // SAFETY: We're maintaining both indices within bounds of the original entries, so
72:             // 0..new_len and current..indices.len() are always valid items for our Drop to keep.
73:             unsafe {
74:                 let item = base.add(self.current);
75:                 if pred(&mut *item) {
76:                     // Extract it!
77:                     self.current += 1;
78:                     return Some(item.read());
79:                 } else {
80:                     // Keep it, shifting it down if needed.
81:                     if self.new_len != self.current {
82:                         debug_assert!(self.new_len < self.current);
83:                         let dest = base.add(self.new_len);
84:                         item.copy_to_nonoverlapping(dest, 1);
85:                     }
86:                     self.current += 1;
87:                     self.new_len += 1;
88:                 }
89:             }
90:         }
91:         None
92:     }
93:     pub(crate) fn remaining(&self) -> usize {
94:         self.end - self.current
95:     }
96: }
````

## File: src/map/disjoint.rs
````rust
 1: #![allow(unsafe_code)]
 2: use crate::GetDisjointMutError;
 3: /// Like `slice::get_disjoint_mut`, although we're not dealing with ranges (yet).
 4: // TODO(MSRV 1.86): remove this in favor of the standard library's method.
 5: pub(super) fn get_disjoint_mut<T, const N: usize>(
 6:     entries: &mut [T],
 7:     indices: [usize; N],
 8: ) -> Result<[&mut T; N], GetDisjointMutError> {
 9:     // SAFETY: Can't allow duplicate indices as we would return mutable refs to the same data.
10:     let len = entries.len();
11:     for i in 0..N {
12:         let idx = indices[i];
13:         if idx >= len {
14:             return Err(GetDisjointMutError::IndexOutOfBounds);
15:         } else if indices[..i].contains(&idx) {
16:             return Err(GetDisjointMutError::OverlappingIndices);
17:         }
18:     }
19:     let entries_ptr = entries.as_mut_ptr();
20:     Ok(indices.map(move |idx| {
21:         // SAFETY: The base pointer is valid as it comes from a slice and the reference is always
22:         // in-bounds & unique as we've already checked the indices above.
23:         unsafe { &mut *(entries_ptr.add(idx)) }
24:     }))
25: }
26: /// Like `slice::get_disjoint_mut` but with optional indices,
27: /// allowing for absent keys from the user's original request.
28: #[track_caller]
29: pub(super) fn get_disjoint_opt_mut<T, const N: usize>(
30:     entries: &mut [T],
31:     indices: [Option<usize>; N],
32: ) -> [Option<&mut T>; N] {
33:     // SAFETY: Can't allow duplicate indices as we would return mutable refs to the same data.
34:     let len = entries.len();
35:     for i in 0..N {
36:         if let Some(idx) = indices[i] {
37:             if idx >= len {
38:                 unreachable!("`get_index_of` returned an out-of-bounds index");
39:             } else if indices[..i].contains(&Some(idx)) {
40:                 panic!("duplicate keys found");
41:             }
42:         }
43:     }
44:     let entries_ptr = entries.as_mut_ptr();
45:     indices.map(move |idx_opt| {
46:         match idx_opt {
47:             Some(idx) => {
48:                 // SAFETY: The base pointer is valid as it comes from a slice and the reference is always
49:                 // in-bounds & unique as we've already checked the indices above.
50:                 Some(unsafe { &mut *entries_ptr.add(idx) })
51:             }
52:             None => None,
53:         }
54:     })
55: }
````

## File: src/map/entry.rs
````rust
  1: use crate::Bucket;
  2: use crate::inner::{Core, OccupiedEntry, VacantEntry};
  3: use core::{fmt, mem};
  4: /// Entry for an existing key-value pair in an [`IndexMap`][crate::IndexMap]
  5: /// or a vacant location to insert one.
  6: pub enum Entry<'a, K, V> {
  7:     /// Existing slot with equivalent key.
  8:     Occupied(OccupiedEntry<'a, K, V>),
  9:     /// Vacant slot (no equivalent key in the map).
 10:     Vacant(VacantEntry<'a, K, V>),
 11: }
 12: impl<'a, K, V> Entry<'a, K, V> {
 13:     /// Return the index where the key-value pair exists or will be inserted.
 14:     pub fn index(&self) -> usize {
 15:         match self {
 16:             Entry::Occupied(entry) => entry.index(),
 17:             Entry::Vacant(entry) => entry.index(),
 18:         }
 19:     }
 20:     /// Sets the value of the entry (after inserting if vacant), and returns an `OccupiedEntry`.
 21:     ///
 22:     /// Computes in **O(1)** time (amortized average).
 23:     pub fn insert_entry(self, value: V) -> OccupiedEntry<'a, K, V> {
 24:         match self {
 25:             Entry::Occupied(mut entry) => {
 26:                 entry.insert(value);
 27:                 entry
 28:             }
 29:             Entry::Vacant(entry) => entry.insert_entry(value),
 30:         }
 31:     }
 32:     /// Inserts the given default value in the entry if it is vacant and returns a mutable
 33:     /// reference to it. Otherwise a mutable reference to an already existent value is returned.
 34:     ///
 35:     /// Computes in **O(1)** time (amortized average).
 36:     pub fn or_insert(self, default: V) -> &'a mut V {
 37:         match self {
 38:             Entry::Occupied(entry) => entry.into_mut(),
 39:             Entry::Vacant(entry) => entry.insert(default),
 40:         }
 41:     }
 42:     /// Inserts the result of the `call` function in the entry if it is vacant and returns a mutable
 43:     /// reference to it. Otherwise a mutable reference to an already existent value is returned.
 44:     ///
 45:     /// Computes in **O(1)** time (amortized average).
 46:     pub fn or_insert_with<F>(self, call: F) -> &'a mut V
 47:     where
 48:         F: FnOnce() -> V,
 49:     {
 50:         match self {
 51:             Entry::Occupied(entry) => entry.into_mut(),
 52:             Entry::Vacant(entry) => entry.insert(call()),
 53:         }
 54:     }
 55:     /// Inserts the result of the `call` function with a reference to the entry's key if it is
 56:     /// vacant, and returns a mutable reference to the new value. Otherwise a mutable reference to
 57:     /// an already existent value is returned.
 58:     ///
 59:     /// Computes in **O(1)** time (amortized average).
 60:     pub fn or_insert_with_key<F>(self, call: F) -> &'a mut V
 61:     where
 62:         F: FnOnce(&K) -> V,
 63:     {
 64:         match self {
 65:             Entry::Occupied(entry) => entry.into_mut(),
 66:             Entry::Vacant(entry) => {
 67:                 let value = call(entry.key());
 68:                 entry.insert(value)
 69:             }
 70:         }
 71:     }
 72:     /// Gets a reference to the entry's key, either within the map if occupied,
 73:     /// or else the new key that was used to find the entry.
 74:     pub fn key(&self) -> &K {
 75:         match *self {
 76:             Entry::Occupied(ref entry) => entry.key(),
 77:             Entry::Vacant(ref entry) => entry.key(),
 78:         }
 79:     }
 80:     /// Modifies the entry if it is occupied.
 81:     pub fn and_modify<F>(mut self, f: F) -> Self
 82:     where
 83:         F: FnOnce(&mut V),
 84:     {
 85:         if let Entry::Occupied(entry) = &mut self {
 86:             f(entry.get_mut());
 87:         }
 88:         self
 89:     }
 90:     /// Inserts a default-constructed value in the entry if it is vacant and returns a mutable
 91:     /// reference to it. Otherwise a mutable reference to an already existent value is returned.
 92:     ///
 93:     /// Computes in **O(1)** time (amortized average).
 94:     pub fn or_default(self) -> &'a mut V
 95:     where
 96:         V: Default,
 97:     {
 98:         match self {
 99:             Entry::Occupied(entry) => entry.into_mut(),
100:             Entry::Vacant(entry) => entry.insert(V::default()),
101:         }
102:     }
103: }
104: impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for Entry<'_, K, V> {
105:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
106:         let mut tuple = f.debug_tuple("Entry");
107:         match self {
108:             Entry::Vacant(v) => tuple.field(v),
109:             Entry::Occupied(o) => tuple.field(o),
110:         };
111:         tuple.finish()
112:     }
113: }
114: impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for OccupiedEntry<'_, K, V> {
115:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
116:         f.debug_struct("OccupiedEntry")
117:             .field("key", self.key())
118:             .field("value", self.get())
119:             .finish()
120:     }
121: }
122: impl<K: fmt::Debug, V> fmt::Debug for VacantEntry<'_, K, V> {
123:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
124:         f.debug_tuple("VacantEntry").field(self.key()).finish()
125:     }
126: }
127: /// A view into an occupied entry in an [`IndexMap`][crate::IndexMap] obtained by index.
128: ///
129: /// This `struct` is created from the [`get_index_entry`][crate::IndexMap::get_index_entry] method.
130: pub struct IndexedEntry<'a, K, V> {
131:     map: &'a mut Core<K, V>,
132:     // We have a mutable reference to the map, which keeps the index
133:     // valid and pointing to the correct entry.
134:     index: usize,
135: }
136: impl<'a, K, V> IndexedEntry<'a, K, V> {
137:     pub(crate) fn new(map: &'a mut Core<K, V>, index: usize) -> Option<Self> {
138:         if index < map.len() {
139:             Some(Self { map, index })
140:         } else {
141:             None
142:         }
143:     }
144:     /// Return the index of the key-value pair
145:     #[inline]
146:     pub fn index(&self) -> usize {
147:         self.index
148:     }
149:     pub(crate) fn into_core(self) -> &'a mut Core<K, V> {
150:         self.map
151:     }
152:     fn get_bucket(&self) -> &Bucket<K, V> {
153:         &self.map.as_entries()[self.index]
154:     }
155:     fn get_bucket_mut(&mut self) -> &mut Bucket<K, V> {
156:         &mut self.map.as_entries_mut()[self.index]
157:     }
158:     fn into_bucket(self) -> &'a mut Bucket<K, V> {
159:         &mut self.map.as_entries_mut()[self.index]
160:     }
161:     /// Gets a reference to the entry's key in the map.
162:     pub fn key(&self) -> &K {
163:         &self.get_bucket().key
164:     }
165:     pub(super) fn key_mut(&mut self) -> &mut K {
166:         &mut self.get_bucket_mut().key
167:     }
168:     /// Gets a reference to the entry's value in the map.
169:     pub fn get(&self) -> &V {
170:         &self.get_bucket().value
171:     }
172:     /// Gets a mutable reference to the entry's value in the map.
173:     ///
174:     /// If you need a reference which may outlive the destruction of the
175:     /// `IndexedEntry` value, see [`into_mut`][Self::into_mut].
176:     pub fn get_mut(&mut self) -> &mut V {
177:         &mut self.get_bucket_mut().value
178:     }
179:     /// Sets the value of the entry to `value`, and returns the entry's old value.
180:     pub fn insert(&mut self, value: V) -> V {
181:         mem::replace(self.get_mut(), value)
182:     }
183:     /// Converts into a mutable reference to the entry's value in the map,
184:     /// with a lifetime bound to the map itself.
185:     pub fn into_mut(self) -> &'a mut V {
186:         &mut self.into_bucket().value
187:     }
188:     /// Remove and return the key, value pair stored in the map for this entry
189:     ///
190:     /// Like [`Vec::swap_remove`][alloc::vec::Vec::swap_remove], the pair is removed by swapping it
191:     /// with the last element of the map and popping it off.
192:     /// **This perturbs the position of what used to be the last element!**
193:     ///
194:     /// Computes in **O(1)** time (average).
195:     pub fn swap_remove_entry(self) -> (K, V) {
196:         self.map.swap_remove_index(self.index).unwrap()
197:     }
198:     /// Remove and return the key, value pair stored in the map for this entry
199:     ///
200:     /// Like [`Vec::remove`][alloc::vec::Vec::remove], the pair is removed by shifting all of the
201:     /// elements that follow it, preserving their relative order.
202:     /// **This perturbs the index of all of those elements!**
203:     ///
204:     /// Computes in **O(n)** time (average).
205:     pub fn shift_remove_entry(self) -> (K, V) {
206:         self.map.shift_remove_index(self.index).unwrap()
207:     }
208:     /// Remove the key, value pair stored in the map for this entry, and return the value.
209:     ///
210:     /// Like [`Vec::swap_remove`][alloc::vec::Vec::swap_remove], the pair is removed by swapping it
211:     /// with the last element of the map and popping it off.
212:     /// **This perturbs the position of what used to be the last element!**
213:     ///
214:     /// Computes in **O(1)** time (average).
215:     pub fn swap_remove(self) -> V {
216:         self.swap_remove_entry().1
217:     }
218:     /// Remove the key, value pair stored in the map for this entry, and return the value.
219:     ///
220:     /// Like [`Vec::remove`][alloc::vec::Vec::remove], the pair is removed by shifting all of the
221:     /// elements that follow it, preserving their relative order.
222:     /// **This perturbs the index of all of those elements!**
223:     ///
224:     /// Computes in **O(n)** time (average).
225:     pub fn shift_remove(self) -> V {
226:         self.shift_remove_entry().1
227:     }
228:     /// Moves the position of the entry to a new index
229:     /// by shifting all other entries in-between.
230:     ///
231:     /// This is equivalent to [`IndexMap::move_index`][`crate::IndexMap::move_index`]
232:     /// coming `from` the current [`.index()`][Self::index].
233:     ///
234:     /// * If `self.index() < to`, the other pairs will shift down while the targeted pair moves up.
235:     /// * If `self.index() > to`, the other pairs will shift up while the targeted pair moves down.
236:     ///
237:     /// ***Panics*** if `to` is out of bounds.
238:     ///
239:     /// Computes in **O(n)** time (average).
240:     #[track_caller]
241:     pub fn move_index(self, to: usize) {
242:         self.map.move_index(self.index, to);
243:     }
244:     /// Swaps the position of entry with another.
245:     ///
246:     /// This is equivalent to [`IndexMap::swap_indices`][`crate::IndexMap::swap_indices`]
247:     /// with the current [`.index()`][Self::index] as one of the two being swapped.
248:     ///
249:     /// ***Panics*** if the `other` index is out of bounds.
250:     ///
251:     /// Computes in **O(1)** time (average).
252:     #[track_caller]
253:     pub fn swap_indices(self, other: usize) {
254:         self.map.swap_indices(self.index, other);
255:     }
256: }
257: impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for IndexedEntry<'_, K, V> {
258:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
259:         f.debug_struct("IndexedEntry")
260:             .field("index", &self.index)
261:             .field("key", self.key())
262:             .field("value", self.get())
263:             .finish()
264:     }
265: }
266: impl<'a, K, V> From<OccupiedEntry<'a, K, V>> for IndexedEntry<'a, K, V> {
267:     fn from(other: OccupiedEntry<'a, K, V>) -> Self {
268:         Self {
269:             index: other.index(),
270:             map: other.into_core(),
271:         }
272:     }
273: }
274: #[test]
275: fn assert_send_sync() {
276:     fn assert_send_sync<T: Send + Sync>() {}
277:     assert_send_sync::<Entry<'_, i32, i32>>();
278:     assert_send_sync::<IndexedEntry<'_, i32, i32>>();
279: }
````

## File: src/map/iter.rs
````rust
  1: use super::{Bucket, HashValue, IndexMap, Slice};
  2: use crate::inner::{Core, ExtractCore};
  3: use alloc::vec::{self, Vec};
  4: use core::fmt;
  5: use core::hash::{BuildHasher, Hash};
  6: use core::iter::FusedIterator;
  7: use core::mem::MaybeUninit;
  8: use core::ops::{Index, RangeBounds};
  9: use core::slice;
 10: impl<'a, K, V, S> IntoIterator for &'a IndexMap<K, V, S> {
 11:     type Item = (&'a K, &'a V);
 12:     type IntoIter = Iter<'a, K, V>;
 13:     fn into_iter(self) -> Self::IntoIter {
 14:         self.iter()
 15:     }
 16: }
 17: impl<'a, K, V, S> IntoIterator for &'a mut IndexMap<K, V, S> {
 18:     type Item = (&'a K, &'a mut V);
 19:     type IntoIter = IterMut<'a, K, V>;
 20:     fn into_iter(self) -> Self::IntoIter {
 21:         self.iter_mut()
 22:     }
 23: }
 24: impl<K, V, S> IntoIterator for IndexMap<K, V, S> {
 25:     type Item = (K, V);
 26:     type IntoIter = IntoIter<K, V>;
 27:     fn into_iter(self) -> Self::IntoIter {
 28:         IntoIter::new(self.into_entries())
 29:     }
 30: }
 31: /// An iterator over the entries of an [`IndexMap`].
 32: ///
 33: /// This `struct` is created by the [`IndexMap::iter`] method.
 34: /// See its documentation for more.
 35: pub struct Iter<'a, K, V> {
 36:     iter: slice::Iter<'a, Bucket<K, V>>,
 37: }
 38: impl<'a, K, V> Iter<'a, K, V> {
 39:     pub(super) fn new(entries: &'a [Bucket<K, V>]) -> Self {
 40:         Self {
 41:             iter: entries.iter(),
 42:         }
 43:     }
 44:     /// Returns a slice of the remaining entries in the iterator.
 45:     pub fn as_slice(&self) -> &'a Slice<K, V> {
 46:         Slice::from_slice(self.iter.as_slice())
 47:     }
 48: }
 49: impl<'a, K, V> Iterator for Iter<'a, K, V> {
 50:     type Item = (&'a K, &'a V);
 51:     iterator_methods!(Bucket::refs);
 52: }
 53: impl<K, V> DoubleEndedIterator for Iter<'_, K, V> {
 54:     double_ended_iterator_methods!(Bucket::refs);
 55: }
 56: impl<K, V> ExactSizeIterator for Iter<'_, K, V> {
 57:     fn len(&self) -> usize {
 58:         self.iter.len()
 59:     }
 60: }
 61: impl<K, V> FusedIterator for Iter<'_, K, V> {}
 62: // FIXME(#26925) Remove in favor of `#[derive(Clone)]`
 63: impl<K, V> Clone for Iter<'_, K, V> {
 64:     fn clone(&self) -> Self {
 65:         Iter {
 66:             iter: self.iter.clone(),
 67:         }
 68:     }
 69: }
 70: impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for Iter<'_, K, V> {
 71:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
 72:         f.debug_list().entries(self.clone()).finish()
 73:     }
 74: }
 75: impl<K, V> Default for Iter<'_, K, V> {
 76:     fn default() -> Self {
 77:         Self { iter: [].iter() }
 78:     }
 79: }
 80: /// A mutable iterator over the entries of an [`IndexMap`].
 81: ///
 82: /// This `struct` is created by the [`IndexMap::iter_mut`] method.
 83: /// See its documentation for more.
 84: pub struct IterMut<'a, K, V> {
 85:     iter: slice::IterMut<'a, Bucket<K, V>>,
 86: }
 87: impl<'a, K, V> IterMut<'a, K, V> {
 88:     pub(super) fn new(entries: &'a mut [Bucket<K, V>]) -> Self {
 89:         Self {
 90:             iter: entries.iter_mut(),
 91:         }
 92:     }
 93:     /// Returns a slice of the remaining entries in the iterator.
 94:     pub fn as_slice(&self) -> &Slice<K, V> {
 95:         Slice::from_slice(self.iter.as_slice())
 96:     }
 97:     /// Returns a mutable slice of the remaining entries in the iterator.
 98:     ///
 99:     /// To avoid creating `&mut` references that alias, this is forced to consume the iterator.
100:     pub fn into_slice(self) -> &'a mut Slice<K, V> {
101:         Slice::from_mut_slice(self.iter.into_slice())
102:     }
103: }
104: impl<'a, K, V> Iterator for IterMut<'a, K, V> {
105:     type Item = (&'a K, &'a mut V);
106:     iterator_methods!(Bucket::ref_mut);
107: }
108: impl<K, V> DoubleEndedIterator for IterMut<'_, K, V> {
109:     double_ended_iterator_methods!(Bucket::ref_mut);
110: }
111: impl<K, V> ExactSizeIterator for IterMut<'_, K, V> {
112:     fn len(&self) -> usize {
113:         self.iter.len()
114:     }
115: }
116: impl<K, V> FusedIterator for IterMut<'_, K, V> {}
117: impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for IterMut<'_, K, V> {
118:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
119:         let iter = self.iter.as_slice().iter().map(Bucket::refs);
120:         f.debug_list().entries(iter).finish()
121:     }
122: }
123: impl<K, V> Default for IterMut<'_, K, V> {
124:     fn default() -> Self {
125:         Self {
126:             iter: [].iter_mut(),
127:         }
128:     }
129: }
130: /// A mutable iterator over the entries of an [`IndexMap`].
131: ///
132: /// This `struct` is created by the [`MutableKeys::iter_mut2`][super::MutableKeys::iter_mut2] method.
133: /// See its documentation for more.
134: pub struct IterMut2<'a, K, V> {
135:     iter: slice::IterMut<'a, Bucket<K, V>>,
136: }
137: impl<'a, K, V> IterMut2<'a, K, V> {
138:     pub(super) fn new(entries: &'a mut [Bucket<K, V>]) -> Self {
139:         Self {
140:             iter: entries.iter_mut(),
141:         }
142:     }
143:     /// Returns a slice of the remaining entries in the iterator.
144:     pub fn as_slice(&self) -> &Slice<K, V> {
145:         Slice::from_slice(self.iter.as_slice())
146:     }
147:     /// Returns a mutable slice of the remaining entries in the iterator.
148:     ///
149:     /// To avoid creating `&mut` references that alias, this is forced to consume the iterator.
150:     pub fn into_slice(self) -> &'a mut Slice<K, V> {
151:         Slice::from_mut_slice(self.iter.into_slice())
152:     }
153: }
154: impl<'a, K, V> Iterator for IterMut2<'a, K, V> {
155:     type Item = (&'a mut K, &'a mut V);
156:     iterator_methods!(Bucket::muts);
157: }
158: impl<K, V> DoubleEndedIterator for IterMut2<'_, K, V> {
159:     double_ended_iterator_methods!(Bucket::muts);
160: }
161: impl<K, V> ExactSizeIterator for IterMut2<'_, K, V> {
162:     fn len(&self) -> usize {
163:         self.iter.len()
164:     }
165: }
166: impl<K, V> FusedIterator for IterMut2<'_, K, V> {}
167: impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for IterMut2<'_, K, V> {
168:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
169:         let iter = self.iter.as_slice().iter().map(Bucket::refs);
170:         f.debug_list().entries(iter).finish()
171:     }
172: }
173: impl<K, V> Default for IterMut2<'_, K, V> {
174:     fn default() -> Self {
175:         Self {
176:             iter: [].iter_mut(),
177:         }
178:     }
179: }
180: /// An owning iterator over the entries of an [`IndexMap`].
181: ///
182: /// This `struct` is created by the [`IndexMap::into_iter`] method
183: /// (provided by the [`IntoIterator`] trait). See its documentation for more.
184: #[derive(Clone)]
185: pub struct IntoIter<K, V> {
186:     iter: vec::IntoIter<Bucket<K, V>>,
187: }
188: impl<K, V> IntoIter<K, V> {
189:     pub(super) fn new(entries: Vec<Bucket<K, V>>) -> Self {
190:         Self {
191:             iter: entries.into_iter(),
192:         }
193:     }
194:     /// Returns a slice of the remaining entries in the iterator.
195:     pub fn as_slice(&self) -> &Slice<K, V> {
196:         Slice::from_slice(self.iter.as_slice())
197:     }
198:     /// Returns a mutable slice of the remaining entries in the iterator.
199:     pub fn as_mut_slice(&mut self) -> &mut Slice<K, V> {
200:         Slice::from_mut_slice(self.iter.as_mut_slice())
201:     }
202: }
203: impl<K, V> Iterator for IntoIter<K, V> {
204:     type Item = (K, V);
205:     iterator_methods!(Bucket::key_value);
206: }
207: impl<K, V> DoubleEndedIterator for IntoIter<K, V> {
208:     double_ended_iterator_methods!(Bucket::key_value);
209: }
210: impl<K, V> ExactSizeIterator for IntoIter<K, V> {
211:     fn len(&self) -> usize {
212:         self.iter.len()
213:     }
214: }
215: impl<K, V> FusedIterator for IntoIter<K, V> {}
216: impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for IntoIter<K, V> {
217:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
218:         let iter = self.iter.as_slice().iter().map(Bucket::refs);
219:         f.debug_list().entries(iter).finish()
220:     }
221: }
222: impl<K, V> Default for IntoIter<K, V> {
223:     fn default() -> Self {
224:         Self {
225:             iter: Vec::new().into_iter(),
226:         }
227:     }
228: }
229: /// A draining iterator over the entries of an [`IndexMap`].
230: ///
231: /// This `struct` is created by the [`IndexMap::drain`] method.
232: /// See its documentation for more.
233: pub struct Drain<'a, K, V> {
234:     iter: vec::Drain<'a, Bucket<K, V>>,
235: }
236: impl<'a, K, V> Drain<'a, K, V> {
237:     pub(super) fn new(iter: vec::Drain<'a, Bucket<K, V>>) -> Self {
238:         Self { iter }
239:     }
240:     /// Returns a slice of the remaining entries in the iterator.
241:     pub fn as_slice(&self) -> &Slice<K, V> {
242:         Slice::from_slice(self.iter.as_slice())
243:     }
244: }
245: impl<K, V> Iterator for Drain<'_, K, V> {
246:     type Item = (K, V);
247:     iterator_methods!(Bucket::key_value);
248: }
249: impl<K, V> DoubleEndedIterator for Drain<'_, K, V> {
250:     double_ended_iterator_methods!(Bucket::key_value);
251: }
252: impl<K, V> ExactSizeIterator for Drain<'_, K, V> {
253:     fn len(&self) -> usize {
254:         self.iter.len()
255:     }
256: }
257: impl<K, V> FusedIterator for Drain<'_, K, V> {}
258: impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for Drain<'_, K, V> {
259:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
260:         let iter = self.iter.as_slice().iter().map(Bucket::refs);
261:         f.debug_list().entries(iter).finish()
262:     }
263: }
264: /// An iterator over the keys of an [`IndexMap`].
265: ///
266: /// This `struct` is created by the [`IndexMap::keys`] method.
267: /// See its documentation for more.
268: pub struct Keys<'a, K, V> {
269:     iter: slice::Iter<'a, Bucket<K, V>>,
270: }
271: impl<'a, K, V> Keys<'a, K, V> {
272:     pub(super) fn new(entries: &'a [Bucket<K, V>]) -> Self {
273:         Self {
274:             iter: entries.iter(),
275:         }
276:     }
277: }
278: impl<'a, K, V> Iterator for Keys<'a, K, V> {
279:     type Item = &'a K;
280:     iterator_methods!(Bucket::key_ref);
281: }
282: impl<K, V> DoubleEndedIterator for Keys<'_, K, V> {
283:     double_ended_iterator_methods!(Bucket::key_ref);
284: }
285: impl<K, V> ExactSizeIterator for Keys<'_, K, V> {
286:     fn len(&self) -> usize {
287:         self.iter.len()
288:     }
289: }
290: impl<K, V> FusedIterator for Keys<'_, K, V> {}
291: // FIXME(#26925) Remove in favor of `#[derive(Clone)]`
292: impl<K, V> Clone for Keys<'_, K, V> {
293:     fn clone(&self) -> Self {
294:         Keys {
295:             iter: self.iter.clone(),
296:         }
297:     }
298: }
299: impl<K: fmt::Debug, V> fmt::Debug for Keys<'_, K, V> {
300:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
301:         f.debug_list().entries(self.clone()).finish()
302:     }
303: }
304: impl<K, V> Default for Keys<'_, K, V> {
305:     fn default() -> Self {
306:         Self { iter: [].iter() }
307:     }
308: }
309: /// Access [`IndexMap`] keys at indexed positions.
310: ///
311: /// While [`Index<usize> for IndexMap`][values] accesses a map's values,
312: /// indexing through [`IndexMap::keys`] offers an alternative to access a map's
313: /// keys instead.
314: ///
315: /// [values]: IndexMap#impl-Index<usize>-for-IndexMap<K,+V,+S>
316: ///
317: /// Since `Keys` is also an iterator, consuming items from the iterator will
318: /// offset the effective indices. Similarly, if `Keys` is obtained from
319: /// [`Slice::keys`], indices will be interpreted relative to the position of
320: /// that slice.
321: ///
322: /// # Examples
323: ///
324: /// ```
325: /// use indexmap::IndexMap;
326: ///
327: /// let mut map = IndexMap::new();
328: /// for word in "Lorem ipsum dolor sit amet".split_whitespace() {
329: ///     map.insert(word.to_lowercase(), word.to_uppercase());
330: /// }
331: ///
332: /// assert_eq!(map[0], "LOREM");
333: /// assert_eq!(map.keys()[0], "lorem");
334: /// assert_eq!(map[1], "IPSUM");
335: /// assert_eq!(map.keys()[1], "ipsum");
336: ///
337: /// map.reverse();
338: /// assert_eq!(map.keys()[0], "amet");
339: /// assert_eq!(map.keys()[1], "sit");
340: ///
341: /// map.sort_keys();
342: /// assert_eq!(map.keys()[0], "amet");
343: /// assert_eq!(map.keys()[1], "dolor");
344: ///
345: /// // Advancing the iterator will offset the indexing
346: /// let mut keys = map.keys();
347: /// assert_eq!(keys[0], "amet");
348: /// assert_eq!(keys.next().map(|s| &**s), Some("amet"));
349: /// assert_eq!(keys[0], "dolor");
350: /// assert_eq!(keys[1], "ipsum");
351: ///
352: /// // Slices may have an offset as well
353: /// let slice = &map[2..];
354: /// assert_eq!(slice[0], "IPSUM");
355: /// assert_eq!(slice.keys()[0], "ipsum");
356: /// ```
357: ///
358: /// ```should_panic
359: /// use indexmap::IndexMap;
360: ///
361: /// let mut map = IndexMap::new();
362: /// map.insert("foo", 1);
363: /// println!("{:?}", map.keys()[10]); // panics!
364: /// ```
365: impl<K, V> Index<usize> for Keys<'_, K, V> {
366:     type Output = K;
367:     /// Returns a reference to the key at the supplied `index`.
368:     ///
369:     /// ***Panics*** if `index` is out of bounds.
370:     fn index(&self, index: usize) -> &K {
371:         &self.iter.as_slice()[index].key
372:     }
373: }
374: /// An owning iterator over the keys of an [`IndexMap`].
375: ///
376: /// This `struct` is created by the [`IndexMap::into_keys`] method.
377: /// See its documentation for more.
378: pub struct IntoKeys<K, V> {
379:     // We eagerly drop the values during construction so we can ignore them in
380:     // `Clone`, but we keep uninit values so the bucket's size and alignment
381:     // remain the same, and therefore the `Vec` conversion should be in-place.
382:     iter: vec::IntoIter<Bucket<K, MaybeUninit<V>>>,
383: }
384: impl<K, V> IntoKeys<K, V> {
385:     pub(super) fn new(entries: Vec<Bucket<K, V>>) -> Self {
386:         // The original values will be dropped here.
387:         // The hash doesn't matter, but "copying" it in-place is free.
388:         let entries = entries
389:             .into_iter()
390:             .map(|Bucket { hash, key, .. }| Bucket {
391:                 hash,
392:                 key,
393:                 value: MaybeUninit::uninit(),
394:             })
395:             .collect::<Vec<_>>();
396:         Self {
397:             iter: entries.into_iter(),
398:         }
399:     }
400: }
401: impl<K: Clone, V> Clone for IntoKeys<K, V> {
402:     fn clone(&self) -> Self {
403:         let entries = self
404:             .iter
405:             .as_slice()
406:             .iter()
407:             .map(|Bucket { key, .. }| Bucket {
408:                 hash: HashValue(0),
409:                 key: key.clone(),
410:                 value: MaybeUninit::uninit(),
411:             })
412:             .collect::<Vec<_>>();
413:         Self {
414:             iter: entries.into_iter(),
415:         }
416:     }
417: }
418: impl<K, V> Iterator for IntoKeys<K, V> {
419:     type Item = K;
420:     iterator_methods!(Bucket::key);
421: }
422: impl<K, V> DoubleEndedIterator for IntoKeys<K, V> {
423:     double_ended_iterator_methods!(Bucket::key);
424: }
425: impl<K, V> ExactSizeIterator for IntoKeys<K, V> {
426:     fn len(&self) -> usize {
427:         self.iter.len()
428:     }
429: }
430: impl<K, V> FusedIterator for IntoKeys<K, V> {}
431: impl<K: fmt::Debug, V> fmt::Debug for IntoKeys<K, V> {
432:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
433:         let iter = self.iter.as_slice().iter().map(Bucket::key_ref);
434:         f.debug_list().entries(iter).finish()
435:     }
436: }
437: impl<K, V> Default for IntoKeys<K, V> {
438:     fn default() -> Self {
439:         Self {
440:             iter: Vec::new().into_iter(),
441:         }
442:     }
443: }
444: /// An iterator over the values of an [`IndexMap`].
445: ///
446: /// This `struct` is created by the [`IndexMap::values`] method.
447: /// See its documentation for more.
448: pub struct Values<'a, K, V> {
449:     iter: slice::Iter<'a, Bucket<K, V>>,
450: }
451: impl<'a, K, V> Values<'a, K, V> {
452:     pub(super) fn new(entries: &'a [Bucket<K, V>]) -> Self {
453:         Self {
454:             iter: entries.iter(),
455:         }
456:     }
457: }
458: impl<'a, K, V> Iterator for Values<'a, K, V> {
459:     type Item = &'a V;
460:     iterator_methods!(Bucket::value_ref);
461: }
462: impl<K, V> DoubleEndedIterator for Values<'_, K, V> {
463:     double_ended_iterator_methods!(Bucket::value_ref);
464: }
465: impl<K, V> ExactSizeIterator for Values<'_, K, V> {
466:     fn len(&self) -> usize {
467:         self.iter.len()
468:     }
469: }
470: impl<K, V> FusedIterator for Values<'_, K, V> {}
471: // FIXME(#26925) Remove in favor of `#[derive(Clone)]`
472: impl<K, V> Clone for Values<'_, K, V> {
473:     fn clone(&self) -> Self {
474:         Values {
475:             iter: self.iter.clone(),
476:         }
477:     }
478: }
479: impl<K, V: fmt::Debug> fmt::Debug for Values<'_, K, V> {
480:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
481:         f.debug_list().entries(self.clone()).finish()
482:     }
483: }
484: impl<K, V> Default for Values<'_, K, V> {
485:     fn default() -> Self {
486:         Self { iter: [].iter() }
487:     }
488: }
489: /// A mutable iterator over the values of an [`IndexMap`].
490: ///
491: /// This `struct` is created by the [`IndexMap::values_mut`] method.
492: /// See its documentation for more.
493: pub struct ValuesMut<'a, K, V> {
494:     iter: slice::IterMut<'a, Bucket<K, V>>,
495: }
496: impl<'a, K, V> ValuesMut<'a, K, V> {
497:     pub(super) fn new(entries: &'a mut [Bucket<K, V>]) -> Self {
498:         Self {
499:             iter: entries.iter_mut(),
500:         }
501:     }
502: }
503: impl<'a, K, V> Iterator for ValuesMut<'a, K, V> {
504:     type Item = &'a mut V;
505:     iterator_methods!(Bucket::value_mut);
506: }
507: impl<K, V> DoubleEndedIterator for ValuesMut<'_, K, V> {
508:     double_ended_iterator_methods!(Bucket::value_mut);
509: }
510: impl<K, V> ExactSizeIterator for ValuesMut<'_, K, V> {
511:     fn len(&self) -> usize {
512:         self.iter.len()
513:     }
514: }
515: impl<K, V> FusedIterator for ValuesMut<'_, K, V> {}
516: impl<K, V: fmt::Debug> fmt::Debug for ValuesMut<'_, K, V> {
517:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
518:         let iter = self.iter.as_slice().iter().map(Bucket::value_ref);
519:         f.debug_list().entries(iter).finish()
520:     }
521: }
522: impl<K, V> Default for ValuesMut<'_, K, V> {
523:     fn default() -> Self {
524:         Self {
525:             iter: [].iter_mut(),
526:         }
527:     }
528: }
529: /// An owning iterator over the values of an [`IndexMap`].
530: ///
531: /// This `struct` is created by the [`IndexMap::into_values`] method.
532: /// See its documentation for more.
533: pub struct IntoValues<K, V> {
534:     // We eagerly drop the keys during construction so we can ignore them in
535:     // `Clone`, but we keep uninit keys so the bucket's size and alignment
536:     // remain the same, and therefore the `Vec` conversion should be in-place.
537:     iter: vec::IntoIter<Bucket<MaybeUninit<K>, V>>,
538: }
539: impl<K, V> IntoValues<K, V> {
540:     pub(super) fn new(entries: Vec<Bucket<K, V>>) -> Self {
541:         // The original keys will be dropped here.
542:         // The hash doesn't matter, but "copying" it in-place is free.
543:         let entries = entries
544:             .into_iter()
545:             .map(|Bucket { hash, value, .. }| Bucket {
546:                 hash,
547:                 key: MaybeUninit::uninit(),
548:                 value,
549:             })
550:             .collect::<Vec<_>>();
551:         Self {
552:             iter: entries.into_iter(),
553:         }
554:     }
555: }
556: impl<K, V: Clone> Clone for IntoValues<K, V> {
557:     fn clone(&self) -> Self {
558:         let entries = self
559:             .iter
560:             .as_slice()
561:             .iter()
562:             .map(|Bucket { value, .. }| Bucket {
563:                 hash: HashValue(0),
564:                 key: MaybeUninit::uninit(),
565:                 value: value.clone(),
566:             })
567:             .collect::<Vec<_>>();
568:         Self {
569:             iter: entries.into_iter(),
570:         }
571:     }
572: }
573: impl<K, V> Iterator for IntoValues<K, V> {
574:     type Item = V;
575:     iterator_methods!(Bucket::value);
576: }
577: impl<K, V> DoubleEndedIterator for IntoValues<K, V> {
578:     double_ended_iterator_methods!(Bucket::value);
579: }
580: impl<K, V> ExactSizeIterator for IntoValues<K, V> {
581:     fn len(&self) -> usize {
582:         self.iter.len()
583:     }
584: }
585: impl<K, V> FusedIterator for IntoValues<K, V> {}
586: impl<K, V: fmt::Debug> fmt::Debug for IntoValues<K, V> {
587:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
588:         let iter = self.iter.as_slice().iter().map(Bucket::value_ref);
589:         f.debug_list().entries(iter).finish()
590:     }
591: }
592: impl<K, V> Default for IntoValues<K, V> {
593:     fn default() -> Self {
594:         Self {
595:             iter: Vec::new().into_iter(),
596:         }
597:     }
598: }
599: /// A splicing iterator for `IndexMap`.
600: ///
601: /// This `struct` is created by [`IndexMap::splice()`].
602: /// See its documentation for more.
603: pub struct Splice<'a, I, K, V, S>
604: where
605:     I: Iterator<Item = (K, V)>,
606:     K: Hash + Eq,
607:     S: BuildHasher,
608: {
609:     map: &'a mut IndexMap<K, V, S>,
610:     tail: Core<K, V>,
611:     drain: vec::IntoIter<Bucket<K, V>>,
612:     replace_with: I,
613: }
614: impl<'a, I, K, V, S> Splice<'a, I, K, V, S>
615: where
616:     I: Iterator<Item = (K, V)>,
617:     K: Hash + Eq,
618:     S: BuildHasher,
619: {
620:     #[track_caller]
621:     pub(super) fn new<R>(map: &'a mut IndexMap<K, V, S>, range: R, replace_with: I) -> Self
622:     where
623:         R: RangeBounds<usize>,
624:     {
625:         let (tail, drain) = map.core.split_splice(range);
626:         Self {
627:             map,
628:             tail,
629:             drain,
630:             replace_with,
631:         }
632:     }
633: }
634: impl<I, K, V, S> Drop for Splice<'_, I, K, V, S>
635: where
636:     I: Iterator<Item = (K, V)>,
637:     K: Hash + Eq,
638:     S: BuildHasher,
639: {
640:     fn drop(&mut self) {
641:         // Finish draining unconsumed items. We don't strictly *have* to do this
642:         // manually, since we already split it into separate memory, but it will
643:         // match the drop order of `vec::Splice` items this way.
644:         let _ = self.drain.nth(usize::MAX);
645:         // Now insert all the new items. If a key matches an existing entry, it
646:         // keeps the original position and only replaces the value, like `insert`.
647:         while let Some((key, value)) = self.replace_with.next() {
648:             // Since the tail is disjoint, we can try to update it first,
649:             // or else insert (update or append) the primary map.
650:             let hash = self.map.hash(&key);
651:             if let Some(i) = self.tail.get_index_of(hash, &key) {
652:                 self.tail.as_entries_mut()[i].value = value;
653:             } else {
654:                 self.map.core.insert_full(hash, key, value);
655:             }
656:         }
657:         // Finally, re-append the tail
658:         self.map.core.append_unchecked(&mut self.tail);
659:     }
660: }
661: impl<I, K, V, S> Iterator for Splice<'_, I, K, V, S>
662: where
663:     I: Iterator<Item = (K, V)>,
664:     K: Hash + Eq,
665:     S: BuildHasher,
666: {
667:     type Item = (K, V);
668:     fn next(&mut self) -> Option<Self::Item> {
669:         self.drain.next().map(Bucket::key_value)
670:     }
671:     fn size_hint(&self) -> (usize, Option<usize>) {
672:         self.drain.size_hint()
673:     }
674: }
675: impl<I, K, V, S> DoubleEndedIterator for Splice<'_, I, K, V, S>
676: where
677:     I: Iterator<Item = (K, V)>,
678:     K: Hash + Eq,
679:     S: BuildHasher,
680: {
681:     fn next_back(&mut self) -> Option<Self::Item> {
682:         self.drain.next_back().map(Bucket::key_value)
683:     }
684: }
685: impl<I, K, V, S> ExactSizeIterator for Splice<'_, I, K, V, S>
686: where
687:     I: Iterator<Item = (K, V)>,
688:     K: Hash + Eq,
689:     S: BuildHasher,
690: {
691:     fn len(&self) -> usize {
692:         self.drain.len()
693:     }
694: }
695: impl<I, K, V, S> FusedIterator for Splice<'_, I, K, V, S>
696: where
697:     I: Iterator<Item = (K, V)>,
698:     K: Hash + Eq,
699:     S: BuildHasher,
700: {
701: }
702: impl<I, K, V, S> fmt::Debug for Splice<'_, I, K, V, S>
703: where
704:     I: fmt::Debug + Iterator<Item = (K, V)>,
705:     K: fmt::Debug + Hash + Eq,
706:     V: fmt::Debug,
707:     S: BuildHasher,
708: {
709:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
710:         // Follow `vec::Splice` in only printing the drain and replacement
711:         f.debug_struct("Splice")
712:             .field("drain", &self.drain)
713:             .field("replace_with", &self.replace_with)
714:             .finish()
715:     }
716: }
717: /// An extracting iterator for `IndexMap`.
718: ///
719: /// This `struct` is created by [`IndexMap::extract_if()`].
720: /// See its documentation for more.
721: pub struct ExtractIf<'a, K, V, F> {
722:     inner: ExtractCore<'a, K, V>,
723:     pred: F,
724: }
725: impl<K, V, F> ExtractIf<'_, K, V, F> {
726:     #[track_caller]
727:     pub(super) fn new<R>(core: &mut Core<K, V>, range: R, pred: F) -> ExtractIf<'_, K, V, F>
728:     where
729:         R: RangeBounds<usize>,
730:         F: FnMut(&K, &mut V) -> bool,
731:     {
732:         ExtractIf {
733:             inner: core.extract(range),
734:             pred,
735:         }
736:     }
737: }
738: impl<K, V, F> Iterator for ExtractIf<'_, K, V, F>
739: where
740:     F: FnMut(&K, &mut V) -> bool,
741: {
742:     type Item = (K, V);
743:     fn next(&mut self) -> Option<Self::Item> {
744:         self.inner
745:             .extract_if(|bucket| {
746:                 let (key, value) = bucket.ref_mut();
747:                 (self.pred)(key, value)
748:             })
749:             .map(Bucket::key_value)
750:     }
751:     fn size_hint(&self) -> (usize, Option<usize>) {
752:         (0, Some(self.inner.remaining()))
753:     }
754: }
755: impl<K, V, F> FusedIterator for ExtractIf<'_, K, V, F> where F: FnMut(&K, &mut V) -> bool {}
756: impl<K, V, F> fmt::Debug for ExtractIf<'_, K, V, F>
757: where
758:     K: fmt::Debug,
759:     V: fmt::Debug,
760: {
761:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
762:         f.debug_struct("ExtractIf").finish_non_exhaustive()
763:     }
764: }
````

## File: src/map/mutable.rs
````rust
  1: use core::hash::{BuildHasher, Hash};
  2: use super::{
  3:     Bucket, Entry, Equivalent, IndexMap, IndexedEntry, IterMut2, OccupiedEntry, VacantEntry,
  4: };
  5: /// Opt-in mutable access to [`IndexMap`] keys.
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
 16: /// `use` this trait to enable its methods for `IndexMap`.
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
 48: /// Opt-in mutable access to [`IndexMap`] keys.
 49: ///
 50: /// See [`MutableKeys`] for more information.
 51: impl<K, V, S> MutableKeys for IndexMap<K, V, S>
 52: where
 53:     S: BuildHasher,
 54: {
 55:     type Key = K;
 56:     type Value = V;
 57:     fn get_full_mut2<Q>(&mut self, key: &Q) -> Option<(usize, &mut K, &mut V)>
 58:     where
 59:         Q: ?Sized + Hash + Equivalent<K>,
 60:     {
 61:         if let Some(i) = self.get_index_of(key) {
 62:             let entry = &mut self.as_entries_mut()[i];
 63:             Some((i, &mut entry.key, &mut entry.value))
 64:         } else {
 65:             None
 66:         }
 67:     }
 68:     fn get_index_mut2(&mut self, index: usize) -> Option<(&mut K, &mut V)> {
 69:         self.as_entries_mut().get_mut(index).map(Bucket::muts)
 70:     }
 71:     fn iter_mut2(&mut self) -> IterMut2<'_, Self::Key, Self::Value> {
 72:         IterMut2::new(self.as_entries_mut())
 73:     }
 74:     fn retain2<F>(&mut self, keep: F)
 75:     where
 76:         F: FnMut(&mut K, &mut V) -> bool,
 77:     {
 78:         self.core.retain_in_order(keep);
 79:     }
 80: }
 81: /// Opt-in mutable access to [`Entry`] keys.
 82: ///
 83: /// These methods expose `&mut K`, mutable references to the key as it is stored
 84: /// in the map.
 85: /// You are allowed to modify the keys in the map **if the modification
 86: /// does not change the key's hash and equality**.
 87: ///
 88: /// If keys are modified erroneously, you can no longer look them up.
 89: /// This is sound (memory safe) but a logical error hazard (just like
 90: /// implementing `PartialEq`, `Eq`, or `Hash` incorrectly would be).
 91: ///
 92: /// `use` this trait to enable its methods for `Entry`.
 93: ///
 94: /// This trait is sealed and cannot be implemented for types outside this crate.
 95: #[expect(private_bounds)]
 96: pub trait MutableEntryKey: Sealed {
 97:     type Key;
 98:     /// Gets a mutable reference to the entry's key, either within the map if occupied,
 99:     /// or else the new key that was used to find the entry.
100:     fn key_mut(&mut self) -> &mut Self::Key;
101: }
102: /// Opt-in mutable access to [`Entry`] keys.
103: ///
104: /// See [`MutableEntryKey`] for more information.
105: impl<K, V> MutableEntryKey for Entry<'_, K, V> {
106:     type Key = K;
107:     fn key_mut(&mut self) -> &mut Self::Key {
108:         match self {
109:             Entry::Occupied(e) => e.key_mut(),
110:             Entry::Vacant(e) => e.key_mut(),
111:         }
112:     }
113: }
114: /// Opt-in mutable access to [`OccupiedEntry`] keys.
115: ///
116: /// See [`MutableEntryKey`] for more information.
117: impl<K, V> MutableEntryKey for OccupiedEntry<'_, K, V> {
118:     type Key = K;
119:     fn key_mut(&mut self) -> &mut Self::Key {
120:         &mut self.get_bucket_mut().key
121:     }
122: }
123: /// Opt-in mutable access to [`VacantEntry`] keys.
124: ///
125: /// See [`MutableEntryKey`] for more information.
126: impl<K, V> MutableEntryKey for VacantEntry<'_, K, V> {
127:     type Key = K;
128:     fn key_mut(&mut self) -> &mut Self::Key {
129:         self.key_mut()
130:     }
131: }
132: /// Opt-in mutable access to [`IndexedEntry`] keys.
133: ///
134: /// See [`MutableEntryKey`] for more information.
135: impl<K, V> MutableEntryKey for IndexedEntry<'_, K, V> {
136:     type Key = K;
137:     fn key_mut(&mut self) -> &mut Self::Key {
138:         self.key_mut()
139:     }
140: }
141: trait Sealed {}
142: impl<K, V, S> Sealed for IndexMap<K, V, S> {}
143: impl<K, V> Sealed for Entry<'_, K, V> {}
144: impl<K, V> Sealed for OccupiedEntry<'_, K, V> {}
145: impl<K, V> Sealed for VacantEntry<'_, K, V> {}
146: impl<K, V> Sealed for IndexedEntry<'_, K, V> {}
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
 10: //! `IndexMap` without such an opt-in trait.
 11: use super::{Core, OccupiedEntry};
 12: use crate::{Equivalent, HashValue, IndexMap};
 13: use core::fmt;
 14: use core::hash::{BuildHasher, Hash};
 15: use core::marker::PhantomData;
 16: use core::mem;
 17: /// Opt-in access to the experimental raw entry API.
 18: ///
 19: /// See the [`raw_entry_v1`][self] module documentation for more information.
 20: #[expect(private_bounds)]
 21: pub trait RawEntryApiV1<K, V, S>: Sealed {
 22:     /// Creates a raw immutable entry builder for the [`IndexMap`].
 23:     ///
 24:     /// Raw entries provide the lowest level of control for searching and
 25:     /// manipulating a map. They must be manually initialized with a hash and
 26:     /// then manually searched.
 27:     ///
 28:     /// This is useful for
 29:     /// * Hash memoization
 30:     /// * Using a search key that doesn't work with the [`Equivalent`] trait
 31:     /// * Using custom comparison logic without newtype wrappers
 32:     ///
 33:     /// Unless you are in such a situation, higher-level and more foolproof APIs like
 34:     /// [`get`][IndexMap::get] should be preferred.
 35:     ///
 36:     /// Immutable raw entries have very limited use; you might instead want
 37:     /// [`raw_entry_mut_v1`][Self::raw_entry_mut_v1].
 38:     ///
 39:     /// # Examples
 40:     ///
 41:     /// ```
 42:     /// use core::hash::BuildHasher;
 43:     /// use indexmap::map::{IndexMap, RawEntryApiV1};
 44:     ///
 45:     /// let mut map = IndexMap::new();
 46:     /// map.extend([("a", 100), ("b", 200), ("c", 300)]);
 47:     ///
 48:     /// for k in ["a", "b", "c", "d", "e", "f"] {
 49:     ///     let hash = map.hasher().hash_one(k);
 50:     ///     let i = map.get_index_of(k);
 51:     ///     let v = map.get(k);
 52:     ///     let kv = map.get_key_value(k);
 53:     ///     let ikv = map.get_full(k);
 54:     ///
 55:     ///     println!("Key: {} and value: {:?}", k, v);
 56:     ///
 57:     ///     assert_eq!(map.raw_entry_v1().from_key(k), kv);
 58:     ///     assert_eq!(map.raw_entry_v1().from_hash(hash, |q| *q == k), kv);
 59:     ///     assert_eq!(map.raw_entry_v1().from_key_hashed_nocheck(hash, k), kv);
 60:     ///     assert_eq!(map.raw_entry_v1().from_hash_full(hash, |q| *q == k), ikv);
 61:     ///     assert_eq!(map.raw_entry_v1().index_from_hash(hash, |q| *q == k), i);
 62:     /// }
 63:     /// ```
 64:     fn raw_entry_v1(&self) -> RawEntryBuilder<'_, K, V, S>;
 65:     /// Creates a raw entry builder for the [`IndexMap`].
 66:     ///
 67:     /// Raw entries provide the lowest level of control for searching and
 68:     /// manipulating a map. They must be manually initialized with a hash and
 69:     /// then manually searched. After this, insertions into a vacant entry
 70:     /// still require an owned key to be provided.
 71:     ///
 72:     /// Raw entries are useful for such exotic situations as:
 73:     ///
 74:     /// * Hash memoization
 75:     /// * Deferring the creation of an owned key until it is known to be required
 76:     /// * Using a search key that doesn't work with the [`Equivalent`] trait
 77:     /// * Using custom comparison logic without newtype wrappers
 78:     ///
 79:     /// Because raw entries provide much more low-level control, it's much easier
 80:     /// to put the `IndexMap` into an inconsistent state which, while memory-safe,
 81:     /// will cause the map to produce seemingly random results. Higher-level and more
 82:     /// foolproof APIs like [`entry`][IndexMap::entry] should be preferred when possible.
 83:     ///
 84:     /// Raw entries give mutable access to the keys. This must not be used
 85:     /// to modify how the key would compare or hash, as the map will not re-evaluate
 86:     /// where the key should go, meaning the keys may become "lost" if their
 87:     /// location does not reflect their state. For instance, if you change a key
 88:     /// so that the map now contains keys which compare equal, search may start
 89:     /// acting erratically, with two keys randomly masking each other. Implementations
 90:     /// are free to assume this doesn't happen (within the limits of memory-safety).
 91:     ///
 92:     /// # Examples
 93:     ///
 94:     /// ```
 95:     /// use core::hash::BuildHasher;
 96:     /// use indexmap::map::{IndexMap, RawEntryApiV1};
 97:     /// use indexmap::map::raw_entry_v1::RawEntryMut;
 98:     ///
 99:     /// let mut map = IndexMap::new();
100:     /// map.extend([("a", 100), ("b", 200), ("c", 300)]);
101:     ///
102:     /// // Existing key (insert and update)
103:     /// match map.raw_entry_mut_v1().from_key("a") {
104:     ///     RawEntryMut::Vacant(_) => unreachable!(),
105:     ///     RawEntryMut::Occupied(mut view) => {
106:     ///         assert_eq!(view.index(), 0);
107:     ///         assert_eq!(view.get(), &100);
108:     ///         let v = view.get_mut();
109:     ///         let new_v = (*v) * 10;
110:     ///         *v = new_v;
111:     ///         assert_eq!(view.insert(1111), 1000);
112:     ///     }
113:     /// }
114:     ///
115:     /// assert_eq!(map["a"], 1111);
116:     /// assert_eq!(map.len(), 3);
117:     ///
118:     /// // Existing key (take)
119:     /// let hash = map.hasher().hash_one("c");
120:     /// match map.raw_entry_mut_v1().from_key_hashed_nocheck(hash, "c") {
121:     ///     RawEntryMut::Vacant(_) => unreachable!(),
122:     ///     RawEntryMut::Occupied(view) => {
123:     ///         assert_eq!(view.index(), 2);
124:     ///         assert_eq!(view.shift_remove_entry(), ("c", 300));
125:     ///     }
126:     /// }
127:     /// assert_eq!(map.raw_entry_v1().from_key("c"), None);
128:     /// assert_eq!(map.len(), 2);
129:     ///
130:     /// // Nonexistent key (insert and update)
131:     /// let key = "d";
132:     /// let hash = map.hasher().hash_one(key);
133:     /// match map.raw_entry_mut_v1().from_hash(hash, |q| *q == key) {
134:     ///     RawEntryMut::Occupied(_) => unreachable!(),
135:     ///     RawEntryMut::Vacant(view) => {
136:     ///         assert_eq!(view.index(), 2);
137:     ///         let (k, value) = view.insert("d", 4000);
138:     ///         assert_eq!((*k, *value), ("d", 4000));
139:     ///         *value = 40000;
140:     ///     }
141:     /// }
142:     /// assert_eq!(map["d"], 40000);
143:     /// assert_eq!(map.len(), 3);
144:     ///
145:     /// match map.raw_entry_mut_v1().from_hash(hash, |q| *q == key) {
146:     ///     RawEntryMut::Vacant(_) => unreachable!(),
147:     ///     RawEntryMut::Occupied(view) => {
148:     ///         assert_eq!(view.index(), 2);
149:     ///         assert_eq!(view.swap_remove_entry(), ("d", 40000));
150:     ///     }
151:     /// }
152:     /// assert_eq!(map.get("d"), None);
153:     /// assert_eq!(map.len(), 2);
154:     /// ```
155:     fn raw_entry_mut_v1(&mut self) -> RawEntryBuilderMut<'_, K, V, S>;
156: }
157: impl<K, V, S> RawEntryApiV1<K, V, S> for IndexMap<K, V, S> {
158:     fn raw_entry_v1(&self) -> RawEntryBuilder<'_, K, V, S> {
159:         RawEntryBuilder { map: self }
160:     }
161:     fn raw_entry_mut_v1(&mut self) -> RawEntryBuilderMut<'_, K, V, S> {
162:         RawEntryBuilderMut { map: self }
163:     }
164: }
165: /// A builder for computing where in an [`IndexMap`] a key-value pair would be stored.
166: ///
167: /// This `struct` is created by the [`IndexMap::raw_entry_v1`] method, provided by the
168: /// [`RawEntryApiV1`] trait. See its documentation for more.
169: pub struct RawEntryBuilder<'a, K, V, S> {
170:     map: &'a IndexMap<K, V, S>,
171: }
172: impl<K, V, S> fmt::Debug for RawEntryBuilder<'_, K, V, S> {
173:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
174:         f.debug_struct("RawEntryBuilder").finish_non_exhaustive()
175:     }
176: }
177: impl<'a, K, V, S> RawEntryBuilder<'a, K, V, S> {
178:     /// Access an entry by key.
179:     pub fn from_key<Q>(self, key: &Q) -> Option<(&'a K, &'a V)>
180:     where
181:         S: BuildHasher,
182:         Q: ?Sized + Hash + Equivalent<K>,
183:     {
184:         self.map.get_key_value(key)
185:     }
186:     /// Access an entry by a key and its hash.
187:     pub fn from_key_hashed_nocheck<Q>(self, hash: u64, key: &Q) -> Option<(&'a K, &'a V)>
188:     where
189:         Q: ?Sized + Equivalent<K>,
190:     {
191:         let hash = HashValue(hash as usize);
192:         let i = self.map.core.get_index_of(hash, key)?;
193:         self.map.get_index(i)
194:     }
195:     /// Access an entry by hash.
196:     pub fn from_hash<F>(self, hash: u64, is_match: F) -> Option<(&'a K, &'a V)>
197:     where
198:         F: FnMut(&K) -> bool,
199:     {
200:         let map = self.map;
201:         let i = self.index_from_hash(hash, is_match)?;
202:         map.get_index(i)
203:     }
204:     /// Access an entry by hash, including its index.
205:     pub fn from_hash_full<F>(self, hash: u64, is_match: F) -> Option<(usize, &'a K, &'a V)>
206:     where
207:         F: FnMut(&K) -> bool,
208:     {
209:         let map = self.map;
210:         let i = self.index_from_hash(hash, is_match)?;
211:         let (key, value) = map.get_index(i)?;
212:         Some((i, key, value))
213:     }
214:     /// Access the index of an entry by hash.
215:     pub fn index_from_hash<F>(self, hash: u64, is_match: F) -> Option<usize>
216:     where
217:         F: FnMut(&K) -> bool,
218:     {
219:         let hash = HashValue(hash as usize);
220:         self.map.core.get_index_of_raw(hash, is_match)
221:     }
222: }
223: /// A builder for computing where in an [`IndexMap`] a key-value pair would be stored.
224: ///
225: /// This `struct` is created by the [`IndexMap::raw_entry_mut_v1`] method, provided by the
226: /// [`RawEntryApiV1`] trait. See its documentation for more.
227: pub struct RawEntryBuilderMut<'a, K, V, S> {
228:     map: &'a mut IndexMap<K, V, S>,
229: }
230: impl<K, V, S> fmt::Debug for RawEntryBuilderMut<'_, K, V, S> {
231:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
232:         f.debug_struct("RawEntryBuilderMut").finish_non_exhaustive()
233:     }
234: }
235: impl<'a, K, V, S> RawEntryBuilderMut<'a, K, V, S> {
236:     /// Access an entry by key.
237:     pub fn from_key<Q>(self, key: &Q) -> RawEntryMut<'a, K, V, S>
238:     where
239:         S: BuildHasher,
240:         Q: ?Sized + Hash + Equivalent<K>,
241:     {
242:         let hash = self.map.hash(key);
243:         self.from_key_hashed_nocheck(hash.get(), key)
244:     }
245:     /// Access an entry by a key and its hash.
246:     pub fn from_key_hashed_nocheck<Q>(self, hash: u64, key: &Q) -> RawEntryMut<'a, K, V, S>
247:     where
248:         Q: ?Sized + Equivalent<K>,
249:     {
250:         self.from_hash(hash, |k| Q::equivalent(key, k))
251:     }
252:     /// Access an entry by hash.
253:     pub fn from_hash<F>(self, hash: u64, is_match: F) -> RawEntryMut<'a, K, V, S>
254:     where
255:         F: FnMut(&K) -> bool,
256:     {
257:         let hash = HashValue(hash as usize);
258:         match OccupiedEntry::from_hash(&mut self.map.core, hash, is_match) {
259:             Ok(inner) => RawEntryMut::Occupied(RawOccupiedEntryMut {
260:                 inner,
261:                 hash_builder: PhantomData,
262:             }),
263:             Err(map) => RawEntryMut::Vacant(RawVacantEntryMut {
264:                 map,
265:                 hash_builder: &self.map.hash_builder,
266:             }),
267:         }
268:     }
269: }
270: /// Raw entry for an existing key-value pair or a vacant location to
271: /// insert one.
272: pub enum RawEntryMut<'a, K, V, S> {
273:     /// Existing slot with equivalent key.
274:     Occupied(RawOccupiedEntryMut<'a, K, V, S>),
275:     /// Vacant slot (no equivalent key in the map).
276:     Vacant(RawVacantEntryMut<'a, K, V, S>),
277: }
278: impl<K: fmt::Debug, V: fmt::Debug, S> fmt::Debug for RawEntryMut<'_, K, V, S> {
279:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
280:         let mut tuple = f.debug_tuple("RawEntryMut");
281:         match self {
282:             Self::Vacant(v) => tuple.field(v),
283:             Self::Occupied(o) => tuple.field(o),
284:         };
285:         tuple.finish()
286:     }
287: }
288: impl<'a, K, V, S> RawEntryMut<'a, K, V, S> {
289:     /// Return the index where the key-value pair exists or may be inserted.
290:     #[inline]
291:     pub fn index(&self) -> usize {
292:         match self {
293:             Self::Occupied(entry) => entry.index(),
294:             Self::Vacant(entry) => entry.index(),
295:         }
296:     }
297:     /// Inserts the given default key and value in the entry if it is vacant and returns mutable
298:     /// references to them. Otherwise mutable references to an already existent pair are returned.
299:     pub fn or_insert(self, default_key: K, default_value: V) -> (&'a mut K, &'a mut V)
300:     where
301:         K: Hash,
302:         S: BuildHasher,
303:     {
304:         match self {
305:             Self::Occupied(entry) => entry.into_key_value_mut(),
306:             Self::Vacant(entry) => entry.insert(default_key, default_value),
307:         }
308:     }
309:     /// Inserts the result of the `call` function in the entry if it is vacant and returns mutable
310:     /// references to them. Otherwise mutable references to an already existent pair are returned.
311:     pub fn or_insert_with<F>(self, call: F) -> (&'a mut K, &'a mut V)
312:     where
313:         F: FnOnce() -> (K, V),
314:         K: Hash,
315:         S: BuildHasher,
316:     {
317:         match self {
318:             Self::Occupied(entry) => entry.into_key_value_mut(),
319:             Self::Vacant(entry) => {
320:                 let (key, value) = call();
321:                 entry.insert(key, value)
322:             }
323:         }
324:     }
325:     /// Modifies the entry if it is occupied.
326:     pub fn and_modify<F>(mut self, f: F) -> Self
327:     where
328:         F: FnOnce(&mut K, &mut V),
329:     {
330:         if let Self::Occupied(entry) = &mut self {
331:             let (k, v) = entry.get_key_value_mut();
332:             f(k, v);
333:         }
334:         self
335:     }
336: }
337: /// A raw view into an occupied entry in an [`IndexMap`].
338: /// It is part of the [`RawEntryMut`] enum.
339: pub struct RawOccupiedEntryMut<'a, K, V, S> {
340:     inner: OccupiedEntry<'a, K, V>,
341:     hash_builder: PhantomData<&'a S>,
342: }
343: impl<K: fmt::Debug, V: fmt::Debug, S> fmt::Debug for RawOccupiedEntryMut<'_, K, V, S> {
344:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
345:         f.debug_struct("RawOccupiedEntryMut")
346:             .field("key", self.key())
347:             .field("value", self.get())
348:             .finish_non_exhaustive()
349:     }
350: }
351: impl<'a, K, V, S> RawOccupiedEntryMut<'a, K, V, S> {
352:     /// Return the index of the key-value pair
353:     #[inline]
354:     pub fn index(&self) -> usize {
355:         self.inner.index()
356:     }
357:     /// Gets a reference to the entry's key in the map.
358:     ///
359:     /// Note that this is not the key that was used to find the entry. There may be an observable
360:     /// difference if the key type has any distinguishing features outside of `Hash` and `Eq`, like
361:     /// extra fields or the memory address of an allocation.
362:     pub fn key(&self) -> &K {
363:         self.inner.key()
364:     }
365:     /// Gets a mutable reference to the entry's key in the map.
366:     ///
367:     /// Note that this is not the key that was used to find the entry. There may be an observable
368:     /// difference if the key type has any distinguishing features outside of `Hash` and `Eq`, like
369:     /// extra fields or the memory address of an allocation.
370:     pub fn key_mut(&mut self) -> &mut K {
371:         &mut self.inner.get_bucket_mut().key
372:     }
373:     /// Converts into a mutable reference to the entry's key in the map,
374:     /// with a lifetime bound to the map itself.
375:     ///
376:     /// Note that this is not the key that was used to find the entry. There may be an observable
377:     /// difference if the key type has any distinguishing features outside of `Hash` and `Eq`, like
378:     /// extra fields or the memory address of an allocation.
379:     pub fn into_key(self) -> &'a mut K {
380:         &mut self.inner.into_bucket().key
381:     }
382:     /// Gets a reference to the entry's value in the map.
383:     pub fn get(&self) -> &V {
384:         self.inner.get()
385:     }
386:     /// Gets a mutable reference to the entry's value in the map.
387:     ///
388:     /// If you need a reference which may outlive the destruction of the
389:     /// [`RawEntryMut`] value, see [`into_mut`][Self::into_mut].
390:     pub fn get_mut(&mut self) -> &mut V {
391:         self.inner.get_mut()
392:     }
393:     /// Converts into a mutable reference to the entry's value in the map,
394:     /// with a lifetime bound to the map itself.
395:     pub fn into_mut(self) -> &'a mut V {
396:         self.inner.into_mut()
397:     }
398:     /// Gets a reference to the entry's key and value in the map.
399:     pub fn get_key_value(&self) -> (&K, &V) {
400:         self.inner.get_bucket().refs()
401:     }
402:     /// Gets a reference to the entry's key and value in the map.
403:     pub fn get_key_value_mut(&mut self) -> (&mut K, &mut V) {
404:         self.inner.get_bucket_mut().muts()
405:     }
406:     /// Converts into a mutable reference to the entry's key and value in the map,
407:     /// with a lifetime bound to the map itself.
408:     pub fn into_key_value_mut(self) -> (&'a mut K, &'a mut V) {
409:         self.inner.into_bucket().muts()
410:     }
411:     /// Sets the value of the entry, and returns the entry's old value.
412:     pub fn insert(&mut self, value: V) -> V {
413:         self.inner.insert(value)
414:     }
415:     /// Sets the key of the entry, and returns the entry's old key.
416:     pub fn insert_key(&mut self, key: K) -> K {
417:         mem::replace(self.key_mut(), key)
418:     }
419:     /// Remove the key, value pair stored in the map for this entry, and return the value.
420:     ///
421:     /// **NOTE:** This is equivalent to [`.swap_remove()`][Self::swap_remove], replacing this
422:     /// entry's position with the last element, and it is deprecated in favor of calling that
423:     /// explicitly. If you need to preserve the relative order of the keys in the map, use
424:     /// [`.shift_remove()`][Self::shift_remove] instead.
425:     #[deprecated(note = "`remove` disrupts the map order -- \
426:         use `swap_remove` or `shift_remove` for explicit behavior.")]
427:     pub fn remove(self) -> V {
428:         self.swap_remove()
429:     }
430:     /// Remove the key, value pair stored in the map for this entry, and return the value.
431:     ///
432:     /// Like [`Vec::swap_remove`][alloc::vec::Vec::swap_remove], the pair is removed by swapping it
433:     /// with the last element of the map and popping it off.
434:     /// **This perturbs the position of what used to be the last element!**
435:     ///
436:     /// Computes in **O(1)** time (average).
437:     pub fn swap_remove(self) -> V {
438:         self.inner.swap_remove()
439:     }
440:     /// Remove the key, value pair stored in the map for this entry, and return the value.
441:     ///
442:     /// Like [`Vec::remove`][alloc::vec::Vec::remove], the pair is removed by shifting all of the
443:     /// elements that follow it, preserving their relative order.
444:     /// **This perturbs the index of all of those elements!**
445:     ///
446:     /// Computes in **O(n)** time (average).
447:     pub fn shift_remove(self) -> V {
448:         self.inner.shift_remove()
449:     }
450:     /// Remove and return the key, value pair stored in the map for this entry
451:     ///
452:     /// **NOTE:** This is equivalent to [`.swap_remove_entry()`][Self::swap_remove_entry],
453:     /// replacing this entry's position with the last element, and it is deprecated in favor of
454:     /// calling that explicitly. If you need to preserve the relative order of the keys in the map,
455:     /// use [`.shift_remove_entry()`][Self::shift_remove_entry] instead.
456:     #[deprecated(note = "`remove_entry` disrupts the map order -- \
457:         use `swap_remove_entry` or `shift_remove_entry` for explicit behavior.")]
458:     pub fn remove_entry(self) -> (K, V) {
459:         self.swap_remove_entry()
460:     }
461:     /// Remove and return the key, value pair stored in the map for this entry
462:     ///
463:     /// Like [`Vec::swap_remove`][alloc::vec::Vec::swap_remove], the pair is removed by swapping it
464:     /// with the last element of the map and popping it off.
465:     /// **This perturbs the position of what used to be the last element!**
466:     ///
467:     /// Computes in **O(1)** time (average).
468:     pub fn swap_remove_entry(self) -> (K, V) {
469:         self.inner.swap_remove_entry()
470:     }
471:     /// Remove and return the key, value pair stored in the map for this entry
472:     ///
473:     /// Like [`Vec::remove`][alloc::vec::Vec::remove], the pair is removed by shifting all of the
474:     /// elements that follow it, preserving their relative order.
475:     /// **This perturbs the index of all of those elements!**
476:     ///
477:     /// Computes in **O(n)** time (average).
478:     pub fn shift_remove_entry(self) -> (K, V) {
479:         self.inner.shift_remove_entry()
480:     }
481:     /// Moves the position of the entry to a new index
482:     /// by shifting all other entries in-between.
483:     ///
484:     /// This is equivalent to [`IndexMap::move_index`]
485:     /// coming `from` the current [`.index()`][Self::index].
486:     ///
487:     /// * If `self.index() < to`, the other pairs will shift down while the targeted pair moves up.
488:     /// * If `self.index() > to`, the other pairs will shift up while the targeted pair moves down.
489:     ///
490:     /// ***Panics*** if `to` is out of bounds.
491:     ///
492:     /// Computes in **O(n)** time (average).
493:     #[track_caller]
494:     pub fn move_index(self, to: usize) {
495:         self.inner.move_index(to);
496:     }
497:     /// Swaps the position of entry with another.
498:     ///
499:     /// This is equivalent to [`IndexMap::swap_indices`]
500:     /// with the current [`.index()`][Self::index] as one of the two being swapped.
501:     ///
502:     /// ***Panics*** if the `other` index is out of bounds.
503:     ///
504:     /// Computes in **O(1)** time (average).
505:     #[track_caller]
506:     pub fn swap_indices(self, other: usize) {
507:         self.inner.swap_indices(other);
508:     }
509: }
510: /// A view into a vacant raw entry in an [`IndexMap`].
511: /// It is part of the [`RawEntryMut`] enum.
512: pub struct RawVacantEntryMut<'a, K, V, S> {
513:     map: &'a mut Core<K, V>,
514:     hash_builder: &'a S,
515: }
516: impl<K, V, S> fmt::Debug for RawVacantEntryMut<'_, K, V, S> {
517:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
518:         f.debug_struct("RawVacantEntryMut").finish_non_exhaustive()
519:     }
520: }
521: impl<'a, K, V, S> RawVacantEntryMut<'a, K, V, S> {
522:     /// Return the index where a key-value pair may be inserted.
523:     pub fn index(&self) -> usize {
524:         self.map.len()
525:     }
526:     /// Inserts the given key and value into the map,
527:     /// and returns mutable references to them.
528:     pub fn insert(self, key: K, value: V) -> (&'a mut K, &'a mut V)
529:     where
530:         K: Hash,
531:         S: BuildHasher,
532:     {
533:         let h = self.hash_builder.hash_one(&key);
534:         self.insert_hashed_nocheck(h, key, value)
535:     }
536:     /// Inserts the given key and value into the map with the provided hash,
537:     /// and returns mutable references to them.
538:     pub fn insert_hashed_nocheck(self, hash: u64, key: K, value: V) -> (&'a mut K, &'a mut V) {
539:         let hash = HashValue(hash as usize);
540:         self.map.insert_unique(hash, key, value).muts()
541:     }
542:     /// Inserts the given key and value into the map at the given index,
543:     /// shifting others to the right, and returns mutable references to them.
544:     ///
545:     /// ***Panics*** if `index` is out of bounds.
546:     ///
547:     /// Computes in **O(n)** time (average).
548:     #[track_caller]
549:     pub fn shift_insert(self, index: usize, key: K, value: V) -> (&'a mut K, &'a mut V)
550:     where
551:         K: Hash,
552:         S: BuildHasher,
553:     {
554:         let h = self.hash_builder.hash_one(&key);
555:         self.shift_insert_hashed_nocheck(index, h, key, value)
556:     }
557:     /// Inserts the given key and value into the map with the provided hash
558:     /// at the given index, and returns mutable references to them.
559:     ///
560:     /// ***Panics*** if `index` is out of bounds.
561:     ///
562:     /// Computes in **O(n)** time (average).
563:     #[track_caller]
564:     pub fn shift_insert_hashed_nocheck(
565:         self,
566:         index: usize,
567:         hash: u64,
568:         key: K,
569:         value: V,
570:     ) -> (&'a mut K, &'a mut V) {
571:         let hash = HashValue(hash as usize);
572:         self.map.shift_insert_unique(index, hash, key, value).muts()
573:     }
574: }
575: trait Sealed {}
576: impl<K, V, S> Sealed for IndexMap<K, V, S> {}
577: #[test]
578: fn assert_send_sync() {
579:     fn assert_send_sync<T: Send + Sync>() {}
580:     assert_send_sync::<RawEntryMut<'_, i32, i32, ()>>();
581: }
````

## File: src/map/serde_seq.rs
````rust
  1: //! Functions to serialize and deserialize an [`IndexMap`] as an ordered sequence.
  2: //!
  3: //! The default `serde` implementation serializes `IndexMap` as a normal map,
  4: //! but there is no guarantee that serialization formats will preserve the order
  5: //! of the key-value pairs. This module serializes `IndexMap` as a sequence of
  6: //! `(key, value)` elements instead, in order.
  7: //!
  8: //! This module may be used in a field attribute for derived implementations:
  9: //!
 10: //! ```
 11: //! # use indexmap::IndexMap;
 12: //! # use serde::{Deserialize, Serialize};
 13: //! #[derive(Deserialize, Serialize)]
 14: //! struct Data {
 15: //!     #[serde(with = "indexmap::map::serde_seq")]
 16: //!     map: IndexMap<i32, u64>,
 17: //!     // ...
 18: //! }
 19: //! ```
 20: use serde_core::de::{Deserialize, Deserializer, SeqAccess, Visitor};
 21: use serde_core::ser::{Serialize, Serializer};
 22: use core::fmt::{self, Formatter};
 23: use core::hash::{BuildHasher, Hash};
 24: use core::marker::PhantomData;
 25: use crate::IndexMap;
 26: use crate::map::Slice as MapSlice;
 27: use crate::serde::cautious_capacity;
 28: use crate::set::Slice as SetSlice;
 29: /// Serializes a [`map::Slice`][MapSlice] as an ordered sequence.
 30: ///
 31: /// This behaves like [`crate::map::serde_seq`] for `IndexMap`, serializing a sequence
 32: /// of `(key, value)` pairs, rather than as a map that might not preserve order.
 33: impl<K, V> Serialize for MapSlice<K, V>
 34: where
 35:     K: Serialize,
 36:     V: Serialize,
 37: {
 38:     fn serialize<T>(&self, serializer: T) -> Result<T::Ok, T::Error>
 39:     where
 40:         T: Serializer,
 41:     {
 42:         serializer.collect_seq(self)
 43:     }
 44: }
 45: /// Serializes a [`set::Slice`][SetSlice] as an ordered sequence.
 46: impl<T> Serialize for SetSlice<T>
 47: where
 48:     T: Serialize,
 49: {
 50:     fn serialize<Se>(&self, serializer: Se) -> Result<Se::Ok, Se::Error>
 51:     where
 52:         Se: Serializer,
 53:     {
 54:         serializer.collect_seq(self)
 55:     }
 56: }
 57: /// Serializes an [`IndexMap`] as an ordered sequence.
 58: ///
 59: /// This function may be used in a field attribute for deriving [`Serialize`]:
 60: ///
 61: /// ```
 62: /// # use indexmap::IndexMap;
 63: /// # use serde::Serialize;
 64: /// #[derive(Serialize)]
 65: /// struct Data {
 66: ///     #[serde(serialize_with = "indexmap::map::serde_seq::serialize")]
 67: ///     map: IndexMap<i32, u64>,
 68: ///     // ...
 69: /// }
 70: /// ```
 71: pub fn serialize<K, V, S, T>(map: &IndexMap<K, V, S>, serializer: T) -> Result<T::Ok, T::Error>
 72: where
 73:     K: Serialize,
 74:     V: Serialize,
 75:     T: Serializer,
 76: {
 77:     serializer.collect_seq(map)
 78: }
 79: /// Visitor to deserialize a *sequenced* `IndexMap`
 80: struct SeqVisitor<K, V, S>(PhantomData<(K, V, S)>);
 81: impl<'de, K, V, S> Visitor<'de> for SeqVisitor<K, V, S>
 82: where
 83:     K: Deserialize<'de> + Eq + Hash,
 84:     V: Deserialize<'de>,
 85:     S: Default + BuildHasher,
 86: {
 87:     type Value = IndexMap<K, V, S>;
 88:     fn expecting(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
 89:         write!(formatter, "a sequenced map")
 90:     }
 91:     fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
 92:     where
 93:         A: SeqAccess<'de>,
 94:     {
 95:         let capacity = cautious_capacity::<K, V>(seq.size_hint());
 96:         let mut map = IndexMap::with_capacity_and_hasher(capacity, S::default());
 97:         while let Some((key, value)) = seq.next_element()? {
 98:             map.insert(key, value);
 99:         }
100:         Ok(map)
101:     }
102: }
103: /// Deserializes an [`IndexMap`] from an ordered sequence.
104: ///
105: /// This function may be used in a field attribute for deriving [`Deserialize`]:
106: ///
107: /// ```
108: /// # use indexmap::IndexMap;
109: /// # use serde::Deserialize;
110: /// #[derive(Deserialize)]
111: /// struct Data {
112: ///     #[serde(deserialize_with = "indexmap::map::serde_seq::deserialize")]
113: ///     map: IndexMap<i32, u64>,
114: ///     // ...
115: /// }
116: /// ```
117: pub fn deserialize<'de, D, K, V, S>(deserializer: D) -> Result<IndexMap<K, V, S>, D::Error>
118: where
119:     D: Deserializer<'de>,
120:     K: Deserialize<'de> + Eq + Hash,
121:     V: Deserialize<'de>,
122:     S: Default + BuildHasher,
123: {
124:     deserializer.deserialize_seq(SeqVisitor(PhantomData))
125: }
````

## File: src/map/slice.rs
````rust
  1: use super::{
  2:     Bucket, IndexMap, IntoIter, IntoKeys, IntoValues, Iter, IterMut, Keys, Values, ValuesMut,
  3: };
  4: use crate::GetDisjointMutError;
  5: use crate::util::{slice_eq, try_simplify_range};
  6: use alloc::boxed::Box;
  7: use alloc::vec::Vec;
  8: use core::cmp::Ordering;
  9: use core::fmt;
 10: use core::hash::{Hash, Hasher};
 11: use core::ops::{self, Bound, Index, IndexMut, RangeBounds};
 12: /// A dynamically-sized slice of key-value pairs in an [`IndexMap`].
 13: ///
 14: /// This supports indexed operations much like a `[(K, V)]` slice,
 15: /// but not any hashed operations on the map keys.
 16: ///
 17: /// Unlike `IndexMap`, `Slice` does consider the order for [`PartialEq`]
 18: /// and [`Eq`], and it also implements [`PartialOrd`], [`Ord`], and [`Hash`].
 19: #[repr(transparent)]
 20: pub struct Slice<K, V> {
 21:     pub(crate) entries: [Bucket<K, V>],
 22: }
 23: // SAFETY: `Slice<K, V>` is a transparent wrapper around `[Bucket<K, V>]`,
 24: // and reference lifetimes are bound together in function signatures.
 25: #[allow(unsafe_code)]
 26: impl<K, V> Slice<K, V> {
 27:     pub(crate) const fn from_slice(entries: &[Bucket<K, V>]) -> &Self {
 28:         unsafe { &*(entries as *const [Bucket<K, V>] as *const Self) }
 29:     }
 30:     pub(super) const fn from_mut_slice(entries: &mut [Bucket<K, V>]) -> &mut Self {
 31:         unsafe { &mut *(entries as *mut [Bucket<K, V>] as *mut Self) }
 32:     }
 33:     pub(super) fn from_boxed(entries: Box<[Bucket<K, V>]>) -> Box<Self> {
 34:         unsafe { Box::from_raw(Box::into_raw(entries) as *mut Self) }
 35:     }
 36:     fn into_boxed(self: Box<Self>) -> Box<[Bucket<K, V>]> {
 37:         unsafe { Box::from_raw(Box::into_raw(self) as *mut [Bucket<K, V>]) }
 38:     }
 39: }
 40: impl<K, V> Slice<K, V> {
 41:     pub(crate) fn into_entries(self: Box<Self>) -> Vec<Bucket<K, V>> {
 42:         self.into_boxed().into_vec()
 43:     }
 44:     /// Returns an empty slice.
 45:     pub const fn new<'a>() -> &'a Self {
 46:         Self::from_slice(&[])
 47:     }
 48:     /// Returns an empty mutable slice.
 49:     pub const fn new_mut<'a>() -> &'a mut Self {
 50:         Self::from_mut_slice(&mut [])
 51:     }
 52:     /// Return the number of key-value pairs in the map slice.
 53:     #[inline]
 54:     pub const fn len(&self) -> usize {
 55:         self.entries.len()
 56:     }
 57:     /// Returns true if the map slice contains no elements.
 58:     #[inline]
 59:     pub const fn is_empty(&self) -> bool {
 60:         self.entries.is_empty()
 61:     }
 62:     /// Get a key-value pair by index.
 63:     ///
 64:     /// Valid indices are `0 <= index < self.len()`.
 65:     pub fn get_index(&self, index: usize) -> Option<(&K, &V)> {
 66:         self.entries.get(index).map(Bucket::refs)
 67:     }
 68:     /// Get a key-value pair by index, with mutable access to the value.
 69:     ///
 70:     /// Valid indices are `0 <= index < self.len()`.
 71:     pub fn get_index_mut(&mut self, index: usize) -> Option<(&K, &mut V)> {
 72:         self.entries.get_mut(index).map(Bucket::ref_mut)
 73:     }
 74:     /// Returns a slice of key-value pairs in the given range of indices.
 75:     ///
 76:     /// Valid indices are `0 <= index < self.len()`.
 77:     pub fn get_range<R: RangeBounds<usize>>(&self, range: R) -> Option<&Self> {
 78:         let range = try_simplify_range(range, self.entries.len())?;
 79:         self.entries.get(range).map(Slice::from_slice)
 80:     }
 81:     /// Returns a mutable slice of key-value pairs in the given range of indices.
 82:     ///
 83:     /// Valid indices are `0 <= index < self.len()`.
 84:     pub fn get_range_mut<R: RangeBounds<usize>>(&mut self, range: R) -> Option<&mut Self> {
 85:         let range = try_simplify_range(range, self.entries.len())?;
 86:         self.entries.get_mut(range).map(Slice::from_mut_slice)
 87:     }
 88:     /// Get the first key-value pair.
 89:     pub const fn first(&self) -> Option<(&K, &V)> {
 90:         if let [first, ..] = &self.entries {
 91:             Some(first.refs())
 92:         } else {
 93:             None
 94:         }
 95:     }
 96:     /// Get the first key-value pair, with mutable access to the value.
 97:     pub const fn first_mut(&mut self) -> Option<(&K, &mut V)> {
 98:         if let [first, ..] = &mut self.entries {
 99:             Some(first.ref_mut())
100:         } else {
101:             None
102:         }
103:     }
104:     /// Get the last key-value pair.
105:     pub const fn last(&self) -> Option<(&K, &V)> {
106:         if let [.., last] = &self.entries {
107:             Some(last.refs())
108:         } else {
109:             None
110:         }
111:     }
112:     /// Get the last key-value pair, with mutable access to the value.
113:     pub const fn last_mut(&mut self) -> Option<(&K, &mut V)> {
114:         if let [.., last] = &mut self.entries {
115:             Some(last.ref_mut())
116:         } else {
117:             None
118:         }
119:     }
120:     /// Divides one slice into two at an index.
121:     ///
122:     /// ***Panics*** if `index > len`.
123:     /// For a non-panicking alternative see [`split_at_checked`][Self::split_at_checked].
124:     #[track_caller]
125:     pub const fn split_at(&self, index: usize) -> (&Self, &Self) {
126:         let (first, second) = self.entries.split_at(index);
127:         (Self::from_slice(first), Self::from_slice(second))
128:     }
129:     /// Divides one mutable slice into two at an index.
130:     ///
131:     /// ***Panics*** if `index > len`.
132:     /// For a non-panicking alternative see [`split_at_mut_checked`][Self::split_at_mut_checked].
133:     #[track_caller]
134:     pub const fn split_at_mut(&mut self, index: usize) -> (&mut Self, &mut Self) {
135:         let (first, second) = self.entries.split_at_mut(index);
136:         (Self::from_mut_slice(first), Self::from_mut_slice(second))
137:     }
138:     /// Divides one slice into two at an index.
139:     ///
140:     /// Returns `None` if `index > len`.
141:     pub const fn split_at_checked(&self, index: usize) -> Option<(&Self, &Self)> {
142:         if let Some((first, second)) = self.entries.split_at_checked(index) {
143:             Some((Self::from_slice(first), Self::from_slice(second)))
144:         } else {
145:             None
146:         }
147:     }
148:     /// Divides one mutable slice into two at an index.
149:     ///
150:     /// Returns `None` if `index > len`.
151:     pub const fn split_at_mut_checked(&mut self, index: usize) -> Option<(&mut Self, &mut Self)> {
152:         if let Some((first, second)) = self.entries.split_at_mut_checked(index) {
153:             Some((Self::from_mut_slice(first), Self::from_mut_slice(second)))
154:         } else {
155:             None
156:         }
157:     }
158:     /// Returns the first key-value pair and the rest of the slice,
159:     /// or `None` if it is empty.
160:     pub const fn split_first(&self) -> Option<((&K, &V), &Self)> {
161:         if let [first, rest @ ..] = &self.entries {
162:             Some((first.refs(), Self::from_slice(rest)))
163:         } else {
164:             None
165:         }
166:     }
167:     /// Returns the first key-value pair and the rest of the slice,
168:     /// with mutable access to the value, or `None` if it is empty.
169:     pub const fn split_first_mut(&mut self) -> Option<((&K, &mut V), &mut Self)> {
170:         if let [first, rest @ ..] = &mut self.entries {
171:             Some((first.ref_mut(), Self::from_mut_slice(rest)))
172:         } else {
173:             None
174:         }
175:     }
176:     /// Returns the last key-value pair and the rest of the slice,
177:     /// or `None` if it is empty.
178:     pub const fn split_last(&self) -> Option<((&K, &V), &Self)> {
179:         if let [rest @ .., last] = &self.entries {
180:             Some((last.refs(), Self::from_slice(rest)))
181:         } else {
182:             None
183:         }
184:     }
185:     /// Returns the last key-value pair and the rest of the slice,
186:     /// with mutable access to the value, or `None` if it is empty.
187:     pub const fn split_last_mut(&mut self) -> Option<((&K, &mut V), &mut Self)> {
188:         if let [rest @ .., last] = &mut self.entries {
189:             Some((last.ref_mut(), Self::from_mut_slice(rest)))
190:         } else {
191:             None
192:         }
193:     }
194:     /// Return an iterator over the key-value pairs of the map slice.
195:     pub fn iter(&self) -> Iter<'_, K, V> {
196:         Iter::new(&self.entries)
197:     }
198:     /// Return an iterator over the key-value pairs of the map slice.
199:     pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
200:         IterMut::new(&mut self.entries)
201:     }
202:     /// Return an iterator over the keys of the map slice.
203:     pub fn keys(&self) -> Keys<'_, K, V> {
204:         Keys::new(&self.entries)
205:     }
206:     /// Return an owning iterator over the keys of the map slice.
207:     pub fn into_keys(self: Box<Self>) -> IntoKeys<K, V> {
208:         IntoKeys::new(self.into_entries())
209:     }
210:     /// Return an iterator over the values of the map slice.
211:     pub fn values(&self) -> Values<'_, K, V> {
212:         Values::new(&self.entries)
213:     }
214:     /// Return an iterator over mutable references to the values of the map slice.
215:     pub fn values_mut(&mut self) -> ValuesMut<'_, K, V> {
216:         ValuesMut::new(&mut self.entries)
217:     }
218:     /// Return an owning iterator over the values of the map slice.
219:     pub fn into_values(self: Box<Self>) -> IntoValues<K, V> {
220:         IntoValues::new(self.into_entries())
221:     }
222:     /// Search over a sorted map for a key.
223:     ///
224:     /// Returns the position where that key is present, or the position where it can be inserted to
225:     /// maintain the sort. See [`slice::binary_search`] for more details.
226:     ///
227:     /// Computes in **O(log(n))** time, which is notably less scalable than looking the key up in
228:     /// the map this is a slice from using [`IndexMap::get_index_of`], but this can also position
229:     /// missing keys.
230:     pub fn binary_search_keys(&self, x: &K) -> Result<usize, usize>
231:     where
232:         K: Ord,
233:     {
234:         self.binary_search_by(|p, _| p.cmp(x))
235:     }
236:     /// Search over a sorted map with a comparator function.
237:     ///
238:     /// Returns the position where that value is present, or the position where it can be inserted
239:     /// to maintain the sort. See [`slice::binary_search_by`] for more details.
240:     ///
241:     /// Computes in **O(log(n))** time.
242:     #[inline]
243:     pub fn binary_search_by<'a, F>(&'a self, mut f: F) -> Result<usize, usize>
244:     where
245:         F: FnMut(&'a K, &'a V) -> Ordering,
246:     {
247:         self.entries.binary_search_by(move |a| f(&a.key, &a.value))
248:     }
249:     /// Search over a sorted map with an extraction function.
250:     ///
251:     /// Returns the position where that value is present, or the position where it can be inserted
252:     /// to maintain the sort. See [`slice::binary_search_by_key`] for more details.
253:     ///
254:     /// Computes in **O(log(n))** time.
255:     #[inline]
256:     pub fn binary_search_by_key<'a, B, F>(&'a self, b: &B, mut f: F) -> Result<usize, usize>
257:     where
258:         F: FnMut(&'a K, &'a V) -> B,
259:         B: Ord,
260:     {
261:         self.binary_search_by(|k, v| f(k, v).cmp(b))
262:     }
263:     /// Checks if the keys of this slice are sorted.
264:     #[inline]
265:     pub fn is_sorted(&self) -> bool
266:     where
267:         K: PartialOrd,
268:     {
269:         self.entries.is_sorted_by(|a, b| a.key <= b.key)
270:     }
271:     /// Checks if this slice is sorted using the given comparator function.
272:     #[inline]
273:     pub fn is_sorted_by<'a, F>(&'a self, mut cmp: F) -> bool
274:     where
275:         F: FnMut(&'a K, &'a V, &'a K, &'a V) -> bool,
276:     {
277:         self.entries
278:             .is_sorted_by(move |a, b| cmp(&a.key, &a.value, &b.key, &b.value))
279:     }
280:     /// Checks if this slice is sorted using the given sort-key function.
281:     #[inline]
282:     pub fn is_sorted_by_key<'a, F, T>(&'a self, mut sort_key: F) -> bool
283:     where
284:         F: FnMut(&'a K, &'a V) -> T,
285:         T: PartialOrd,
286:     {
287:         self.entries
288:             .is_sorted_by_key(move |a| sort_key(&a.key, &a.value))
289:     }
290:     /// Returns the index of the partition point of a sorted map according to the given predicate
291:     /// (the index of the first element of the second partition).
292:     ///
293:     /// See [`slice::partition_point`] for more details.
294:     ///
295:     /// Computes in **O(log(n))** time.
296:     #[must_use]
297:     pub fn partition_point<P>(&self, mut pred: P) -> usize
298:     where
299:         P: FnMut(&K, &V) -> bool,
300:     {
301:         self.entries
302:             .partition_point(move |a| pred(&a.key, &a.value))
303:     }
304:     /// Get an array of `N` key-value pairs by `N` indices
305:     ///
306:     /// Valid indices are *0 <= index < self.len()* and each index needs to be unique.
307:     pub fn get_disjoint_mut<const N: usize>(
308:         &mut self,
309:         indices: [usize; N],
310:     ) -> Result<[(&K, &mut V); N], GetDisjointMutError> {
311:         // TODO(MSRV 1.86): use the standard library's `slice::get_disjoint_mut`
312:         let entries = super::disjoint::get_disjoint_mut(&mut self.entries, indices)?;
313:         Ok(entries.map(Bucket::ref_mut))
314:     }
315: }
316: impl<'a, K, V> IntoIterator for &'a Slice<K, V> {
317:     type IntoIter = Iter<'a, K, V>;
318:     type Item = (&'a K, &'a V);
319:     fn into_iter(self) -> Self::IntoIter {
320:         self.iter()
321:     }
322: }
323: impl<'a, K, V> IntoIterator for &'a mut Slice<K, V> {
324:     type IntoIter = IterMut<'a, K, V>;
325:     type Item = (&'a K, &'a mut V);
326:     fn into_iter(self) -> Self::IntoIter {
327:         self.iter_mut()
328:     }
329: }
330: impl<K, V> IntoIterator for Box<Slice<K, V>> {
331:     type IntoIter = IntoIter<K, V>;
332:     type Item = (K, V);
333:     fn into_iter(self) -> Self::IntoIter {
334:         IntoIter::new(self.into_entries())
335:     }
336: }
337: impl<K, V> Default for &'_ Slice<K, V> {
338:     fn default() -> Self {
339:         Slice::from_slice(&[])
340:     }
341: }
342: impl<K, V> Default for &'_ mut Slice<K, V> {
343:     fn default() -> Self {
344:         Slice::from_mut_slice(&mut [])
345:     }
346: }
347: impl<K, V> Default for Box<Slice<K, V>> {
348:     fn default() -> Self {
349:         Slice::from_boxed(Box::default())
350:     }
351: }
352: impl<K: Clone, V: Clone> Clone for Box<Slice<K, V>> {
353:     fn clone(&self) -> Self {
354:         Slice::from_boxed(self.entries.to_vec().into_boxed_slice())
355:     }
356: }
357: impl<K: Copy, V: Copy> From<&Slice<K, V>> for Box<Slice<K, V>> {
358:     fn from(slice: &Slice<K, V>) -> Self {
359:         Slice::from_boxed(Box::from(&slice.entries))
360:     }
361: }
362: impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for Slice<K, V> {
363:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
364:         f.debug_list().entries(self).finish()
365:     }
366: }
367: impl<K, V, K2, V2> PartialEq<Slice<K2, V2>> for Slice<K, V>
368: where
369:     K: PartialEq<K2>,
370:     V: PartialEq<V2>,
371: {
372:     fn eq(&self, other: &Slice<K2, V2>) -> bool {
373:         slice_eq(&self.entries, &other.entries, |b1, b2| {
374:             b1.key == b2.key && b1.value == b2.value
375:         })
376:     }
377: }
378: impl<K, V, K2, V2> PartialEq<[(K2, V2)]> for Slice<K, V>
379: where
380:     K: PartialEq<K2>,
381:     V: PartialEq<V2>,
382: {
383:     fn eq(&self, other: &[(K2, V2)]) -> bool {
384:         slice_eq(&self.entries, other, |b, t| b.key == t.0 && b.value == t.1)
385:     }
386: }
387: impl<K, V, K2, V2> PartialEq<Slice<K2, V2>> for [(K, V)]
388: where
389:     K: PartialEq<K2>,
390:     V: PartialEq<V2>,
391: {
392:     fn eq(&self, other: &Slice<K2, V2>) -> bool {
393:         slice_eq(self, &other.entries, |t, b| t.0 == b.key && t.1 == b.value)
394:     }
395: }
396: impl<K, V, K2, V2, const N: usize> PartialEq<[(K2, V2); N]> for Slice<K, V>
397: where
398:     K: PartialEq<K2>,
399:     V: PartialEq<V2>,
400: {
401:     fn eq(&self, other: &[(K2, V2); N]) -> bool {
402:         <Self as PartialEq<[_]>>::eq(self, other)
403:     }
404: }
405: impl<K, V, const N: usize, K2, V2> PartialEq<Slice<K2, V2>> for [(K, V); N]
406: where
407:     K: PartialEq<K2>,
408:     V: PartialEq<V2>,
409: {
410:     fn eq(&self, other: &Slice<K2, V2>) -> bool {
411:         <[_] as PartialEq<_>>::eq(self, other)
412:     }
413: }
414: impl<K: Eq, V: Eq> Eq for Slice<K, V> {}
415: impl<K: PartialOrd, V: PartialOrd> PartialOrd for Slice<K, V> {
416:     fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
417:         self.iter().partial_cmp(other)
418:     }
419: }
420: impl<K: Ord, V: Ord> Ord for Slice<K, V> {
421:     fn cmp(&self, other: &Self) -> Ordering {
422:         self.iter().cmp(other)
423:     }
424: }
425: impl<K: Hash, V: Hash> Hash for Slice<K, V> {
426:     fn hash<H: Hasher>(&self, state: &mut H) {
427:         self.len().hash(state);
428:         for (key, value) in self {
429:             key.hash(state);
430:             value.hash(state);
431:         }
432:     }
433: }
434: impl<K, V> Index<usize> for Slice<K, V> {
435:     type Output = V;
436:     fn index(&self, index: usize) -> &V {
437:         &self.entries[index].value
438:     }
439: }
440: impl<K, V> IndexMut<usize> for Slice<K, V> {
441:     fn index_mut(&mut self, index: usize) -> &mut V {
442:         &mut self.entries[index].value
443:     }
444: }
445: // We can't have `impl<I: RangeBounds<usize>> Index<I>` because that conflicts
446: // both upstream with `Index<usize>` and downstream with `Index<&Q>`.
447: // Instead, we repeat the implementations for all the core range types.
448: macro_rules! impl_index {
449:     ($($range:ty),*) => {$(
450:         impl<K, V, S> Index<$range> for IndexMap<K, V, S> {
451:             type Output = Slice<K, V>;
452:             fn index(&self, range: $range) -> &Self::Output {
453:                 Slice::from_slice(&self.as_entries()[range])
454:             }
455:         }
456:         impl<K, V, S> IndexMut<$range> for IndexMap<K, V, S> {
457:             fn index_mut(&mut self, range: $range) -> &mut Self::Output {
458:                 Slice::from_mut_slice(&mut self.as_entries_mut()[range])
459:             }
460:         }
461:         impl<K, V> Index<$range> for Slice<K, V> {
462:             type Output = Slice<K, V>;
463:             fn index(&self, range: $range) -> &Self {
464:                 Self::from_slice(&self.entries[range])
465:             }
466:         }
467:         impl<K, V> IndexMut<$range> for Slice<K, V> {
468:             fn index_mut(&mut self, range: $range) -> &mut Self {
469:                 Self::from_mut_slice(&mut self.entries[range])
470:             }
471:         }
472:     )*}
473: }
474: impl_index!(
475:     ops::Range<usize>,
476:     ops::RangeFrom<usize>,
477:     ops::RangeFull,
478:     ops::RangeInclusive<usize>,
479:     ops::RangeTo<usize>,
480:     ops::RangeToInclusive<usize>,
481:     (Bound<usize>, Bound<usize>)
482: );
483: #[cfg(test)]
484: mod tests {
485:     use super::*;
486:     #[test]
487:     fn slice_index() {
488:         fn check(
489:             vec_slice: &[(i32, i32)],
490:             map_slice: &Slice<i32, i32>,
491:             sub_slice: &Slice<i32, i32>,
492:         ) {
493:             assert_eq!(map_slice as *const _, sub_slice as *const _);
494:             itertools::assert_equal(
495:                 vec_slice.iter().copied(),
496:                 map_slice.iter().map(|(&k, &v)| (k, v)),
497:             );
498:             itertools::assert_equal(vec_slice.iter().map(|(k, _)| k), map_slice.keys());
499:             itertools::assert_equal(vec_slice.iter().map(|(_, v)| v), map_slice.values());
500:         }
501:         let vec: Vec<(i32, i32)> = (0..10).map(|i| (i, i * i)).collect();
502:         let map: IndexMap<i32, i32> = vec.iter().cloned().collect();
503:         let slice = map.as_slice();
504:         // RangeFull
505:         #[expect(clippy::redundant_slicing)]
506:         check(&vec[..], &map[..], &slice[..]);
507:         for i in 0usize..10 {
508:             // Index
509:             assert_eq!(vec[i].1, map[i]);
510:             assert_eq!(vec[i].1, slice[i]);
511:             assert_eq!(map[&(i as i32)], map[i]);
512:             assert_eq!(map[&(i as i32)], slice[i]);
513:             // RangeFrom
514:             check(&vec[i..], &map[i..], &slice[i..]);
515:             // RangeTo
516:             check(&vec[..i], &map[..i], &slice[..i]);
517:             // RangeToInclusive
518:             check(&vec[..=i], &map[..=i], &slice[..=i]);
519:             // (Bound<usize>, Bound<usize>)
520:             let bounds = (Bound::Excluded(i), Bound::Unbounded);
521:             check(&vec[i + 1..], &map[bounds], &slice[bounds]);
522:             for j in i..=10 {
523:                 // Range
524:                 check(&vec[i..j], &map[i..j], &slice[i..j]);
525:             }
526:             for j in i..10 {
527:                 // RangeInclusive
528:                 check(&vec[i..=j], &map[i..=j], &slice[i..=j]);
529:             }
530:         }
531:     }
532:     #[test]
533:     fn slice_index_mut() {
534:         fn check_mut(
535:             vec_slice: &[(i32, i32)],
536:             map_slice: &mut Slice<i32, i32>,
537:             sub_slice: &mut Slice<i32, i32>,
538:         ) {
539:             assert_eq!(map_slice, sub_slice);
540:             itertools::assert_equal(
541:                 vec_slice.iter().copied(),
542:                 map_slice.iter_mut().map(|(&k, &mut v)| (k, v)),
543:             );
544:             itertools::assert_equal(
545:                 vec_slice.iter().map(|&(_, v)| v),
546:                 map_slice.values_mut().map(|&mut v| v),
547:             );
548:         }
549:         let vec: Vec<(i32, i32)> = (0..10).map(|i| (i, i * i)).collect();
550:         let mut map: IndexMap<i32, i32> = vec.iter().cloned().collect();
551:         let mut map2 = map.clone();
552:         let slice = map2.as_mut_slice();
553:         // RangeFull
554:         check_mut(&vec[..], &mut map[..], &mut slice[..]);
555:         for i in 0usize..10 {
556:             // IndexMut
557:             assert_eq!(&mut map[i], &mut slice[i]);
558:             // RangeFrom
559:             check_mut(&vec[i..], &mut map[i..], &mut slice[i..]);
560:             // RangeTo
561:             check_mut(&vec[..i], &mut map[..i], &mut slice[..i]);
562:             // RangeToInclusive
563:             check_mut(&vec[..=i], &mut map[..=i], &mut slice[..=i]);
564:             // (Bound<usize>, Bound<usize>)
565:             let bounds = (Bound::Excluded(i), Bound::Unbounded);
566:             check_mut(&vec[i + 1..], &mut map[bounds], &mut slice[bounds]);
567:             for j in i..=10 {
568:                 // Range
569:                 check_mut(&vec[i..j], &mut map[i..j], &mut slice[i..j]);
570:             }
571:             for j in i..10 {
572:                 // RangeInclusive
573:                 check_mut(&vec[i..=j], &mut map[i..=j], &mut slice[i..=j]);
574:             }
575:         }
576:     }
577:     #[test]
578:     fn slice_new() {
579:         let slice: &Slice<i32, i32> = Slice::new();
580:         assert!(slice.is_empty());
581:         assert_eq!(slice.len(), 0);
582:     }
583:     #[test]
584:     fn slice_new_mut() {
585:         let slice: &mut Slice<i32, i32> = Slice::new_mut();
586:         assert!(slice.is_empty());
587:         assert_eq!(slice.len(), 0);
588:     }
589:     #[test]
590:     fn slice_get_index_mut() {
591:         let mut map: IndexMap<i32, i32> = (0..10).map(|i| (i, i * i)).collect();
592:         let slice: &mut Slice<i32, i32> = map.as_mut_slice();
593:         {
594:             let (key, value) = slice.get_index_mut(0).unwrap();
595:             assert_eq!(*key, 0);
596:             assert_eq!(*value, 0);
597:             *value = 11;
598:         }
599:         assert_eq!(slice[0], 11);
600:         {
601:             let result = slice.get_index_mut(11);
602:             assert!(result.is_none());
603:         }
604:     }
605:     #[test]
606:     fn slice_split_first() {
607:         let slice: &mut Slice<i32, i32> = Slice::new_mut();
608:         let result = slice.split_first();
609:         assert!(result.is_none());
610:         let mut map: IndexMap<i32, i32> = (0..10).map(|i| (i, i * i)).collect();
611:         let slice: &mut Slice<i32, i32> = map.as_mut_slice();
612:         {
613:             let (first, rest) = slice.split_first().unwrap();
614:             assert_eq!(first, (&0, &0));
615:             assert_eq!(rest.len(), 9);
616:         }
617:         assert_eq!(slice.len(), 10);
618:     }
619:     #[test]
620:     fn slice_split_first_mut() {
621:         let slice: &mut Slice<i32, i32> = Slice::new_mut();
622:         let result = slice.split_first_mut();
623:         assert!(result.is_none());
624:         let mut map: IndexMap<i32, i32> = (0..10).map(|i| (i, i * i)).collect();
625:         let slice: &mut Slice<i32, i32> = map.as_mut_slice();
626:         {
627:             let (first, rest) = slice.split_first_mut().unwrap();
628:             assert_eq!(first, (&0, &mut 0));
629:             assert_eq!(rest.len(), 9);
630:             *first.1 = 11;
631:         }
632:         assert_eq!(slice.len(), 10);
633:         assert_eq!(slice[0], 11);
634:     }
635:     #[test]
636:     fn slice_split_last() {
637:         let slice: &mut Slice<i32, i32> = Slice::new_mut();
638:         let result = slice.split_last();
639:         assert!(result.is_none());
640:         let mut map: IndexMap<i32, i32> = (0..10).map(|i| (i, i * i)).collect();
641:         let slice: &mut Slice<i32, i32> = map.as_mut_slice();
642:         {
643:             let (last, rest) = slice.split_last().unwrap();
644:             assert_eq!(last, (&9, &81));
645:             assert_eq!(rest.len(), 9);
646:         }
647:         assert_eq!(slice.len(), 10);
648:     }
649:     #[test]
650:     fn slice_split_last_mut() {
651:         let slice: &mut Slice<i32, i32> = Slice::new_mut();
652:         let result = slice.split_last_mut();
653:         assert!(result.is_none());
654:         let mut map: IndexMap<i32, i32> = (0..10).map(|i| (i, i * i)).collect();
655:         let slice: &mut Slice<i32, i32> = map.as_mut_slice();
656:         {
657:             let (last, rest) = slice.split_last_mut().unwrap();
658:             assert_eq!(last, (&9, &mut 81));
659:             assert_eq!(rest.len(), 9);
660:             *last.1 = 100;
661:         }
662:         assert_eq!(slice.len(), 10);
663:         assert_eq!(slice[slice.len() - 1], 100);
664:     }
665:     #[test]
666:     fn slice_get_range() {
667:         let mut map: IndexMap<i32, i32> = (0..10).map(|i| (i, i * i)).collect();
668:         let slice: &mut Slice<i32, i32> = map.as_mut_slice();
669:         let subslice = slice.get_range(3..6).unwrap();
670:         assert_eq!(subslice.len(), 3);
671:         assert_eq!(subslice, &[(3, 9), (4, 16), (5, 25)]);
672:     }
673: }
````

## File: src/map/tests.rs
````rust
   1: use super::*;
   2: use std::string::String;
   3: #[test]
   4: fn it_works() {
   5:     let mut map = IndexMap::new();
   6:     assert_eq!(map.is_empty(), true);
   7:     map.insert(1, ());
   8:     map.insert(1, ());
   9:     assert_eq!(map.len(), 1);
  10:     assert!(map.get(&1).is_some());
  11:     assert_eq!(map.is_empty(), false);
  12: }
  13: #[test]
  14: fn new() {
  15:     let map = IndexMap::<String, String>::new();
  16:     println!("{:?}", map);
  17:     assert_eq!(map.capacity(), 0);
  18:     assert_eq!(map.len(), 0);
  19:     assert_eq!(map.is_empty(), true);
  20: }
  21: #[test]
  22: fn insert() {
  23:     let insert = [0, 4, 2, 12, 8, 7, 11, 5];
  24:     let not_present = [1, 3, 6, 9, 10];
  25:     let mut map = IndexMap::with_capacity(insert.len());
  26:     for (i, &elt) in insert.iter().enumerate() {
  27:         assert_eq!(map.len(), i);
  28:         map.insert(elt, elt);
  29:         assert_eq!(map.len(), i + 1);
  30:         assert_eq!(map.get(&elt), Some(&elt));
  31:         assert_eq!(map[&elt], elt);
  32:     }
  33:     println!("{:?}", map);
  34:     for &elt in &not_present {
  35:         assert!(map.get(&elt).is_none());
  36:     }
  37: }
  38: #[test]
  39: fn insert_full() {
  40:     let insert = [9, 2, 7, 1, 4, 6, 13];
  41:     let present = [1, 6, 2];
  42:     let mut map = IndexMap::with_capacity(insert.len());
  43:     for (i, &elt) in insert.iter().enumerate() {
  44:         assert_eq!(map.len(), i);
  45:         let (index, existing) = map.insert_full(elt, elt);
  46:         assert_eq!(existing, None);
  47:         assert_eq!(Some(index), map.get_full(&elt).map(|x| x.0));
  48:         assert_eq!(map.len(), i + 1);
  49:     }
  50:     let len = map.len();
  51:     for &elt in &present {
  52:         let (index, existing) = map.insert_full(elt, elt);
  53:         assert_eq!(existing, Some(elt));
  54:         assert_eq!(Some(index), map.get_full(&elt).map(|x| x.0));
  55:         assert_eq!(map.len(), len);
  56:     }
  57: }
  58: #[test]
  59: fn insert_2() {
  60:     let mut map = IndexMap::with_capacity(16);
  61:     let mut keys = vec![];
  62:     keys.extend(0..16);
  63:     keys.extend(if cfg!(miri) { 32..64 } else { 128..267 });
  64:     for &i in &keys {
  65:         let old_map = map.clone();
  66:         map.insert(i, ());
  67:         for key in old_map.keys() {
  68:             if map.get(key).is_none() {
  69:                 println!("old_map: {:?}", old_map);
  70:                 println!("map: {:?}", map);
  71:                 panic!("did not find {} in map", key);
  72:             }
  73:         }
  74:     }
  75:     for &i in &keys {
  76:         assert!(map.get(&i).is_some(), "did not find {}", i);
  77:     }
  78: }
  79: #[test]
  80: fn insert_order() {
  81:     let insert = [0, 4, 2, 12, 8, 7, 11, 5, 3, 17, 19, 22, 23];
  82:     let mut map = IndexMap::new();
  83:     for &elt in &insert {
  84:         map.insert(elt, ());
  85:     }
  86:     assert_eq!(map.keys().count(), map.len());
  87:     assert_eq!(map.keys().count(), insert.len());
  88:     for (a, b) in insert.iter().zip(map.keys()) {
  89:         assert_eq!(a, b);
  90:     }
  91:     for (i, k) in (0..insert.len()).zip(map.keys()) {
  92:         assert_eq!(map.get_index(i).unwrap().0, k);
  93:     }
  94: }
  95: #[test]
  96: fn shift_insert() {
  97:     let insert = [0, 4, 2, 12, 8, 7, 11, 5, 3, 17, 19, 22, 23];
  98:     let mut map = IndexMap::new();
  99:     for &elt in &insert {
 100:         map.shift_insert(0, elt, ());
 101:     }
 102:     assert_eq!(map.keys().count(), map.len());
 103:     assert_eq!(map.keys().count(), insert.len());
 104:     for (a, b) in insert.iter().rev().zip(map.keys()) {
 105:         assert_eq!(a, b);
 106:     }
 107:     for (i, k) in (0..insert.len()).zip(map.keys()) {
 108:         assert_eq!(map.get_index(i).unwrap().0, k);
 109:     }
 110:     // "insert" that moves an existing entry
 111:     map.shift_insert(0, insert[0], ());
 112:     assert_eq!(map.keys().count(), insert.len());
 113:     assert_eq!(insert[0], map.keys()[0]);
 114:     for (a, b) in insert[1..].iter().rev().zip(map.keys().skip(1)) {
 115:         assert_eq!(a, b);
 116:     }
 117: }
 118: #[test]
 119: fn insert_sorted_bad() {
 120:     let mut map = IndexMap::new();
 121:     map.insert(10, ());
 122:     for i in 0..10 {
 123:         map.insert(i, ());
 124:     }
 125:     // The binary search will want to insert this at the end (index == len()),
 126:     // but that's only possible for *new* inserts. It should still be handled
 127:     // without panicking though, and in this case it's simple enough that we
 128:     // know the exact result. (But don't read this as an API guarantee!)
 129:     assert_eq!(map.first(), Some((&10, &())));
 130:     map.insert_sorted(10, ());
 131:     assert_eq!(map.last(), Some((&10, &())));
 132:     assert!(map.keys().copied().eq(0..=10));
 133:     // Other out-of-order entries can also "insert" to a binary-searched
 134:     // position, moving in either direction.
 135:     map.move_index(5, 0);
 136:     map.move_index(6, 10);
 137:     assert_eq!(map.first(), Some((&5, &())));
 138:     assert_eq!(map.last(), Some((&6, &())));
 139:     map.insert_sorted(5, ()); // moves back up
 140:     map.insert_sorted(6, ()); // moves back down
 141:     assert!(map.keys().copied().eq(0..=10));
 142: }
 143: #[test]
 144: fn grow() {
 145:     let insert = [0, 4, 2, 12, 8, 7, 11];
 146:     let not_present = [1, 3, 6, 9, 10];
 147:     let mut map = IndexMap::with_capacity(insert.len());
 148:     for (i, &elt) in insert.iter().enumerate() {
 149:         assert_eq!(map.len(), i);
 150:         map.insert(elt, elt);
 151:         assert_eq!(map.len(), i + 1);
 152:         assert_eq!(map.get(&elt), Some(&elt));
 153:         assert_eq!(map[&elt], elt);
 154:     }
 155:     println!("{:?}", map);
 156:     for &elt in &insert {
 157:         map.insert(elt * 10, elt);
 158:     }
 159:     for &elt in &insert {
 160:         map.insert(elt * 100, elt);
 161:     }
 162:     for (i, &elt) in insert.iter().cycle().enumerate().take(100) {
 163:         map.insert(elt * 100 + i as i32, elt);
 164:     }
 165:     println!("{:?}", map);
 166:     for &elt in &not_present {
 167:         assert!(map.get(&elt).is_none());
 168:     }
 169: }
 170: #[test]
 171: fn reserve() {
 172:     let mut map = IndexMap::<usize, usize>::new();
 173:     assert_eq!(map.capacity(), 0);
 174:     map.reserve(100);
 175:     let capacity = map.capacity();
 176:     assert!(capacity >= 100);
 177:     for i in 0..capacity {
 178:         assert_eq!(map.len(), i);
 179:         map.insert(i, i * i);
 180:         assert_eq!(map.len(), i + 1);
 181:         assert_eq!(map.capacity(), capacity);
 182:         assert_eq!(map.get(&i), Some(&(i * i)));
 183:     }
 184:     map.insert(capacity, usize::MAX);
 185:     assert_eq!(map.len(), capacity + 1);
 186:     assert!(map.capacity() > capacity);
 187:     assert_eq!(map.get(&capacity), Some(&usize::MAX));
 188: }
 189: #[test]
 190: fn try_reserve() {
 191:     let mut map = IndexMap::<usize, usize>::new();
 192:     assert_eq!(map.capacity(), 0);
 193:     assert_eq!(map.try_reserve(100), Ok(()));
 194:     assert!(map.capacity() >= 100);
 195:     assert!(map.try_reserve(usize::MAX).is_err());
 196: }
 197: #[test]
 198: fn shrink_to_fit() {
 199:     let mut map = IndexMap::<usize, usize>::new();
 200:     assert_eq!(map.capacity(), 0);
 201:     for i in 0..100 {
 202:         assert_eq!(map.len(), i);
 203:         map.insert(i, i * i);
 204:         assert_eq!(map.len(), i + 1);
 205:         assert!(map.capacity() > i);
 206:         assert_eq!(map.get(&i), Some(&(i * i)));
 207:         map.shrink_to_fit();
 208:         assert_eq!(map.len(), i + 1);
 209:         assert_eq!(map.capacity(), i + 1);
 210:         assert_eq!(map.get(&i), Some(&(i * i)));
 211:     }
 212: }
 213: #[test]
 214: fn remove() {
 215:     let insert = [0, 4, 2, 12, 8, 7, 11, 5, 3, 17, 19, 22, 23];
 216:     let mut map = IndexMap::new();
 217:     for &elt in &insert {
 218:         map.insert(elt, elt);
 219:     }
 220:     assert_eq!(map.keys().count(), map.len());
 221:     assert_eq!(map.keys().count(), insert.len());
 222:     for (a, b) in insert.iter().zip(map.keys()) {
 223:         assert_eq!(a, b);
 224:     }
 225:     let remove_fail = [99, 77];
 226:     let remove = [4, 12, 8, 7];
 227:     for &key in &remove_fail {
 228:         assert!(map.swap_remove_full(&key).is_none());
 229:     }
 230:     println!("{:?}", map);
 231:     for &key in &remove {
 232:         //println!("{:?}", map);
 233:         let index = map.get_full(&key).unwrap().0;
 234:         assert_eq!(map.swap_remove_full(&key), Some((index, key, key)));
 235:     }
 236:     println!("{:?}", map);
 237:     for key in &insert {
 238:         assert_eq!(map.get(key).is_some(), !remove.contains(key));
 239:     }
 240:     assert_eq!(map.len(), insert.len() - remove.len());
 241:     assert_eq!(map.keys().count(), insert.len() - remove.len());
 242: }
 243: #[test]
 244: fn remove_to_empty() {
 245:     let mut map = indexmap! { 0 => 0, 4 => 4, 5 => 5 };
 246:     map.swap_remove(&5).unwrap();
 247:     map.swap_remove(&4).unwrap();
 248:     map.swap_remove(&0).unwrap();
 249:     assert!(map.is_empty());
 250: }
 251: #[test]
 252: fn swap_remove_index() {
 253:     let insert = [0, 4, 2, 12, 8, 7, 11, 5, 3, 17, 19, 22, 23];
 254:     let mut map = IndexMap::new();
 255:     for &elt in &insert {
 256:         map.insert(elt, elt * 2);
 257:     }
 258:     let mut vector = insert.to_vec();
 259:     let remove_sequence = &[3, 3, 10, 4, 5, 4, 3, 0, 1];
 260:     // check that the same swap remove sequence on vec and map
 261:     // have the same result.
 262:     for &rm in remove_sequence {
 263:         let out_vec = vector.swap_remove(rm);
 264:         let (out_map, _) = map.swap_remove_index(rm).unwrap();
 265:         assert_eq!(out_vec, out_map);
 266:     }
 267:     assert_eq!(vector.len(), map.len());
 268:     for (a, b) in vector.iter().zip(map.keys()) {
 269:         assert_eq!(a, b);
 270:     }
 271: }
 272: #[test]
 273: fn partial_eq_and_eq() {
 274:     let mut map_a = IndexMap::new();
 275:     map_a.insert(1, "1");
 276:     map_a.insert(2, "2");
 277:     let mut map_b = map_a.clone();
 278:     assert_eq!(map_a, map_b);
 279:     map_b.swap_remove(&1);
 280:     assert_ne!(map_a, map_b);
 281:     let map_c: IndexMap<_, String> = map_b.into_iter().map(|(k, v)| (k, v.into())).collect();
 282:     assert_ne!(map_a, map_c);
 283:     assert_ne!(map_c, map_a);
 284: }
 285: #[test]
 286: fn extend() {
 287:     let mut map = IndexMap::new();
 288:     map.extend([(&1, &2), (&3, &4)]);
 289:     map.extend([(5, 6)]);
 290:     assert_eq!(
 291:         map.into_iter().collect::<Vec<_>>(),
 292:         [(1, 2), (3, 4), (5, 6)]
 293:     );
 294: }
 295: #[test]
 296: fn entry() {
 297:     let mut map = IndexMap::new();
 298:     map.insert(1, "1");
 299:     map.insert(2, "2");
 300:     {
 301:         let e = map.entry(3);
 302:         assert_eq!(e.index(), 2);
 303:         let e = e.or_insert("3");
 304:         assert_eq!(e, &"3");
 305:     }
 306:     let e = map.entry(2);
 307:     assert_eq!(e.index(), 1);
 308:     assert_eq!(e.key(), &2);
 309:     match e {
 310:         Entry::Occupied(ref e) => assert_eq!(e.get(), &"2"),
 311:         Entry::Vacant(_) => panic!(),
 312:     }
 313:     assert_eq!(e.or_insert("4"), &"2");
 314: }
 315: #[test]
 316: fn entry_and_modify() {
 317:     let mut map = IndexMap::new();
 318:     map.insert(1, "1");
 319:     map.entry(1).and_modify(|x| *x = "2");
 320:     assert_eq!(Some(&"2"), map.get(&1));
 321:     map.entry(2).and_modify(|x| *x = "doesn't exist");
 322:     assert_eq!(None, map.get(&2));
 323: }
 324: #[test]
 325: fn entry_or_default() {
 326:     let mut map = IndexMap::new();
 327:     #[derive(Debug, Default, PartialEq)]
 328:     enum TestEnum {
 329:         #[default]
 330:         DefaultValue,
 331:         NonDefaultValue,
 332:     }
 333:     map.insert(1, TestEnum::NonDefaultValue);
 334:     assert_eq!(&mut TestEnum::NonDefaultValue, map.entry(1).or_default());
 335:     assert_eq!(&mut TestEnum::DefaultValue, map.entry(2).or_default());
 336: }
 337: #[test]
 338: fn occupied_entry_key() {
 339:     // These keys match hash and equality, but their addresses are distinct.
 340:     let (k1, k2) = (&mut 1, &mut 1);
 341:     let k1_ptr = k1 as *const i32;
 342:     let k2_ptr = k2 as *const i32;
 343:     assert_ne!(k1_ptr, k2_ptr);
 344:     let mut map = IndexMap::new();
 345:     map.insert(k1, "value");
 346:     match map.entry(k2) {
 347:         Entry::Occupied(ref e) => {
 348:             // `OccupiedEntry::key` should reference the key in the map,
 349:             // not the key that was used to find the entry.
 350:             let ptr = *e.key() as *const i32;
 351:             assert_eq!(ptr, k1_ptr);
 352:             assert_ne!(ptr, k2_ptr);
 353:         }
 354:         Entry::Vacant(_) => panic!(),
 355:     }
 356: }
 357: #[test]
 358: fn get_index_entry() {
 359:     let mut map = IndexMap::new();
 360:     assert!(map.get_index_entry(0).is_none());
 361:     assert!(map.first_entry().is_none());
 362:     assert!(map.last_entry().is_none());
 363:     map.insert(0, "0");
 364:     map.insert(1, "1");
 365:     map.insert(2, "2");
 366:     map.insert(3, "3");
 367:     assert!(map.get_index_entry(4).is_none());
 368:     {
 369:         let e = map.get_index_entry(1).unwrap();
 370:         assert_eq!(*e.key(), 1);
 371:         assert_eq!(*e.get(), "1");
 372:         assert_eq!(e.swap_remove(), "1");
 373:     }
 374:     {
 375:         let mut e = map.get_index_entry(1).unwrap();
 376:         assert_eq!(*e.key(), 3);
 377:         assert_eq!(*e.get(), "3");
 378:         assert_eq!(e.insert("4"), "3");
 379:     }
 380:     assert_eq!(*map.get(&3).unwrap(), "4");
 381:     {
 382:         let e = map.first_entry().unwrap();
 383:         assert_eq!(*e.key(), 0);
 384:         assert_eq!(*e.get(), "0");
 385:     }
 386:     {
 387:         let e = map.last_entry().unwrap();
 388:         assert_eq!(*e.key(), 2);
 389:         assert_eq!(*e.get(), "2");
 390:     }
 391: }
 392: #[test]
 393: fn from_entries() {
 394:     let mut map = IndexMap::from([(1, "1"), (2, "2"), (3, "3")]);
 395:     {
 396:         let e = match map.entry(1) {
 397:             Entry::Occupied(e) => IndexedEntry::from(e),
 398:             Entry::Vacant(_) => panic!(),
 399:         };
 400:         assert_eq!(e.index(), 0);
 401:         assert_eq!(*e.key(), 1);
 402:         assert_eq!(*e.get(), "1");
 403:     }
 404:     {
 405:         let e = match map.get_index_entry(1) {
 406:             Some(e) => OccupiedEntry::from(e),
 407:             None => panic!(),
 408:         };
 409:         assert_eq!(e.index(), 1);
 410:         assert_eq!(*e.key(), 2);
 411:         assert_eq!(*e.get(), "2");
 412:     }
 413: }
 414: #[test]
 415: fn keys() {
 416:     let map = IndexMap::<_, _>::from_iter([(1, 'a'), (2, 'b'), (3, 'c')]);
 417:     let keys: Vec<_> = map.keys().copied().collect();
 418:     assert_eq!(keys.len(), 3);
 419:     assert!(keys.contains(&1));
 420:     assert!(keys.contains(&2));
 421:     assert!(keys.contains(&3));
 422: }
 423: #[test]
 424: fn into_keys() {
 425:     let map = IndexMap::<_, _>::from_iter([(1, 'a'), (2, 'b'), (3, 'c')]);
 426:     let keys: Vec<i32> = map.into_keys().collect();
 427:     assert_eq!(keys.len(), 3);
 428:     assert!(keys.contains(&1));
 429:     assert!(keys.contains(&2));
 430:     assert!(keys.contains(&3));
 431: }
 432: #[test]
 433: fn values() {
 434:     let map = IndexMap::<_, _>::from_iter([(1, 'a'), (2, 'b'), (3, 'c')]);
 435:     let values: Vec<_> = map.values().copied().collect();
 436:     assert_eq!(values.len(), 3);
 437:     assert!(values.contains(&'a'));
 438:     assert!(values.contains(&'b'));
 439:     assert!(values.contains(&'c'));
 440: }
 441: #[test]
 442: fn values_mut() {
 443:     let mut map = IndexMap::<_, _>::from_iter([(1, 1), (2, 2), (3, 3)]);
 444:     for value in map.values_mut() {
 445:         *value *= 2
 446:     }
 447:     let values: Vec<_> = map.values().copied().collect();
 448:     assert_eq!(values.len(), 3);
 449:     assert!(values.contains(&2));
 450:     assert!(values.contains(&4));
 451:     assert!(values.contains(&6));
 452: }
 453: #[test]
 454: fn into_values() {
 455:     let map = IndexMap::<_, _>::from_iter([(1, 'a'), (2, 'b'), (3, 'c')]);
 456:     let values: Vec<char> = map.into_values().collect();
 457:     assert_eq!(values.len(), 3);
 458:     assert!(values.contains(&'a'));
 459:     assert!(values.contains(&'b'));
 460:     assert!(values.contains(&'c'));
 461: }
 462: #[test]
 463: fn drain_range() {
 464:     // Test the various heuristics of `erase_indices`
 465:     for range in [
 466:         0..0,   // nothing erased
 467:         10..90, // reinsert the few kept (..10 and 90..)
 468:         80..90, // update the few to adjust (80..)
 469:         20..30, // sweep everything
 470:     ] {
 471:         let mut vec = Vec::from_iter(0..100);
 472:         let mut map: IndexMap<i32, ()> = (0..100).map(|i| (i, ())).collect();
 473:         drop(vec.drain(range.clone()));
 474:         drop(map.drain(range));
 475:         assert!(vec.iter().eq(map.keys()));
 476:         for (i, x) in vec.iter().enumerate() {
 477:             assert_eq!(map.get_index_of(x), Some(i));
 478:         }
 479:     }
 480: }
 481: #[test]
 482: #[cfg(feature = "std")]
 483: fn from_array() {
 484:     let map = IndexMap::from([(1, 2), (3, 4)]);
 485:     let mut expected = IndexMap::new();
 486:     expected.insert(1, 2);
 487:     expected.insert(3, 4);
 488:     assert_eq!(map, expected)
 489: }
 490: #[test]
 491: fn iter_default() {
 492:     struct K;
 493:     struct V;
 494:     fn assert_default<T>()
 495:     where
 496:         T: Default + Iterator,
 497:     {
 498:         assert!(T::default().next().is_none());
 499:     }
 500:     assert_default::<Iter<'static, K, V>>();
 501:     assert_default::<IterMut<'static, K, V>>();
 502:     assert_default::<IterMut2<'static, K, V>>();
 503:     assert_default::<IntoIter<K, V>>();
 504:     assert_default::<Keys<'static, K, V>>();
 505:     assert_default::<IntoKeys<K, V>>();
 506:     assert_default::<Values<'static, K, V>>();
 507:     assert_default::<ValuesMut<'static, K, V>>();
 508:     assert_default::<IntoValues<K, V>>();
 509: }
 510: #[test]
 511: fn get_index_mut2() {
 512:     let mut map: IndexMap<i32, i32> = IndexMap::new();
 513:     map.insert(1, 2);
 514:     map.insert(3, 4);
 515:     map.insert(5, 6);
 516:     {
 517:         let (key, value) = map.get_index_mut2(0).unwrap();
 518:         assert_eq!(*key, 1);
 519:         assert_eq!(*value, 2);
 520:         *value = 7;
 521:     }
 522:     assert_eq!(map[0], 7);
 523:     {
 524:         let (key, _) = map.get_index_mut2(0).unwrap();
 525:         *key = 8;
 526:     }
 527:     assert_eq!(map.get_index(0).unwrap().0, &8);
 528: }
 529: #[test]
 530: fn shift_shift_remove_index() {
 531:     let mut map: IndexMap<i32, i32> = IndexMap::new();
 532:     map.insert(1, 2);
 533:     map.insert(3, 4);
 534:     map.insert(5, 6);
 535:     map.insert(7, 8);
 536:     map.insert(9, 10);
 537:     let result = map.shift_remove_index(1);
 538:     assert_eq!(result, Some((3, 4)));
 539:     assert_eq!(map.len(), 4);
 540:     assert_eq!(map.as_slice(), &[(1, 2), (5, 6), (7, 8), (9, 10)]);
 541:     let result = map.shift_remove_index(1);
 542:     assert_eq!(result, Some((5, 6)));
 543:     assert_eq!(map.len(), 3);
 544:     assert_eq!(map.as_slice(), &[(1, 2), (7, 8), (9, 10)]);
 545:     let result = map.shift_remove_index(2);
 546:     assert_eq!(result, Some((9, 10)));
 547:     assert_eq!(map.len(), 2);
 548:     assert_eq!(map.as_slice(), &[(1, 2), (7, 8)]);
 549:     let result = map.shift_remove_index(2);
 550:     assert_eq!(result, None);
 551:     assert_eq!(map.len(), 2);
 552:     assert_eq!(map.as_slice(), &[(1, 2), (7, 8)]);
 553: }
 554: #[test]
 555: fn shift_remove_entry() {
 556:     let mut map: IndexMap<i32, i32> = IndexMap::new();
 557:     map.insert(1, 2);
 558:     map.insert(3, 4);
 559:     map.insert(5, 6);
 560:     map.insert(7, 8);
 561:     map.insert(9, 10);
 562:     let result = map.shift_remove_entry(&3);
 563:     assert_eq!(result, Some((3, 4)));
 564:     assert_eq!(map.len(), 4);
 565:     assert_eq!(map.as_slice(), &[(1, 2), (5, 6), (7, 8), (9, 10)]);
 566:     let result = map.shift_remove_entry(&9);
 567:     assert_eq!(result, Some((9, 10)));
 568:     assert_eq!(map.len(), 3);
 569:     assert_eq!(map.as_slice(), &[(1, 2), (5, 6), (7, 8)]);
 570:     let result = map.shift_remove_entry(&9);
 571:     assert_eq!(result, None);
 572:     assert_eq!(map.len(), 3);
 573:     assert_eq!(map.as_slice(), &[(1, 2), (5, 6), (7, 8)]);
 574: }
 575: #[test]
 576: fn shift_remove_full() {
 577:     let mut map: IndexMap<i32, i32> = IndexMap::new();
 578:     map.insert(1, 2);
 579:     map.insert(3, 4);
 580:     map.insert(5, 6);
 581:     map.insert(7, 8);
 582:     map.insert(9, 10);
 583:     let result = map.shift_remove_full(&3);
 584:     assert_eq!(result, Some((1, 3, 4)));
 585:     assert_eq!(map.len(), 4);
 586:     assert_eq!(map.as_slice(), &[(1, 2), (5, 6), (7, 8), (9, 10)]);
 587:     let result = map.shift_remove_full(&9);
 588:     assert_eq!(result, Some((3, 9, 10)));
 589:     assert_eq!(map.len(), 3);
 590:     assert_eq!(map.as_slice(), &[(1, 2), (5, 6), (7, 8)]);
 591:     let result = map.shift_remove_full(&9);
 592:     assert_eq!(result, None);
 593:     assert_eq!(map.len(), 3);
 594:     assert_eq!(map.as_slice(), &[(1, 2), (5, 6), (7, 8)]);
 595: }
 596: #[test]
 597: fn sorted_unstable_by() {
 598:     let map = IndexMap::<i32, i32>::from_iter([(1, 10), (2, 20), (3, 30), (4, 40), (5, 50)]);
 599:     let sorted = map.sorted_unstable_by(|_a, b, _c, d| d.cmp(&b));
 600:     assert_eq!(
 601:         sorted.as_slice(),
 602:         &[(5, 50), (4, 40), (3, 30), (2, 20), (1, 10)]
 603:     );
 604: }
 605: #[test]
 606: fn into_boxed_slice() {
 607:     let mut map: IndexMap<i32, i32> = IndexMap::new();
 608:     for i in 0..5 {
 609:         map.insert(i, i * 10);
 610:     }
 611:     let boxed_slice: Box<Slice<i32, i32>> = map.into_boxed_slice();
 612:     assert_eq!(boxed_slice.len(), 5);
 613:     assert_eq!(
 614:         boxed_slice.as_ref(),
 615:         &[(0, 0), (1, 10), (2, 20), (3, 30), (4, 40)]
 616:     );
 617: }
 618: #[test]
 619: fn last_mut() {
 620:     let mut map: IndexMap<&str, i32> = IndexMap::new();
 621:     let last_entry = map.last_mut();
 622:     assert_eq!(last_entry, None);
 623:     map.insert("key1", 1);
 624:     map.insert("key2", 2);
 625:     map.insert("key3", 3);
 626:     let last_entry = map.last_mut();
 627:     assert_eq!(last_entry, Some((&"key3", &mut 3)));
 628:     *last_entry.unwrap().1 = 4;
 629:     assert_eq!(map.get("key3"), Some(&4));
 630: }
 631: #[test]
 632: #[should_panic = "index out of bounds"]
 633: fn insert_before_oob() {
 634:     let mut map: IndexMap<char, ()> = IndexMap::new();
 635:     let _ = map.insert_before(0, 'a', ());
 636:     let _ = map.insert_before(1, 'b', ());
 637:     map.insert_before(3, 'd', ());
 638: }
 639: #[test]
 640: fn clear() {
 641:     let mut map = IndexMap::<i32, i32>::from_iter([(1, 10), (2, 20), (3, 30), (4, 40), (5, 50)]);
 642:     assert_ne!(map.len(), 0);
 643:     map.clear();
 644:     assert_eq!(map.len(), 0);
 645: }
 646: #[test]
 647: fn get_range() {
 648:     let map = IndexMap::<i32, i32>::from_iter([(1, 10), (2, 20), (3, 30), (4, 40), (5, 50)]);
 649:     let result = map.get_range(2..2);
 650:     assert!(result.unwrap().is_empty());
 651:     #[expect(clippy::reversed_empty_ranges)]
 652:     let result = map.get_range(4..2);
 653:     assert!(result.is_none());
 654:     let result = map.get_range(2..4);
 655:     let slice: &Slice<i32, i32> = result.unwrap();
 656:     assert_eq!(slice.len(), 2);
 657:     assert_eq!(slice, &[(3, 30), (4, 40)]);
 658: }
 659: #[test]
 660: fn get_range_mut() {
 661:     let mut index_map: IndexMap<i32, i32> = IndexMap::new();
 662:     index_map.insert(1, 10);
 663:     index_map.insert(2, 20);
 664:     index_map.insert(3, 30);
 665:     index_map.insert(4, 40);
 666:     index_map.insert(5, 50);
 667:     let result = index_map.get_range_mut(2..2);
 668:     assert!(result.unwrap().is_empty());
 669:     #[expect(clippy::reversed_empty_ranges)]
 670:     let result = index_map.get_range_mut(4..2);
 671:     assert!(result.is_none());
 672:     let result = index_map.get_range_mut(2..4);
 673:     let slice: &mut Slice<i32, i32> = result.unwrap();
 674:     assert_eq!(slice.len(), 2);
 675:     assert_eq!(slice, &mut [(3, 30), (4, 40)]);
 676:     for i in 0..slice.len() {
 677:         slice[i] += 1;
 678:     }
 679:     assert_eq!(slice, &mut [(3, 31), (4, 41)]);
 680: }
 681: #[test]
 682: #[should_panic = "index out of bounds"]
 683: fn shift_insert_oob() {
 684:     let mut map: IndexMap<u32, u32> = IndexMap::new();
 685:     map.shift_insert(0, 1, 10);
 686:     map.shift_insert(1, 2, 20);
 687:     map.shift_insert(2, 3, 30);
 688:     map.shift_insert(5, 4, 40);
 689: }
 690: #[test]
 691: fn test_binary_search_by() {
 692:     // adapted from std's test for binary_search
 693:     let b: IndexMap<_, i32> = []
 694:         .into_iter()
 695:         .enumerate()
 696:         .map(|(i, x)| (i + 100, x))
 697:         .collect();
 698:     assert_eq!(b.binary_search_by(|_, x| x.cmp(&5)), Err(0));
 699:     let b: IndexMap<_, i32> = [4]
 700:         .into_iter()
 701:         .enumerate()
 702:         .map(|(i, x)| (i + 100, x))
 703:         .collect();
 704:     assert_eq!(b.binary_search_by(|_, x| x.cmp(&3)), Err(0));
 705:     assert_eq!(b.binary_search_by(|_, x| x.cmp(&4)), Ok(0));
 706:     assert_eq!(b.binary_search_by(|_, x| x.cmp(&5)), Err(1));
 707:     let b: IndexMap<_, i32> = [1, 2, 4, 6, 8, 9]
 708:         .into_iter()
 709:         .enumerate()
 710:         .map(|(i, x)| (i + 100, x))
 711:         .collect();
 712:     assert_eq!(b.binary_search_by(|_, x| x.cmp(&5)), Err(3));
 713:     assert_eq!(b.binary_search_by(|_, x| x.cmp(&6)), Ok(3));
 714:     assert_eq!(b.binary_search_by(|_, x| x.cmp(&7)), Err(4));
 715:     assert_eq!(b.binary_search_by(|_, x| x.cmp(&8)), Ok(4));
 716:     let b: IndexMap<_, i32> = [1, 2, 4, 5, 6, 8]
 717:         .into_iter()
 718:         .enumerate()
 719:         .map(|(i, x)| (i + 100, x))
 720:         .collect();
 721:     assert_eq!(b.binary_search_by(|_, x| x.cmp(&9)), Err(6));
 722:     let b: IndexMap<_, i32> = [1, 2, 4, 6, 7, 8, 9]
 723:         .into_iter()
 724:         .enumerate()
 725:         .map(|(i, x)| (i + 100, x))
 726:         .collect();
 727:     assert_eq!(b.binary_search_by(|_, x| x.cmp(&6)), Ok(3));
 728:     assert_eq!(b.binary_search_by(|_, x| x.cmp(&5)), Err(3));
 729:     assert_eq!(b.binary_search_by(|_, x| x.cmp(&8)), Ok(5));
 730:     let b: IndexMap<_, i32> = [1, 2, 4, 5, 6, 8, 9]
 731:         .into_iter()
 732:         .enumerate()
 733:         .map(|(i, x)| (i + 100, x))
 734:         .collect();
 735:     assert_eq!(b.binary_search_by(|_, x| x.cmp(&7)), Err(5));
 736:     assert_eq!(b.binary_search_by(|_, x| x.cmp(&0)), Err(0));
 737:     let b: IndexMap<_, i32> = [1, 3, 3, 3, 7]
 738:         .into_iter()
 739:         .enumerate()
 740:         .map(|(i, x)| (i + 100, x))
 741:         .collect();
 742:     assert_eq!(b.binary_search_by(|_, x| x.cmp(&0)), Err(0));
 743:     assert_eq!(b.binary_search_by(|_, x| x.cmp(&1)), Ok(0));
 744:     assert_eq!(b.binary_search_by(|_, x| x.cmp(&2)), Err(1));
 745:     assert!(match b.binary_search_by(|_, x| x.cmp(&3)) {
 746:         Ok(1..=3) => true,
 747:         _ => false,
 748:     });
 749:     assert!(match b.binary_search_by(|_, x| x.cmp(&3)) {
 750:         Ok(1..=3) => true,
 751:         _ => false,
 752:     });
 753:     assert_eq!(b.binary_search_by(|_, x| x.cmp(&4)), Err(4));
 754:     assert_eq!(b.binary_search_by(|_, x| x.cmp(&5)), Err(4));
 755:     assert_eq!(b.binary_search_by(|_, x| x.cmp(&6)), Err(4));
 756:     assert_eq!(b.binary_search_by(|_, x| x.cmp(&7)), Ok(4));
 757:     assert_eq!(b.binary_search_by(|_, x| x.cmp(&8)), Err(5));
 758: }
 759: #[test]
 760: fn test_binary_search_by_key() {
 761:     // adapted from std's test for binary_search
 762:     let b: IndexMap<_, i32> = []
 763:         .into_iter()
 764:         .enumerate()
 765:         .map(|(i, x)| (i + 100, x))
 766:         .collect();
 767:     assert_eq!(b.binary_search_by_key(&5, |_, &x| x), Err(0));
 768:     let b: IndexMap<_, i32> = [4]
 769:         .into_iter()
 770:         .enumerate()
 771:         .map(|(i, x)| (i + 100, x))
 772:         .collect();
 773:     assert_eq!(b.binary_search_by_key(&3, |_, &x| x), Err(0));
 774:     assert_eq!(b.binary_search_by_key(&4, |_, &x| x), Ok(0));
 775:     assert_eq!(b.binary_search_by_key(&5, |_, &x| x), Err(1));
 776:     let b: IndexMap<_, i32> = [1, 2, 4, 6, 8, 9]
 777:         .into_iter()
 778:         .enumerate()
 779:         .map(|(i, x)| (i + 100, x))
 780:         .collect();
 781:     assert_eq!(b.binary_search_by_key(&5, |_, &x| x), Err(3));
 782:     assert_eq!(b.binary_search_by_key(&6, |_, &x| x), Ok(3));
 783:     assert_eq!(b.binary_search_by_key(&7, |_, &x| x), Err(4));
 784:     assert_eq!(b.binary_search_by_key(&8, |_, &x| x), Ok(4));
 785:     let b: IndexMap<_, i32> = [1, 2, 4, 5, 6, 8]
 786:         .into_iter()
 787:         .enumerate()
 788:         .map(|(i, x)| (i + 100, x))
 789:         .collect();
 790:     assert_eq!(b.binary_search_by_key(&9, |_, &x| x), Err(6));
 791:     let b: IndexMap<_, i32> = [1, 2, 4, 6, 7, 8, 9]
 792:         .into_iter()
 793:         .enumerate()
 794:         .map(|(i, x)| (i + 100, x))
 795:         .collect();
 796:     assert_eq!(b.binary_search_by_key(&6, |_, &x| x), Ok(3));
 797:     assert_eq!(b.binary_search_by_key(&5, |_, &x| x), Err(3));
 798:     assert_eq!(b.binary_search_by_key(&8, |_, &x| x), Ok(5));
 799:     let b: IndexMap<_, i32> = [1, 2, 4, 5, 6, 8, 9]
 800:         .into_iter()
 801:         .enumerate()
 802:         .map(|(i, x)| (i + 100, x))
 803:         .collect();
 804:     assert_eq!(b.binary_search_by_key(&7, |_, &x| x), Err(5));
 805:     assert_eq!(b.binary_search_by_key(&0, |_, &x| x), Err(0));
 806:     let b: IndexMap<_, i32> = [1, 3, 3, 3, 7]
 807:         .into_iter()
 808:         .enumerate()
 809:         .map(|(i, x)| (i + 100, x))
 810:         .collect();
 811:     assert_eq!(b.binary_search_by_key(&0, |_, &x| x), Err(0));
 812:     assert_eq!(b.binary_search_by_key(&1, |_, &x| x), Ok(0));
 813:     assert_eq!(b.binary_search_by_key(&2, |_, &x| x), Err(1));
 814:     assert!(match b.binary_search_by_key(&3, |_, &x| x) {
 815:         Ok(1..=3) => true,
 816:         _ => false,
 817:     });
 818:     assert!(match b.binary_search_by_key(&3, |_, &x| x) {
 819:         Ok(1..=3) => true,
 820:         _ => false,
 821:     });
 822:     assert_eq!(b.binary_search_by_key(&4, |_, &x| x), Err(4));
 823:     assert_eq!(b.binary_search_by_key(&5, |_, &x| x), Err(4));
 824:     assert_eq!(b.binary_search_by_key(&6, |_, &x| x), Err(4));
 825:     assert_eq!(b.binary_search_by_key(&7, |_, &x| x), Ok(4));
 826:     assert_eq!(b.binary_search_by_key(&8, |_, &x| x), Err(5));
 827: }
 828: #[test]
 829: fn test_partition_point() {
 830:     // adapted from std's test for partition_point
 831:     let b: IndexMap<_, i32> = []
 832:         .into_iter()
 833:         .enumerate()
 834:         .map(|(i, x)| (i + 100, x))
 835:         .collect();
 836:     assert_eq!(b.partition_point(|_, &x| x < 5), 0);
 837:     let b: IndexMap<_, i32> = [4]
 838:         .into_iter()
 839:         .enumerate()
 840:         .map(|(i, x)| (i + 100, x))
 841:         .collect();
 842:     assert_eq!(b.partition_point(|_, &x| x < 3), 0);
 843:     assert_eq!(b.partition_point(|_, &x| x < 4), 0);
 844:     assert_eq!(b.partition_point(|_, &x| x < 5), 1);
 845:     let b: IndexMap<_, i32> = [1, 2, 4, 6, 8, 9]
 846:         .into_iter()
 847:         .enumerate()
 848:         .map(|(i, x)| (i + 100, x))
 849:         .collect();
 850:     assert_eq!(b.partition_point(|_, &x| x < 5), 3);
 851:     assert_eq!(b.partition_point(|_, &x| x < 6), 3);
 852:     assert_eq!(b.partition_point(|_, &x| x < 7), 4);
 853:     assert_eq!(b.partition_point(|_, &x| x < 8), 4);
 854:     let b: IndexMap<_, i32> = [1, 2, 4, 5, 6, 8]
 855:         .into_iter()
 856:         .enumerate()
 857:         .map(|(i, x)| (i + 100, x))
 858:         .collect();
 859:     assert_eq!(b.partition_point(|_, &x| x < 9), 6);
 860:     let b: IndexMap<_, i32> = [1, 2, 4, 6, 7, 8, 9]
 861:         .into_iter()
 862:         .enumerate()
 863:         .map(|(i, x)| (i + 100, x))
 864:         .collect();
 865:     assert_eq!(b.partition_point(|_, &x| x < 6), 3);
 866:     assert_eq!(b.partition_point(|_, &x| x < 5), 3);
 867:     assert_eq!(b.partition_point(|_, &x| x < 8), 5);
 868:     let b: IndexMap<_, i32> = [1, 2, 4, 5, 6, 8, 9]
 869:         .into_iter()
 870:         .enumerate()
 871:         .map(|(i, x)| (i + 100, x))
 872:         .collect();
 873:     assert_eq!(b.partition_point(|_, &x| x < 7), 5);
 874:     assert_eq!(b.partition_point(|_, &x| x < 0), 0);
 875:     let b: IndexMap<_, i32> = [1, 3, 3, 3, 7]
 876:         .into_iter()
 877:         .enumerate()
 878:         .map(|(i, x)| (i + 100, x))
 879:         .collect();
 880:     assert_eq!(b.partition_point(|_, &x| x < 0), 0);
 881:     assert_eq!(b.partition_point(|_, &x| x < 1), 0);
 882:     assert_eq!(b.partition_point(|_, &x| x < 2), 1);
 883:     assert_eq!(b.partition_point(|_, &x| x < 3), 1);
 884:     assert_eq!(b.partition_point(|_, &x| x < 4), 4);
 885:     assert_eq!(b.partition_point(|_, &x| x < 5), 4);
 886:     assert_eq!(b.partition_point(|_, &x| x < 6), 4);
 887:     assert_eq!(b.partition_point(|_, &x| x < 7), 4);
 888:     assert_eq!(b.partition_point(|_, &x| x < 8), 5);
 889: }
 890: macro_rules! move_index_oob {
 891:     ($test:ident, $from:expr, $to:expr) => {
 892:         #[test]
 893:         #[should_panic(expected = "index out of bounds")]
 894:         fn $test() {
 895:             let mut map: IndexMap<i32, ()> = (0..10).map(|k| (k, ())).collect();
 896:             map.move_index($from, $to);
 897:         }
 898:     };
 899: }
 900: move_index_oob!(test_move_index_out_of_bounds_0_10, 0, 10);
 901: move_index_oob!(test_move_index_out_of_bounds_0_max, 0, usize::MAX);
 902: move_index_oob!(test_move_index_out_of_bounds_10_0, 10, 0);
 903: move_index_oob!(test_move_index_out_of_bounds_max_0, usize::MAX, 0);
 904: #[test]
 905: fn disjoint_mut_empty_map() {
 906:     let mut map: IndexMap<u32, u32> = IndexMap::default();
 907:     assert_eq!(
 908:         map.get_disjoint_mut([&0, &1, &2, &3]),
 909:         [None, None, None, None]
 910:     );
 911: }
 912: #[test]
 913: fn disjoint_mut_empty_param() {
 914:     let mut map: IndexMap<u32, u32> = IndexMap::default();
 915:     map.insert(1, 10);
 916:     assert_eq!(map.get_disjoint_mut([] as [&u32; 0]), []);
 917: }
 918: #[test]
 919: fn disjoint_mut_single_fail() {
 920:     let mut map: IndexMap<u32, u32> = IndexMap::default();
 921:     map.insert(1, 10);
 922:     assert_eq!(map.get_disjoint_mut([&0]), [None]);
 923: }
 924: #[test]
 925: fn disjoint_mut_single_success() {
 926:     let mut map: IndexMap<u32, u32> = IndexMap::default();
 927:     map.insert(1, 10);
 928:     assert_eq!(map.get_disjoint_mut([&1]), [Some(&mut 10)]);
 929: }
 930: #[test]
 931: fn disjoint_mut_multi_success() {
 932:     let mut map: IndexMap<u32, u32> = IndexMap::default();
 933:     map.insert(1, 100);
 934:     map.insert(2, 200);
 935:     map.insert(3, 300);
 936:     map.insert(4, 400);
 937:     assert_eq!(
 938:         map.get_disjoint_mut([&1, &2]),
 939:         [Some(&mut 100), Some(&mut 200)]
 940:     );
 941:     assert_eq!(
 942:         map.get_disjoint_mut([&1, &3]),
 943:         [Some(&mut 100), Some(&mut 300)]
 944:     );
 945:     assert_eq!(
 946:         map.get_disjoint_mut([&3, &1, &4, &2]),
 947:         [
 948:             Some(&mut 300),
 949:             Some(&mut 100),
 950:             Some(&mut 400),
 951:             Some(&mut 200)
 952:         ]
 953:     );
 954: }
 955: #[test]
 956: fn disjoint_mut_multi_success_unsized_key() {
 957:     let mut map: IndexMap<&'static str, u32> = IndexMap::default();
 958:     map.insert("1", 100);
 959:     map.insert("2", 200);
 960:     map.insert("3", 300);
 961:     map.insert("4", 400);
 962:     assert_eq!(
 963:         map.get_disjoint_mut(["1", "2"]),
 964:         [Some(&mut 100), Some(&mut 200)]
 965:     );
 966:     assert_eq!(
 967:         map.get_disjoint_mut(["1", "3"]),
 968:         [Some(&mut 100), Some(&mut 300)]
 969:     );
 970:     assert_eq!(
 971:         map.get_disjoint_mut(["3", "1", "4", "2"]),
 972:         [
 973:             Some(&mut 300),
 974:             Some(&mut 100),
 975:             Some(&mut 400),
 976:             Some(&mut 200)
 977:         ]
 978:     );
 979: }
 980: #[test]
 981: fn disjoint_mut_multi_success_borrow_key() {
 982:     let mut map: IndexMap<String, u32> = IndexMap::default();
 983:     map.insert("1".into(), 100);
 984:     map.insert("2".into(), 200);
 985:     map.insert("3".into(), 300);
 986:     map.insert("4".into(), 400);
 987:     assert_eq!(
 988:         map.get_disjoint_mut(["1", "2"]),
 989:         [Some(&mut 100), Some(&mut 200)]
 990:     );
 991:     assert_eq!(
 992:         map.get_disjoint_mut(["1", "3"]),
 993:         [Some(&mut 100), Some(&mut 300)]
 994:     );
 995:     assert_eq!(
 996:         map.get_disjoint_mut(["3", "1", "4", "2"]),
 997:         [
 998:             Some(&mut 300),
 999:             Some(&mut 100),
1000:             Some(&mut 400),
1001:             Some(&mut 200)
1002:         ]
1003:     );
1004: }
1005: #[test]
1006: fn disjoint_mut_multi_fail_missing() {
1007:     let mut map: IndexMap<u32, u32> = IndexMap::default();
1008:     map.insert(1, 100);
1009:     map.insert(2, 200);
1010:     map.insert(3, 300);
1011:     map.insert(4, 400);
1012:     assert_eq!(map.get_disjoint_mut([&1, &5]), [Some(&mut 100), None]);
1013:     assert_eq!(map.get_disjoint_mut([&5, &6]), [None, None]);
1014:     assert_eq!(
1015:         map.get_disjoint_mut([&1, &5, &4]),
1016:         [Some(&mut 100), None, Some(&mut 400)]
1017:     );
1018: }
1019: #[test]
1020: #[should_panic]
1021: fn disjoint_mut_multi_fail_duplicate_panic() {
1022:     let mut map: IndexMap<u32, u32> = IndexMap::default();
1023:     map.insert(1, 100);
1024:     map.get_disjoint_mut([&1, &2, &1]);
1025: }
1026: #[test]
1027: fn disjoint_indices_mut_fail_oob() {
1028:     let mut map: IndexMap<u32, u32> = IndexMap::default();
1029:     map.insert(1, 10);
1030:     map.insert(321, 20);
1031:     assert_eq!(
1032:         map.get_disjoint_indices_mut([1, 3]),
1033:         Err(crate::GetDisjointMutError::IndexOutOfBounds)
1034:     );
1035: }
1036: #[test]
1037: fn disjoint_indices_mut_empty() {
1038:     let mut map: IndexMap<u32, u32> = IndexMap::default();
1039:     map.insert(1, 10);
1040:     map.insert(321, 20);
1041:     assert_eq!(map.get_disjoint_indices_mut([]), Ok([]));
1042: }
1043: #[test]
1044: fn disjoint_indices_mut_success() {
1045:     let mut map: IndexMap<u32, u32> = IndexMap::default();
1046:     map.insert(1, 10);
1047:     map.insert(321, 20);
1048:     assert_eq!(map.get_disjoint_indices_mut([0]), Ok([(&1, &mut 10)]));
1049:     assert_eq!(map.get_disjoint_indices_mut([1]), Ok([(&321, &mut 20)]));
1050:     assert_eq!(
1051:         map.get_disjoint_indices_mut([0, 1]),
1052:         Ok([(&1, &mut 10), (&321, &mut 20)])
1053:     );
1054: }
1055: #[test]
1056: fn disjoint_indices_mut_fail_duplicate() {
1057:     let mut map: IndexMap<u32, u32> = IndexMap::default();
1058:     map.insert(1, 10);
1059:     map.insert(321, 20);
1060:     assert_eq!(
1061:         map.get_disjoint_indices_mut([1, 0, 1]),
1062:         Err(crate::GetDisjointMutError::OverlappingIndices)
1063:     );
1064: }
1065: #[test]
1066: fn insert_sorted_by_key() {
1067:     let mut values = [(-1, 8), (3, 18), (-27, 2), (-2, 5)];
1068:     let mut map: IndexMap<i32, i32> = IndexMap::new();
1069:     for (key, value) in values {
1070:         let (_, old) = map.insert_sorted_by_key(key, value, |k, _| k.abs());
1071:         assert_eq!(old, None);
1072:     }
1073:     values.sort_by_key(|(key, _)| key.abs());
1074:     assert_eq!(values, *map.as_slice());
1075:     for (key, value) in &mut values {
1076:         let (_, old) = map.insert_sorted_by_key(*key, -*value, |k, _| k.abs());
1077:         assert_eq!(old, Some(*value));
1078:         *value = -*value;
1079:     }
1080:     assert_eq!(values, *map.as_slice());
1081: }
1082: #[test]
1083: fn insert_sorted_by() {
1084:     let mut values = [(1, 1), (2, 2), (3, 3), (4, 4), (5, 5)];
1085:     let mut map: IndexMap<i32, i32> = IndexMap::new();
1086:     for (key, value) in values {
1087:         let (_, old) = map.insert_sorted_by(key, value, |key1, _, key2, _| key2.cmp(key1));
1088:         assert_eq!(old, None);
1089:     }
1090:     values.reverse();
1091:     assert_eq!(values, *map.as_slice());
1092:     for (key, value) in &mut values {
1093:         let (_, old) = map.insert_sorted_by(*key, -*value, |key1, _, key2, _| key2.cmp(key1));
1094:         assert_eq!(old, Some(*value));
1095:         *value = -*value;
1096:     }
1097:     assert_eq!(values, *map.as_slice());
1098: }
1099: #[test]
1100: fn is_sorted() {
1101:     fn expect(map: &IndexMap<i32, i32>, e: [bool; 7]) {
1102:         assert_eq!(e[0], map.is_sorted());
1103:         assert_eq!(e[1], map.is_sorted_by(|k1, _, k2, _| k1 < k2));
1104:         assert_eq!(e[2], map.is_sorted_by(|k1, _, k2, _| k1 > k2));
1105:         assert_eq!(e[3], map.is_sorted_by(|_, v1, _, v2| v1 < v2));
1106:         assert_eq!(e[4], map.is_sorted_by(|_, v1, _, v2| v1 > v2));
1107:         assert_eq!(e[5], map.is_sorted_by_key(|k, _| k));
1108:         assert_eq!(e[6], map.is_sorted_by_key(|_, v| v));
1109:     }
1110:     let mut map = IndexMap::from_iter((0..10).map(|i| (i, i * i)));
1111:     expect(&map, [true, true, false, true, false, true, true]);
1112:     map[5] = -1;
1113:     expect(&map, [true, true, false, false, false, true, false]);
1114:     map[5] = 25;
1115:     map.replace_index(5, -1).unwrap();
1116:     expect(&map, [false, false, false, true, false, false, true]);
1117: }
1118: #[test]
1119: fn is_sorted_trivial() {
1120:     fn expect(map: &IndexMap<i32, i32>, e: [bool; 5]) {
1121:         assert_eq!(e[0], map.is_sorted());
1122:         assert_eq!(e[1], map.is_sorted_by(|_, _, _, _| true));
1123:         assert_eq!(e[2], map.is_sorted_by(|_, _, _, _| false));
1124:         assert_eq!(e[3], map.is_sorted_by_key(|_, _| 0f64));
1125:         assert_eq!(e[4], map.is_sorted_by_key(|_, _| f64::NAN));
1126:     }
1127:     let mut map = IndexMap::new();
1128:     expect(&map, [true, true, true, true, true]);
1129:     map.insert(0, 0);
1130:     expect(&map, [true, true, true, true, true]);
1131:     map.insert(1, 1);
1132:     expect(&map, [true, true, false, true, false]);
1133:     map.reverse();
1134:     expect(&map, [false, true, false, true, false]);
1135: }
````

## File: src/rayon/map.rs
````rust
  1: //! Parallel iterator types for [`IndexMap`] with [`rayon`][::rayon].
  2: //!
  3: //! You will rarely need to interact with this module directly unless you need to name one of the
  4: //! iterator types.
  5: use super::collect;
  6: use rayon::iter::plumbing::{Consumer, ProducerCallback, UnindexedConsumer};
  7: use rayon::prelude::*;
  8: use alloc::boxed::Box;
  9: use alloc::vec::Vec;
 10: use core::cmp::Ordering;
 11: use core::fmt;
 12: use core::hash::{BuildHasher, Hash};
 13: use core::ops::RangeBounds;
 14: use crate::Bucket;
 15: use crate::IndexMap;
 16: use crate::map::Slice;
 17: impl<K, V, S> IntoParallelIterator for IndexMap<K, V, S>
 18: where
 19:     K: Send,
 20:     V: Send,
 21: {
 22:     type Item = (K, V);
 23:     type Iter = IntoParIter<K, V>;
 24:     fn into_par_iter(self) -> Self::Iter {
 25:         IntoParIter {
 26:             entries: self.into_entries(),
 27:         }
 28:     }
 29: }
 30: impl<K, V> IntoParallelIterator for Box<Slice<K, V>>
 31: where
 32:     K: Send,
 33:     V: Send,
 34: {
 35:     type Item = (K, V);
 36:     type Iter = IntoParIter<K, V>;
 37:     fn into_par_iter(self) -> Self::Iter {
 38:         IntoParIter {
 39:             entries: self.into_entries(),
 40:         }
 41:     }
 42: }
 43: /// A parallel owning iterator over the entries of an [`IndexMap`].
 44: ///
 45: /// This `struct` is created by the [`IndexMap::into_par_iter`] method
 46: /// (provided by rayon's [`IntoParallelIterator`] trait). See its documentation for more.
 47: pub struct IntoParIter<K, V> {
 48:     entries: Vec<Bucket<K, V>>,
 49: }
 50: impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for IntoParIter<K, V> {
 51:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
 52:         let iter = self.entries.iter().map(Bucket::refs);
 53:         f.debug_list().entries(iter).finish()
 54:     }
 55: }
 56: impl<K: Send, V: Send> ParallelIterator for IntoParIter<K, V> {
 57:     type Item = (K, V);
 58:     parallel_iterator_methods!(Bucket::key_value);
 59: }
 60: impl<K: Send, V: Send> IndexedParallelIterator for IntoParIter<K, V> {
 61:     indexed_parallel_iterator_methods!(Bucket::key_value);
 62: }
 63: impl<'a, K, V, S> IntoParallelIterator for &'a IndexMap<K, V, S>
 64: where
 65:     K: Sync,
 66:     V: Sync,
 67: {
 68:     type Item = (&'a K, &'a V);
 69:     type Iter = ParIter<'a, K, V>;
 70:     fn into_par_iter(self) -> Self::Iter {
 71:         ParIter {
 72:             entries: self.as_entries(),
 73:         }
 74:     }
 75: }
 76: impl<'a, K, V> IntoParallelIterator for &'a Slice<K, V>
 77: where
 78:     K: Sync,
 79:     V: Sync,
 80: {
 81:     type Item = (&'a K, &'a V);
 82:     type Iter = ParIter<'a, K, V>;
 83:     fn into_par_iter(self) -> Self::Iter {
 84:         ParIter {
 85:             entries: &self.entries,
 86:         }
 87:     }
 88: }
 89: /// A parallel iterator over the entries of an [`IndexMap`].
 90: ///
 91: /// This `struct` is created by the [`IndexMap::par_iter`] method
 92: /// (provided by rayon's [`IntoParallelRefIterator`] trait). See its documentation for more.
 93: ///
 94: /// [`IndexMap::par_iter`]: ../struct.IndexMap.html#method.par_iter
 95: pub struct ParIter<'a, K, V> {
 96:     entries: &'a [Bucket<K, V>],
 97: }
 98: impl<K, V> Clone for ParIter<'_, K, V> {
 99:     fn clone(&self) -> Self {
100:         ParIter { ..*self }
101:     }
102: }
103: impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for ParIter<'_, K, V> {
104:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
105:         let iter = self.entries.iter().map(Bucket::refs);
106:         f.debug_list().entries(iter).finish()
107:     }
108: }
109: impl<'a, K: Sync, V: Sync> ParallelIterator for ParIter<'a, K, V> {
110:     type Item = (&'a K, &'a V);
111:     parallel_iterator_methods!(Bucket::refs);
112: }
113: impl<K: Sync, V: Sync> IndexedParallelIterator for ParIter<'_, K, V> {
114:     indexed_parallel_iterator_methods!(Bucket::refs);
115: }
116: impl<'a, K, V, S> IntoParallelIterator for &'a mut IndexMap<K, V, S>
117: where
118:     K: Sync + Send,
119:     V: Send,
120: {
121:     type Item = (&'a K, &'a mut V);
122:     type Iter = ParIterMut<'a, K, V>;
123:     fn into_par_iter(self) -> Self::Iter {
124:         ParIterMut {
125:             entries: self.as_entries_mut(),
126:         }
127:     }
128: }
129: impl<'a, K, V> IntoParallelIterator for &'a mut Slice<K, V>
130: where
131:     K: Sync + Send,
132:     V: Send,
133: {
134:     type Item = (&'a K, &'a mut V);
135:     type Iter = ParIterMut<'a, K, V>;
136:     fn into_par_iter(self) -> Self::Iter {
137:         ParIterMut {
138:             entries: &mut self.entries,
139:         }
140:     }
141: }
142: /// A parallel mutable iterator over the entries of an [`IndexMap`].
143: ///
144: /// This `struct` is created by the [`IndexMap::par_iter_mut`] method
145: /// (provided by rayon's [`IntoParallelRefMutIterator`] trait). See its documentation for more.
146: ///
147: /// [`IndexMap::par_iter_mut`]: ../struct.IndexMap.html#method.par_iter_mut
148: pub struct ParIterMut<'a, K, V> {
149:     entries: &'a mut [Bucket<K, V>],
150: }
151: impl<K: fmt::Debug, V: fmt::Debug> fmt::Debug for ParIterMut<'_, K, V> {
152:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
153:         let iter = self.entries.iter().map(Bucket::refs);
154:         f.debug_list().entries(iter).finish()
155:     }
156: }
157: impl<'a, K: Sync + Send, V: Send> ParallelIterator for ParIterMut<'a, K, V> {
158:     type Item = (&'a K, &'a mut V);
159:     parallel_iterator_methods!(Bucket::ref_mut);
160: }
161: impl<K: Sync + Send, V: Send> IndexedParallelIterator for ParIterMut<'_, K, V> {
162:     indexed_parallel_iterator_methods!(Bucket::ref_mut);
163: }
164: impl<'a, K, V, S> ParallelDrainRange<usize> for &'a mut IndexMap<K, V, S>
165: where
166:     K: Send,
167:     V: Send,
168: {
169:     type Item = (K, V);
170:     type Iter = ParDrain<'a, K, V>;
171:     fn par_drain<R: RangeBounds<usize>>(self, range: R) -> Self::Iter {
172:         ParDrain {
173:             entries: self.core.par_drain(range),
174:         }
175:     }
176: }
177: /// A parallel draining iterator over the entries of an [`IndexMap`].
178: ///
179: /// This `struct` is created by the [`IndexMap::par_drain`] method
180: /// (provided by rayon's [`ParallelDrainRange`] trait). See its documentation for more.
181: ///
182: /// [`IndexMap::par_drain`]: ../struct.IndexMap.html#method.par_drain
183: pub struct ParDrain<'a, K: Send, V: Send> {
184:     entries: rayon::vec::Drain<'a, Bucket<K, V>>,
185: }
186: impl<K: Send, V: Send> ParallelIterator for ParDrain<'_, K, V> {
187:     type Item = (K, V);
188:     parallel_iterator_methods!(Bucket::key_value);
189: }
190: impl<K: Send, V: Send> IndexedParallelIterator for ParDrain<'_, K, V> {
191:     indexed_parallel_iterator_methods!(Bucket::key_value);
192: }
193: /// Parallel iterator methods and other parallel methods.
194: ///
195: /// The following methods **require crate feature `"rayon"`**.
196: ///
197: /// See also the `IntoParallelIterator` implementations.
198: impl<K, V, S> IndexMap<K, V, S>
199: where
200:     K: Sync,
201:     V: Sync,
202: {
203:     /// Return a parallel iterator over the keys of the map.
204:     ///
205:     /// While parallel iterators can process items in any order, their relative order
206:     /// in the map is still preserved for operations like `reduce` and `collect`.
207:     pub fn par_keys(&self) -> ParKeys<'_, K, V> {
208:         ParKeys {
209:             entries: self.as_entries(),
210:         }
211:     }
212:     /// Return a parallel iterator over the values of the map.
213:     ///
214:     /// While parallel iterators can process items in any order, their relative order
215:     /// in the map is still preserved for operations like `reduce` and `collect`.
216:     pub fn par_values(&self) -> ParValues<'_, K, V> {
217:         ParValues {
218:             entries: self.as_entries(),
219:         }
220:     }
221: }
222: /// Parallel iterator methods and other parallel methods.
223: ///
224: /// The following methods **require crate feature `"rayon"`**.
225: ///
226: /// See also the `IntoParallelIterator` implementations.
227: impl<K, V> Slice<K, V>
228: where
229:     K: Sync,
230:     V: Sync,
231: {
232:     /// Return a parallel iterator over the keys of the map slice.
233:     ///
234:     /// While parallel iterators can process items in any order, their relative order
235:     /// in the slice is still preserved for operations like `reduce` and `collect`.
236:     pub fn par_keys(&self) -> ParKeys<'_, K, V> {
237:         ParKeys {
238:             entries: &self.entries,
239:         }
240:     }
241:     /// Return a parallel iterator over the values of the map slice.
242:     ///
243:     /// While parallel iterators can process items in any order, their relative order
244:     /// in the slice is still preserved for operations like `reduce` and `collect`.
245:     pub fn par_values(&self) -> ParValues<'_, K, V> {
246:         ParValues {
247:             entries: &self.entries,
248:         }
249:     }
250: }
251: impl<K, V, S> IndexMap<K, V, S>
252: where
253:     K: Hash + Eq + Sync,
254:     V: Sync,
255:     S: BuildHasher,
256: {
257:     /// Returns `true` if `self` contains all of the same key-value pairs as `other`,
258:     /// regardless of each map's indexed order, determined in parallel.
259:     pub fn par_eq<V2, S2>(&self, other: &IndexMap<K, V2, S2>) -> bool
260:     where
261:         V: PartialEq<V2>,
262:         V2: Sync,
263:         S2: BuildHasher + Sync,
264:     {
265:         self.len() == other.len()
266:             && self
267:                 .par_iter()
268:                 .all(move |(key, value)| other.get(key).map_or(false, |v| *value == *v))
269:     }
270: }
271: /// A parallel iterator over the keys of an [`IndexMap`].
272: ///
273: /// This `struct` is created by the [`IndexMap::par_keys`] method.
274: /// See its documentation for more.
275: pub struct ParKeys<'a, K, V> {
276:     entries: &'a [Bucket<K, V>],
277: }
278: impl<K, V> Clone for ParKeys<'_, K, V> {
279:     fn clone(&self) -> Self {
280:         ParKeys { ..*self }
281:     }
282: }
283: impl<K: fmt::Debug, V> fmt::Debug for ParKeys<'_, K, V> {
284:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
285:         let iter = self.entries.iter().map(Bucket::key_ref);
286:         f.debug_list().entries(iter).finish()
287:     }
288: }
289: impl<'a, K: Sync, V: Sync> ParallelIterator for ParKeys<'a, K, V> {
290:     type Item = &'a K;
291:     parallel_iterator_methods!(Bucket::key_ref);
292: }
293: impl<K: Sync, V: Sync> IndexedParallelIterator for ParKeys<'_, K, V> {
294:     indexed_parallel_iterator_methods!(Bucket::key_ref);
295: }
296: /// A parallel iterator over the values of an [`IndexMap`].
297: ///
298: /// This `struct` is created by the [`IndexMap::par_values`] method.
299: /// See its documentation for more.
300: pub struct ParValues<'a, K, V> {
301:     entries: &'a [Bucket<K, V>],
302: }
303: impl<K, V> Clone for ParValues<'_, K, V> {
304:     fn clone(&self) -> Self {
305:         ParValues { ..*self }
306:     }
307: }
308: impl<K, V: fmt::Debug> fmt::Debug for ParValues<'_, K, V> {
309:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
310:         let iter = self.entries.iter().map(Bucket::value_ref);
311:         f.debug_list().entries(iter).finish()
312:     }
313: }
314: impl<'a, K: Sync, V: Sync> ParallelIterator for ParValues<'a, K, V> {
315:     type Item = &'a V;
316:     parallel_iterator_methods!(Bucket::value_ref);
317: }
318: impl<K: Sync, V: Sync> IndexedParallelIterator for ParValues<'_, K, V> {
319:     indexed_parallel_iterator_methods!(Bucket::value_ref);
320: }
321: impl<K, V, S> IndexMap<K, V, S>
322: where
323:     K: Send,
324:     V: Send,
325: {
326:     /// Return a parallel iterator over mutable references to the values of the map
327:     ///
328:     /// While parallel iterators can process items in any order, their relative order
329:     /// in the map is still preserved for operations like `reduce` and `collect`.
330:     pub fn par_values_mut(&mut self) -> ParValuesMut<'_, K, V> {
331:         ParValuesMut {
332:             entries: self.as_entries_mut(),
333:         }
334:     }
335: }
336: impl<K, V> Slice<K, V>
337: where
338:     K: Send,
339:     V: Send,
340: {
341:     /// Return a parallel iterator over mutable references to the values of the map slice.
342:     ///
343:     /// While parallel iterators can process items in any order, their relative order
344:     /// in the slice is still preserved for operations like `reduce` and `collect`.
345:     pub fn par_values_mut(&mut self) -> ParValuesMut<'_, K, V> {
346:         ParValuesMut {
347:             entries: &mut self.entries,
348:         }
349:     }
350: }
351: impl<K, V, S> IndexMap<K, V, S>
352: where
353:     K: Send,
354:     V: Send,
355: {
356:     /// Sort the map's key-value pairs in parallel, by the default ordering of the keys.
357:     pub fn par_sort_keys(&mut self)
358:     where
359:         K: Ord,
360:     {
361:         self.with_entries(|entries| {
362:             entries.par_sort_by(|a, b| K::cmp(&a.key, &b.key));
363:         });
364:     }
365:     /// Sort the map's key-value pairs in place and in parallel, using the comparison
366:     /// function `cmp`.
367:     ///
368:     /// The comparison function receives two key and value pairs to compare (you
369:     /// can sort by keys or values or their combination as needed).
370:     pub fn par_sort_by<F>(&mut self, cmp: F)
371:     where
372:         F: Fn(&K, &V, &K, &V) -> Ordering + Sync,
373:     {
374:         self.with_entries(|entries| {
375:             entries.par_sort_by(move |a, b| cmp(&a.key, &a.value, &b.key, &b.value));
376:         });
377:     }
378:     /// Sort the key-value pairs of the map in parallel and return a by-value parallel
379:     /// iterator of the key-value pairs with the result.
380:     pub fn par_sorted_by<F>(self, cmp: F) -> IntoParIter<K, V>
381:     where
382:         F: Fn(&K, &V, &K, &V) -> Ordering + Sync,
383:     {
384:         let mut entries = self.into_entries();
385:         entries.par_sort_by(move |a, b| cmp(&a.key, &a.value, &b.key, &b.value));
386:         IntoParIter { entries }
387:     }
388:     /// Sort the map's key-value pairs in place and in parallel, using a sort-key extraction
389:     /// function.
390:     pub fn par_sort_by_key<T, F>(&mut self, sort_key: F)
391:     where
392:         T: Ord,
393:         F: Fn(&K, &V) -> T + Sync,
394:     {
395:         self.with_entries(move |entries| {
396:             entries.par_sort_by_key(move |a| sort_key(&a.key, &a.value));
397:         });
398:     }
399:     /// Sort the map's key-value pairs in parallel, by the default ordering of the keys.
400:     pub fn par_sort_unstable_keys(&mut self)
401:     where
402:         K: Ord,
403:     {
404:         self.with_entries(|entries| {
405:             entries.par_sort_unstable_by(|a, b| K::cmp(&a.key, &b.key));
406:         });
407:     }
408:     /// Sort the map's key-value pairs in place and in parallel, using the comparison
409:     /// function `cmp`.
410:     ///
411:     /// The comparison function receives two key and value pairs to compare (you
412:     /// can sort by keys or values or their combination as needed).
413:     pub fn par_sort_unstable_by<F>(&mut self, cmp: F)
414:     where
415:         F: Fn(&K, &V, &K, &V) -> Ordering + Sync,
416:     {
417:         self.with_entries(|entries| {
418:             entries.par_sort_unstable_by(move |a, b| cmp(&a.key, &a.value, &b.key, &b.value));
419:         });
420:     }
421:     /// Sort the key-value pairs of the map in parallel and return a by-value parallel
422:     /// iterator of the key-value pairs with the result.
423:     pub fn par_sorted_unstable_by<F>(self, cmp: F) -> IntoParIter<K, V>
424:     where
425:         F: Fn(&K, &V, &K, &V) -> Ordering + Sync,
426:     {
427:         let mut entries = self.into_entries();
428:         entries.par_sort_unstable_by(move |a, b| cmp(&a.key, &a.value, &b.key, &b.value));
429:         IntoParIter { entries }
430:     }
431:     /// Sort the map's key-value pairs in place and in parallel, using a sort-key extraction
432:     /// function.
433:     pub fn par_sort_unstable_by_key<T, F>(&mut self, sort_key: F)
434:     where
435:         T: Ord,
436:         F: Fn(&K, &V) -> T + Sync,
437:     {
438:         self.with_entries(move |entries| {
439:             entries.par_sort_unstable_by_key(move |a| sort_key(&a.key, &a.value));
440:         });
441:     }
442:     /// Sort the map's key-value pairs in place and in parallel, using a sort-key extraction
443:     /// function.
444:     pub fn par_sort_by_cached_key<T, F>(&mut self, sort_key: F)
445:     where
446:         T: Ord + Send,
447:         F: Fn(&K, &V) -> T + Sync,
448:     {
449:         self.with_entries(move |entries| {
450:             entries.par_sort_by_cached_key(move |a| sort_key(&a.key, &a.value));
451:         });
452:     }
453: }
454: /// A parallel mutable iterator over the values of an [`IndexMap`].
455: ///
456: /// This `struct` is created by the [`IndexMap::par_values_mut`] method.
457: /// See its documentation for more.
458: pub struct ParValuesMut<'a, K, V> {
459:     entries: &'a mut [Bucket<K, V>],
460: }
461: impl<K, V: fmt::Debug> fmt::Debug for ParValuesMut<'_, K, V> {
462:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
463:         let iter = self.entries.iter().map(Bucket::value_ref);
464:         f.debug_list().entries(iter).finish()
465:     }
466: }
467: impl<'a, K: Send, V: Send> ParallelIterator for ParValuesMut<'a, K, V> {
468:     type Item = &'a mut V;
469:     parallel_iterator_methods!(Bucket::value_mut);
470: }
471: impl<K: Send, V: Send> IndexedParallelIterator for ParValuesMut<'_, K, V> {
472:     indexed_parallel_iterator_methods!(Bucket::value_mut);
473: }
474: impl<K, V, S> FromParallelIterator<(K, V)> for IndexMap<K, V, S>
475: where
476:     K: Eq + Hash + Send,
477:     V: Send,
478:     S: BuildHasher + Default + Send,
479: {
480:     fn from_par_iter<I>(iter: I) -> Self
481:     where
482:         I: IntoParallelIterator<Item = (K, V)>,
483:     {
484:         let list = collect(iter);
485:         let len = list.iter().map(Vec::len).sum();
486:         let mut map = Self::with_capacity_and_hasher(len, S::default());
487:         for vec in list {
488:             map.extend(vec);
489:         }
490:         map
491:     }
492: }
493: impl<K, V, S> ParallelExtend<(K, V)> for IndexMap<K, V, S>
494: where
495:     K: Eq + Hash + Send,
496:     V: Send,
497:     S: BuildHasher + Send,
498: {
499:     fn par_extend<I>(&mut self, iter: I)
500:     where
501:         I: IntoParallelIterator<Item = (K, V)>,
502:     {
503:         for vec in collect(iter) {
504:             self.extend(vec);
505:         }
506:     }
507: }
508: impl<'a, K: 'a, V: 'a, S> ParallelExtend<(&'a K, &'a V)> for IndexMap<K, V, S>
509: where
510:     K: Copy + Eq + Hash + Send + Sync,
511:     V: Copy + Send + Sync,
512:     S: BuildHasher + Send,
513: {
514:     fn par_extend<I>(&mut self, iter: I)
515:     where
516:         I: IntoParallelIterator<Item = (&'a K, &'a V)>,
517:     {
518:         for vec in collect(iter) {
519:             self.extend(vec);
520:         }
521:     }
522: }
523: #[cfg(test)]
524: mod tests {
525:     use super::*;
526:     use std::string::String;
527:     #[test]
528:     fn insert_order() {
529:         let insert = [0, 4, 2, 12, 8, 7, 11, 5, 3, 17, 19, 22, 23];
530:         let mut map = IndexMap::new();
531:         for &elt in &insert {
532:             map.insert(elt, ());
533:         }
534:         assert_eq!(map.par_keys().count(), map.len());
535:         assert_eq!(map.par_keys().count(), insert.len());
536:         insert.par_iter().zip(map.par_keys()).for_each(|(a, b)| {
537:             assert_eq!(a, b);
538:         });
539:         (0..insert.len())
540:             .into_par_iter()
541:             .zip(map.par_keys())
542:             .for_each(|(i, k)| {
543:                 assert_eq!(map.get_index(i).unwrap().0, k);
544:             });
545:     }
546:     #[test]
547:     fn partial_eq_and_eq() {
548:         let mut map_a = IndexMap::new();
549:         map_a.insert(1, "1");
550:         map_a.insert(2, "2");
551:         let mut map_b = map_a.clone();
552:         assert!(map_a.par_eq(&map_b));
553:         map_b.swap_remove(&1);
554:         assert!(!map_a.par_eq(&map_b));
555:         map_b.insert(3, "3");
556:         assert!(!map_a.par_eq(&map_b));
557:         let map_c: IndexMap<_, String> =
558:             map_b.into_par_iter().map(|(k, v)| (k, v.into())).collect();
559:         assert!(!map_a.par_eq(&map_c));
560:         assert!(!map_c.par_eq(&map_a));
561:     }
562:     #[test]
563:     fn extend() {
564:         let mut map = IndexMap::new();
565:         map.par_extend(vec![(&1, &2), (&3, &4)]);
566:         map.par_extend(vec![(5, 6)]);
567:         assert_eq!(
568:             map.into_par_iter().collect::<Vec<_>>(),
569:             vec![(1, 2), (3, 4), (5, 6)]
570:         );
571:     }
572:     #[test]
573:     fn keys() {
574:         let vec = vec![(1, 'a'), (2, 'b'), (3, 'c')];
575:         let map: IndexMap<_, _> = vec.into_par_iter().collect();
576:         let keys: Vec<_> = map.par_keys().copied().collect();
577:         assert_eq!(keys.len(), 3);
578:         assert!(keys.contains(&1));
579:         assert!(keys.contains(&2));
580:         assert!(keys.contains(&3));
581:     }
582:     #[test]
583:     fn values() {
584:         let vec = vec![(1, 'a'), (2, 'b'), (3, 'c')];
585:         let map: IndexMap<_, _> = vec.into_par_iter().collect();
586:         let values: Vec<_> = map.par_values().copied().collect();
587:         assert_eq!(values.len(), 3);
588:         assert!(values.contains(&'a'));
589:         assert!(values.contains(&'b'));
590:         assert!(values.contains(&'c'));
591:     }
592:     #[test]
593:     fn values_mut() {
594:         let vec = vec![(1, 1), (2, 2), (3, 3)];
595:         let mut map: IndexMap<_, _> = vec.into_par_iter().collect();
596:         map.par_values_mut().for_each(|value| *value *= 2);
597:         let values: Vec<_> = map.par_values().copied().collect();
598:         assert_eq!(values.len(), 3);
599:         assert!(values.contains(&2));
600:         assert!(values.contains(&4));
601:         assert!(values.contains(&6));
602:     }
603: }
````

## File: src/rayon/mod.rs
````rust
 1: #![cfg_attr(docsrs, doc(cfg(feature = "rayon")))]
 2: use rayon::prelude::*;
 3: use alloc::collections::LinkedList;
 4: use alloc::vec::Vec;
 5: pub mod map;
 6: pub mod set;
 7: // This form of intermediate collection is also how Rayon collects `HashMap`.
 8: // Note that the order will also be preserved!
 9: fn collect<I: IntoParallelIterator>(iter: I) -> LinkedList<Vec<I::Item>> {
10:     iter.into_par_iter().collect_vec_list()
11: }
````

## File: src/rayon/set.rs
````rust
  1: //! Parallel iterator types for [`IndexSet`] with [rayon][::rayon].
  2: //!
  3: //! You will rarely need to interact with this module directly unless you need to name one of the
  4: //! iterator types.
  5: use super::collect;
  6: use rayon::iter::plumbing::{Consumer, ProducerCallback, UnindexedConsumer};
  7: use rayon::prelude::*;
  8: use alloc::boxed::Box;
  9: use alloc::vec::Vec;
 10: use core::cmp::Ordering;
 11: use core::fmt;
 12: use core::hash::{BuildHasher, Hash};
 13: use core::ops::RangeBounds;
 14: use crate::IndexSet;
 15: use crate::set::Slice;
 16: type Bucket<T> = crate::Bucket<T, ()>;
 17: impl<T, S> IntoParallelIterator for IndexSet<T, S>
 18: where
 19:     T: Send,
 20: {
 21:     type Item = T;
 22:     type Iter = IntoParIter<T>;
 23:     fn into_par_iter(self) -> Self::Iter {
 24:         IntoParIter {
 25:             entries: self.into_entries(),
 26:         }
 27:     }
 28: }
 29: impl<T> IntoParallelIterator for Box<Slice<T>>
 30: where
 31:     T: Send,
 32: {
 33:     type Item = T;
 34:     type Iter = IntoParIter<T>;
 35:     fn into_par_iter(self) -> Self::Iter {
 36:         IntoParIter {
 37:             entries: self.into_entries(),
 38:         }
 39:     }
 40: }
 41: /// A parallel owning iterator over the items of an [`IndexSet`].
 42: ///
 43: /// This `struct` is created by the [`IndexSet::into_par_iter`] method
 44: /// (provided by rayon's [`IntoParallelIterator`] trait). See its documentation for more.
 45: pub struct IntoParIter<T> {
 46:     entries: Vec<Bucket<T>>,
 47: }
 48: impl<T: fmt::Debug> fmt::Debug for IntoParIter<T> {
 49:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
 50:         let iter = self.entries.iter().map(Bucket::key_ref);
 51:         f.debug_list().entries(iter).finish()
 52:     }
 53: }
 54: impl<T: Send> ParallelIterator for IntoParIter<T> {
 55:     type Item = T;
 56:     parallel_iterator_methods!(Bucket::key);
 57: }
 58: impl<T: Send> IndexedParallelIterator for IntoParIter<T> {
 59:     indexed_parallel_iterator_methods!(Bucket::key);
 60: }
 61: impl<'a, T, S> IntoParallelIterator for &'a IndexSet<T, S>
 62: where
 63:     T: Sync,
 64: {
 65:     type Item = &'a T;
 66:     type Iter = ParIter<'a, T>;
 67:     fn into_par_iter(self) -> Self::Iter {
 68:         ParIter {
 69:             entries: self.as_entries(),
 70:         }
 71:     }
 72: }
 73: impl<'a, T> IntoParallelIterator for &'a Slice<T>
 74: where
 75:     T: Sync,
 76: {
 77:     type Item = &'a T;
 78:     type Iter = ParIter<'a, T>;
 79:     fn into_par_iter(self) -> Self::Iter {
 80:         ParIter {
 81:             entries: &self.entries,
 82:         }
 83:     }
 84: }
 85: /// A parallel iterator over the items of an [`IndexSet`].
 86: ///
 87: /// This `struct` is created by the [`IndexSet::par_iter`] method
 88: /// (provided by rayon's [`IntoParallelRefIterator`] trait). See its documentation for more.
 89: ///
 90: /// [`IndexSet::par_iter`]: ../struct.IndexSet.html#method.par_iter
 91: pub struct ParIter<'a, T> {
 92:     entries: &'a [Bucket<T>],
 93: }
 94: impl<T> Clone for ParIter<'_, T> {
 95:     fn clone(&self) -> Self {
 96:         ParIter { ..*self }
 97:     }
 98: }
 99: impl<T: fmt::Debug> fmt::Debug for ParIter<'_, T> {
100:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
101:         let iter = self.entries.iter().map(Bucket::key_ref);
102:         f.debug_list().entries(iter).finish()
103:     }
104: }
105: impl<'a, T: Sync> ParallelIterator for ParIter<'a, T> {
106:     type Item = &'a T;
107:     parallel_iterator_methods!(Bucket::key_ref);
108: }
109: impl<T: Sync> IndexedParallelIterator for ParIter<'_, T> {
110:     indexed_parallel_iterator_methods!(Bucket::key_ref);
111: }
112: impl<'a, T, S> ParallelDrainRange<usize> for &'a mut IndexSet<T, S>
113: where
114:     T: Send,
115: {
116:     type Item = T;
117:     type Iter = ParDrain<'a, T>;
118:     fn par_drain<R: RangeBounds<usize>>(self, range: R) -> Self::Iter {
119:         ParDrain {
120:             entries: self.map.core.par_drain(range),
121:         }
122:     }
123: }
124: /// A parallel draining iterator over the items of an [`IndexSet`].
125: ///
126: /// This `struct` is created by the [`IndexSet::par_drain`] method
127: /// (provided by rayon's [`ParallelDrainRange`] trait). See its documentation for more.
128: ///
129: /// [`IndexSet::par_drain`]: ../struct.IndexSet.html#method.par_drain
130: pub struct ParDrain<'a, T: Send> {
131:     entries: rayon::vec::Drain<'a, Bucket<T>>,
132: }
133: impl<T: Send> ParallelIterator for ParDrain<'_, T> {
134:     type Item = T;
135:     parallel_iterator_methods!(Bucket::key);
136: }
137: impl<T: Send> IndexedParallelIterator for ParDrain<'_, T> {
138:     indexed_parallel_iterator_methods!(Bucket::key);
139: }
140: /// Parallel iterator methods and other parallel methods.
141: ///
142: /// The following methods **require crate feature `"rayon"`**.
143: ///
144: /// See also the `IntoParallelIterator` implementations.
145: impl<T, S> IndexSet<T, S>
146: where
147:     T: Hash + Eq + Sync,
148:     S: BuildHasher + Sync,
149: {
150:     /// Return a parallel iterator over the values that are in `self` but not `other`.
151:     ///
152:     /// While parallel iterators can process items in any order, their relative order
153:     /// in the `self` set is still preserved for operations like `reduce` and `collect`.
154:     pub fn par_difference<'a, S2>(
155:         &'a self,
156:         other: &'a IndexSet<T, S2>,
157:     ) -> ParDifference<'a, T, S, S2>
158:     where
159:         S2: BuildHasher + Sync,
160:     {
161:         ParDifference {
162:             set1: self,
163:             set2: other,
164:         }
165:     }
166:     /// Return a parallel iterator over the values that are in `self` or `other`,
167:     /// but not in both.
168:     ///
169:     /// While parallel iterators can process items in any order, their relative order
170:     /// in the sets is still preserved for operations like `reduce` and `collect`.
171:     /// Values from `self` are produced in their original order, followed by
172:     /// values from `other` in their original order.
173:     pub fn par_symmetric_difference<'a, S2>(
174:         &'a self,
175:         other: &'a IndexSet<T, S2>,
176:     ) -> ParSymmetricDifference<'a, T, S, S2>
177:     where
178:         S2: BuildHasher + Sync,
179:     {
180:         ParSymmetricDifference {
181:             set1: self,
182:             set2: other,
183:         }
184:     }
185:     /// Return a parallel iterator over the values that are in both `self` and `other`.
186:     ///
187:     /// While parallel iterators can process items in any order, their relative order
188:     /// in the `self` set is still preserved for operations like `reduce` and `collect`.
189:     pub fn par_intersection<'a, S2>(
190:         &'a self,
191:         other: &'a IndexSet<T, S2>,
192:     ) -> ParIntersection<'a, T, S, S2>
193:     where
194:         S2: BuildHasher + Sync,
195:     {
196:         ParIntersection {
197:             set1: self,
198:             set2: other,
199:         }
200:     }
201:     /// Return a parallel iterator over all values that are in `self` or `other`.
202:     ///
203:     /// While parallel iterators can process items in any order, their relative order
204:     /// in the sets is still preserved for operations like `reduce` and `collect`.
205:     /// Values from `self` are produced in their original order, followed by
206:     /// values that are unique to `other` in their original order.
207:     pub fn par_union<'a, S2>(&'a self, other: &'a IndexSet<T, S2>) -> ParUnion<'a, T, S, S2>
208:     where
209:         S2: BuildHasher + Sync,
210:     {
211:         ParUnion {
212:             set1: self,
213:             set2: other,
214:         }
215:     }
216:     /// Returns `true` if `self` contains all of the same values as `other`,
217:     /// regardless of each set's indexed order, determined in parallel.
218:     pub fn par_eq<S2>(&self, other: &IndexSet<T, S2>) -> bool
219:     where
220:         S2: BuildHasher + Sync,
221:     {
222:         self.len() == other.len() && self.par_is_subset(other)
223:     }
224:     /// Returns `true` if `self` has no elements in common with `other`,
225:     /// determined in parallel.
226:     pub fn par_is_disjoint<S2>(&self, other: &IndexSet<T, S2>) -> bool
227:     where
228:         S2: BuildHasher + Sync,
229:     {
230:         if self.len() <= other.len() {
231:             self.par_iter().all(move |value| !other.contains(value))
232:         } else {
233:             other.par_iter().all(move |value| !self.contains(value))
234:         }
235:     }
236:     /// Returns `true` if all elements of `other` are contained in `self`,
237:     /// determined in parallel.
238:     pub fn par_is_superset<S2>(&self, other: &IndexSet<T, S2>) -> bool
239:     where
240:         S2: BuildHasher + Sync,
241:     {
242:         other.par_is_subset(self)
243:     }
244:     /// Returns `true` if all elements of `self` are contained in `other`,
245:     /// determined in parallel.
246:     pub fn par_is_subset<S2>(&self, other: &IndexSet<T, S2>) -> bool
247:     where
248:         S2: BuildHasher + Sync,
249:     {
250:         self.len() <= other.len() && self.par_iter().all(move |value| other.contains(value))
251:     }
252: }
253: /// A parallel iterator producing elements in the difference of [`IndexSet`]s.
254: ///
255: /// This `struct` is created by the [`IndexSet::par_difference`] method.
256: /// See its documentation for more.
257: pub struct ParDifference<'a, T, S1, S2> {
258:     set1: &'a IndexSet<T, S1>,
259:     set2: &'a IndexSet<T, S2>,
260: }
261: impl<T, S1, S2> Clone for ParDifference<'_, T, S1, S2> {
262:     fn clone(&self) -> Self {
263:         ParDifference { ..*self }
264:     }
265: }
266: impl<T, S1, S2> fmt::Debug for ParDifference<'_, T, S1, S2>
267: where
268:     T: fmt::Debug + Eq + Hash,
269:     S1: BuildHasher,
270:     S2: BuildHasher,
271: {
272:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
273:         f.debug_list()
274:             .entries(self.set1.difference(self.set2))
275:             .finish()
276:     }
277: }
278: impl<'a, T, S1, S2> ParallelIterator for ParDifference<'a, T, S1, S2>
279: where
280:     T: Hash + Eq + Sync,
281:     S1: BuildHasher + Sync,
282:     S2: BuildHasher + Sync,
283: {
284:     type Item = &'a T;
285:     fn drive_unindexed<C>(self, consumer: C) -> C::Result
286:     where
287:         C: UnindexedConsumer<Self::Item>,
288:     {
289:         let Self { set1, set2 } = self;
290:         set1.par_iter()
291:             .filter(move |&item| !set2.contains(item))
292:             .drive_unindexed(consumer)
293:     }
294: }
295: /// A parallel iterator producing elements in the intersection of [`IndexSet`]s.
296: ///
297: /// This `struct` is created by the [`IndexSet::par_intersection`] method.
298: /// See its documentation for more.
299: pub struct ParIntersection<'a, T, S1, S2> {
300:     set1: &'a IndexSet<T, S1>,
301:     set2: &'a IndexSet<T, S2>,
302: }
303: impl<T, S1, S2> Clone for ParIntersection<'_, T, S1, S2> {
304:     fn clone(&self) -> Self {
305:         ParIntersection { ..*self }
306:     }
307: }
308: impl<T, S1, S2> fmt::Debug for ParIntersection<'_, T, S1, S2>
309: where
310:     T: fmt::Debug + Eq + Hash,
311:     S1: BuildHasher,
312:     S2: BuildHasher,
313: {
314:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
315:         f.debug_list()
316:             .entries(self.set1.intersection(self.set2))
317:             .finish()
318:     }
319: }
320: impl<'a, T, S1, S2> ParallelIterator for ParIntersection<'a, T, S1, S2>
321: where
322:     T: Hash + Eq + Sync,
323:     S1: BuildHasher + Sync,
324:     S2: BuildHasher + Sync,
325: {
326:     type Item = &'a T;
327:     fn drive_unindexed<C>(self, consumer: C) -> C::Result
328:     where
329:         C: UnindexedConsumer<Self::Item>,
330:     {
331:         let Self { set1, set2 } = self;
332:         set1.par_iter()
333:             .filter(move |&item| set2.contains(item))
334:             .drive_unindexed(consumer)
335:     }
336: }
337: /// A parallel iterator producing elements in the symmetric difference of [`IndexSet`]s.
338: ///
339: /// This `struct` is created by the [`IndexSet::par_symmetric_difference`] method.
340: /// See its documentation for more.
341: pub struct ParSymmetricDifference<'a, T, S1, S2> {
342:     set1: &'a IndexSet<T, S1>,
343:     set2: &'a IndexSet<T, S2>,
344: }
345: impl<T, S1, S2> Clone for ParSymmetricDifference<'_, T, S1, S2> {
346:     fn clone(&self) -> Self {
347:         ParSymmetricDifference { ..*self }
348:     }
349: }
350: impl<T, S1, S2> fmt::Debug for ParSymmetricDifference<'_, T, S1, S2>
351: where
352:     T: fmt::Debug + Eq + Hash,
353:     S1: BuildHasher,
354:     S2: BuildHasher,
355: {
356:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
357:         f.debug_list()
358:             .entries(self.set1.symmetric_difference(self.set2))
359:             .finish()
360:     }
361: }
362: impl<'a, T, S1, S2> ParallelIterator for ParSymmetricDifference<'a, T, S1, S2>
363: where
364:     T: Hash + Eq + Sync,
365:     S1: BuildHasher + Sync,
366:     S2: BuildHasher + Sync,
367: {
368:     type Item = &'a T;
369:     fn drive_unindexed<C>(self, consumer: C) -> C::Result
370:     where
371:         C: UnindexedConsumer<Self::Item>,
372:     {
373:         let Self { set1, set2 } = self;
374:         set1.par_difference(set2)
375:             .chain(set2.par_difference(set1))
376:             .drive_unindexed(consumer)
377:     }
378: }
379: /// A parallel iterator producing elements in the union of [`IndexSet`]s.
380: ///
381: /// This `struct` is created by the [`IndexSet::par_union`] method.
382: /// See its documentation for more.
383: pub struct ParUnion<'a, T, S1, S2> {
384:     set1: &'a IndexSet<T, S1>,
385:     set2: &'a IndexSet<T, S2>,
386: }
387: impl<T, S1, S2> Clone for ParUnion<'_, T, S1, S2> {
388:     fn clone(&self) -> Self {
389:         ParUnion { ..*self }
390:     }
391: }
392: impl<T, S1, S2> fmt::Debug for ParUnion<'_, T, S1, S2>
393: where
394:     T: fmt::Debug + Eq + Hash,
395:     S1: BuildHasher,
396:     S2: BuildHasher,
397: {
398:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
399:         f.debug_list().entries(self.set1.union(self.set2)).finish()
400:     }
401: }
402: impl<'a, T, S1, S2> ParallelIterator for ParUnion<'a, T, S1, S2>
403: where
404:     T: Hash + Eq + Sync,
405:     S1: BuildHasher + Sync,
406:     S2: BuildHasher + Sync,
407: {
408:     type Item = &'a T;
409:     fn drive_unindexed<C>(self, consumer: C) -> C::Result
410:     where
411:         C: UnindexedConsumer<Self::Item>,
412:     {
413:         let Self { set1, set2 } = self;
414:         set1.par_iter()
415:             .chain(set2.par_difference(set1))
416:             .drive_unindexed(consumer)
417:     }
418: }
419: /// Parallel sorting methods.
420: ///
421: /// The following methods **require crate feature `"rayon"`**.
422: impl<T, S> IndexSet<T, S>
423: where
424:     T: Send,
425: {
426:     /// Sort the set's values in parallel by their default ordering.
427:     pub fn par_sort(&mut self)
428:     where
429:         T: Ord,
430:     {
431:         self.with_entries(|entries| {
432:             entries.par_sort_by(|a, b| T::cmp(&a.key, &b.key));
433:         });
434:     }
435:     /// Sort the set's values in place and in parallel, using the comparison function `cmp`.
436:     pub fn par_sort_by<F>(&mut self, cmp: F)
437:     where
438:         F: Fn(&T, &T) -> Ordering + Sync,
439:     {
440:         self.with_entries(|entries| {
441:             entries.par_sort_by(move |a, b| cmp(&a.key, &b.key));
442:         });
443:     }
444:     /// Sort the values of the set in parallel and return a by-value parallel iterator of
445:     /// the values with the result.
446:     pub fn par_sorted_by<F>(self, cmp: F) -> IntoParIter<T>
447:     where
448:         F: Fn(&T, &T) -> Ordering + Sync,
449:     {
450:         let mut entries = self.into_entries();
451:         entries.par_sort_by(move |a, b| cmp(&a.key, &b.key));
452:         IntoParIter { entries }
453:     }
454:     /// Sort the set's values in place and in parallel, using a key extraction function.
455:     pub fn par_sort_by_key<K, F>(&mut self, sort_key: F)
456:     where
457:         K: Ord,
458:         F: Fn(&T) -> K + Sync,
459:     {
460:         self.with_entries(move |entries| {
461:             entries.par_sort_by_key(move |a| sort_key(&a.key));
462:         });
463:     }
464:     /// Sort the set's values in parallel by their default ordering.
465:     pub fn par_sort_unstable(&mut self)
466:     where
467:         T: Ord,
468:     {
469:         self.with_entries(|entries| {
470:             entries.par_sort_unstable_by(|a, b| T::cmp(&a.key, &b.key));
471:         });
472:     }
473:     /// Sort the set's values in place and in parallel, using the comparison function `cmp`.
474:     pub fn par_sort_unstable_by<F>(&mut self, cmp: F)
475:     where
476:         F: Fn(&T, &T) -> Ordering + Sync,
477:     {
478:         self.with_entries(|entries| {
479:             entries.par_sort_unstable_by(move |a, b| cmp(&a.key, &b.key));
480:         });
481:     }
482:     /// Sort the values of the set in parallel and return a by-value parallel iterator of
483:     /// the values with the result.
484:     pub fn par_sorted_unstable_by<F>(self, cmp: F) -> IntoParIter<T>
485:     where
486:         F: Fn(&T, &T) -> Ordering + Sync,
487:     {
488:         let mut entries = self.into_entries();
489:         entries.par_sort_unstable_by(move |a, b| cmp(&a.key, &b.key));
490:         IntoParIter { entries }
491:     }
492:     /// Sort the set's values in place and in parallel, using a key extraction function.
493:     pub fn par_sort_unstable_by_key<K, F>(&mut self, sort_key: F)
494:     where
495:         K: Ord,
496:         F: Fn(&T) -> K + Sync,
497:     {
498:         self.with_entries(move |entries| {
499:             entries.par_sort_unstable_by_key(move |a| sort_key(&a.key));
500:         });
501:     }
502:     /// Sort the set's values in place and in parallel, using a key extraction function.
503:     pub fn par_sort_by_cached_key<K, F>(&mut self, sort_key: F)
504:     where
505:         K: Ord + Send,
506:         F: Fn(&T) -> K + Sync,
507:     {
508:         self.with_entries(move |entries| {
509:             entries.par_sort_by_cached_key(move |a| sort_key(&a.key));
510:         });
511:     }
512: }
513: impl<T, S> FromParallelIterator<T> for IndexSet<T, S>
514: where
515:     T: Eq + Hash + Send,
516:     S: BuildHasher + Default + Send,
517: {
518:     fn from_par_iter<I>(iter: I) -> Self
519:     where
520:         I: IntoParallelIterator<Item = T>,
521:     {
522:         let list = collect(iter);
523:         let len = list.iter().map(Vec::len).sum();
524:         let mut set = Self::with_capacity_and_hasher(len, S::default());
525:         for vec in list {
526:             set.extend(vec);
527:         }
528:         set
529:     }
530: }
531: impl<T, S> ParallelExtend<T> for IndexSet<T, S>
532: where
533:     T: Eq + Hash + Send,
534:     S: BuildHasher + Send,
535: {
536:     fn par_extend<I>(&mut self, iter: I)
537:     where
538:         I: IntoParallelIterator<Item = T>,
539:     {
540:         for vec in collect(iter) {
541:             self.extend(vec);
542:         }
543:     }
544: }
545: impl<'a, T: 'a, S> ParallelExtend<&'a T> for IndexSet<T, S>
546: where
547:     T: Copy + Eq + Hash + Send + Sync,
548:     S: BuildHasher + Send,
549: {
550:     fn par_extend<I>(&mut self, iter: I)
551:     where
552:         I: IntoParallelIterator<Item = &'a T>,
553:     {
554:         for vec in collect(iter) {
555:             self.extend(vec);
556:         }
557:     }
558: }
559: #[cfg(test)]
560: mod tests {
561:     use super::*;
562:     #[test]
563:     fn insert_order() {
564:         let insert = [0, 4, 2, 12, 8, 7, 11, 5, 3, 17, 19, 22, 23];
565:         let mut set = IndexSet::new();
566:         for &elt in &insert {
567:             set.insert(elt);
568:         }
569:         assert_eq!(set.par_iter().count(), set.len());
570:         assert_eq!(set.par_iter().count(), insert.len());
571:         insert.par_iter().zip(&set).for_each(|(a, b)| {
572:             assert_eq!(a, b);
573:         });
574:         (0..insert.len())
575:             .into_par_iter()
576:             .zip(&set)
577:             .for_each(|(i, v)| {
578:                 assert_eq!(set.get_index(i).unwrap(), v);
579:             });
580:     }
581:     #[test]
582:     fn partial_eq_and_eq() {
583:         let mut set_a = IndexSet::new();
584:         set_a.insert(1);
585:         set_a.insert(2);
586:         let mut set_b = set_a.clone();
587:         assert!(set_a.par_eq(&set_b));
588:         set_b.swap_remove(&1);
589:         assert!(!set_a.par_eq(&set_b));
590:         set_b.insert(3);
591:         assert!(!set_a.par_eq(&set_b));
592:         let set_c: IndexSet<_> = set_b.into_par_iter().collect();
593:         assert!(!set_a.par_eq(&set_c));
594:         assert!(!set_c.par_eq(&set_a));
595:     }
596:     #[test]
597:     fn extend() {
598:         let mut set = IndexSet::new();
599:         set.par_extend(vec![&1, &2, &3, &4]);
600:         set.par_extend(vec![5, 6]);
601:         assert_eq!(
602:             set.into_par_iter().collect::<Vec<_>>(),
603:             vec![1, 2, 3, 4, 5, 6]
604:         );
605:     }
606:     #[test]
607:     fn comparisons() {
608:         let set_a: IndexSet<_> = (0..3).collect();
609:         let set_b: IndexSet<_> = (3..6).collect();
610:         let set_c: IndexSet<_> = (0..6).collect();
611:         let set_d: IndexSet<_> = (3..9).collect();
612:         assert!(!set_a.par_is_disjoint(&set_a));
613:         assert!(set_a.par_is_subset(&set_a));
614:         assert!(set_a.par_is_superset(&set_a));
615:         assert!(set_a.par_is_disjoint(&set_b));
616:         assert!(set_b.par_is_disjoint(&set_a));
617:         assert!(!set_a.par_is_subset(&set_b));
618:         assert!(!set_b.par_is_subset(&set_a));
619:         assert!(!set_a.par_is_superset(&set_b));
620:         assert!(!set_b.par_is_superset(&set_a));
621:         assert!(!set_a.par_is_disjoint(&set_c));
622:         assert!(!set_c.par_is_disjoint(&set_a));
623:         assert!(set_a.par_is_subset(&set_c));
624:         assert!(!set_c.par_is_subset(&set_a));
625:         assert!(!set_a.par_is_superset(&set_c));
626:         assert!(set_c.par_is_superset(&set_a));
627:         assert!(!set_c.par_is_disjoint(&set_d));
628:         assert!(!set_d.par_is_disjoint(&set_c));
629:         assert!(!set_c.par_is_subset(&set_d));
630:         assert!(!set_d.par_is_subset(&set_c));
631:         assert!(!set_c.par_is_superset(&set_d));
632:         assert!(!set_d.par_is_superset(&set_c));
633:     }
634:     #[test]
635:     fn iter_comparisons() {
636:         use std::iter::empty;
637:         fn check<'a, I1, I2>(iter1: I1, iter2: I2)
638:         where
639:             I1: ParallelIterator<Item = &'a i32>,
640:             I2: Iterator<Item = i32>,
641:         {
642:             let v1: Vec<_> = iter1.copied().collect();
643:             let v2: Vec<_> = iter2.collect();
644:             assert_eq!(v1, v2);
645:         }
646:         let set_a: IndexSet<_> = (0..3).collect();
647:         let set_b: IndexSet<_> = (3..6).collect();
648:         let set_c: IndexSet<_> = (0..6).collect();
649:         let set_d: IndexSet<_> = (3..9).rev().collect();
650:         check(set_a.par_difference(&set_a), empty());
651:         check(set_a.par_symmetric_difference(&set_a), empty());
652:         check(set_a.par_intersection(&set_a), 0..3);
653:         check(set_a.par_union(&set_a), 0..3);
654:         check(set_a.par_difference(&set_b), 0..3);
655:         check(set_b.par_difference(&set_a), 3..6);
656:         check(set_a.par_symmetric_difference(&set_b), 0..6);
657:         check(set_b.par_symmetric_difference(&set_a), (3..6).chain(0..3));
658:         check(set_a.par_intersection(&set_b), empty());
659:         check(set_b.par_intersection(&set_a), empty());
660:         check(set_a.par_union(&set_b), 0..6);
661:         check(set_b.par_union(&set_a), (3..6).chain(0..3));
662:         check(set_a.par_difference(&set_c), empty());
663:         check(set_c.par_difference(&set_a), 3..6);
664:         check(set_a.par_symmetric_difference(&set_c), 3..6);
665:         check(set_c.par_symmetric_difference(&set_a), 3..6);
666:         check(set_a.par_intersection(&set_c), 0..3);
667:         check(set_c.par_intersection(&set_a), 0..3);
668:         check(set_a.par_union(&set_c), 0..6);
669:         check(set_c.par_union(&set_a), 0..6);
670:         check(set_c.par_difference(&set_d), 0..3);
671:         check(set_d.par_difference(&set_c), (6..9).rev());
672:         check(
673:             set_c.par_symmetric_difference(&set_d),
674:             (0..3).chain((6..9).rev()),
675:         );
676:         check(
677:             set_d.par_symmetric_difference(&set_c),
678:             (6..9).rev().chain(0..3),
679:         );
680:         check(set_c.par_intersection(&set_d), 3..6);
681:         check(set_d.par_intersection(&set_c), (3..6).rev());
682:         check(set_c.par_union(&set_d), (0..6).chain((6..9).rev()));
683:         check(set_d.par_union(&set_c), (3..9).rev().chain(0..3));
684:     }
685: }
````

## File: src/set/iter.rs
````rust
  1: use super::{Bucket, IndexSet, Slice};
  2: use crate::inner::{Core, ExtractCore};
  3: use alloc::vec::{self, Vec};
  4: use core::fmt;
  5: use core::hash::{BuildHasher, Hash};
  6: use core::iter::{Chain, FusedIterator};
  7: use core::ops::RangeBounds;
  8: use core::slice::Iter as SliceIter;
  9: impl<'a, T, S> IntoIterator for &'a IndexSet<T, S> {
 10:     type Item = &'a T;
 11:     type IntoIter = Iter<'a, T>;
 12:     fn into_iter(self) -> Self::IntoIter {
 13:         self.iter()
 14:     }
 15: }
 16: impl<T, S> IntoIterator for IndexSet<T, S> {
 17:     type Item = T;
 18:     type IntoIter = IntoIter<T>;
 19:     fn into_iter(self) -> Self::IntoIter {
 20:         IntoIter::new(self.into_entries())
 21:     }
 22: }
 23: /// An iterator over the items of an [`IndexSet`].
 24: ///
 25: /// This `struct` is created by the [`IndexSet::iter`] method.
 26: /// See its documentation for more.
 27: pub struct Iter<'a, T> {
 28:     iter: SliceIter<'a, Bucket<T>>,
 29: }
 30: impl<'a, T> Iter<'a, T> {
 31:     pub(super) fn new(entries: &'a [Bucket<T>]) -> Self {
 32:         Self {
 33:             iter: entries.iter(),
 34:         }
 35:     }
 36:     /// Returns a slice of the remaining entries in the iterator.
 37:     pub fn as_slice(&self) -> &'a Slice<T> {
 38:         Slice::from_slice(self.iter.as_slice())
 39:     }
 40: }
 41: impl<'a, T> Iterator for Iter<'a, T> {
 42:     type Item = &'a T;
 43:     iterator_methods!(Bucket::key_ref);
 44: }
 45: impl<T> DoubleEndedIterator for Iter<'_, T> {
 46:     double_ended_iterator_methods!(Bucket::key_ref);
 47: }
 48: impl<T> ExactSizeIterator for Iter<'_, T> {
 49:     fn len(&self) -> usize {
 50:         self.iter.len()
 51:     }
 52: }
 53: impl<T> FusedIterator for Iter<'_, T> {}
 54: impl<T> Clone for Iter<'_, T> {
 55:     fn clone(&self) -> Self {
 56:         Iter {
 57:             iter: self.iter.clone(),
 58:         }
 59:     }
 60: }
 61: impl<T: fmt::Debug> fmt::Debug for Iter<'_, T> {
 62:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
 63:         f.debug_list().entries(self.clone()).finish()
 64:     }
 65: }
 66: impl<T> Default for Iter<'_, T> {
 67:     fn default() -> Self {
 68:         Self { iter: [].iter() }
 69:     }
 70: }
 71: /// An owning iterator over the items of an [`IndexSet`].
 72: ///
 73: /// This `struct` is created by the [`IndexSet::into_iter`] method
 74: /// (provided by the [`IntoIterator`] trait). See its documentation for more.
 75: #[derive(Clone)]
 76: pub struct IntoIter<T> {
 77:     iter: vec::IntoIter<Bucket<T>>,
 78: }
 79: impl<T> IntoIter<T> {
 80:     pub(super) fn new(entries: Vec<Bucket<T>>) -> Self {
 81:         Self {
 82:             iter: entries.into_iter(),
 83:         }
 84:     }
 85:     /// Returns a slice of the remaining entries in the iterator.
 86:     pub fn as_slice(&self) -> &Slice<T> {
 87:         Slice::from_slice(self.iter.as_slice())
 88:     }
 89: }
 90: impl<T> Iterator for IntoIter<T> {
 91:     type Item = T;
 92:     iterator_methods!(Bucket::key);
 93: }
 94: impl<T> DoubleEndedIterator for IntoIter<T> {
 95:     double_ended_iterator_methods!(Bucket::key);
 96: }
 97: impl<T> ExactSizeIterator for IntoIter<T> {
 98:     fn len(&self) -> usize {
 99:         self.iter.len()
100:     }
101: }
102: impl<T> FusedIterator for IntoIter<T> {}
103: impl<T: fmt::Debug> fmt::Debug for IntoIter<T> {
104:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
105:         let iter = self.iter.as_slice().iter().map(Bucket::key_ref);
106:         f.debug_list().entries(iter).finish()
107:     }
108: }
109: impl<T> Default for IntoIter<T> {
110:     fn default() -> Self {
111:         Self {
112:             iter: Vec::new().into_iter(),
113:         }
114:     }
115: }
116: /// A draining iterator over the items of an [`IndexSet`].
117: ///
118: /// This `struct` is created by the [`IndexSet::drain`] method.
119: /// See its documentation for more.
120: pub struct Drain<'a, T> {
121:     iter: vec::Drain<'a, Bucket<T>>,
122: }
123: impl<'a, T> Drain<'a, T> {
124:     pub(super) fn new(iter: vec::Drain<'a, Bucket<T>>) -> Self {
125:         Self { iter }
126:     }
127:     /// Returns a slice of the remaining entries in the iterator.
128:     pub fn as_slice(&self) -> &Slice<T> {
129:         Slice::from_slice(self.iter.as_slice())
130:     }
131: }
132: impl<T> Iterator for Drain<'_, T> {
133:     type Item = T;
134:     iterator_methods!(Bucket::key);
135: }
136: impl<T> DoubleEndedIterator for Drain<'_, T> {
137:     double_ended_iterator_methods!(Bucket::key);
138: }
139: impl<T> ExactSizeIterator for Drain<'_, T> {
140:     fn len(&self) -> usize {
141:         self.iter.len()
142:     }
143: }
144: impl<T> FusedIterator for Drain<'_, T> {}
145: impl<T: fmt::Debug> fmt::Debug for Drain<'_, T> {
146:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
147:         let iter = self.iter.as_slice().iter().map(Bucket::key_ref);
148:         f.debug_list().entries(iter).finish()
149:     }
150: }
151: /// A lazy iterator producing elements in the difference of [`IndexSet`]s.
152: ///
153: /// This `struct` is created by the [`IndexSet::difference`] method.
154: /// See its documentation for more.
155: pub struct Difference<'a, T, S> {
156:     iter: Iter<'a, T>,
157:     other: &'a IndexSet<T, S>,
158: }
159: impl<'a, T, S> Difference<'a, T, S> {
160:     pub(super) fn new<S1>(set: &'a IndexSet<T, S1>, other: &'a IndexSet<T, S>) -> Self {
161:         Self {
162:             iter: set.iter(),
163:             other,
164:         }
165:     }
166: }
167: impl<'a, T, S> Iterator for Difference<'a, T, S>
168: where
169:     T: Eq + Hash,
170:     S: BuildHasher,
171: {
172:     type Item = &'a T;
173:     fn next(&mut self) -> Option<Self::Item> {
174:         while let Some(item) = self.iter.next() {
175:             if !self.other.contains(item) {
176:                 return Some(item);
177:             }
178:         }
179:         None
180:     }
181:     fn size_hint(&self) -> (usize, Option<usize>) {
182:         (0, self.iter.size_hint().1)
183:     }
184: }
185: impl<T, S> DoubleEndedIterator for Difference<'_, T, S>
186: where
187:     T: Eq + Hash,
188:     S: BuildHasher,
189: {
190:     fn next_back(&mut self) -> Option<Self::Item> {
191:         while let Some(item) = self.iter.next_back() {
192:             if !self.other.contains(item) {
193:                 return Some(item);
194:             }
195:         }
196:         None
197:     }
198: }
199: impl<T, S> FusedIterator for Difference<'_, T, S>
200: where
201:     T: Eq + Hash,
202:     S: BuildHasher,
203: {
204: }
205: impl<T, S> Clone for Difference<'_, T, S> {
206:     fn clone(&self) -> Self {
207:         Difference {
208:             iter: self.iter.clone(),
209:             ..*self
210:         }
211:     }
212: }
213: impl<T, S> fmt::Debug for Difference<'_, T, S>
214: where
215:     T: fmt::Debug + Eq + Hash,
216:     S: BuildHasher,
217: {
218:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
219:         f.debug_list().entries(self.clone()).finish()
220:     }
221: }
222: /// A lazy iterator producing elements in the intersection of [`IndexSet`]s.
223: ///
224: /// This `struct` is created by the [`IndexSet::intersection`] method.
225: /// See its documentation for more.
226: pub struct Intersection<'a, T, S> {
227:     iter: Iter<'a, T>,
228:     other: &'a IndexSet<T, S>,
229: }
230: impl<'a, T, S> Intersection<'a, T, S> {
231:     pub(super) fn new<S1>(set: &'a IndexSet<T, S1>, other: &'a IndexSet<T, S>) -> Self {
232:         Self {
233:             iter: set.iter(),
234:             other,
235:         }
236:     }
237: }
238: impl<'a, T, S> Iterator for Intersection<'a, T, S>
239: where
240:     T: Eq + Hash,
241:     S: BuildHasher,
242: {
243:     type Item = &'a T;
244:     fn next(&mut self) -> Option<Self::Item> {
245:         while let Some(item) = self.iter.next() {
246:             if self.other.contains(item) {
247:                 return Some(item);
248:             }
249:         }
250:         None
251:     }
252:     fn size_hint(&self) -> (usize, Option<usize>) {
253:         (0, self.iter.size_hint().1)
254:     }
255: }
256: impl<T, S> DoubleEndedIterator for Intersection<'_, T, S>
257: where
258:     T: Eq + Hash,
259:     S: BuildHasher,
260: {
261:     fn next_back(&mut self) -> Option<Self::Item> {
262:         while let Some(item) = self.iter.next_back() {
263:             if self.other.contains(item) {
264:                 return Some(item);
265:             }
266:         }
267:         None
268:     }
269: }
270: impl<T, S> FusedIterator for Intersection<'_, T, S>
271: where
272:     T: Eq + Hash,
273:     S: BuildHasher,
274: {
275: }
276: impl<T, S> Clone for Intersection<'_, T, S> {
277:     fn clone(&self) -> Self {
278:         Intersection {
279:             iter: self.iter.clone(),
280:             ..*self
281:         }
282:     }
283: }
284: impl<T, S> fmt::Debug for Intersection<'_, T, S>
285: where
286:     T: fmt::Debug + Eq + Hash,
287:     S: BuildHasher,
288: {
289:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
290:         f.debug_list().entries(self.clone()).finish()
291:     }
292: }
293: /// A lazy iterator producing elements in the symmetric difference of [`IndexSet`]s.
294: ///
295: /// This `struct` is created by the [`IndexSet::symmetric_difference`] method.
296: /// See its documentation for more.
297: pub struct SymmetricDifference<'a, T, S1, S2> {
298:     iter: Chain<Difference<'a, T, S2>, Difference<'a, T, S1>>,
299: }
300: impl<'a, T, S1, S2> SymmetricDifference<'a, T, S1, S2>
301: where
302:     T: Eq + Hash,
303:     S1: BuildHasher,
304:     S2: BuildHasher,
305: {
306:     pub(super) fn new(set1: &'a IndexSet<T, S1>, set2: &'a IndexSet<T, S2>) -> Self {
307:         let diff1 = set1.difference(set2);
308:         let diff2 = set2.difference(set1);
309:         Self {
310:             iter: diff1.chain(diff2),
311:         }
312:     }
313: }
314: impl<'a, T, S1, S2> Iterator for SymmetricDifference<'a, T, S1, S2>
315: where
316:     T: Eq + Hash,
317:     S1: BuildHasher,
318:     S2: BuildHasher,
319: {
320:     type Item = &'a T;
321:     fn next(&mut self) -> Option<Self::Item> {
322:         self.iter.next()
323:     }
324:     fn size_hint(&self) -> (usize, Option<usize>) {
325:         self.iter.size_hint()
326:     }
327:     fn fold<B, F>(self, init: B, f: F) -> B
328:     where
329:         F: FnMut(B, Self::Item) -> B,
330:     {
331:         self.iter.fold(init, f)
332:     }
333: }
334: impl<T, S1, S2> DoubleEndedIterator for SymmetricDifference<'_, T, S1, S2>
335: where
336:     T: Eq + Hash,
337:     S1: BuildHasher,
338:     S2: BuildHasher,
339: {
340:     fn next_back(&mut self) -> Option<Self::Item> {
341:         self.iter.next_back()
342:     }
343:     fn rfold<B, F>(self, init: B, f: F) -> B
344:     where
345:         F: FnMut(B, Self::Item) -> B,
346:     {
347:         self.iter.rfold(init, f)
348:     }
349: }
350: impl<T, S1, S2> FusedIterator for SymmetricDifference<'_, T, S1, S2>
351: where
352:     T: Eq + Hash,
353:     S1: BuildHasher,
354:     S2: BuildHasher,
355: {
356: }
357: impl<T, S1, S2> Clone for SymmetricDifference<'_, T, S1, S2> {
358:     fn clone(&self) -> Self {
359:         SymmetricDifference {
360:             iter: self.iter.clone(),
361:         }
362:     }
363: }
364: impl<T, S1, S2> fmt::Debug for SymmetricDifference<'_, T, S1, S2>
365: where
366:     T: fmt::Debug + Eq + Hash,
367:     S1: BuildHasher,
368:     S2: BuildHasher,
369: {
370:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
371:         f.debug_list().entries(self.clone()).finish()
372:     }
373: }
374: /// A lazy iterator producing elements in the union of [`IndexSet`]s.
375: ///
376: /// This `struct` is created by the [`IndexSet::union`] method.
377: /// See its documentation for more.
378: pub struct Union<'a, T, S> {
379:     iter: Chain<Iter<'a, T>, Difference<'a, T, S>>,
380: }
381: impl<'a, T, S> Union<'a, T, S>
382: where
383:     T: Eq + Hash,
384:     S: BuildHasher,
385: {
386:     pub(super) fn new<S2>(set1: &'a IndexSet<T, S>, set2: &'a IndexSet<T, S2>) -> Self
387:     where
388:         S2: BuildHasher,
389:     {
390:         Self {
391:             iter: set1.iter().chain(set2.difference(set1)),
392:         }
393:     }
394: }
395: impl<'a, T, S> Iterator for Union<'a, T, S>
396: where
397:     T: Eq + Hash,
398:     S: BuildHasher,
399: {
400:     type Item = &'a T;
401:     fn next(&mut self) -> Option<Self::Item> {
402:         self.iter.next()
403:     }
404:     fn size_hint(&self) -> (usize, Option<usize>) {
405:         self.iter.size_hint()
406:     }
407:     fn fold<B, F>(self, init: B, f: F) -> B
408:     where
409:         F: FnMut(B, Self::Item) -> B,
410:     {
411:         self.iter.fold(init, f)
412:     }
413: }
414: impl<T, S> DoubleEndedIterator for Union<'_, T, S>
415: where
416:     T: Eq + Hash,
417:     S: BuildHasher,
418: {
419:     fn next_back(&mut self) -> Option<Self::Item> {
420:         self.iter.next_back()
421:     }
422:     fn rfold<B, F>(self, init: B, f: F) -> B
423:     where
424:         F: FnMut(B, Self::Item) -> B,
425:     {
426:         self.iter.rfold(init, f)
427:     }
428: }
429: impl<T, S> FusedIterator for Union<'_, T, S>
430: where
431:     T: Eq + Hash,
432:     S: BuildHasher,
433: {
434: }
435: impl<T, S> Clone for Union<'_, T, S> {
436:     fn clone(&self) -> Self {
437:         Union {
438:             iter: self.iter.clone(),
439:         }
440:     }
441: }
442: impl<T, S> fmt::Debug for Union<'_, T, S>
443: where
444:     T: fmt::Debug + Eq + Hash,
445:     S: BuildHasher,
446: {
447:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
448:         f.debug_list().entries(self.clone()).finish()
449:     }
450: }
451: /// A splicing iterator for `IndexSet`.
452: ///
453: /// This `struct` is created by [`IndexSet::splice()`].
454: /// See its documentation for more.
455: pub struct Splice<'a, I, T, S>
456: where
457:     I: Iterator<Item = T>,
458:     T: Hash + Eq,
459:     S: BuildHasher,
460: {
461:     iter: crate::map::Splice<'a, UnitValue<I>, T, (), S>,
462: }
463: impl<'a, I, T, S> Splice<'a, I, T, S>
464: where
465:     I: Iterator<Item = T>,
466:     T: Hash + Eq,
467:     S: BuildHasher,
468: {
469:     #[track_caller]
470:     pub(super) fn new<R>(set: &'a mut IndexSet<T, S>, range: R, replace_with: I) -> Self
471:     where
472:         R: RangeBounds<usize>,
473:     {
474:         Self {
475:             iter: set.map.splice(range, UnitValue(replace_with)),
476:         }
477:     }
478: }
479: impl<I, T, S> Iterator for Splice<'_, I, T, S>
480: where
481:     I: Iterator<Item = T>,
482:     T: Hash + Eq,
483:     S: BuildHasher,
484: {
485:     type Item = T;
486:     fn next(&mut self) -> Option<Self::Item> {
487:         Some(self.iter.next()?.0)
488:     }
489:     fn size_hint(&self) -> (usize, Option<usize>) {
490:         self.iter.size_hint()
491:     }
492: }
493: impl<I, T, S> DoubleEndedIterator for Splice<'_, I, T, S>
494: where
495:     I: Iterator<Item = T>,
496:     T: Hash + Eq,
497:     S: BuildHasher,
498: {
499:     fn next_back(&mut self) -> Option<Self::Item> {
500:         Some(self.iter.next_back()?.0)
501:     }
502: }
503: impl<I, T, S> ExactSizeIterator for Splice<'_, I, T, S>
504: where
505:     I: Iterator<Item = T>,
506:     T: Hash + Eq,
507:     S: BuildHasher,
508: {
509:     fn len(&self) -> usize {
510:         self.iter.len()
511:     }
512: }
513: impl<I, T, S> FusedIterator for Splice<'_, I, T, S>
514: where
515:     I: Iterator<Item = T>,
516:     T: Hash + Eq,
517:     S: BuildHasher,
518: {
519: }
520: struct UnitValue<I>(I);
521: impl<I: Iterator> Iterator for UnitValue<I> {
522:     type Item = (I::Item, ());
523:     fn next(&mut self) -> Option<Self::Item> {
524:         self.0.next().map(|x| (x, ()))
525:     }
526: }
527: impl<I, T, S> fmt::Debug for Splice<'_, I, T, S>
528: where
529:     I: fmt::Debug + Iterator<Item = T>,
530:     T: fmt::Debug + Hash + Eq,
531:     S: BuildHasher,
532: {
533:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
534:         fmt::Debug::fmt(&self.iter, f)
535:     }
536: }
537: impl<I: fmt::Debug> fmt::Debug for UnitValue<I> {
538:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
539:         fmt::Debug::fmt(&self.0, f)
540:     }
541: }
542: /// An extracting iterator for `IndexSet`.
543: ///
544: /// This `struct` is created by [`IndexSet::extract_if()`].
545: /// See its documentation for more.
546: pub struct ExtractIf<'a, T, F> {
547:     inner: ExtractCore<'a, T, ()>,
548:     pred: F,
549: }
550: impl<T, F> ExtractIf<'_, T, F> {
551:     #[track_caller]
552:     pub(super) fn new<R>(core: &mut Core<T, ()>, range: R, pred: F) -> ExtractIf<'_, T, F>
553:     where
554:         R: RangeBounds<usize>,
555:         F: FnMut(&T) -> bool,
556:     {
557:         ExtractIf {
558:             inner: core.extract(range),
559:             pred,
560:         }
561:     }
562: }
563: impl<T, F> Iterator for ExtractIf<'_, T, F>
564: where
565:     F: FnMut(&T) -> bool,
566: {
567:     type Item = T;
568:     fn next(&mut self) -> Option<Self::Item> {
569:         self.inner
570:             .extract_if(|bucket| (self.pred)(bucket.key_ref()))
571:             .map(Bucket::key)
572:     }
573:     fn size_hint(&self) -> (usize, Option<usize>) {
574:         (0, Some(self.inner.remaining()))
575:     }
576: }
577: impl<T, F> FusedIterator for ExtractIf<'_, T, F> where F: FnMut(&T) -> bool {}
578: impl<T, F> fmt::Debug for ExtractIf<'_, T, F>
579: where
580:     T: fmt::Debug,
581: {
582:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
583:         f.debug_struct("ExtractIf").finish_non_exhaustive()
584:     }
585: }
````

## File: src/set/mutable.rs
````rust
 1: use core::hash::{BuildHasher, Hash};
 2: use super::{Equivalent, IndexSet};
 3: use crate::map::MutableKeys;
 4: /// Opt-in mutable access to [`IndexSet`] values.
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
15: /// `use` this trait to enable its methods for `IndexSet`.
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
43: /// Opt-in mutable access to [`IndexSet`] values.
44: ///
45: /// See [`MutableValues`] for more information.
46: impl<T, S> MutableValues for IndexSet<T, S>
47: where
48:     S: BuildHasher,
49: {
50:     type Value = T;
51:     fn get_full_mut2<Q>(&mut self, value: &Q) -> Option<(usize, &mut T)>
52:     where
53:         Q: ?Sized + Hash + Equivalent<T>,
54:     {
55:         match self.map.get_full_mut2(value) {
56:             Some((index, value, ())) => Some((index, value)),
57:             None => None,
58:         }
59:     }
60:     fn get_index_mut2(&mut self, index: usize) -> Option<&mut T> {
61:         match self.map.get_index_mut2(index) {
62:             Some((value, ())) => Some(value),
63:             None => None,
64:         }
65:     }
66:     fn retain2<F>(&mut self, mut keep: F)
67:     where
68:         F: FnMut(&mut T) -> bool,
69:     {
70:         self.map.retain2(move |value, ()| keep(value));
71:     }
72: }
73: trait Sealed {}
74: impl<T, S> Sealed for IndexSet<T, S> {}
````

## File: src/set/slice.rs
````rust
  1: use super::{Bucket, IndexSet, IntoIter, Iter};
  2: use crate::util::{slice_eq, try_simplify_range};
  3: use alloc::boxed::Box;
  4: use alloc::vec::Vec;
  5: use core::cmp::Ordering;
  6: use core::fmt;
  7: use core::hash::{Hash, Hasher};
  8: use core::ops::{self, Bound, Index, RangeBounds};
  9: /// A dynamically-sized slice of values in an [`IndexSet`].
 10: ///
 11: /// This supports indexed operations much like a `[T]` slice,
 12: /// but not any hashed operations on the values.
 13: ///
 14: /// Unlike `IndexSet`, `Slice` does consider the order for [`PartialEq`]
 15: /// and [`Eq`], and it also implements [`PartialOrd`], [`Ord`], and [`Hash`].
 16: #[repr(transparent)]
 17: pub struct Slice<T> {
 18:     pub(crate) entries: [Bucket<T>],
 19: }
 20: // SAFETY: `Slice<T>` is a transparent wrapper around `[Bucket<T>]`,
 21: // and reference lifetimes are bound together in function signatures.
 22: #[allow(unsafe_code)]
 23: impl<T> Slice<T> {
 24:     pub(super) const fn from_slice(entries: &[Bucket<T>]) -> &Self {
 25:         unsafe { &*(entries as *const [Bucket<T>] as *const Self) }
 26:     }
 27:     pub(super) fn from_boxed(entries: Box<[Bucket<T>]>) -> Box<Self> {
 28:         unsafe { Box::from_raw(Box::into_raw(entries) as *mut Self) }
 29:     }
 30:     fn into_boxed(self: Box<Self>) -> Box<[Bucket<T>]> {
 31:         unsafe { Box::from_raw(Box::into_raw(self) as *mut [Bucket<T>]) }
 32:     }
 33: }
 34: impl<T> Slice<T> {
 35:     pub(crate) fn into_entries(self: Box<Self>) -> Vec<Bucket<T>> {
 36:         self.into_boxed().into_vec()
 37:     }
 38:     /// Returns an empty slice.
 39:     pub const fn new<'a>() -> &'a Self {
 40:         Self::from_slice(&[])
 41:     }
 42:     /// Return the number of elements in the set slice.
 43:     pub const fn len(&self) -> usize {
 44:         self.entries.len()
 45:     }
 46:     /// Returns true if the set slice contains no elements.
 47:     pub const fn is_empty(&self) -> bool {
 48:         self.entries.is_empty()
 49:     }
 50:     /// Get a value by index.
 51:     ///
 52:     /// Valid indices are `0 <= index < self.len()`.
 53:     pub fn get_index(&self, index: usize) -> Option<&T> {
 54:         self.entries.get(index).map(Bucket::key_ref)
 55:     }
 56:     /// Returns a slice of values in the given range of indices.
 57:     ///
 58:     /// Valid indices are `0 <= index < self.len()`.
 59:     pub fn get_range<R: RangeBounds<usize>>(&self, range: R) -> Option<&Self> {
 60:         let range = try_simplify_range(range, self.entries.len())?;
 61:         self.entries.get(range).map(Self::from_slice)
 62:     }
 63:     /// Get the first value.
 64:     pub const fn first(&self) -> Option<&T> {
 65:         if let [first, ..] = &self.entries {
 66:             Some(&first.key)
 67:         } else {
 68:             None
 69:         }
 70:     }
 71:     /// Get the last value.
 72:     pub const fn last(&self) -> Option<&T> {
 73:         if let [.., last] = &self.entries {
 74:             Some(&last.key)
 75:         } else {
 76:             None
 77:         }
 78:     }
 79:     /// Divides one slice into two at an index.
 80:     ///
 81:     /// ***Panics*** if `index > len`.
 82:     /// For a non-panicking alternative see [`split_at_checked`][Self::split_at_checked].
 83:     #[track_caller]
 84:     pub const fn split_at(&self, index: usize) -> (&Self, &Self) {
 85:         let (first, second) = self.entries.split_at(index);
 86:         (Self::from_slice(first), Self::from_slice(second))
 87:     }
 88:     /// Divides one slice into two at an index.
 89:     ///
 90:     /// Returns `None` if `index > len`.
 91:     pub const fn split_at_checked(&self, index: usize) -> Option<(&Self, &Self)> {
 92:         if let Some((first, second)) = self.entries.split_at_checked(index) {
 93:             Some((Self::from_slice(first), Self::from_slice(second)))
 94:         } else {
 95:             None
 96:         }
 97:     }
 98:     /// Returns the first value and the rest of the slice,
 99:     /// or `None` if it is empty.
100:     pub const fn split_first(&self) -> Option<(&T, &Self)> {
101:         if let [first, rest @ ..] = &self.entries {
102:             Some((&first.key, Self::from_slice(rest)))
103:         } else {
104:             None
105:         }
106:     }
107:     /// Returns the last value and the rest of the slice,
108:     /// or `None` if it is empty.
109:     pub const fn split_last(&self) -> Option<(&T, &Self)> {
110:         if let [rest @ .., last] = &self.entries {
111:             Some((&last.key, Self::from_slice(rest)))
112:         } else {
113:             None
114:         }
115:     }
116:     /// Return an iterator over the values of the set slice.
117:     pub fn iter(&self) -> Iter<'_, T> {
118:         Iter::new(&self.entries)
119:     }
120:     /// Search over a sorted set for a value.
121:     ///
122:     /// Returns the position where that value is present, or the position where it can be inserted
123:     /// to maintain the sort. See [`slice::binary_search`] for more details.
124:     ///
125:     /// Computes in **O(log(n))** time, which is notably less scalable than looking the value up in
126:     /// the set this is a slice from using [`IndexSet::get_index_of`], but this can also position
127:     /// missing values.
128:     pub fn binary_search(&self, x: &T) -> Result<usize, usize>
129:     where
130:         T: Ord,
131:     {
132:         self.binary_search_by(|p| p.cmp(x))
133:     }
134:     /// Search over a sorted set with a comparator function.
135:     ///
136:     /// Returns the position where that value is present, or the position where it can be inserted
137:     /// to maintain the sort. See [`slice::binary_search_by`] for more details.
138:     ///
139:     /// Computes in **O(log(n))** time.
140:     #[inline]
141:     pub fn binary_search_by<'a, F>(&'a self, mut f: F) -> Result<usize, usize>
142:     where
143:         F: FnMut(&'a T) -> Ordering,
144:     {
145:         self.entries.binary_search_by(move |a| f(&a.key))
146:     }
147:     /// Search over a sorted set with an extraction function.
148:     ///
149:     /// Returns the position where that value is present, or the position where it can be inserted
150:     /// to maintain the sort. See [`slice::binary_search_by_key`] for more details.
151:     ///
152:     /// Computes in **O(log(n))** time.
153:     #[inline]
154:     pub fn binary_search_by_key<'a, B, F>(&'a self, b: &B, mut f: F) -> Result<usize, usize>
155:     where
156:         F: FnMut(&'a T) -> B,
157:         B: Ord,
158:     {
159:         self.binary_search_by(|k| f(k).cmp(b))
160:     }
161:     /// Checks if the values of this slice are sorted.
162:     #[inline]
163:     pub fn is_sorted(&self) -> bool
164:     where
165:         T: PartialOrd,
166:     {
167:         self.entries.is_sorted_by(|a, b| a.key <= b.key)
168:     }
169:     /// Checks if this slice is sorted using the given comparator function.
170:     #[inline]
171:     pub fn is_sorted_by<'a, F>(&'a self, mut cmp: F) -> bool
172:     where
173:         F: FnMut(&'a T, &'a T) -> bool,
174:     {
175:         self.entries.is_sorted_by(move |a, b| cmp(&a.key, &b.key))
176:     }
177:     /// Checks if this slice is sorted using the given sort-key function.
178:     #[inline]
179:     pub fn is_sorted_by_key<'a, F, K>(&'a self, mut sort_key: F) -> bool
180:     where
181:         F: FnMut(&'a T) -> K,
182:         K: PartialOrd,
183:     {
184:         self.entries.is_sorted_by_key(move |a| sort_key(&a.key))
185:     }
186:     /// Returns the index of the partition point of a sorted set according to the given predicate
187:     /// (the index of the first element of the second partition).
188:     ///
189:     /// See [`slice::partition_point`] for more details.
190:     ///
191:     /// Computes in **O(log(n))** time.
192:     #[must_use]
193:     pub fn partition_point<P>(&self, mut pred: P) -> usize
194:     where
195:         P: FnMut(&T) -> bool,
196:     {
197:         self.entries.partition_point(move |a| pred(&a.key))
198:     }
199: }
200: impl<'a, T> IntoIterator for &'a Slice<T> {
201:     type IntoIter = Iter<'a, T>;
202:     type Item = &'a T;
203:     fn into_iter(self) -> Self::IntoIter {
204:         self.iter()
205:     }
206: }
207: impl<T> IntoIterator for Box<Slice<T>> {
208:     type IntoIter = IntoIter<T>;
209:     type Item = T;
210:     fn into_iter(self) -> Self::IntoIter {
211:         IntoIter::new(self.into_entries())
212:     }
213: }
214: impl<T> Default for &'_ Slice<T> {
215:     fn default() -> Self {
216:         Slice::from_slice(&[])
217:     }
218: }
219: impl<T> Default for Box<Slice<T>> {
220:     fn default() -> Self {
221:         Slice::from_boxed(Box::default())
222:     }
223: }
224: impl<T: Clone> Clone for Box<Slice<T>> {
225:     fn clone(&self) -> Self {
226:         Slice::from_boxed(self.entries.to_vec().into_boxed_slice())
227:     }
228: }
229: impl<T: Copy> From<&Slice<T>> for Box<Slice<T>> {
230:     fn from(slice: &Slice<T>) -> Self {
231:         Slice::from_boxed(Box::from(&slice.entries))
232:     }
233: }
234: impl<T: fmt::Debug> fmt::Debug for Slice<T> {
235:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
236:         f.debug_list().entries(self).finish()
237:     }
238: }
239: impl<T, U> PartialEq<Slice<U>> for Slice<T>
240: where
241:     T: PartialEq<U>,
242: {
243:     fn eq(&self, other: &Slice<U>) -> bool {
244:         slice_eq(&self.entries, &other.entries, |b1, b2| b1.key == b2.key)
245:     }
246: }
247: impl<T, U> PartialEq<[U]> for Slice<T>
248: where
249:     T: PartialEq<U>,
250: {
251:     fn eq(&self, other: &[U]) -> bool {
252:         slice_eq(&self.entries, other, |b, o| b.key == *o)
253:     }
254: }
255: impl<T, U> PartialEq<Slice<U>> for [T]
256: where
257:     T: PartialEq<U>,
258: {
259:     fn eq(&self, other: &Slice<U>) -> bool {
260:         slice_eq(self, &other.entries, |o, b| *o == b.key)
261:     }
262: }
263: impl<T, U, const N: usize> PartialEq<[U; N]> for Slice<T>
264: where
265:     T: PartialEq<U>,
266: {
267:     fn eq(&self, other: &[U; N]) -> bool {
268:         <Self as PartialEq<[U]>>::eq(self, other)
269:     }
270: }
271: impl<T, const N: usize, U> PartialEq<Slice<U>> for [T; N]
272: where
273:     T: PartialEq<U>,
274: {
275:     fn eq(&self, other: &Slice<U>) -> bool {
276:         <[T] as PartialEq<Slice<U>>>::eq(self, other)
277:     }
278: }
279: impl<T: Eq> Eq for Slice<T> {}
280: impl<T: PartialOrd> PartialOrd for Slice<T> {
281:     fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
282:         self.iter().partial_cmp(other)
283:     }
284: }
285: impl<T: Ord> Ord for Slice<T> {
286:     fn cmp(&self, other: &Self) -> Ordering {
287:         self.iter().cmp(other)
288:     }
289: }
290: impl<T: Hash> Hash for Slice<T> {
291:     fn hash<H: Hasher>(&self, state: &mut H) {
292:         self.len().hash(state);
293:         for value in self {
294:             value.hash(state);
295:         }
296:     }
297: }
298: impl<T> Index<usize> for Slice<T> {
299:     type Output = T;
300:     fn index(&self, index: usize) -> &Self::Output {
301:         &self.entries[index].key
302:     }
303: }
304: // We can't have `impl<I: RangeBounds<usize>> Index<I>` because that conflicts with `Index<usize>`.
305: // Instead, we repeat the implementations for all the core range types.
306: macro_rules! impl_index {
307:     ($($range:ty),*) => {$(
308:         impl<T, S> Index<$range> for IndexSet<T, S> {
309:             type Output = Slice<T>;
310:             fn index(&self, range: $range) -> &Self::Output {
311:                 Slice::from_slice(&self.as_entries()[range])
312:             }
313:         }
314:         impl<T> Index<$range> for Slice<T> {
315:             type Output = Self;
316:             fn index(&self, range: $range) -> &Self::Output {
317:                 Slice::from_slice(&self.entries[range])
318:             }
319:         }
320:     )*}
321: }
322: impl_index!(
323:     ops::Range<usize>,
324:     ops::RangeFrom<usize>,
325:     ops::RangeFull,
326:     ops::RangeInclusive<usize>,
327:     ops::RangeTo<usize>,
328:     ops::RangeToInclusive<usize>,
329:     (Bound<usize>, Bound<usize>)
330: );
331: #[cfg(test)]
332: mod tests {
333:     use super::*;
334:     #[test]
335:     fn slice_index() {
336:         fn check(vec_slice: &[i32], set_slice: &Slice<i32>, sub_slice: &Slice<i32>) {
337:             assert_eq!(set_slice as *const _, sub_slice as *const _);
338:             itertools::assert_equal(vec_slice, set_slice);
339:         }
340:         let vec: Vec<i32> = (0..10).map(|i| i * i).collect();
341:         let set: IndexSet<i32> = vec.iter().cloned().collect();
342:         let slice = set.as_slice();
343:         // RangeFull
344:         #[expect(clippy::redundant_slicing)]
345:         check(&vec[..], &set[..], &slice[..]);
346:         for i in 0usize..10 {
347:             // Index
348:             assert_eq!(vec[i], set[i]);
349:             assert_eq!(vec[i], slice[i]);
350:             // RangeFrom
351:             check(&vec[i..], &set[i..], &slice[i..]);
352:             // RangeTo
353:             check(&vec[..i], &set[..i], &slice[..i]);
354:             // RangeToInclusive
355:             check(&vec[..=i], &set[..=i], &slice[..=i]);
356:             // (Bound<usize>, Bound<usize>)
357:             let bounds = (Bound::Excluded(i), Bound::Unbounded);
358:             check(&vec[i + 1..], &set[bounds], &slice[bounds]);
359:             for j in i..=10 {
360:                 // Range
361:                 check(&vec[i..j], &set[i..j], &slice[i..j]);
362:             }
363:             for j in i..10 {
364:                 // RangeInclusive
365:                 check(&vec[i..=j], &set[i..=j], &slice[i..=j]);
366:             }
367:         }
368:     }
369: }
````

## File: src/set/tests.rs
````rust
  1: use super::*;
  2: use std::string::String;
  3: #[test]
  4: fn it_works() {
  5:     let mut set = IndexSet::new();
  6:     assert_eq!(set.is_empty(), true);
  7:     set.insert(1);
  8:     set.insert(1);
  9:     assert_eq!(set.len(), 1);
 10:     assert!(set.get(&1).is_some());
 11:     assert_eq!(set.is_empty(), false);
 12: }
 13: #[test]
 14: fn new() {
 15:     let set = IndexSet::<String>::new();
 16:     println!("{:?}", set);
 17:     assert_eq!(set.capacity(), 0);
 18:     assert_eq!(set.len(), 0);
 19:     assert_eq!(set.is_empty(), true);
 20: }
 21: #[test]
 22: fn insert() {
 23:     let insert = [0, 4, 2, 12, 8, 7, 11, 5];
 24:     let not_present = [1, 3, 6, 9, 10];
 25:     let mut set = IndexSet::with_capacity(insert.len());
 26:     for (i, &elt) in insert.iter().enumerate() {
 27:         assert_eq!(set.len(), i);
 28:         set.insert(elt);
 29:         assert_eq!(set.len(), i + 1);
 30:         assert_eq!(set.get(&elt), Some(&elt));
 31:     }
 32:     println!("{:?}", set);
 33:     for &elt in &not_present {
 34:         assert!(set.get(&elt).is_none());
 35:     }
 36: }
 37: #[test]
 38: fn insert_full() {
 39:     let insert = [9, 2, 7, 1, 4, 6, 13];
 40:     let present = [1, 6, 2];
 41:     let mut set = IndexSet::with_capacity(insert.len());
 42:     for (i, &elt) in insert.iter().enumerate() {
 43:         assert_eq!(set.len(), i);
 44:         let (index, success) = set.insert_full(elt);
 45:         assert!(success);
 46:         assert_eq!(Some(index), set.get_full(&elt).map(|x| x.0));
 47:         assert_eq!(set.len(), i + 1);
 48:     }
 49:     let len = set.len();
 50:     for &elt in &present {
 51:         let (index, success) = set.insert_full(elt);
 52:         assert!(!success);
 53:         assert_eq!(Some(index), set.get_full(&elt).map(|x| x.0));
 54:         assert_eq!(set.len(), len);
 55:     }
 56: }
 57: #[test]
 58: fn insert_2() {
 59:     let mut set = IndexSet::with_capacity(16);
 60:     let mut values = vec![];
 61:     values.extend(0..16);
 62:     values.extend(if cfg!(miri) { 32..64 } else { 128..267 });
 63:     for &i in &values {
 64:         let old_set = set.clone();
 65:         set.insert(i);
 66:         for value in old_set.iter() {
 67:             if set.get(value).is_none() {
 68:                 println!("old_set: {:?}", old_set);
 69:                 println!("set: {:?}", set);
 70:                 panic!("did not find {} in set", value);
 71:             }
 72:         }
 73:     }
 74:     for &i in &values {
 75:         assert!(set.get(&i).is_some(), "did not find {}", i);
 76:     }
 77: }
 78: #[test]
 79: fn insert_dup() {
 80:     let mut set = IndexSet::<u8>::from_iter([0, 2, 4, 6, 8]);
 81:     {
 82:         let (i, v) = set.get_full(&0).unwrap();
 83:         assert_eq!(set.len(), 5);
 84:         assert_eq!(i, 0);
 85:         assert_eq!(*v, 0);
 86:     }
 87:     {
 88:         let inserted = set.insert(0);
 89:         let (i, v) = set.get_full(&0).unwrap();
 90:         assert_eq!(set.len(), 5);
 91:         assert_eq!(inserted, false);
 92:         assert_eq!(i, 0);
 93:         assert_eq!(*v, 0);
 94:     }
 95: }
 96: #[test]
 97: fn insert_order() {
 98:     let insert = [0, 4, 2, 12, 8, 7, 11, 5, 3, 17, 19, 22, 23];
 99:     let mut set = IndexSet::new();
100:     for &elt in &insert {
101:         set.insert(elt);
102:     }
103:     assert_eq!(set.iter().count(), set.len());
104:     assert_eq!(set.iter().count(), insert.len());
105:     for (a, b) in insert.iter().zip(set.iter()) {
106:         assert_eq!(a, b);
107:     }
108:     for (i, v) in (0..insert.len()).zip(set.iter()) {
109:         assert_eq!(set.get_index(i).unwrap(), v);
110:     }
111: }
112: #[test]
113: fn shift_insert() {
114:     let insert = [0, 4, 2, 12, 8, 7, 11, 5, 3, 17, 19, 22, 23];
115:     let mut set = IndexSet::new();
116:     for &elt in &insert {
117:         set.shift_insert(0, elt);
118:     }
119:     assert_eq!(set.iter().count(), set.len());
120:     assert_eq!(set.iter().count(), insert.len());
121:     for (a, b) in insert.iter().rev().zip(set.iter()) {
122:         assert_eq!(a, b);
123:     }
124:     for (i, v) in (0..insert.len()).zip(set.iter()) {
125:         assert_eq!(set.get_index(i).unwrap(), v);
126:     }
127:     // "insert" that moves an existing entry
128:     set.shift_insert(0, insert[0]);
129:     assert_eq!(set.iter().count(), insert.len());
130:     assert_eq!(insert[0], set[0]);
131:     for (a, b) in insert[1..].iter().rev().zip(set.iter().skip(1)) {
132:         assert_eq!(a, b);
133:     }
134: }
135: #[test]
136: fn replace() {
137:     let replace = [0, 4, 2, 12, 8, 7, 11, 5];
138:     let not_present = [1, 3, 6, 9, 10];
139:     let mut set = IndexSet::with_capacity(replace.len());
140:     for (i, &elt) in replace.iter().enumerate() {
141:         assert_eq!(set.len(), i);
142:         set.replace(elt);
143:         assert_eq!(set.len(), i + 1);
144:         assert_eq!(set.get(&elt), Some(&elt));
145:     }
146:     println!("{:?}", set);
147:     for &elt in &not_present {
148:         assert!(set.get(&elt).is_none());
149:     }
150: }
151: #[test]
152: fn replace_full() {
153:     let replace = [9, 2, 7, 1, 4, 6, 13];
154:     let present = [1, 6, 2];
155:     let mut set = IndexSet::with_capacity(replace.len());
156:     for (i, &elt) in replace.iter().enumerate() {
157:         assert_eq!(set.len(), i);
158:         let (index, replaced) = set.replace_full(elt);
159:         assert!(replaced.is_none());
160:         assert_eq!(Some(index), set.get_full(&elt).map(|x| x.0));
161:         assert_eq!(set.len(), i + 1);
162:     }
163:     let len = set.len();
164:     for &elt in &present {
165:         let (index, replaced) = set.replace_full(elt);
166:         assert_eq!(Some(elt), replaced);
167:         assert_eq!(Some(index), set.get_full(&elt).map(|x| x.0));
168:         assert_eq!(set.len(), len);
169:     }
170: }
171: #[test]
172: fn replace_2() {
173:     let mut set = IndexSet::with_capacity(16);
174:     let mut values = vec![];
175:     values.extend(0..16);
176:     values.extend(if cfg!(miri) { 32..64 } else { 128..267 });
177:     for &i in &values {
178:         let old_set = set.clone();
179:         set.replace(i);
180:         for value in old_set.iter() {
181:             if set.get(value).is_none() {
182:                 println!("old_set: {:?}", old_set);
183:                 println!("set: {:?}", set);
184:                 panic!("did not find {} in set", value);
185:             }
186:         }
187:     }
188:     for &i in &values {
189:         assert!(set.get(&i).is_some(), "did not find {}", i);
190:     }
191: }
192: #[test]
193: fn replace_dup() {
194:     let mut set = IndexSet::<u8>::from_iter([0, 2, 4, 6, 8]);
195:     {
196:         let (i, v) = set.get_full(&0).unwrap();
197:         assert_eq!(set.len(), 5);
198:         assert_eq!(i, 0);
199:         assert_eq!(*v, 0);
200:     }
201:     {
202:         let replaced = set.replace(0);
203:         let (i, v) = set.get_full(&0).unwrap();
204:         assert_eq!(set.len(), 5);
205:         assert_eq!(replaced, Some(0));
206:         assert_eq!(i, 0);
207:         assert_eq!(*v, 0);
208:     }
209: }
210: #[test]
211: fn replace_order() {
212:     let replace = [0, 4, 2, 12, 8, 7, 11, 5, 3, 17, 19, 22, 23];
213:     let mut set = IndexSet::new();
214:     for &elt in &replace {
215:         set.replace(elt);
216:     }
217:     assert_eq!(set.iter().count(), set.len());
218:     assert_eq!(set.iter().count(), replace.len());
219:     for (a, b) in replace.iter().zip(set.iter()) {
220:         assert_eq!(a, b);
221:     }
222:     for (i, v) in (0..replace.len()).zip(set.iter()) {
223:         assert_eq!(set.get_index(i).unwrap(), v);
224:     }
225: }
226: #[test]
227: fn replace_change() {
228:     // Check pointers to make sure it really changes
229:     let mut set = indexset!(vec![42]);
230:     let old_ptr = set[0].as_ptr();
231:     let new = set[0].clone();
232:     let new_ptr = new.as_ptr();
233:     assert_ne!(old_ptr, new_ptr);
234:     let replaced = set.replace(new).unwrap();
235:     assert_eq!(replaced.as_ptr(), old_ptr);
236: }
237: #[test]
238: fn grow() {
239:     let insert = [0, 4, 2, 12, 8, 7, 11];
240:     let not_present = [1, 3, 6, 9, 10];
241:     let mut set = IndexSet::with_capacity(insert.len());
242:     for (i, &elt) in insert.iter().enumerate() {
243:         assert_eq!(set.len(), i);
244:         set.insert(elt);
245:         assert_eq!(set.len(), i + 1);
246:         assert_eq!(set.get(&elt), Some(&elt));
247:     }
248:     println!("{:?}", set);
249:     for &elt in &insert {
250:         set.insert(elt * 10);
251:     }
252:     for &elt in &insert {
253:         set.insert(elt * 100);
254:     }
255:     for (i, &elt) in insert.iter().cycle().enumerate().take(100) {
256:         set.insert(elt * 100 + i as i32);
257:     }
258:     println!("{:?}", set);
259:     for &elt in &not_present {
260:         assert!(set.get(&elt).is_none());
261:     }
262: }
263: #[test]
264: fn reserve() {
265:     let mut set = IndexSet::<usize>::new();
266:     assert_eq!(set.capacity(), 0);
267:     set.reserve(100);
268:     let capacity = set.capacity();
269:     assert!(capacity >= 100);
270:     for i in 0..capacity {
271:         assert_eq!(set.len(), i);
272:         set.insert(i);
273:         assert_eq!(set.len(), i + 1);
274:         assert_eq!(set.capacity(), capacity);
275:         assert_eq!(set.get(&i), Some(&i));
276:     }
277:     set.insert(capacity);
278:     assert_eq!(set.len(), capacity + 1);
279:     assert!(set.capacity() > capacity);
280:     assert_eq!(set.get(&capacity), Some(&capacity));
281: }
282: #[test]
283: fn try_reserve() {
284:     let mut set = IndexSet::<usize>::new();
285:     assert_eq!(set.capacity(), 0);
286:     assert_eq!(set.try_reserve(100), Ok(()));
287:     assert!(set.capacity() >= 100);
288:     assert!(set.try_reserve(usize::MAX).is_err());
289: }
290: #[test]
291: fn shrink_to_fit() {
292:     let mut set = IndexSet::<usize>::new();
293:     assert_eq!(set.capacity(), 0);
294:     for i in 0..100 {
295:         assert_eq!(set.len(), i);
296:         set.insert(i);
297:         assert_eq!(set.len(), i + 1);
298:         assert!(set.capacity() > i);
299:         assert_eq!(set.get(&i), Some(&i));
300:         set.shrink_to_fit();
301:         assert_eq!(set.len(), i + 1);
302:         assert_eq!(set.capacity(), i + 1);
303:         assert_eq!(set.get(&i), Some(&i));
304:     }
305: }
306: #[test]
307: fn remove() {
308:     let insert = [0, 4, 2, 12, 8, 7, 11, 5, 3, 17, 19, 22, 23];
309:     let mut set = IndexSet::new();
310:     for &elt in &insert {
311:         set.insert(elt);
312:     }
313:     assert_eq!(set.iter().count(), set.len());
314:     assert_eq!(set.iter().count(), insert.len());
315:     for (a, b) in insert.iter().zip(set.iter()) {
316:         assert_eq!(a, b);
317:     }
318:     let remove_fail = [99, 77];
319:     let remove = [4, 12, 8, 7];
320:     for &value in &remove_fail {
321:         assert!(set.swap_remove_full(&value).is_none());
322:     }
323:     println!("{:?}", set);
324:     for &value in &remove {
325:         //println!("{:?}", set);
326:         let index = set.get_full(&value).unwrap().0;
327:         assert_eq!(set.swap_remove_full(&value), Some((index, value)));
328:     }
329:     println!("{:?}", set);
330:     for value in &insert {
331:         assert_eq!(set.get(value).is_some(), !remove.contains(value));
332:     }
333:     assert_eq!(set.len(), insert.len() - remove.len());
334:     assert_eq!(set.iter().count(), insert.len() - remove.len());
335: }
336: #[test]
337: fn swap_remove_index() {
338:     let insert = [0, 4, 2, 12, 8, 7, 11, 5, 3, 17, 19, 22, 23];
339:     let mut set = IndexSet::new();
340:     for &elt in &insert {
341:         set.insert(elt);
342:     }
343:     let mut vector = insert.to_vec();
344:     let remove_sequence = &[3, 3, 10, 4, 5, 4, 3, 0, 1];
345:     // check that the same swap remove sequence on vec and set
346:     // have the same result.
347:     for &rm in remove_sequence {
348:         let out_vec = vector.swap_remove(rm);
349:         let out_set = set.swap_remove_index(rm).unwrap();
350:         assert_eq!(out_vec, out_set);
351:     }
352:     assert_eq!(vector.len(), set.len());
353:     for (a, b) in vector.iter().zip(set.iter()) {
354:         assert_eq!(a, b);
355:     }
356: }
357: #[test]
358: fn partial_eq_and_eq() {
359:     let mut set_a = IndexSet::new();
360:     set_a.insert(1);
361:     set_a.insert(2);
362:     let mut set_b = set_a.clone();
363:     assert_eq!(set_a, set_b);
364:     set_b.swap_remove(&1);
365:     assert_ne!(set_a, set_b);
366:     let set_c: IndexSet<_> = set_b.into_iter().collect();
367:     assert_ne!(set_a, set_c);
368:     assert_ne!(set_c, set_a);
369: }
370: #[test]
371: fn extend() {
372:     let mut set = IndexSet::new();
373:     set.extend([&1, &2, &3, &4]);
374:     set.extend([5, 6]);
375:     assert_eq!(set.into_iter().collect::<Vec<_>>(), [1, 2, 3, 4, 5, 6]);
376: }
377: #[test]
378: fn comparisons() {
379:     let set_a: IndexSet<_> = (0..3).collect();
380:     let set_b: IndexSet<_> = (3..6).collect();
381:     let set_c: IndexSet<_> = (0..6).collect();
382:     let set_d: IndexSet<_> = (3..9).collect();
383:     assert!(!set_a.is_disjoint(&set_a));
384:     assert!(set_a.is_subset(&set_a));
385:     assert!(set_a.is_superset(&set_a));
386:     assert!(set_a.is_disjoint(&set_b));
387:     assert!(set_b.is_disjoint(&set_a));
388:     assert!(!set_a.is_subset(&set_b));
389:     assert!(!set_b.is_subset(&set_a));
390:     assert!(!set_a.is_superset(&set_b));
391:     assert!(!set_b.is_superset(&set_a));
392:     assert!(!set_a.is_disjoint(&set_c));
393:     assert!(!set_c.is_disjoint(&set_a));
394:     assert!(set_a.is_subset(&set_c));
395:     assert!(!set_c.is_subset(&set_a));
396:     assert!(!set_a.is_superset(&set_c));
397:     assert!(set_c.is_superset(&set_a));
398:     assert!(!set_c.is_disjoint(&set_d));
399:     assert!(!set_d.is_disjoint(&set_c));
400:     assert!(!set_c.is_subset(&set_d));
401:     assert!(!set_d.is_subset(&set_c));
402:     assert!(!set_c.is_superset(&set_d));
403:     assert!(!set_d.is_superset(&set_c));
404: }
405: #[test]
406: fn iter_comparisons() {
407:     use std::iter::empty;
408:     fn check<'a, I1, I2>(iter1: I1, iter2: I2)
409:     where
410:         I1: Iterator<Item = &'a i32>,
411:         I2: Iterator<Item = i32>,
412:     {
413:         assert!(iter1.copied().eq(iter2));
414:     }
415:     let set_a: IndexSet<_> = (0..3).collect();
416:     let set_b: IndexSet<_> = (3..6).collect();
417:     let set_c: IndexSet<_> = (0..6).collect();
418:     let set_d: IndexSet<_> = (3..9).rev().collect();
419:     check(set_a.difference(&set_a), empty());
420:     check(set_a.symmetric_difference(&set_a), empty());
421:     check(set_a.intersection(&set_a), 0..3);
422:     check(set_a.union(&set_a), 0..3);
423:     check(set_a.difference(&set_b), 0..3);
424:     check(set_b.difference(&set_a), 3..6);
425:     check(set_a.symmetric_difference(&set_b), 0..6);
426:     check(set_b.symmetric_difference(&set_a), (3..6).chain(0..3));
427:     check(set_a.intersection(&set_b), empty());
428:     check(set_b.intersection(&set_a), empty());
429:     check(set_a.union(&set_b), 0..6);
430:     check(set_b.union(&set_a), (3..6).chain(0..3));
431:     check(set_a.difference(&set_c), empty());
432:     check(set_c.difference(&set_a), 3..6);
433:     check(set_a.symmetric_difference(&set_c), 3..6);
434:     check(set_c.symmetric_difference(&set_a), 3..6);
435:     check(set_a.intersection(&set_c), 0..3);
436:     check(set_c.intersection(&set_a), 0..3);
437:     check(set_a.union(&set_c), 0..6);
438:     check(set_c.union(&set_a), 0..6);
439:     check(set_c.difference(&set_d), 0..3);
440:     check(set_d.difference(&set_c), (6..9).rev());
441:     check(
442:         set_c.symmetric_difference(&set_d),
443:         (0..3).chain((6..9).rev()),
444:     );
445:     check(set_d.symmetric_difference(&set_c), (6..9).rev().chain(0..3));
446:     check(set_c.intersection(&set_d), 3..6);
447:     check(set_d.intersection(&set_c), (3..6).rev());
448:     check(set_c.union(&set_d), (0..6).chain((6..9).rev()));
449:     check(set_d.union(&set_c), (3..9).rev().chain(0..3));
450: }
451: #[test]
452: fn ops() {
453:     let empty = IndexSet::<i32>::new();
454:     let set_a: IndexSet<_> = (0..3).collect();
455:     let set_b: IndexSet<_> = (3..6).collect();
456:     let set_c: IndexSet<_> = (0..6).collect();
457:     let set_d: IndexSet<_> = (3..9).rev().collect();
458:     #[allow(clippy::eq_op)]
459:     {
460:         assert_eq!(&set_a & &set_a, set_a);
461:         assert_eq!(&set_a | &set_a, set_a);
462:         assert_eq!(&set_a ^ &set_a, empty);
463:         assert_eq!(&set_a - &set_a, empty);
464:     }
465:     assert_eq!(&set_a & &set_b, empty);
466:     assert_eq!(&set_b & &set_a, empty);
467:     assert_eq!(&set_a | &set_b, set_c);
468:     assert_eq!(&set_b | &set_a, set_c);
469:     assert_eq!(&set_a ^ &set_b, set_c);
470:     assert_eq!(&set_b ^ &set_a, set_c);
471:     assert_eq!(&set_a - &set_b, set_a);
472:     assert_eq!(&set_b - &set_a, set_b);
473:     assert_eq!(&set_a & &set_c, set_a);
474:     assert_eq!(&set_c & &set_a, set_a);
475:     assert_eq!(&set_a | &set_c, set_c);
476:     assert_eq!(&set_c | &set_a, set_c);
477:     assert_eq!(&set_a ^ &set_c, set_b);
478:     assert_eq!(&set_c ^ &set_a, set_b);
479:     assert_eq!(&set_a - &set_c, empty);
480:     assert_eq!(&set_c - &set_a, set_b);
481:     assert_eq!(&set_c & &set_d, set_b);
482:     assert_eq!(&set_d & &set_c, set_b);
483:     assert_eq!(&set_c | &set_d, &set_a | &set_d);
484:     assert_eq!(&set_d | &set_c, &set_a | &set_d);
485:     assert_eq!(&set_c ^ &set_d, &set_a | &(&set_d - &set_b));
486:     assert_eq!(&set_d ^ &set_c, &set_a | &(&set_d - &set_b));
487:     assert_eq!(&set_c - &set_d, set_a);
488:     assert_eq!(&set_d - &set_c, &set_d - &set_b);
489: }
490: #[test]
491: #[cfg(feature = "std")]
492: fn from_array() {
493:     let set1 = IndexSet::from([1, 2, 3, 4]);
494:     let set2: IndexSet<_> = [1, 2, 3, 4].into();
495:     assert_eq!(set1, set2);
496: }
497: #[test]
498: fn iter_default() {
499:     struct Item;
500:     fn assert_default<T>()
501:     where
502:         T: Default + Iterator,
503:     {
504:         assert!(T::default().next().is_none());
505:     }
506:     assert_default::<Iter<'static, Item>>();
507:     assert_default::<IntoIter<Item>>();
508: }
509: #[test]
510: #[allow(deprecated)]
511: fn take() {
512:     let mut index_set: IndexSet<i32> = IndexSet::new();
513:     index_set.insert(10);
514:     assert_eq!(index_set.len(), 1);
515:     let result = index_set.take(&10);
516:     assert_eq!(result, Some(10));
517:     assert_eq!(index_set.len(), 0);
518:     let result = index_set.take(&20);
519:     assert_eq!(result, None);
520: }
521: #[test]
522: fn swap_take() {
523:     let mut index_set: IndexSet<i32> = IndexSet::new();
524:     index_set.insert(10);
525:     index_set.insert(20);
526:     index_set.insert(30);
527:     index_set.insert(40);
528:     assert_eq!(index_set.len(), 4);
529:     let result = index_set.swap_take(&20);
530:     assert_eq!(result, Some(20));
531:     assert_eq!(index_set.len(), 3);
532:     assert_eq!(index_set.as_slice(), &[10, 40, 30]);
533:     let result = index_set.swap_take(&50);
534:     assert_eq!(result, None);
535: }
536: #[test]
537: fn sort_unstable() {
538:     let mut index_set: IndexSet<i32> = IndexSet::new();
539:     index_set.insert(30);
540:     index_set.insert(20);
541:     index_set.insert(10);
542:     index_set.sort_unstable();
543:     assert_eq!(index_set.as_slice(), &[10, 20, 30]);
544: }
545: #[test]
546: fn try_reserve_exact() {
547:     let mut index_set: IndexSet<i32> = IndexSet::new();
548:     index_set.insert(10);
549:     index_set.insert(20);
550:     index_set.insert(30);
551:     index_set.shrink_to_fit();
552:     assert_eq!(index_set.capacity(), 3);
553:     index_set.try_reserve_exact(2).unwrap();
554:     assert_eq!(index_set.capacity(), 5);
555: }
556: #[test]
557: fn shift_remove_full() {
558:     let mut set: IndexSet<i32> = IndexSet::new();
559:     set.insert(10);
560:     set.insert(20);
561:     set.insert(30);
562:     set.insert(40);
563:     set.insert(50);
564:     let result = set.shift_remove_full(&20);
565:     assert_eq!(result, Some((1, 20)));
566:     assert_eq!(set.len(), 4);
567:     assert_eq!(set.as_slice(), &[10, 30, 40, 50]);
568:     let result = set.shift_remove_full(&50);
569:     assert_eq!(result, Some((3, 50)));
570:     assert_eq!(set.len(), 3);
571:     assert_eq!(set.as_slice(), &[10, 30, 40]);
572:     let result = set.shift_remove_full(&60);
573:     assert_eq!(result, None);
574:     assert_eq!(set.len(), 3);
575:     assert_eq!(set.as_slice(), &[10, 30, 40]);
576: }
577: #[test]
578: fn shift_remove_index() {
579:     let mut set: IndexSet<i32> = IndexSet::new();
580:     set.insert(10);
581:     set.insert(20);
582:     set.insert(30);
583:     set.insert(40);
584:     set.insert(50);
585:     let result = set.shift_remove_index(1);
586:     assert_eq!(result, Some(20));
587:     assert_eq!(set.len(), 4);
588:     assert_eq!(set.as_slice(), &[10, 30, 40, 50]);
589:     let result = set.shift_remove_index(1);
590:     assert_eq!(result, Some(30));
591:     assert_eq!(set.len(), 3);
592:     assert_eq!(set.as_slice(), &[10, 40, 50]);
593:     let result = set.shift_remove_index(3);
594:     assert_eq!(result, None);
595:     assert_eq!(set.len(), 3);
596:     assert_eq!(set.as_slice(), &[10, 40, 50]);
597: }
598: #[test]
599: fn sort_unstable_by() {
600:     let mut set: IndexSet<i32> = IndexSet::from([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
601:     set.sort_unstable_by(|a, b| b.cmp(a));
602:     assert_eq!(set.as_slice(), &[10, 9, 8, 7, 6, 5, 4, 3, 2, 1]);
603: }
604: #[test]
605: fn sort_by() {
606:     let mut set: IndexSet<i32> = IndexSet::new();
607:     set.insert(3);
608:     set.insert(1);
609:     set.insert(2);
610:     set.sort_by(|a, b| a.cmp(b));
611:     assert_eq!(set.as_slice(), &[1, 2, 3]);
612: }
613: #[test]
614: fn drain() {
615:     let mut set: IndexSet<i32> = IndexSet::new();
616:     set.insert(1);
617:     set.insert(2);
618:     set.insert(3);
619:     {
620:         let drain = set.drain(0..2);
621:         assert_eq!(drain.as_slice(), &[1, 2]);
622:     }
623:     assert_eq!(set.len(), 1);
624:     assert_eq!(set.as_slice(), &[3]);
625: }
626: #[test]
627: fn split_off() {
628:     let mut set: IndexSet<i32> = IndexSet::from([1, 2, 3, 4, 5]);
629:     let split_set: IndexSet<i32> = set.split_off(3);
630:     assert_eq!(split_set.len(), 2);
631:     assert_eq!(split_set.as_slice(), &[4, 5]);
632:     assert_eq!(set.len(), 3);
633:     assert_eq!(set.as_slice(), &[1, 2, 3]);
634: }
635: #[test]
636: fn retain() {
637:     let mut set: IndexSet<i32> = IndexSet::from([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
638:     set.retain(|&x| x > 4);
639:     assert_eq!(set.len(), 6);
640:     assert_eq!(set.as_slice(), &[5, 6, 7, 8, 9, 10]);
641:     set.retain(|_| false);
642:     assert_eq!(set.len(), 0);
643: }
644: #[test]
645: fn first() {
646:     let mut index_set: IndexSet<i32> = IndexSet::new();
647:     index_set.insert(10);
648:     index_set.insert(20);
649:     index_set.insert(30);
650:     let result = index_set.first();
651:     assert_eq!(*result.unwrap(), 10);
652:     index_set.clear();
653:     let result = index_set.first();
654:     assert!(result.is_none());
655: }
656: #[test]
657: fn sort_by_key() {
658:     let mut index_set: IndexSet<i32> = IndexSet::new();
659:     index_set.insert(3);
660:     index_set.insert(1);
661:     index_set.insert(2);
662:     index_set.insert(0);
663:     index_set.sort_by_key(|&x| -x);
664:     assert_eq!(index_set.as_slice(), &[3, 2, 1, 0]);
665: }
666: #[test]
667: fn sort_unstable_by_key() {
668:     let mut index_set: IndexSet<i32> = IndexSet::new();
669:     index_set.insert(3);
670:     index_set.insert(1);
671:     index_set.insert(2);
672:     index_set.insert(0);
673:     index_set.sort_unstable_by_key(|&x| -x);
674:     assert_eq!(index_set.as_slice(), &[3, 2, 1, 0]);
675: }
676: #[test]
677: fn sort_by_cached_key() {
678:     let mut index_set: IndexSet<i32> = IndexSet::new();
679:     index_set.insert(3);
680:     index_set.insert(1);
681:     index_set.insert(2);
682:     index_set.insert(0);
683:     index_set.sort_by_cached_key(|&x| -x);
684:     assert_eq!(index_set.as_slice(), &[3, 2, 1, 0]);
685: }
686: #[test]
687: fn insert_sorted() {
688:     let mut set: IndexSet<i32> = IndexSet::<i32>::new();
689:     set.insert_sorted(1);
690:     set.insert_sorted(3);
691:     assert_eq!(set.insert_sorted(2), (1, true));
692: }
693: #[test]
694: fn binary_search() {
695:     let mut set: IndexSet<i32> = IndexSet::new();
696:     set.insert(100);
697:     set.insert(300);
698:     set.insert(200);
699:     set.insert(400);
700:     let result = set.binary_search(&200);
701:     assert_eq!(result, Ok(2));
702:     let result = set.binary_search(&500);
703:     assert_eq!(result, Err(4));
704: }
705: #[test]
706: fn sorted_unstable_by() {
707:     let mut set: IndexSet<i32> = IndexSet::from([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
708:     set.sort_unstable_by(|a, b| b.cmp(a));
709:     assert_eq!(set.as_slice(), &[10, 9, 8, 7, 6, 5, 4, 3, 2, 1]);
710: }
711: #[test]
712: fn last() {
713:     let mut set: IndexSet<i32> = IndexSet::new();
714:     set.insert(1);
715:     set.insert(2);
716:     set.insert(3);
717:     set.insert(4);
718:     set.insert(5);
719:     set.insert(6);
720:     assert_eq!(set.last(), Some(&6));
721:     set.pop();
722:     assert_eq!(set.last(), Some(&5));
723:     set.clear();
724:     assert_eq!(set.last(), None);
725: }
726: #[test]
727: fn get_range() {
728:     let set: IndexSet<i32> = IndexSet::from([1, 2, 3, 4, 5]);
729:     let result = set.get_range(0..3);
730:     let slice: &Slice<i32> = result.unwrap();
731:     assert_eq!(slice, &[1, 2, 3]);
732:     let result = set.get_range(0..0);
733:     assert_eq!(result.unwrap().len(), 0);
734:     #[expect(clippy::reversed_empty_ranges)]
735:     let result = set.get_range(2..1);
736:     assert!(result.is_none());
737: }
738: #[test]
739: fn shift_take() {
740:     let mut set: IndexSet<i32> = IndexSet::new();
741:     set.insert(1);
742:     set.insert(2);
743:     set.insert(3);
744:     set.insert(4);
745:     set.insert(5);
746:     let result = set.shift_take(&2);
747:     assert_eq!(result, Some(2));
748:     assert_eq!(set.len(), 4);
749:     assert_eq!(set.as_slice(), &[1, 3, 4, 5]);
750:     let result = set.shift_take(&5);
751:     assert_eq!(result, Some(5));
752:     assert_eq!(set.len(), 3);
753:     assert_eq!(set.as_slice(), &[1, 3, 4]);
754:     let result = set.shift_take(&5);
755:     assert_eq!(result, None);
756:     assert_eq!(set.len(), 3);
757:     assert_eq!(set.as_slice(), &[1, 3, 4]);
758: }
759: #[test]
760: fn test_binary_search_by() {
761:     // adapted from std's test for binary_search
762:     let b: IndexSet<i32> = [].into();
763:     assert_eq!(b.binary_search_by(|x| x.cmp(&5)), Err(0));
764:     let b: IndexSet<i32> = [4].into();
765:     assert_eq!(b.binary_search_by(|x| x.cmp(&3)), Err(0));
766:     assert_eq!(b.binary_search_by(|x| x.cmp(&4)), Ok(0));
767:     assert_eq!(b.binary_search_by(|x| x.cmp(&5)), Err(1));
768:     let b: IndexSet<i32> = [1, 2, 4, 6, 8, 9].into();
769:     assert_eq!(b.binary_search_by(|x| x.cmp(&5)), Err(3));
770:     assert_eq!(b.binary_search_by(|x| x.cmp(&6)), Ok(3));
771:     assert_eq!(b.binary_search_by(|x| x.cmp(&7)), Err(4));
772:     assert_eq!(b.binary_search_by(|x| x.cmp(&8)), Ok(4));
773:     let b: IndexSet<i32> = [1, 2, 4, 5, 6, 8].into();
774:     assert_eq!(b.binary_search_by(|x| x.cmp(&9)), Err(6));
775:     let b: IndexSet<i32> = [1, 2, 4, 6, 7, 8, 9].into();
776:     assert_eq!(b.binary_search_by(|x| x.cmp(&6)), Ok(3));
777:     assert_eq!(b.binary_search_by(|x| x.cmp(&5)), Err(3));
778:     assert_eq!(b.binary_search_by(|x| x.cmp(&8)), Ok(5));
779:     let b: IndexSet<i32> = [1, 2, 4, 5, 6, 8, 9].into();
780:     assert_eq!(b.binary_search_by(|x| x.cmp(&7)), Err(5));
781:     assert_eq!(b.binary_search_by(|x| x.cmp(&0)), Err(0));
782:     let b: IndexSet<i32> = [1, 3, 3, 3, 7].into();
783:     assert_eq!(b.binary_search_by(|x| x.cmp(&0)), Err(0));
784:     assert_eq!(b.binary_search_by(|x| x.cmp(&1)), Ok(0));
785:     assert_eq!(b.binary_search_by(|x| x.cmp(&2)), Err(1));
786:     // diff from std as set merges the duplicate keys
787:     assert!(match b.binary_search_by(|x| x.cmp(&3)) {
788:         Ok(1..=2) => true,
789:         _ => false,
790:     });
791:     assert!(match b.binary_search_by(|x| x.cmp(&3)) {
792:         Ok(1..=2) => true,
793:         _ => false,
794:     });
795:     assert_eq!(b.binary_search_by(|x| x.cmp(&4)), Err(2));
796:     assert_eq!(b.binary_search_by(|x| x.cmp(&5)), Err(2));
797:     assert_eq!(b.binary_search_by(|x| x.cmp(&6)), Err(2));
798:     assert_eq!(b.binary_search_by(|x| x.cmp(&7)), Ok(2));
799:     assert_eq!(b.binary_search_by(|x| x.cmp(&8)), Err(3));
800: }
801: #[test]
802: fn test_binary_search_by_key() {
803:     // adapted from std's test for binary_search
804:     let b: IndexSet<i32> = [].into();
805:     assert_eq!(b.binary_search_by_key(&5, |&x| x), Err(0));
806:     let b: IndexSet<i32> = [4].into();
807:     assert_eq!(b.binary_search_by_key(&3, |&x| x), Err(0));
808:     assert_eq!(b.binary_search_by_key(&4, |&x| x), Ok(0));
809:     assert_eq!(b.binary_search_by_key(&5, |&x| x), Err(1));
810:     let b: IndexSet<i32> = [1, 2, 4, 6, 8, 9].into();
811:     assert_eq!(b.binary_search_by_key(&5, |&x| x), Err(3));
812:     assert_eq!(b.binary_search_by_key(&6, |&x| x), Ok(3));
813:     assert_eq!(b.binary_search_by_key(&7, |&x| x), Err(4));
814:     assert_eq!(b.binary_search_by_key(&8, |&x| x), Ok(4));
815:     let b: IndexSet<i32> = [1, 2, 4, 5, 6, 8].into();
816:     assert_eq!(b.binary_search_by_key(&9, |&x| x), Err(6));
817:     let b: IndexSet<i32> = [1, 2, 4, 6, 7, 8, 9].into();
818:     assert_eq!(b.binary_search_by_key(&6, |&x| x), Ok(3));
819:     assert_eq!(b.binary_search_by_key(&5, |&x| x), Err(3));
820:     assert_eq!(b.binary_search_by_key(&8, |&x| x), Ok(5));
821:     let b: IndexSet<i32> = [1, 2, 4, 5, 6, 8, 9].into();
822:     assert_eq!(b.binary_search_by_key(&7, |&x| x), Err(5));
823:     assert_eq!(b.binary_search_by_key(&0, |&x| x), Err(0));
824:     let b: IndexSet<i32> = [1, 3, 3, 3, 7].into();
825:     assert_eq!(b.binary_search_by_key(&0, |&x| x), Err(0));
826:     assert_eq!(b.binary_search_by_key(&1, |&x| x), Ok(0));
827:     assert_eq!(b.binary_search_by_key(&2, |&x| x), Err(1));
828:     // diff from std as set merges the duplicate keys
829:     assert!(match b.binary_search_by_key(&3, |&x| x) {
830:         Ok(1..=2) => true,
831:         _ => false,
832:     });
833:     assert!(match b.binary_search_by_key(&3, |&x| x) {
834:         Ok(1..=2) => true,
835:         _ => false,
836:     });
837:     assert_eq!(b.binary_search_by_key(&4, |&x| x), Err(2));
838:     assert_eq!(b.binary_search_by_key(&5, |&x| x), Err(2));
839:     assert_eq!(b.binary_search_by_key(&6, |&x| x), Err(2));
840:     assert_eq!(b.binary_search_by_key(&7, |&x| x), Ok(2));
841:     assert_eq!(b.binary_search_by_key(&8, |&x| x), Err(3));
842: }
843: #[test]
844: fn test_partition_point() {
845:     // adapted from std's test for partition_point
846:     let b: IndexSet<i32> = [].into();
847:     assert_eq!(b.partition_point(|&x| x < 5), 0);
848:     let b: IndexSet<_> = [4].into();
849:     assert_eq!(b.partition_point(|&x| x < 3), 0);
850:     assert_eq!(b.partition_point(|&x| x < 4), 0);
851:     assert_eq!(b.partition_point(|&x| x < 5), 1);
852:     let b: IndexSet<_> = [1, 2, 4, 6, 8, 9].into();
853:     assert_eq!(b.partition_point(|&x| x < 5), 3);
854:     assert_eq!(b.partition_point(|&x| x < 6), 3);
855:     assert_eq!(b.partition_point(|&x| x < 7), 4);
856:     assert_eq!(b.partition_point(|&x| x < 8), 4);
857:     let b: IndexSet<_> = [1, 2, 4, 5, 6, 8].into();
858:     assert_eq!(b.partition_point(|&x| x < 9), 6);
859:     let b: IndexSet<_> = [1, 2, 4, 6, 7, 8, 9].into();
860:     assert_eq!(b.partition_point(|&x| x < 6), 3);
861:     assert_eq!(b.partition_point(|&x| x < 5), 3);
862:     assert_eq!(b.partition_point(|&x| x < 8), 5);
863:     let b: IndexSet<_> = [1, 2, 4, 5, 6, 8, 9].into();
864:     assert_eq!(b.partition_point(|&x| x < 7), 5);
865:     assert_eq!(b.partition_point(|&x| x < 0), 0);
866:     let b: IndexSet<_> = [1, 3, 3, 3, 7].into();
867:     assert_eq!(b.partition_point(|&x| x < 0), 0);
868:     assert_eq!(b.partition_point(|&x| x < 1), 0);
869:     assert_eq!(b.partition_point(|&x| x < 2), 1);
870:     assert_eq!(b.partition_point(|&x| x < 3), 1);
871:     assert_eq!(b.partition_point(|&x| x < 4), 2); // diff from std as set merges the duplicate keys
872:     assert_eq!(b.partition_point(|&x| x < 5), 2);
873:     assert_eq!(b.partition_point(|&x| x < 6), 2);
874:     assert_eq!(b.partition_point(|&x| x < 7), 2);
875:     assert_eq!(b.partition_point(|&x| x < 8), 3);
876: }
877: #[test]
878: fn is_sorted() {
879:     fn expect(set: &IndexSet<i32>, e: [bool; 4]) {
880:         assert_eq!(e[0], set.is_sorted());
881:         assert_eq!(e[1], set.is_sorted_by(|v1, v2| v1 < v2));
882:         assert_eq!(e[2], set.is_sorted_by(|v1, v2| v1 > v2));
883:         assert_eq!(e[3], set.is_sorted_by_key(|v| v));
884:     }
885:     let mut set = IndexSet::<i32>::from_iter(0..10);
886:     expect(&set, [true, true, false, true]);
887:     set.replace_index(5, -1).unwrap();
888:     expect(&set, [false, false, false, false]);
889: }
890: #[test]
891: fn is_sorted_trivial() {
892:     fn expect(set: &IndexSet<i32>, e: [bool; 5]) {
893:         assert_eq!(e[0], set.is_sorted());
894:         assert_eq!(e[1], set.is_sorted_by(|_, _| true));
895:         assert_eq!(e[2], set.is_sorted_by(|_, _| false));
896:         assert_eq!(e[3], set.is_sorted_by_key(|_| 0f64));
897:         assert_eq!(e[4], set.is_sorted_by_key(|_| f64::NAN));
898:     }
899:     let mut set = IndexSet::<i32>::default();
900:     expect(&set, [true, true, true, true, true]);
901:     set.insert(0);
902:     expect(&set, [true, true, true, true, true]);
903:     set.insert(1);
904:     expect(&set, [true, true, false, true, false]);
905:     set.reverse();
906:     expect(&set, [false, true, false, true, false]);
907: }
````

## File: src/arbitrary.rs
````rust
 1: #[cfg(feature = "arbitrary")]
 2: #[cfg_attr(docsrs, doc(cfg(feature = "arbitrary")))]
 3: mod impl_arbitrary {
 4:     use crate::{IndexMap, IndexSet};
 5:     use arbitrary::{Arbitrary, Result, Unstructured};
 6:     use core::hash::{BuildHasher, Hash};
 7:     impl<'a, K, V, S> Arbitrary<'a> for IndexMap<K, V, S>
 8:     where
 9:         K: Arbitrary<'a> + Hash + Eq,
10:         V: Arbitrary<'a>,
11:         S: BuildHasher + Default,
12:     {
13:         fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
14:             u.arbitrary_iter()?.collect()
15:         }
16:         fn arbitrary_take_rest(u: Unstructured<'a>) -> Result<Self> {
17:             u.arbitrary_take_rest_iter()?.collect()
18:         }
19:     }
20:     impl<'a, T, S> Arbitrary<'a> for IndexSet<T, S>
21:     where
22:         T: Arbitrary<'a> + Hash + Eq,
23:         S: BuildHasher + Default,
24:     {
25:         fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
26:             u.arbitrary_iter()?.collect()
27:         }
28:         fn arbitrary_take_rest(u: Unstructured<'a>) -> Result<Self> {
29:             u.arbitrary_take_rest_iter()?.collect()
30:         }
31:     }
32: }
33: #[cfg(feature = "quickcheck")]
34: #[cfg_attr(docsrs, doc(cfg(feature = "quickcheck")))]
35: mod impl_quickcheck {
36:     use crate::{IndexMap, IndexSet};
37:     use alloc::boxed::Box;
38:     use alloc::vec::Vec;
39:     use core::hash::{BuildHasher, Hash};
40:     use quickcheck::{Arbitrary, Gen};
41:     impl<K, V, S> Arbitrary for IndexMap<K, V, S>
42:     where
43:         K: Arbitrary + Hash + Eq,
44:         V: Arbitrary,
45:         S: BuildHasher + Default + Clone + 'static,
46:     {
47:         fn arbitrary(g: &mut Gen) -> Self {
48:             Self::from_iter(Vec::arbitrary(g))
49:         }
50:         fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
51:             let vec = Vec::from_iter(self.clone());
52:             Box::new(vec.shrink().map(Self::from_iter))
53:         }
54:     }
55:     impl<T, S> Arbitrary for IndexSet<T, S>
56:     where
57:         T: Arbitrary + Hash + Eq,
58:         S: BuildHasher + Default + Clone + 'static,
59:     {
60:         fn arbitrary(g: &mut Gen) -> Self {
61:             Self::from_iter(Vec::arbitrary(g))
62:         }
63:         fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
64:             let vec = Vec::from_iter(self.clone());
65:             Box::new(vec.shrink().map(Self::from_iter))
66:         }
67:     }
68: }
````

## File: src/borsh.rs
````rust
  1: #![cfg_attr(docsrs, doc(cfg(feature = "borsh")))]
  2: use alloc::vec::Vec;
  3: use core::hash::BuildHasher;
  4: use core::hash::Hash;
  5: use borsh::error::ERROR_ZST_FORBIDDEN;
  6: use borsh::io::{Error, ErrorKind, Read, Result, Write};
  7: use borsh::{BorshDeserialize, BorshSerialize};
  8: use crate::map::IndexMap;
  9: use crate::set::IndexSet;
 10: // NOTE: the real `#[deprecated]` attribute doesn't work for trait implementations,
 11: // but we can get close by mimicking the message style for documentation.
 12: /// <div class="stab deprecated"><span class="emoji">👎</span><span>Deprecated: use borsh's <code>indexmap</code> feature instead.</span></div>
 13: impl<K, V, S> BorshSerialize for IndexMap<K, V, S>
 14: where
 15:     K: BorshSerialize,
 16:     V: BorshSerialize,
 17: {
 18:     #[inline]
 19:     fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
 20:         check_zst::<K>()?;
 21:         let iterator = self.iter();
 22:         u32::try_from(iterator.len())
 23:             .map_err(|_| ErrorKind::InvalidData)?
 24:             .serialize(writer)?;
 25:         for (key, value) in iterator {
 26:             key.serialize(writer)?;
 27:             value.serialize(writer)?;
 28:         }
 29:         Ok(())
 30:     }
 31: }
 32: /// <div class="stab deprecated"><span class="emoji">👎</span><span>Deprecated: use borsh's <code>indexmap</code> feature instead.</span></div>
 33: impl<K, V, S> BorshDeserialize for IndexMap<K, V, S>
 34: where
 35:     K: BorshDeserialize + Eq + Hash,
 36:     V: BorshDeserialize,
 37:     S: BuildHasher + Default,
 38: {
 39:     #[inline]
 40:     fn deserialize_reader<R: Read>(reader: &mut R) -> Result<Self> {
 41:         check_zst::<K>()?;
 42:         let vec = <Vec<(K, V)>>::deserialize_reader(reader)?;
 43:         Ok(vec.into_iter().collect::<IndexMap<K, V, S>>())
 44:     }
 45: }
 46: /// <div class="stab deprecated"><span class="emoji">👎</span><span>Deprecated: use borsh's <code>indexmap</code> feature instead.</span></div>
 47: impl<T, S> BorshSerialize for IndexSet<T, S>
 48: where
 49:     T: BorshSerialize,
 50: {
 51:     #[inline]
 52:     fn serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
 53:         check_zst::<T>()?;
 54:         let iterator = self.iter();
 55:         u32::try_from(iterator.len())
 56:             .map_err(|_| ErrorKind::InvalidData)?
 57:             .serialize(writer)?;
 58:         for item in iterator {
 59:             item.serialize(writer)?;
 60:         }
 61:         Ok(())
 62:     }
 63: }
 64: /// <div class="stab deprecated"><span class="emoji">👎</span><span>Deprecated: use borsh's <code>indexmap</code> feature instead.</span></div>
 65: impl<T, S> BorshDeserialize for IndexSet<T, S>
 66: where
 67:     T: BorshDeserialize + Eq + Hash,
 68:     S: BuildHasher + Default,
 69: {
 70:     #[inline]
 71:     fn deserialize_reader<R: Read>(reader: &mut R) -> Result<Self> {
 72:         check_zst::<T>()?;
 73:         let vec = <Vec<T>>::deserialize_reader(reader)?;
 74:         Ok(vec.into_iter().collect::<IndexSet<T, S>>())
 75:     }
 76: }
 77: fn check_zst<T>() -> Result<()> {
 78:     if size_of::<T>() == 0 {
 79:         return Err(Error::new(ErrorKind::InvalidData, ERROR_ZST_FORBIDDEN));
 80:     }
 81:     Ok(())
 82: }
 83: #[cfg(test)]
 84: mod borsh_tests {
 85:     use super::*;
 86:     #[test]
 87:     fn map_borsh_roundtrip() {
 88:         let original_map: IndexMap<i32, i32> = {
 89:             let mut map = IndexMap::new();
 90:             map.insert(1, 2);
 91:             map.insert(3, 4);
 92:             map.insert(5, 6);
 93:             map
 94:         };
 95:         let serialized_map = borsh::to_vec(&original_map).unwrap();
 96:         let deserialized_map: IndexMap<i32, i32> =
 97:             BorshDeserialize::try_from_slice(&serialized_map).unwrap();
 98:         assert_eq!(original_map, deserialized_map);
 99:     }
100:     #[test]
101:     fn set_borsh_roundtrip() {
102:         let original_map: IndexSet<i32> = [1, 2, 3, 4, 5, 6].into_iter().collect();
103:         let serialized_map = borsh::to_vec(&original_map).unwrap();
104:         let deserialized_map: IndexSet<i32> =
105:             BorshDeserialize::try_from_slice(&serialized_map).unwrap();
106:         assert_eq!(original_map, deserialized_map);
107:     }
108: }
````

## File: src/inner.rs
````rust
  1: //! This is the core implementation that doesn't depend on the hasher at all.
  2: //!
  3: //! The methods of `Core` don't use any Hash properties of K.
  4: //!
  5: //! It's cleaner to separate them out, then the compiler checks that we are not
  6: //! using Hash at all in these methods.
  7: //!
  8: //! However, we should probably not let this show in the public API or docs.
  9: mod entry;
 10: mod extract;
 11: use alloc::vec::{self, Vec};
 12: use core::mem;
 13: use core::ops::RangeBounds;
 14: use hashbrown::hash_table;
 15: use crate::util::{assert_index_le, assert_index_lt, simplify_range};
 16: use crate::{Bucket, Equivalent, HashValue, TryReserveError};
 17: type Indices = hash_table::HashTable<usize>;
 18: type Entries<K, V> = Vec<Bucket<K, V>>;
 19: pub use entry::{OccupiedEntry, VacantEntry};
 20: pub(crate) use extract::ExtractCore;
 21: /// Core of the map that does not depend on S
 22: #[cfg_attr(feature = "test_debug", derive(Debug))]
 23: pub(crate) struct Core<K, V> {
 24:     /// indices mapping from the entry hash to its index.
 25:     indices: Indices,
 26:     /// entries is a dense vec maintaining entry order.
 27:     entries: Entries<K, V>,
 28: }
 29: #[inline(always)]
 30: fn get_hash<K, V>(entries: &[Bucket<K, V>]) -> impl Fn(&usize) -> u64 + use<'_, K, V> {
 31:     move |&i| entries[i].hash.get()
 32: }
 33: #[inline]
 34: fn equal<'a, K: Eq, V>(
 35:     key: &'a K,
 36:     entries: &'a [Bucket<K, V>],
 37: ) -> impl Fn(&usize) -> bool + use<'a, K, V> {
 38:     move |&i| K::eq(key, &entries[i].key)
 39: }
 40: #[inline]
 41: fn equivalent<'a, K, V, Q: ?Sized + Equivalent<K>>(
 42:     key: &'a Q,
 43:     entries: &'a [Bucket<K, V>],
 44: ) -> impl Fn(&usize) -> bool + use<'a, K, V, Q> {
 45:     move |&i| Q::equivalent(key, &entries[i].key)
 46: }
 47: #[inline]
 48: fn erase_index(table: &mut Indices, hash: HashValue, index: usize) {
 49:     if let Ok(entry) = table.find_entry(hash.get(), move |&i| i == index) {
 50:         entry.remove();
 51:     } else if cfg!(debug_assertions) {
 52:         panic!("index not found");
 53:     }
 54: }
 55: #[inline]
 56: fn update_index(table: &mut Indices, hash: HashValue, old: usize, new: usize) {
 57:     let index = table
 58:         .find_mut(hash.get(), move |&i| i == old)
 59:         .expect("index not found");
 60:     *index = new;
 61: }
 62: /// Inserts many entries into the indices table without reallocating,
 63: /// and without regard for duplication.
 64: ///
 65: /// ***Panics*** if there is not sufficient capacity already.
 66: fn insert_bulk_no_grow<K, V>(indices: &mut Indices, entries: &[Bucket<K, V>]) {
 67:     assert!(indices.capacity() - indices.len() >= entries.len());
 68:     for entry in entries {
 69:         indices.insert_unique(entry.hash.get(), indices.len(), |_| unreachable!());
 70:     }
 71: }
 72: impl<K, V> Clone for Core<K, V>
 73: where
 74:     K: Clone,
 75:     V: Clone,
 76: {
 77:     fn clone(&self) -> Self {
 78:         let mut new = Self::new();
 79:         new.clone_from(self);
 80:         new
 81:     }
 82:     fn clone_from(&mut self, other: &Self) {
 83:         self.indices.clone_from(&other.indices);
 84:         if self.entries.capacity() < other.entries.len() {
 85:             // If we must resize, match the indices capacity.
 86:             let additional = other.entries.len() - self.entries.len();
 87:             self.reserve_entries(additional);
 88:         }
 89:         self.entries.clone_from(&other.entries);
 90:     }
 91: }
 92: impl<K, V> Core<K, V> {
 93:     /// The maximum capacity before the `entries` allocation would exceed `isize::MAX`.
 94:     const MAX_ENTRIES_CAPACITY: usize = (isize::MAX as usize) / size_of::<Bucket<K, V>>();
 95:     #[inline]
 96:     pub(crate) const fn new() -> Self {
 97:         Core {
 98:             indices: Indices::new(),
 99:             entries: Vec::new(),
100:         }
101:     }
102:     #[inline]
103:     pub(crate) fn with_capacity(n: usize) -> Self {
104:         Core {
105:             indices: Indices::with_capacity(n),
106:             entries: Vec::with_capacity(n),
107:         }
108:     }
109:     #[inline]
110:     pub(crate) fn into_entries(self) -> Entries<K, V> {
111:         self.entries
112:     }
113:     #[inline]
114:     pub(crate) fn as_entries(&self) -> &[Bucket<K, V>] {
115:         &self.entries
116:     }
117:     #[inline]
118:     pub(crate) fn as_entries_mut(&mut self) -> &mut [Bucket<K, V>] {
119:         &mut self.entries
120:     }
121:     pub(crate) fn with_entries<F>(&mut self, f: F)
122:     where
123:         F: FnOnce(&mut [Bucket<K, V>]),
124:     {
125:         f(&mut self.entries);
126:         self.rebuild_hash_table();
127:     }
128:     #[inline]
129:     pub(crate) fn len(&self) -> usize {
130:         debug_assert_eq!(self.entries.len(), self.indices.len());
131:         self.indices.len()
132:     }
133:     #[inline]
134:     pub(crate) fn capacity(&self) -> usize {
135:         Ord::min(self.indices.capacity(), self.entries.capacity())
136:     }
137:     pub(crate) fn clear(&mut self) {
138:         self.indices.clear();
139:         self.entries.clear();
140:     }
141:     pub(crate) fn truncate(&mut self, len: usize) {
142:         if len < self.len() {
143:             self.erase_indices(len, self.entries.len());
144:             self.entries.truncate(len);
145:         }
146:     }
147:     #[track_caller]
148:     pub(crate) fn drain<R>(&mut self, range: R) -> vec::Drain<'_, Bucket<K, V>>
149:     where
150:         R: RangeBounds<usize>,
151:     {
152:         let range = simplify_range(range, self.entries.len());
153:         self.erase_indices(range.start, range.end);
154:         self.entries.drain(range)
155:     }
156:     #[cfg(feature = "rayon")]
157:     pub(crate) fn par_drain<R>(&mut self, range: R) -> rayon::vec::Drain<'_, Bucket<K, V>>
158:     where
159:         K: Send,
160:         V: Send,
161:         R: RangeBounds<usize>,
162:     {
163:         use rayon::iter::ParallelDrainRange;
164:         let range = simplify_range(range, self.entries.len());
165:         self.erase_indices(range.start, range.end);
166:         self.entries.par_drain(range)
167:     }
168:     #[track_caller]
169:     pub(crate) fn split_off(&mut self, at: usize) -> Self {
170:         assert_index_le(at, self.len());
171:         self.erase_indices(at, self.entries.len());
172:         let entries = self.entries.split_off(at);
173:         let mut indices = Indices::with_capacity(entries.len());
174:         insert_bulk_no_grow(&mut indices, &entries);
175:         Self { indices, entries }
176:     }
177:     #[track_caller]
178:     pub(crate) fn split_splice<R>(&mut self, range: R) -> (Self, vec::IntoIter<Bucket<K, V>>)
179:     where
180:         R: RangeBounds<usize>,
181:     {
182:         let range = simplify_range(range, self.len());
183:         self.erase_indices(range.start, self.entries.len());
184:         let entries = self.entries.split_off(range.end);
185:         let drained = self.entries.split_off(range.start);
186:         let mut indices = Indices::with_capacity(entries.len());
187:         insert_bulk_no_grow(&mut indices, &entries);
188:         (Self { indices, entries }, drained.into_iter())
189:     }
190:     /// Append from another map without checking whether items already exist.
191:     pub(crate) fn append_unchecked(&mut self, other: &mut Self) {
192:         self.reserve(other.len());
193:         insert_bulk_no_grow(&mut self.indices, &other.entries);
194:         self.entries.append(&mut other.entries);
195:         other.indices.clear();
196:     }
197:     /// Reserve capacity for `additional` more key-value pairs.
198:     pub(crate) fn reserve(&mut self, additional: usize) {
199:         self.indices.reserve(additional, get_hash(&self.entries));
200:         // Only grow entries if necessary, since we also round up capacity.
201:         if additional > self.entries.capacity() - self.entries.len() {
202:             self.reserve_entries(additional);
203:         }
204:     }
205:     /// Reserve capacity for `additional` more key-value pairs, without over-allocating.
206:     pub(crate) fn reserve_exact(&mut self, additional: usize) {
207:         self.indices.reserve(additional, get_hash(&self.entries));
208:         self.entries.reserve_exact(additional);
209:     }
210:     /// Try to reserve capacity for `additional` more key-value pairs.
211:     pub(crate) fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
212:         self.indices
213:             .try_reserve(additional, get_hash(&self.entries))
214:             .map_err(TryReserveError::from_hashbrown)?;
215:         // Only grow entries if necessary, since we also round up capacity.
216:         if additional > self.entries.capacity() - self.entries.len() {
217:             self.try_reserve_entries(additional)
218:         } else {
219:             Ok(())
220:         }
221:     }
222:     /// Try to reserve entries capacity, rounded up to match the indices
223:     fn try_reserve_entries(&mut self, additional: usize) -> Result<(), TryReserveError> {
224:         // Use a soft-limit on the maximum capacity, but if the caller explicitly
225:         // requested more, do it and let them have the resulting error.
226:         let new_capacity = Ord::min(self.indices.capacity(), Self::MAX_ENTRIES_CAPACITY);
227:         let try_add = new_capacity - self.entries.len();
228:         if try_add > additional && self.entries.try_reserve_exact(try_add).is_ok() {
229:             return Ok(());
230:         }
231:         self.entries
232:             .try_reserve_exact(additional)
233:             .map_err(TryReserveError::from_alloc)
234:     }
235:     /// Try to reserve capacity for `additional` more key-value pairs, without over-allocating.
236:     pub(crate) fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
237:         self.indices
238:             .try_reserve(additional, get_hash(&self.entries))
239:             .map_err(TryReserveError::from_hashbrown)?;
240:         self.entries
241:             .try_reserve_exact(additional)
242:             .map_err(TryReserveError::from_alloc)
243:     }
244:     /// Shrink the capacity of the map with a lower bound
245:     pub(crate) fn shrink_to(&mut self, min_capacity: usize) {
246:         self.indices
247:             .shrink_to(min_capacity, get_hash(&self.entries));
248:         self.entries.shrink_to(min_capacity);
249:     }
250:     /// Remove the last key-value pair
251:     pub(crate) fn pop(&mut self) -> Option<(K, V)> {
252:         if let Some(entry) = self.entries.pop() {
253:             let last = self.entries.len();
254:             erase_index(&mut self.indices, entry.hash, last);
255:             Some((entry.key, entry.value))
256:         } else {
257:             None
258:         }
259:     }
260:     /// Return the index in `entries` where an equivalent key can be found
261:     pub(crate) fn get_index_of<Q>(&self, hash: HashValue, key: &Q) -> Option<usize>
262:     where
263:         Q: ?Sized + Equivalent<K>,
264:     {
265:         let eq = equivalent(key, &self.entries);
266:         self.indices.find(hash.get(), eq).copied()
267:     }
268:     /// Return the index in `entries` where an equivalent key can be found
269:     pub(crate) fn get_index_of_raw<F>(&self, hash: HashValue, mut is_match: F) -> Option<usize>
270:     where
271:         F: FnMut(&K) -> bool,
272:     {
273:         let eq = move |&i: &usize| is_match(&self.entries[i].key);
274:         self.indices.find(hash.get(), eq).copied()
275:     }
276:     /// Append a key-value pair to `entries`,
277:     /// *without* checking whether it already exists.
278:     fn push_entry(&mut self, hash: HashValue, key: K, value: V) {
279:         if self.entries.len() == self.entries.capacity() {
280:             // Reserve our own capacity synced to the indices,
281:             // rather than letting `Vec::push` just double it.
282:             self.reserve_entries(1);
283:         }
284:         self.entries.push(Bucket { hash, key, value });
285:     }
286:     pub(crate) fn insert_full(&mut self, hash: HashValue, key: K, value: V) -> (usize, Option<V>)
287:     where
288:         K: Eq,
289:     {
290:         let eq = equal(&key, &self.entries);
291:         let hasher = get_hash(&self.entries);
292:         match self.indices.entry(hash.get(), eq, hasher) {
293:             hash_table::Entry::Occupied(entry) => {
294:                 let i = *entry.get();
295:                 (i, Some(mem::replace(&mut self.entries[i].value, value)))
296:             }
297:             hash_table::Entry::Vacant(entry) => {
298:                 let i = self.entries.len();
299:                 entry.insert(i);
300:                 self.push_entry(hash, key, value);
301:                 debug_assert_eq!(self.indices.len(), self.entries.len());
302:                 (i, None)
303:             }
304:         }
305:     }
306:     /// Same as `insert_full`, except it also replaces the key
307:     pub(crate) fn replace_full(
308:         &mut self,
309:         hash: HashValue,
310:         key: K,
311:         value: V,
312:     ) -> (usize, Option<(K, V)>)
313:     where
314:         K: Eq,
315:     {
316:         let eq = equal(&key, &self.entries);
317:         let hasher = get_hash(&self.entries);
318:         match self.indices.entry(hash.get(), eq, hasher) {
319:             hash_table::Entry::Occupied(entry) => {
320:                 let i = *entry.get();
321:                 let entry = &mut self.entries[i];
322:                 let kv = (
323:                     mem::replace(&mut entry.key, key),
324:                     mem::replace(&mut entry.value, value),
325:                 );
326:                 (i, Some(kv))
327:             }
328:             hash_table::Entry::Vacant(entry) => {
329:                 let i = self.entries.len();
330:                 entry.insert(i);
331:                 self.push_entry(hash, key, value);
332:                 debug_assert_eq!(self.indices.len(), self.entries.len());
333:                 (i, None)
334:             }
335:         }
336:     }
337:     /// Remove an entry by shifting all entries that follow it
338:     pub(crate) fn shift_remove_full<Q>(&mut self, hash: HashValue, key: &Q) -> Option<(usize, K, V)>
339:     where
340:         Q: ?Sized + Equivalent<K>,
341:     {
342:         let eq = equivalent(key, &self.entries);
343:         let (index, _) = self.indices.find_entry(hash.get(), eq).ok()?.remove();
344:         let (key, value) = self.shift_remove_finish(index);
345:         Some((index, key, value))
346:     }
347:     /// Remove an entry by swapping it with the last
348:     pub(crate) fn swap_remove_full<Q>(&mut self, hash: HashValue, key: &Q) -> Option<(usize, K, V)>
349:     where
350:         Q: ?Sized + Equivalent<K>,
351:     {
352:         let eq = equivalent(key, &self.entries);
353:         let (index, _) = self.indices.find_entry(hash.get(), eq).ok()?.remove();
354:         let (key, value) = self.swap_remove_finish(index);
355:         Some((index, key, value))
356:     }
357:     /// Erase `start..end` from `indices`, and shift `end..` indices down to `start..`
358:     ///
359:     /// All of these items should still be at their original location in `entries`.
360:     /// This is used by `drain`, which will let `Vec::drain` do the work on `entries`.
361:     fn erase_indices(&mut self, start: usize, end: usize) {
362:         let (init, shifted_entries) = self.entries.split_at(end);
363:         let (start_entries, erased_entries) = init.split_at(start);
364:         let erased = erased_entries.len();
365:         let shifted = shifted_entries.len();
366:         let half_capacity = self.indices.capacity() / 2;
367:         // Use a heuristic between different strategies
368:         if erased == 0 {
369:             // Degenerate case, nothing to do
370:         } else if start + shifted < half_capacity && start < erased {
371:             // Reinsert everything, as there are few kept indices
372:             self.indices.clear();
373:             // Reinsert stable indices, then shifted indices
374:             insert_bulk_no_grow(&mut self.indices, start_entries);
375:             insert_bulk_no_grow(&mut self.indices, shifted_entries);
376:         } else if erased + shifted < half_capacity {
377:             // Find each affected index, as there are few to adjust
378:             // Find erased indices
379:             for (i, entry) in (start..).zip(erased_entries) {
380:                 erase_index(&mut self.indices, entry.hash, i);
381:             }
382:             // Find shifted indices
383:             for ((new, old), entry) in (start..).zip(end..).zip(shifted_entries) {
384:                 update_index(&mut self.indices, entry.hash, old, new);
385:             }
386:         } else {
387:             // Sweep the whole table for adjustments
388:             let offset = end - start;
389:             self.indices.retain(move |i| {
390:                 if *i >= end {
391:                     *i -= offset;
392:                     true
393:                 } else {
394:                     *i < start
395:                 }
396:             });
397:         }
398:         debug_assert_eq!(self.indices.len(), start + shifted);
399:     }
400:     pub(crate) fn retain_in_order<F>(&mut self, mut keep: F)
401:     where
402:         F: FnMut(&mut K, &mut V) -> bool,
403:     {
404:         self.entries
405:             .retain_mut(|entry| keep(&mut entry.key, &mut entry.value));
406:         if self.entries.len() < self.indices.len() {
407:             self.rebuild_hash_table();
408:         }
409:     }
410:     fn rebuild_hash_table(&mut self) {
411:         self.indices.clear();
412:         insert_bulk_no_grow(&mut self.indices, &self.entries);
413:     }
414:     pub(crate) fn reverse(&mut self) {
415:         self.entries.reverse();
416:         // No need to save hash indices, can easily calculate what they should
417:         // be, given that this is an in-place reversal.
418:         let len = self.entries.len();
419:         for i in &mut self.indices {
420:             *i = len - *i - 1;
421:         }
422:     }
423:     /// Reserve entries capacity, rounded up to match the indices
424:     #[inline]
425:     fn reserve_entries(&mut self, additional: usize) {
426:         // Use a soft-limit on the maximum capacity, but if the caller explicitly
427:         // requested more, do it and let them have the resulting panic.
428:         let try_capacity = Ord::min(self.indices.capacity(), Self::MAX_ENTRIES_CAPACITY);
429:         let try_add = try_capacity - self.entries.len();
430:         if try_add > additional && self.entries.try_reserve_exact(try_add).is_ok() {
431:             return;
432:         }
433:         self.entries.reserve_exact(additional);
434:     }
435:     /// Insert a key-value pair in `entries`,
436:     /// *without* checking whether it already exists.
437:     pub(super) fn insert_unique(&mut self, hash: HashValue, key: K, value: V) -> &mut Bucket<K, V> {
438:         let i = self.indices.len();
439:         debug_assert_eq!(i, self.entries.len());
440:         self.indices
441:             .insert_unique(hash.get(), i, get_hash(&self.entries));
442:         self.push_entry(hash, key, value);
443:         &mut self.entries[i]
444:     }
445:     /// Replaces the key at the given index,
446:     /// *without* checking whether it already exists.
447:     #[track_caller]
448:     pub(crate) fn replace_index_unique(&mut self, index: usize, hash: HashValue, key: K) -> K {
449:         // NB: This removal and insertion isn't "no grow" (with unreachable hasher)
450:         // because hashbrown's tombstones might force a resize anyway.
451:         erase_index(&mut self.indices, self.entries[index].hash, index);
452:         self.indices
453:             .insert_unique(hash.get(), index, get_hash(&self.entries));
454:         let entry = &mut self.entries[index];
455:         entry.hash = hash;
456:         mem::replace(&mut entry.key, key)
457:     }
458:     /// Insert a key-value pair in `entries` at a particular index,
459:     /// *without* checking whether it already exists.
460:     pub(crate) fn shift_insert_unique(
461:         &mut self,
462:         index: usize,
463:         hash: HashValue,
464:         key: K,
465:         value: V,
466:     ) -> &mut Bucket<K, V> {
467:         let end = self.indices.len();
468:         assert!(index <= end);
469:         // Increment others first so we don't have duplicate indices.
470:         self.increment_indices(index, end);
471:         let entries = &*self.entries;
472:         self.indices.insert_unique(hash.get(), index, move |&i| {
473:             // Adjust for the incremented indices to find hashes.
474:             debug_assert_ne!(i, index);
475:             let i = if i < index { i } else { i - 1 };
476:             entries[i].hash.get()
477:         });
478:         if self.entries.len() == self.entries.capacity() {
479:             // Reserve our own capacity synced to the indices,
480:             // rather than letting `Vec::insert` just double it.
481:             self.reserve_entries(1);
482:         }
483:         self.entries.insert(index, Bucket { hash, key, value });
484:         &mut self.entries[index]
485:     }
486:     /// Remove an entry by shifting all entries that follow it
487:     pub(crate) fn shift_remove_index(&mut self, index: usize) -> Option<(K, V)> {
488:         match self.entries.get(index) {
489:             Some(entry) => {
490:                 erase_index(&mut self.indices, entry.hash, index);
491:                 Some(self.shift_remove_finish(index))
492:             }
493:             None => None,
494:         }
495:     }
496:     /// Remove an entry by shifting all entries that follow it
497:     ///
498:     /// The index should already be removed from `self.indices`.
499:     fn shift_remove_finish(&mut self, index: usize) -> (K, V) {
500:         // Correct indices that point to the entries that followed the removed entry.
501:         self.decrement_indices(index + 1, self.entries.len());
502:         // Use Vec::remove to actually remove the entry.
503:         let entry = self.entries.remove(index);
504:         (entry.key, entry.value)
505:     }
506:     /// Remove an entry by swapping it with the last
507:     pub(crate) fn swap_remove_index(&mut self, index: usize) -> Option<(K, V)> {
508:         match self.entries.get(index) {
509:             Some(entry) => {
510:                 erase_index(&mut self.indices, entry.hash, index);
511:                 Some(self.swap_remove_finish(index))
512:             }
513:             None => None,
514:         }
515:     }
516:     /// Finish removing an entry by swapping it with the last
517:     ///
518:     /// The index should already be removed from `self.indices`.
519:     fn swap_remove_finish(&mut self, index: usize) -> (K, V) {
520:         // use swap_remove, but then we need to update the index that points
521:         // to the other entry that has to move
522:         let entry = self.entries.swap_remove(index);
523:         // correct index that points to the entry that had to swap places
524:         if let Some(entry) = self.entries.get(index) {
525:             // was not last element
526:             // examine new element in `index` and find it in indices
527:             let last = self.entries.len();
528:             update_index(&mut self.indices, entry.hash, last, index);
529:         }
530:         (entry.key, entry.value)
531:     }
532:     /// Decrement all indices in the range `start..end`.
533:     ///
534:     /// The index `start - 1` should not exist in `self.indices`.
535:     /// All entries should still be in their original positions.
536:     fn decrement_indices(&mut self, start: usize, end: usize) {
537:         // Use a heuristic between a full sweep vs. a `find()` for every shifted item.
538:         let shifted_entries = &self.entries[start..end];
539:         if shifted_entries.len() > self.indices.capacity() / 2 {
540:             // Shift all indices in range.
541:             for i in &mut self.indices {
542:                 if start <= *i && *i < end {
543:                     *i -= 1;
544:                 }
545:             }
546:         } else {
547:             // Find each entry in range to shift its index.
548:             for (i, entry) in (start..end).zip(shifted_entries) {
549:                 update_index(&mut self.indices, entry.hash, i, i - 1);
550:             }
551:         }
552:     }
553:     /// Increment all indices in the range `start..end`.
554:     ///
555:     /// The index `end` should not exist in `self.indices`.
556:     /// All entries should still be in their original positions.
557:     fn increment_indices(&mut self, start: usize, end: usize) {
558:         // Use a heuristic between a full sweep vs. a `find()` for every shifted item.
559:         let shifted_entries = &self.entries[start..end];
560:         if shifted_entries.len() > self.indices.capacity() / 2 {
561:             // Shift all indices in range.
562:             for i in &mut self.indices {
563:                 if start <= *i && *i < end {
564:                     *i += 1;
565:                 }
566:             }
567:         } else {
568:             // Find each entry in range to shift its index, updated in reverse so
569:             // we never have duplicated indices that might have a hash collision.
570:             for (i, entry) in (start..end).zip(shifted_entries).rev() {
571:                 update_index(&mut self.indices, entry.hash, i, i + 1);
572:             }
573:         }
574:     }
575:     #[track_caller]
576:     pub(super) fn move_index(&mut self, from: usize, to: usize) {
577:         assert_index_lt(from, self.len());
578:         let from_hash = self.entries[from].hash;
579:         if from != to {
580:             assert_index_lt(to, self.len());
581:             // Find the bucket index first so we won't lose it among other updated indices.
582:             let bucket = self
583:                 .indices
584:                 .find_bucket_index(from_hash.get(), move |&i| i == from)
585:                 .expect("index not found");
586:             self.move_index_inner(from, to);
587:             *self.indices.get_bucket_mut(bucket).unwrap() = to;
588:         }
589:     }
590:     fn move_index_inner(&mut self, from: usize, to: usize) {
591:         // Update all other indices and rotate the entry positions.
592:         if from < to {
593:             self.decrement_indices(from + 1, to + 1);
594:             self.entries[from..=to].rotate_left(1);
595:         } else if to < from {
596:             self.increment_indices(to, from);
597:             self.entries[to..=from].rotate_right(1);
598:         }
599:     }
600:     #[track_caller]
601:     pub(crate) fn swap_indices(&mut self, a: usize, b: usize) {
602:         assert_index_lt(a, self.len());
603:         if a == b {
604:             // If they're equal, there's nothing to do.
605:             return;
606:         }
607:         assert_index_lt(b, self.len());
608:         // Since the indices are in-bounds, we expect to find them in the table as well.
609:         match self.indices.get_disjoint_mut(
610:             [self.entries[a].hash.get(), self.entries[b].hash.get()],
611:             move |i, &x| if i == 0 { x == a } else { x == b },
612:         ) {
613:             [Some(ref_a), Some(ref_b)] => {
614:                 mem::swap(ref_a, ref_b);
615:                 self.entries.swap(a, b);
616:             }
617:             _ => panic!("indices not found"),
618:         }
619:     }
620: }
621: #[test]
622: fn assert_send_sync() {
623:     fn assert_send_sync<T: Send + Sync>() {}
624:     assert_send_sync::<Core<i32, i32>>();
625: }
````

## File: src/lib.rs
````rust
  1: #![no_std]
  2: //! [`IndexMap`] is a hash table where the iteration order of the key-value
  3: //! pairs is independent of the hash values of the keys.
  4: //!
  5: //! [`IndexSet`] is a corresponding hash set using the same implementation and
  6: //! with similar properties.
  7: //!
  8: //! ### Highlights
  9: //!
 10: //! [`IndexMap`] and [`IndexSet`] are drop-in compatible with the std `HashMap`
 11: //! and `HashSet`, but they also have some features of note:
 12: //!
 13: //! - The ordering semantics (see their documentation for details)
 14: //! - Sorting methods and the [`.pop()`][IndexMap::pop] methods.
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
 31: //!   to [`IndexMap`] and [`IndexSet`]. Alternative implementations for
 32: //!   (de)serializing [`IndexMap`] as an ordered sequence are available in the
 33: //!   [`map::serde_seq`] module.
 34: //! * `arbitrary`: Adds implementations for the [`arbitrary::Arbitrary`] trait
 35: //!   to [`IndexMap`] and [`IndexSet`].
 36: //! * `quickcheck`: Adds implementations for the [`quickcheck::Arbitrary`] trait
 37: //!   to [`IndexMap`] and [`IndexSet`].
 38: //! * `borsh` (**deprecated**): Adds implementations for [`BorshSerialize`] and
 39: //!   [`BorshDeserialize`] to [`IndexMap`] and [`IndexSet`]. Due to a cyclic
 40: //!   dependency that arose between [`borsh`] and `indexmap`, `borsh v1.5.6`
 41: //!   added an `indexmap` feature that should be used instead of enabling the
 42: //!   feature here.
 43: //!
 44: //! _Note: only the `std` feature is enabled by default._
 45: //!
 46: //! [feature flags]: https://doc.rust-lang.org/cargo/reference/manifest.html#the-features-section
 47: //! [`no_std`]: #no-standard-library-targets
 48: //! [`Serialize`]: `::serde_core::Serialize`
 49: //! [`Deserialize`]: `::serde_core::Deserialize`
 50: //! [`BorshSerialize`]: `::borsh::BorshSerialize`
 51: //! [`BorshDeserialize`]: `::borsh::BorshDeserialize`
 52: //! [`borsh`]: `::borsh`
 53: //! [`arbitrary::Arbitrary`]: `::arbitrary::Arbitrary`
 54: //! [`quickcheck::Arbitrary`]: `::quickcheck::Arbitrary`
 55: //!
 56: //! ### Alternate Hashers
 57: //!
 58: //! [`IndexMap`] and [`IndexSet`] have a default hasher type
 59: //! [`S = RandomState`][std::hash::RandomState],
 60: //! just like the standard `HashMap` and `HashSet`, which is resistant to
 61: //! HashDoS attacks but not the most performant. Type aliases can make it easier
 62: //! to use alternate hashers:
 63: //!
 64: //! ```
 65: //! use fnv::FnvBuildHasher;
 66: //! use indexmap::{IndexMap, IndexSet};
 67: //!
 68: //! type FnvIndexMap<K, V> = IndexMap<K, V, FnvBuildHasher>;
 69: //! type FnvIndexSet<T> = IndexSet<T, FnvBuildHasher>;
 70: //!
 71: //! let std: IndexSet<i32> = (0..100).collect();
 72: //! let fnv: FnvIndexSet<i32> = (0..100).collect();
 73: //! assert_eq!(std, fnv);
 74: //! ```
 75: //!
 76: //! ### Rust Version
 77: //!
 78: //! This version of indexmap requires Rust 1.85 or later.
 79: //!
 80: //! The indexmap 2.x release series will use a carefully considered version
 81: //! upgrade policy, where in a later 2.x version, we will raise the minimum
 82: //! required Rust version.
 83: //!
 84: //! ## No Standard Library Targets
 85: //!
 86: //! This crate supports being built without `std`, requiring `alloc` instead.
 87: //! This is chosen by disabling the default "std" cargo feature, by adding
 88: //! `default-features = false` to your dependency specification.
 89: //!
 90: //! - Creating maps and sets using [`new`][IndexMap::new] and
 91: //!   [`with_capacity`][IndexMap::with_capacity] is unavailable without `std`.
 92: //!   Use methods [`IndexMap::default`], [`with_hasher`][IndexMap::with_hasher],
 93: //!   [`with_capacity_and_hasher`][IndexMap::with_capacity_and_hasher] instead.
 94: //!   A no-std compatible hasher will be needed as well, for example
 95: //!   from the crate `twox-hash`.
 96: //! - Macros [`indexmap!`] and [`indexset!`] are unavailable without `std`. Use
 97: //!   the macros [`indexmap_with_default!`] and [`indexset_with_default!`] instead.
 98: #![cfg_attr(docsrs, feature(doc_cfg))]
 99: extern crate alloc;
100: #[cfg(feature = "std")]
101: #[macro_use]
102: extern crate std;
103: mod arbitrary;
104: mod inner;
105: #[macro_use]
106: mod macros;
107: #[cfg(feature = "borsh")]
108: mod borsh;
109: #[cfg(feature = "serde")]
110: mod serde;
111: #[cfg(feature = "sval")]
112: mod sval;
113: mod util;
114: pub mod map;
115: pub mod set;
116: // Placed after `map` and `set` so new `rayon` methods on the types
117: // are documented after the "normal" methods.
118: #[cfg(feature = "rayon")]
119: mod rayon;
120: pub use crate::map::IndexMap;
121: pub use crate::set::IndexSet;
122: pub use equivalent::Equivalent;
123: // shared private items
124: /// Hash value newtype. Not larger than usize, since anything larger
125: /// isn't used for selecting position anyway.
126: #[derive(Clone, Copy, Debug, PartialEq)]
127: struct HashValue(usize);
128: impl HashValue {
129:     #[inline(always)]
130:     fn get(self) -> u64 {
131:         self.0 as u64
132:     }
133: }
134: #[derive(Copy, Debug)]
135: struct Bucket<K, V> {
136:     hash: HashValue,
137:     key: K,
138:     value: V,
139: }
140: impl<K, V> Clone for Bucket<K, V>
141: where
142:     K: Clone,
143:     V: Clone,
144: {
145:     fn clone(&self) -> Self {
146:         Bucket {
147:             hash: self.hash,
148:             key: self.key.clone(),
149:             value: self.value.clone(),
150:         }
151:     }
152:     fn clone_from(&mut self, other: &Self) {
153:         self.hash = other.hash;
154:         self.key.clone_from(&other.key);
155:         self.value.clone_from(&other.value);
156:     }
157: }
158: impl<K, V> Bucket<K, V> {
159:     // field accessors -- used for `f` instead of closures in `.map(f)`
160:     const fn key_ref(&self) -> &K {
161:         &self.key
162:     }
163:     const fn value_ref(&self) -> &V {
164:         &self.value
165:     }
166:     const fn value_mut(&mut self) -> &mut V {
167:         &mut self.value
168:     }
169:     fn key(self) -> K {
170:         self.key
171:     }
172:     fn value(self) -> V {
173:         self.value
174:     }
175:     fn key_value(self) -> (K, V) {
176:         (self.key, self.value)
177:     }
178:     const fn refs(&self) -> (&K, &V) {
179:         (&self.key, &self.value)
180:     }
181:     const fn ref_mut(&mut self) -> (&K, &mut V) {
182:         (&self.key, &mut self.value)
183:     }
184:     const fn muts(&mut self) -> (&mut K, &mut V) {
185:         (&mut self.key, &mut self.value)
186:     }
187: }
188: /// The error type for [`try_reserve`][IndexMap::try_reserve] methods.
189: #[derive(Clone, PartialEq, Eq, Debug)]
190: pub struct TryReserveError {
191:     kind: TryReserveErrorKind,
192: }
193: #[derive(Clone, PartialEq, Eq, Debug)]
194: enum TryReserveErrorKind {
195:     // The standard library's kind is currently opaque to us, otherwise we could unify this.
196:     Std(alloc::collections::TryReserveError),
197:     CapacityOverflow,
198:     AllocError { layout: alloc::alloc::Layout },
199: }
200: // These are not `From` so we don't expose them in our public API.
201: impl TryReserveError {
202:     fn from_alloc(error: alloc::collections::TryReserveError) -> Self {
203:         Self {
204:             kind: TryReserveErrorKind::Std(error),
205:         }
206:     }
207:     fn from_hashbrown(error: hashbrown::TryReserveError) -> Self {
208:         Self {
209:             kind: match error {
210:                 hashbrown::TryReserveError::CapacityOverflow => {
211:                     TryReserveErrorKind::CapacityOverflow
212:                 }
213:                 hashbrown::TryReserveError::AllocError { layout } => {
214:                     TryReserveErrorKind::AllocError { layout }
215:                 }
216:             },
217:         }
218:     }
219: }
220: impl core::fmt::Display for TryReserveError {
221:     fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
222:         let reason = match &self.kind {
223:             TryReserveErrorKind::Std(e) => return core::fmt::Display::fmt(e, f),
224:             TryReserveErrorKind::CapacityOverflow => {
225:                 " because the computed capacity exceeded the collection's maximum"
226:             }
227:             TryReserveErrorKind::AllocError { .. } => {
228:                 " because the memory allocator returned an error"
229:             }
230:         };
231:         f.write_str("memory allocation failed")?;
232:         f.write_str(reason)
233:     }
234: }
235: impl core::error::Error for TryReserveError {}
236: // NOTE: This is copied from the slice module in the std lib.
237: /// The error type returned by [`get_disjoint_indices_mut`][`IndexMap::get_disjoint_indices_mut`].
238: ///
239: /// It indicates one of two possible errors:
240: /// - An index is out-of-bounds.
241: /// - The same index appeared multiple times in the array.
242: //    (or different but overlapping indices when ranges are provided)
243: #[derive(Debug, Clone, PartialEq, Eq)]
244: pub enum GetDisjointMutError {
245:     /// An index provided was out-of-bounds for the slice.
246:     IndexOutOfBounds,
247:     /// Two indices provided were overlapping.
248:     OverlappingIndices,
249: }
250: impl core::fmt::Display for GetDisjointMutError {
251:     fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
252:         let msg = match self {
253:             GetDisjointMutError::IndexOutOfBounds => "an index is out of bounds",
254:             GetDisjointMutError::OverlappingIndices => "there were overlapping indices",
255:         };
256:         core::fmt::Display::fmt(msg, f)
257:     }
258: }
259: impl core::error::Error for GetDisjointMutError {}
````

## File: src/macros.rs
````rust
  1: /// Create an [`IndexMap`][crate::IndexMap] from a list of key-value pairs
  2: /// and a [`BuildHasherDefault`][core::hash::BuildHasherDefault]-wrapped custom hasher.
  3: ///
  4: /// ## Example
  5: ///
  6: /// ```
  7: /// use indexmap::indexmap_with_default;
  8: /// use fnv::FnvHasher;
  9: ///
 10: /// let map = indexmap_with_default!{
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
 26: /// use indexmap::{IndexMap, indexmap_with_default};
 27: /// use fnv::FnvBuildHasher; // = BuildHasherDefault<FnvHasher>
 28: /// use std::sync::Mutex;
 29: ///
 30: /// static GLOBAL: Mutex<IndexMap<String, i32, FnvBuildHasher>> =
 31: ///     Mutex::new(indexmap_with_default!());
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
 42: macro_rules! indexmap_with_default {
 43:     () => { const {
 44:         $crate::IndexMap::with_hasher(
 45:             // Let type inference figure out the hasher:
 46:             ::core::hash::BuildHasherDefault::new(),
 47:         )
 48:     }};
 49:     ($H:ty $(;)?) => { const {
 50:         $crate::IndexMap::with_hasher(
 51:             // Specify your custom `H` (must implement Default + Hasher) as the hasher:
 52:             ::core::hash::BuildHasherDefault::<$H>::new(),
 53:         )
 54:     }};
 55:     ($H:ty; $($key:expr => $value:expr),+ $(,)?) => {{
 56:         let mut map = $crate::IndexMap::with_capacity_and_hasher(
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
 72: /// Create an [`IndexMap`][crate::IndexMap] from a list of key-value pairs
 73: ///
 74: /// ## Example
 75: ///
 76: /// ```
 77: /// use indexmap::indexmap;
 78: ///
 79: /// let map = indexmap!{
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
 90: macro_rules! indexmap {
 91:     () => { $crate::IndexMap::new() };
 92:     ($($key:expr => $value:expr),+ $(,)?) => {{
 93:         let mut map = $crate::IndexMap::with_capacity(
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
104: /// Create an [`IndexSet`][crate::IndexSet] from a list of values
105: /// and a [`BuildHasherDefault`][core::hash::BuildHasherDefault]-wrapped custom hasher.
106: ///
107: /// ## Example
108: ///
109: /// ```
110: /// use indexmap::indexset_with_default;
111: /// use fnv::FnvHasher;
112: ///
113: /// let set = indexset_with_default!{
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
129: /// use indexmap::{IndexSet, indexset_with_default};
130: /// use fnv::FnvBuildHasher; // = BuildHasherDefault<FnvHasher>
131: /// use std::sync::Mutex;
132: ///
133: /// static INTERN: Mutex<IndexSet<String, FnvBuildHasher>> =
134: ///     Mutex::new(indexset_with_default!());
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
147: macro_rules! indexset_with_default {
148:     () => { const {
149:         $crate::IndexSet::with_hasher(
150:             // Let type inference figure out the hasher:
151:             ::core::hash::BuildHasherDefault::new(),
152:         )
153:     }};
154:     ($H:ty $(;)?) => { const {
155:         $crate::IndexSet::with_hasher(
156:             // Specify your custom `H` (must implement Default + Hasher) as the hasher:
157:             ::core::hash::BuildHasherDefault::<$H>::new(),
158:         )
159:     }};
160:     ($H:ty; $($value:expr),+ $(,)?) => {{
161:         let mut set = $crate::IndexSet::with_capacity_and_hasher(
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
177: /// Create an [`IndexSet`][crate::IndexSet] from a list of values
178: ///
179: /// ## Example
180: ///
181: /// ```
182: /// use indexmap::indexset;
183: ///
184: /// let set = indexset!{
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
195: macro_rules! indexset {
196:     () => { $crate::IndexSet::new() };
197:     ($($value:expr),+ $(,)?) => {{
198:         let mut set = $crate::IndexSet::with_capacity(
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
209: // generate all the Iterator methods by just forwarding to the underlying
210: // self.iter and mapping its element.
211: macro_rules! iterator_methods {
212:     // $map_elt is the mapping function from the underlying iterator's element
213:     // same mapping function for both options and iterators
214:     ($map_elt:expr) => {
215:         fn next(&mut self) -> Option<Self::Item> {
216:             self.iter.next().map($map_elt)
217:         }
218:         fn size_hint(&self) -> (usize, Option<usize>) {
219:             self.iter.size_hint()
220:         }
221:         fn count(self) -> usize {
222:             self.iter.len()
223:         }
224:         fn nth(&mut self, n: usize) -> Option<Self::Item> {
225:             self.iter.nth(n).map($map_elt)
226:         }
227:         fn last(mut self) -> Option<Self::Item> {
228:             self.next_back()
229:         }
230:         fn collect<C>(self) -> C
231:         where
232:             C: FromIterator<Self::Item>,
233:         {
234:             // NB: forwarding this directly to standard iterators will
235:             // allow it to leverage unstable traits like `TrustedLen`.
236:             self.iter.map($map_elt).collect()
237:         }
238:     };
239: }
240: macro_rules! double_ended_iterator_methods {
241:     // $map_elt is the mapping function from the underlying iterator's element
242:     // same mapping function for both options and iterators
243:     ($map_elt:expr) => {
244:         fn next_back(&mut self) -> Option<Self::Item> {
245:             self.iter.next_back().map($map_elt)
246:         }
247:         fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
248:             self.iter.nth_back(n).map($map_elt)
249:         }
250:     };
251: }
252: // generate `ParallelIterator` methods by just forwarding to the underlying
253: // self.entries and mapping its elements.
254: #[cfg(feature = "rayon")]
255: macro_rules! parallel_iterator_methods {
256:     // $map_elt is the mapping function from the underlying iterator's element
257:     ($map_elt:expr) => {
258:         fn drive_unindexed<C>(self, consumer: C) -> C::Result
259:         where
260:             C: UnindexedConsumer<Self::Item>,
261:         {
262:             self.entries
263:                 .into_par_iter()
264:                 .map($map_elt)
265:                 .drive_unindexed(consumer)
266:         }
267:         // NB: This allows indexed collection, e.g. directly into a `Vec`, but the
268:         // underlying iterator must really be indexed.  We should remove this if we
269:         // start having tombstones that must be filtered out.
270:         fn opt_len(&self) -> Option<usize> {
271:             Some(self.entries.len())
272:         }
273:     };
274: }
275: // generate `IndexedParallelIterator` methods by just forwarding to the underlying
276: // self.entries and mapping its elements.
277: #[cfg(feature = "rayon")]
278: macro_rules! indexed_parallel_iterator_methods {
279:     // $map_elt is the mapping function from the underlying iterator's element
280:     ($map_elt:expr) => {
281:         fn drive<C>(self, consumer: C) -> C::Result
282:         where
283:             C: Consumer<Self::Item>,
284:         {
285:             self.entries.into_par_iter().map($map_elt).drive(consumer)
286:         }
287:         fn len(&self) -> usize {
288:             self.entries.len()
289:         }
290:         fn with_producer<CB>(self, callback: CB) -> CB::Output
291:         where
292:             CB: ProducerCallback<Self::Item>,
293:         {
294:             self.entries
295:                 .into_par_iter()
296:                 .map($map_elt)
297:                 .with_producer(callback)
298:         }
299:     };
300: }
````

## File: src/map.rs
````rust
   1: //! [`IndexMap`] is a hash table where the iteration order of the key-value
   2: //! pairs is independent of the hash values of the keys.
   3: mod disjoint;
   4: mod entry;
   5: mod iter;
   6: mod mutable;
   7: mod slice;
   8: pub mod raw_entry_v1;
   9: #[cfg(feature = "serde")]
  10: #[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
  11: pub mod serde_seq;
  12: #[cfg(test)]
  13: mod tests;
  14: pub use self::entry::{Entry, IndexedEntry};
  15: pub use crate::inner::{OccupiedEntry, VacantEntry};
  16: pub use self::iter::{
  17:     Drain, ExtractIf, IntoIter, IntoKeys, IntoValues, Iter, IterMut, IterMut2, Keys, Splice,
  18:     Values, ValuesMut,
  19: };
  20: pub use self::mutable::MutableEntryKey;
  21: pub use self::mutable::MutableKeys;
  22: pub use self::raw_entry_v1::RawEntryApiV1;
  23: pub use self::slice::Slice;
  24: #[cfg(feature = "rayon")]
  25: pub use crate::rayon::map as rayon;
  26: use alloc::boxed::Box;
  27: use alloc::vec::Vec;
  28: use core::cmp::Ordering;
  29: use core::fmt;
  30: use core::hash::{BuildHasher, Hash};
  31: use core::mem;
  32: use core::ops::{Index, IndexMut, RangeBounds};
  33: #[cfg(feature = "std")]
  34: use std::hash::RandomState;
  35: use crate::inner::Core;
  36: use crate::util::{assert_index_le, assert_index_lt, third, try_simplify_range};
  37: use crate::{Bucket, Equivalent, GetDisjointMutError, HashValue, TryReserveError};
  38: /// A hash table where the iteration order of the key-value pairs is independent
  39: /// of the hash values of the keys.
  40: ///
  41: /// The interface is closely compatible with the standard
  42: /// [`HashMap`][std::collections::HashMap],
  43: /// but also has additional features.
  44: ///
  45: /// # Order
  46: ///
  47: /// The key-value pairs have a consistent order that is determined by
  48: /// the sequence of insertion and removal calls on the map. The order does
  49: /// not depend on the keys or the hash function at all.
  50: ///
  51: /// All iterators traverse the map in *the order*.
  52: ///
  53: /// The insertion order is preserved, with **notable exceptions** like the
  54: /// [`.remove()`][Self::remove] or [`.swap_remove()`][Self::swap_remove] methods.
  55: /// Methods such as [`.sort_by()`][Self::sort_by] of
  56: /// course result in a new order, depending on the sorting order.
  57: ///
  58: /// # Indices
  59: ///
  60: /// The key-value pairs are indexed in a compact range without holes in the
  61: /// range `0..self.len()`. For example, the method `.get_full` looks up the
  62: /// index for a key, and the method `.get_index` looks up the key-value pair by
  63: /// index.
  64: ///
  65: /// # Examples
  66: ///
  67: /// ```
  68: /// use indexmap::IndexMap;
  69: ///
  70: /// // count the frequency of each letter in a sentence.
  71: /// let mut letters = IndexMap::new();
  72: /// for ch in "a short treatise on fungi".chars() {
  73: ///     *letters.entry(ch).or_insert(0) += 1;
  74: /// }
  75: ///
  76: /// assert_eq!(letters[&'s'], 2);
  77: /// assert_eq!(letters[&'t'], 3);
  78: /// assert_eq!(letters[&'u'], 1);
  79: /// assert_eq!(letters.get(&'y'), None);
  80: /// ```
  81: #[cfg(feature = "std")]
  82: pub struct IndexMap<K, V, S = RandomState> {
  83:     pub(crate) core: Core<K, V>,
  84:     hash_builder: S,
  85: }
  86: #[cfg(not(feature = "std"))]
  87: pub struct IndexMap<K, V, S> {
  88:     pub(crate) core: Core<K, V>,
  89:     hash_builder: S,
  90: }
  91: impl<K, V, S> Clone for IndexMap<K, V, S>
  92: where
  93:     K: Clone,
  94:     V: Clone,
  95:     S: Clone,
  96: {
  97:     fn clone(&self) -> Self {
  98:         IndexMap {
  99:             core: self.core.clone(),
 100:             hash_builder: self.hash_builder.clone(),
 101:         }
 102:     }
 103:     fn clone_from(&mut self, other: &Self) {
 104:         self.core.clone_from(&other.core);
 105:         self.hash_builder.clone_from(&other.hash_builder);
 106:     }
 107: }
 108: impl<K, V, S> fmt::Debug for IndexMap<K, V, S>
 109: where
 110:     K: fmt::Debug,
 111:     V: fmt::Debug,
 112: {
 113:     #[cfg(not(feature = "test_debug"))]
 114:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
 115:         f.debug_map().entries(self.iter()).finish()
 116:     }
 117:     #[cfg(feature = "test_debug")]
 118:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
 119:         // Let the inner `Core` print all of its details
 120:         f.debug_struct("IndexMap")
 121:             .field("core", &self.core)
 122:             .finish()
 123:     }
 124: }
 125: #[cfg(feature = "std")]
 126: #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
 127: impl<K, V> IndexMap<K, V> {
 128:     /// Create a new map. (Does not allocate.)
 129:     #[inline]
 130:     pub fn new() -> Self {
 131:         Self::with_capacity(0)
 132:     }
 133:     /// Create a new map with capacity for `n` key-value pairs. (Does not
 134:     /// allocate if `n` is zero.)
 135:     ///
 136:     /// Computes in **O(n)** time.
 137:     #[inline]
 138:     pub fn with_capacity(n: usize) -> Self {
 139:         Self::with_capacity_and_hasher(n, <_>::default())
 140:     }
 141: }
 142: impl<K, V, S> IndexMap<K, V, S> {
 143:     /// Create a new map with capacity for `n` key-value pairs. (Does not
 144:     /// allocate if `n` is zero.)
 145:     ///
 146:     /// Computes in **O(n)** time.
 147:     #[inline]
 148:     pub fn with_capacity_and_hasher(n: usize, hash_builder: S) -> Self {
 149:         if n == 0 {
 150:             Self::with_hasher(hash_builder)
 151:         } else {
 152:             IndexMap {
 153:                 core: Core::with_capacity(n),
 154:                 hash_builder,
 155:             }
 156:         }
 157:     }
 158:     /// Create a new map with `hash_builder`.
 159:     ///
 160:     /// This function is `const`, so it
 161:     /// can be called in `static` contexts.
 162:     pub const fn with_hasher(hash_builder: S) -> Self {
 163:         IndexMap {
 164:             core: Core::new(),
 165:             hash_builder,
 166:         }
 167:     }
 168:     #[inline]
 169:     pub(crate) fn into_entries(self) -> Vec<Bucket<K, V>> {
 170:         self.core.into_entries()
 171:     }
 172:     #[inline]
 173:     pub(crate) fn as_entries(&self) -> &[Bucket<K, V>] {
 174:         self.core.as_entries()
 175:     }
 176:     #[inline]
 177:     pub(crate) fn as_entries_mut(&mut self) -> &mut [Bucket<K, V>] {
 178:         self.core.as_entries_mut()
 179:     }
 180:     pub(crate) fn with_entries<F>(&mut self, f: F)
 181:     where
 182:         F: FnOnce(&mut [Bucket<K, V>]),
 183:     {
 184:         self.core.with_entries(f);
 185:     }
 186:     /// Return the number of elements the map can hold without reallocating.
 187:     ///
 188:     /// This number is a lower bound; the map might be able to hold more,
 189:     /// but is guaranteed to be able to hold at least this many.
 190:     ///
 191:     /// Computes in **O(1)** time.
 192:     pub fn capacity(&self) -> usize {
 193:         self.core.capacity()
 194:     }
 195:     /// Return a reference to the map's `BuildHasher`.
 196:     pub fn hasher(&self) -> &S {
 197:         &self.hash_builder
 198:     }
 199:     /// Return the number of key-value pairs in the map.
 200:     ///
 201:     /// Computes in **O(1)** time.
 202:     #[inline]
 203:     pub fn len(&self) -> usize {
 204:         self.core.len()
 205:     }
 206:     /// Returns true if the map contains no elements.
 207:     ///
 208:     /// Computes in **O(1)** time.
 209:     #[inline]
 210:     pub fn is_empty(&self) -> bool {
 211:         self.len() == 0
 212:     }
 213:     /// Return an iterator over the key-value pairs of the map, in their order
 214:     pub fn iter(&self) -> Iter<'_, K, V> {
 215:         Iter::new(self.as_entries())
 216:     }
 217:     /// Return an iterator over the key-value pairs of the map, in their order
 218:     pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
 219:         IterMut::new(self.as_entries_mut())
 220:     }
 221:     /// Return an iterator over the keys of the map, in their order
 222:     pub fn keys(&self) -> Keys<'_, K, V> {
 223:         Keys::new(self.as_entries())
 224:     }
 225:     /// Return an owning iterator over the keys of the map, in their order
 226:     pub fn into_keys(self) -> IntoKeys<K, V> {
 227:         IntoKeys::new(self.into_entries())
 228:     }
 229:     /// Return an iterator over the values of the map, in their order
 230:     pub fn values(&self) -> Values<'_, K, V> {
 231:         Values::new(self.as_entries())
 232:     }
 233:     /// Return an iterator over mutable references to the values of the map,
 234:     /// in their order
 235:     pub fn values_mut(&mut self) -> ValuesMut<'_, K, V> {
 236:         ValuesMut::new(self.as_entries_mut())
 237:     }
 238:     /// Return an owning iterator over the values of the map, in their order
 239:     pub fn into_values(self) -> IntoValues<K, V> {
 240:         IntoValues::new(self.into_entries())
 241:     }
 242:     /// Remove all key-value pairs in the map, while preserving its capacity.
 243:     ///
 244:     /// Computes in **O(n)** time.
 245:     pub fn clear(&mut self) {
 246:         self.core.clear();
 247:     }
 248:     /// Shortens the map, keeping the first `len` elements and dropping the rest.
 249:     ///
 250:     /// If `len` is greater than the map's current length, this has no effect.
 251:     pub fn truncate(&mut self, len: usize) {
 252:         self.core.truncate(len);
 253:     }
 254:     /// Clears the `IndexMap` in the given index range, returning those
 255:     /// key-value pairs as a drain iterator.
 256:     ///
 257:     /// The range may be any type that implements [`RangeBounds<usize>`],
 258:     /// including all of the `std::ops::Range*` types, or even a tuple pair of
 259:     /// `Bound` start and end values. To drain the map entirely, use `RangeFull`
 260:     /// like `map.drain(..)`.
 261:     ///
 262:     /// This shifts down all entries following the drained range to fill the
 263:     /// gap, and keeps the allocated memory for reuse.
 264:     ///
 265:     /// ***Panics*** if the starting point is greater than the end point or if
 266:     /// the end point is greater than the length of the map.
 267:     #[track_caller]
 268:     pub fn drain<R>(&mut self, range: R) -> Drain<'_, K, V>
 269:     where
 270:         R: RangeBounds<usize>,
 271:     {
 272:         Drain::new(self.core.drain(range))
 273:     }
 274:     /// Creates an iterator which uses a closure to determine if an element should be removed,
 275:     /// for all elements in the given range.
 276:     ///
 277:     /// If the closure returns true, the element is removed from the map and yielded.
 278:     /// If the closure returns false, or panics, the element remains in the map and will not be
 279:     /// yielded.
 280:     ///
 281:     /// Note that `extract_if` lets you mutate every value in the filter closure, regardless of
 282:     /// whether you choose to keep or remove it.
 283:     ///
 284:     /// The range may be any type that implements [`RangeBounds<usize>`],
 285:     /// including all of the `std::ops::Range*` types, or even a tuple pair of
 286:     /// `Bound` start and end values. To check the entire map, use `RangeFull`
 287:     /// like `map.extract_if(.., predicate)`.
 288:     ///
 289:     /// If the returned `ExtractIf` is not exhausted, e.g. because it is dropped without iterating
 290:     /// or the iteration short-circuits, then the remaining elements will be retained.
 291:     /// Use [`retain`] with a negated predicate if you do not need the returned iterator.
 292:     ///
 293:     /// [`retain`]: IndexMap::retain
 294:     ///
 295:     /// ***Panics*** if the starting point is greater than the end point or if
 296:     /// the end point is greater than the length of the map.
 297:     ///
 298:     /// # Examples
 299:     ///
 300:     /// Splitting a map into even and odd keys, reusing the original map:
 301:     ///
 302:     /// ```
 303:     /// use indexmap::IndexMap;
 304:     ///
 305:     /// let mut map: IndexMap<i32, i32> = (0..8).map(|x| (x, x)).collect();
 306:     /// let extracted: IndexMap<i32, i32> = map.extract_if(.., |k, _v| k % 2 == 0).collect();
 307:     ///
 308:     /// let evens = extracted.keys().copied().collect::<Vec<_>>();
 309:     /// let odds = map.keys().copied().collect::<Vec<_>>();
 310:     ///
 311:     /// assert_eq!(evens, vec![0, 2, 4, 6]);
 312:     /// assert_eq!(odds, vec![1, 3, 5, 7]);
 313:     /// ```
 314:     #[track_caller]
 315:     pub fn extract_if<F, R>(&mut self, range: R, pred: F) -> ExtractIf<'_, K, V, F>
 316:     where
 317:         F: FnMut(&K, &mut V) -> bool,
 318:         R: RangeBounds<usize>,
 319:     {
 320:         ExtractIf::new(&mut self.core, range, pred)
 321:     }
 322:     /// Splits the collection into two at the given index.
 323:     ///
 324:     /// Returns a newly allocated map containing the elements in the range
 325:     /// `[at, len)`. After the call, the original map will be left containing
 326:     /// the elements `[0, at)` with its previous capacity unchanged.
 327:     ///
 328:     /// ***Panics*** if `at > len`.
 329:     #[track_caller]
 330:     pub fn split_off(&mut self, at: usize) -> Self
 331:     where
 332:         S: Clone,
 333:     {
 334:         Self {
 335:             core: self.core.split_off(at),
 336:             hash_builder: self.hash_builder.clone(),
 337:         }
 338:     }
 339:     /// Reserve capacity for `additional` more key-value pairs.
 340:     ///
 341:     /// Computes in **O(n)** time.
 342:     pub fn reserve(&mut self, additional: usize) {
 343:         self.core.reserve(additional);
 344:     }
 345:     /// Reserve capacity for `additional` more key-value pairs, without over-allocating.
 346:     ///
 347:     /// Unlike `reserve`, this does not deliberately over-allocate the entry capacity to avoid
 348:     /// frequent re-allocations. However, the underlying data structures may still have internal
 349:     /// capacity requirements, and the allocator itself may give more space than requested, so this
 350:     /// cannot be relied upon to be precisely minimal.
 351:     ///
 352:     /// Computes in **O(n)** time.
 353:     pub fn reserve_exact(&mut self, additional: usize) {
 354:         self.core.reserve_exact(additional);
 355:     }
 356:     /// Try to reserve capacity for `additional` more key-value pairs.
 357:     ///
 358:     /// Computes in **O(n)** time.
 359:     pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
 360:         self.core.try_reserve(additional)
 361:     }
 362:     /// Try to reserve capacity for `additional` more key-value pairs, without over-allocating.
 363:     ///
 364:     /// Unlike `try_reserve`, this does not deliberately over-allocate the entry capacity to avoid
 365:     /// frequent re-allocations. However, the underlying data structures may still have internal
 366:     /// capacity requirements, and the allocator itself may give more space than requested, so this
 367:     /// cannot be relied upon to be precisely minimal.
 368:     ///
 369:     /// Computes in **O(n)** time.
 370:     pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
 371:         self.core.try_reserve_exact(additional)
 372:     }
 373:     /// Shrink the capacity of the map as much as possible.
 374:     ///
 375:     /// Computes in **O(n)** time.
 376:     pub fn shrink_to_fit(&mut self) {
 377:         self.core.shrink_to(0);
 378:     }
 379:     /// Shrink the capacity of the map with a lower limit.
 380:     ///
 381:     /// Computes in **O(n)** time.
 382:     pub fn shrink_to(&mut self, min_capacity: usize) {
 383:         self.core.shrink_to(min_capacity);
 384:     }
 385: }
 386: impl<K, V, S> IndexMap<K, V, S>
 387: where
 388:     K: Hash + Eq,
 389:     S: BuildHasher,
 390: {
 391:     /// Insert a key-value pair in the map.
 392:     ///
 393:     /// If an equivalent key already exists in the map: the key remains and
 394:     /// retains in its place in the order, its corresponding value is updated
 395:     /// with `value`, and the older value is returned inside `Some(_)`.
 396:     ///
 397:     /// If no equivalent key existed in the map: the new key-value pair is
 398:     /// inserted, last in order, and `None` is returned.
 399:     ///
 400:     /// Computes in **O(1)** time (amortized average).
 401:     ///
 402:     /// See also [`entry`][Self::entry] if you want to insert *or* modify,
 403:     /// or [`insert_full`][Self::insert_full] if you need to get the index of
 404:     /// the corresponding key-value pair.
 405:     pub fn insert(&mut self, key: K, value: V) -> Option<V> {
 406:         self.insert_full(key, value).1
 407:     }
 408:     /// Insert a key-value pair in the map, and get their index.
 409:     ///
 410:     /// If an equivalent key already exists in the map: the key remains and
 411:     /// retains in its place in the order, its corresponding value is updated
 412:     /// with `value`, and the older value is returned inside `(index, Some(_))`.
 413:     ///
 414:     /// If no equivalent key existed in the map: the new key-value pair is
 415:     /// inserted, last in order, and `(index, None)` is returned.
 416:     ///
 417:     /// Computes in **O(1)** time (amortized average).
 418:     ///
 419:     /// See also [`entry`][Self::entry] if you want to insert *or* modify.
 420:     pub fn insert_full(&mut self, key: K, value: V) -> (usize, Option<V>) {
 421:         let hash = self.hash(&key);
 422:         self.core.insert_full(hash, key, value)
 423:     }
 424:     /// Insert a key-value pair in the map at its ordered position among sorted keys.
 425:     ///
 426:     /// This is equivalent to finding the position with
 427:     /// [`binary_search_keys`][Self::binary_search_keys], then either updating
 428:     /// it or calling [`insert_before`][Self::insert_before] for a new key.
 429:     ///
 430:     /// If the sorted key is found in the map, its corresponding value is
 431:     /// updated with `value`, and the older value is returned inside
 432:     /// `(index, Some(_))`. Otherwise, the new key-value pair is inserted at
 433:     /// the sorted position, and `(index, None)` is returned.
 434:     ///
 435:     /// If the existing keys are **not** already sorted, then the insertion
 436:     /// index is unspecified (like [`slice::binary_search`]), but the key-value
 437:     /// pair is moved to or inserted at that position regardless.
 438:     ///
 439:     /// Computes in **O(n)** time (average). Instead of repeating calls to
 440:     /// `insert_sorted`, it may be faster to call batched [`insert`][Self::insert]
 441:     /// or [`extend`][Self::extend] and only call [`sort_keys`][Self::sort_keys]
 442:     /// or [`sort_unstable_keys`][Self::sort_unstable_keys] once.
 443:     pub fn insert_sorted(&mut self, key: K, value: V) -> (usize, Option<V>)
 444:     where
 445:         K: Ord,
 446:     {
 447:         match self.binary_search_keys(&key) {
 448:             Ok(i) => (i, Some(mem::replace(&mut self[i], value))),
 449:             Err(i) => self.insert_before(i, key, value),
 450:         }
 451:     }
 452:     /// Insert a key-value pair in the map at its ordered position among keys
 453:     /// sorted by `cmp`.
 454:     ///
 455:     /// This is equivalent to finding the position with
 456:     /// [`binary_search_by`][Self::binary_search_by], then calling
 457:     /// [`insert_before`][Self::insert_before] with the given key and value.
 458:     ///
 459:     /// If the existing keys are **not** already sorted, then the insertion
 460:     /// index is unspecified (like [`slice::binary_search`]), but the key-value
 461:     /// pair is moved to or inserted at that position regardless.
 462:     ///
 463:     /// Computes in **O(n)** time (average).
 464:     pub fn insert_sorted_by<F>(&mut self, key: K, value: V, mut cmp: F) -> (usize, Option<V>)
 465:     where
 466:         F: FnMut(&K, &V, &K, &V) -> Ordering,
 467:     {
 468:         let (Ok(i) | Err(i)) = self.binary_search_by(|k, v| cmp(k, v, &key, &value));
 469:         self.insert_before(i, key, value)
 470:     }
 471:     /// Insert a key-value pair in the map at its ordered position
 472:     /// using a sort-key extraction function.
 473:     ///
 474:     /// This is equivalent to finding the position with
 475:     /// [`binary_search_by_key`][Self::binary_search_by_key] with `sort_key(key)`, then
 476:     /// calling [`insert_before`][Self::insert_before] with the given key and value.
 477:     ///
 478:     /// If the existing keys are **not** already sorted, then the insertion
 479:     /// index is unspecified (like [`slice::binary_search`]), but the key-value
 480:     /// pair is moved to or inserted at that position regardless.
 481:     ///
 482:     /// Computes in **O(n)** time (average).
 483:     pub fn insert_sorted_by_key<B, F>(
 484:         &mut self,
 485:         key: K,
 486:         value: V,
 487:         mut sort_key: F,
 488:     ) -> (usize, Option<V>)
 489:     where
 490:         B: Ord,
 491:         F: FnMut(&K, &V) -> B,
 492:     {
 493:         let search_key = sort_key(&key, &value);
 494:         let (Ok(i) | Err(i)) = self.binary_search_by_key(&search_key, sort_key);
 495:         self.insert_before(i, key, value)
 496:     }
 497:     /// Insert a key-value pair in the map before the entry at the given index, or at the end.
 498:     ///
 499:     /// If an equivalent key already exists in the map: the key remains and
 500:     /// is moved to the new position in the map, its corresponding value is updated
 501:     /// with `value`, and the older value is returned inside `Some(_)`. The returned index
 502:     /// will either be the given index or one less, depending on how the entry moved.
 503:     /// (See [`shift_insert`](Self::shift_insert) for different behavior here.)
 504:     ///
 505:     /// If no equivalent key existed in the map: the new key-value pair is
 506:     /// inserted exactly at the given index, and `None` is returned.
 507:     ///
 508:     /// ***Panics*** if `index` is out of bounds.
 509:     /// Valid indices are `0..=map.len()` (inclusive).
 510:     ///
 511:     /// Computes in **O(n)** time (average).
 512:     ///
 513:     /// See also [`entry`][Self::entry] if you want to insert *or* modify,
 514:     /// perhaps only using the index for new entries with [`VacantEntry::shift_insert`].
 515:     ///
 516:     /// # Examples
 517:     ///
 518:     /// ```
 519:     /// use indexmap::IndexMap;
 520:     /// let mut map: IndexMap<char, ()> = ('a'..='z').map(|c| (c, ())).collect();
 521:     ///
 522:     /// // The new key '*' goes exactly at the given index.
 523:     /// assert_eq!(map.get_index_of(&'*'), None);
 524:     /// assert_eq!(map.insert_before(10, '*', ()), (10, None));
 525:     /// assert_eq!(map.get_index_of(&'*'), Some(10));
 526:     ///
 527:     /// // Moving the key 'a' up will shift others down, so this moves *before* 10 to index 9.
 528:     /// assert_eq!(map.insert_before(10, 'a', ()), (9, Some(())));
 529:     /// assert_eq!(map.get_index_of(&'a'), Some(9));
 530:     /// assert_eq!(map.get_index_of(&'*'), Some(10));
 531:     ///
 532:     /// // Moving the key 'z' down will shift others up, so this moves to exactly 10.
 533:     /// assert_eq!(map.insert_before(10, 'z', ()), (10, Some(())));
 534:     /// assert_eq!(map.get_index_of(&'z'), Some(10));
 535:     /// assert_eq!(map.get_index_of(&'*'), Some(11));
 536:     ///
 537:     /// // Moving or inserting before the endpoint is also valid.
 538:     /// assert_eq!(map.len(), 27);
 539:     /// assert_eq!(map.insert_before(map.len(), '*', ()), (26, Some(())));
 540:     /// assert_eq!(map.get_index_of(&'*'), Some(26));
 541:     /// assert_eq!(map.insert_before(map.len(), '+', ()), (27, None));
 542:     /// assert_eq!(map.get_index_of(&'+'), Some(27));
 543:     /// assert_eq!(map.len(), 28);
 544:     /// ```
 545:     #[track_caller]
 546:     pub fn insert_before(&mut self, mut index: usize, key: K, value: V) -> (usize, Option<V>) {
 547:         assert_index_le(index, self.len());
 548:         match self.entry(key) {
 549:             Entry::Occupied(mut entry) => {
 550:                 if index > entry.index() {
 551:                     // Some entries will shift down when this one moves up,
 552:                     // so "insert before index" becomes "move to index - 1",
 553:                     // keeping the entry at the original index unmoved.
 554:                     index -= 1;
 555:                 }
 556:                 let old = mem::replace(entry.get_mut(), value);
 557:                 entry.move_index(index);
 558:                 (index, Some(old))
 559:             }
 560:             Entry::Vacant(entry) => {
 561:                 entry.shift_insert(index, value);
 562:                 (index, None)
 563:             }
 564:         }
 565:     }
 566:     /// Insert a key-value pair in the map at the given index.
 567:     ///
 568:     /// If an equivalent key already exists in the map: the key remains and
 569:     /// is moved to the given index in the map, its corresponding value is updated
 570:     /// with `value`, and the older value is returned inside `Some(_)`.
 571:     /// Note that existing entries **cannot** be moved to `index == map.len()`!
 572:     /// (See [`insert_before`](Self::insert_before) for different behavior here.)
 573:     ///
 574:     /// If no equivalent key existed in the map: the new key-value pair is
 575:     /// inserted at the given index, and `None` is returned.
 576:     ///
 577:     /// ***Panics*** if `index` is out of bounds.
 578:     /// Valid indices are `0..map.len()` (exclusive) when moving an existing entry, or
 579:     /// `0..=map.len()` (inclusive) when inserting a new key.
 580:     ///
 581:     /// Computes in **O(n)** time (average).
 582:     ///
 583:     /// See also [`entry`][Self::entry] if you want to insert *or* modify,
 584:     /// perhaps only using the index for new entries with [`VacantEntry::shift_insert`].
 585:     ///
 586:     /// # Examples
 587:     ///
 588:     /// ```
 589:     /// use indexmap::IndexMap;
 590:     /// let mut map: IndexMap<char, ()> = ('a'..='z').map(|c| (c, ())).collect();
 591:     ///
 592:     /// // The new key '*' goes exactly at the given index.
 593:     /// assert_eq!(map.get_index_of(&'*'), None);
 594:     /// assert_eq!(map.shift_insert(10, '*', ()), None);
 595:     /// assert_eq!(map.get_index_of(&'*'), Some(10));
 596:     ///
 597:     /// // Moving the key 'a' up to 10 will shift others down, including the '*' that was at 10.
 598:     /// assert_eq!(map.shift_insert(10, 'a', ()), Some(()));
 599:     /// assert_eq!(map.get_index_of(&'a'), Some(10));
 600:     /// assert_eq!(map.get_index_of(&'*'), Some(9));
 601:     ///
 602:     /// // Moving the key 'z' down to 9 will shift others up, including the '*' that was at 9.
 603:     /// assert_eq!(map.shift_insert(9, 'z', ()), Some(()));
 604:     /// assert_eq!(map.get_index_of(&'z'), Some(9));
 605:     /// assert_eq!(map.get_index_of(&'*'), Some(10));
 606:     ///
 607:     /// // Existing keys can move to len-1 at most, but new keys can insert at the endpoint.
 608:     /// assert_eq!(map.len(), 27);
 609:     /// assert_eq!(map.shift_insert(map.len() - 1, '*', ()), Some(()));
 610:     /// assert_eq!(map.get_index_of(&'*'), Some(26));
 611:     /// assert_eq!(map.shift_insert(map.len(), '+', ()), None);
 612:     /// assert_eq!(map.get_index_of(&'+'), Some(27));
 613:     /// assert_eq!(map.len(), 28);
 614:     /// ```
 615:     ///
 616:     /// ```should_panic
 617:     /// use indexmap::IndexMap;
 618:     /// let mut map: IndexMap<char, ()> = ('a'..='z').map(|c| (c, ())).collect();
 619:     ///
 620:     /// // This is an invalid index for moving an existing key!
 621:     /// map.shift_insert(map.len(), 'a', ());
 622:     /// ```
 623:     #[track_caller]
 624:     pub fn shift_insert(&mut self, index: usize, key: K, value: V) -> Option<V> {
 625:         let len = self.len();
 626:         match self.entry(key) {
 627:             Entry::Occupied(mut entry) => {
 628:                 assert_index_lt(index, len);
 629:                 let old = mem::replace(entry.get_mut(), value);
 630:                 entry.move_index(index);
 631:                 Some(old)
 632:             }
 633:             Entry::Vacant(entry) => {
 634:                 assert_index_le(index, len);
 635:                 entry.shift_insert(index, value);
 636:                 None
 637:             }
 638:         }
 639:     }
 640:     /// Replaces the key at the given index. The new key does not need to be
 641:     /// equivalent to the one it is replacing, but it must be unique to the rest
 642:     /// of the map.
 643:     ///
 644:     /// Returns `Ok(old_key)` if successful, or `Err((other_index, key))` if an
 645:     /// equivalent key already exists at a different index. The map will be
 646:     /// unchanged in the error case.
 647:     ///
 648:     /// Direct indexing can be used to change the corresponding value: simply
 649:     /// `map[index] = value`, or `mem::replace(&mut map[index], value)` to
 650:     /// retrieve the old value as well.
 651:     ///
 652:     /// ***Panics*** if `index` is out of bounds.
 653:     ///
 654:     /// Computes in **O(1)** time (average).
 655:     #[track_caller]
 656:     pub fn replace_index(&mut self, index: usize, key: K) -> Result<K, (usize, K)> {
 657:         assert_index_lt(index, self.len());
 658:         // If there's a direct match, we don't even need to hash it.
 659:         let entry = &mut self.as_entries_mut()[index];
 660:         if key == entry.key {
 661:             return Ok(mem::replace(&mut entry.key, key));
 662:         }
 663:         let hash = self.hash(&key);
 664:         if let Some(i) = self.core.get_index_of(hash, &key) {
 665:             debug_assert_ne!(i, index);
 666:             return Err((i, key));
 667:         }
 668:         Ok(self.core.replace_index_unique(index, hash, key))
 669:     }
 670:     /// Get the given key's corresponding entry in the map for insertion and/or
 671:     /// in-place manipulation.
 672:     ///
 673:     /// Computes in **O(1)** time (amortized average).
 674:     pub fn entry(&mut self, key: K) -> Entry<'_, K, V> {
 675:         let hash = self.hash(&key);
 676:         Entry::new(&mut self.core, hash, key)
 677:     }
 678:     /// Creates a splicing iterator that replaces the specified range in the map
 679:     /// with the given `replace_with` key-value iterator and yields the removed
 680:     /// items. `replace_with` does not need to be the same length as `range`.
 681:     ///
 682:     /// The `range` is removed even if the iterator is not consumed until the
 683:     /// end. It is unspecified how many elements are removed from the map if the
 684:     /// `Splice` value is leaked.
 685:     ///
 686:     /// The input iterator `replace_with` is only consumed when the `Splice`
 687:     /// value is dropped. If a key from the iterator matches an existing entry
 688:     /// in the map (outside of `range`), then the value will be updated in that
 689:     /// position. Otherwise, the new key-value pair will be inserted in the
 690:     /// replaced `range`.
 691:     ///
 692:     /// ***Panics*** if the starting point is greater than the end point or if
 693:     /// the end point is greater than the length of the map.
 694:     ///
 695:     /// # Examples
 696:     ///
 697:     /// ```
 698:     /// use indexmap::IndexMap;
 699:     ///
 700:     /// let mut map = IndexMap::from([(0, '_'), (1, 'a'), (2, 'b'), (3, 'c'), (4, 'd')]);
 701:     /// let new = [(5, 'E'), (4, 'D'), (3, 'C'), (2, 'B'), (1, 'A')];
 702:     /// let removed: Vec<_> = map.splice(2..4, new).collect();
 703:     ///
 704:     /// // 1 and 4 got new values, while 5, 3, and 2 were newly inserted.
 705:     /// assert!(map.into_iter().eq([(0, '_'), (1, 'A'), (5, 'E'), (3, 'C'), (2, 'B'), (4, 'D')]));
 706:     /// assert_eq!(removed, &[(2, 'b'), (3, 'c')]);
 707:     /// ```
 708:     #[track_caller]
 709:     pub fn splice<R, I>(&mut self, range: R, replace_with: I) -> Splice<'_, I::IntoIter, K, V, S>
 710:     where
 711:         R: RangeBounds<usize>,
 712:         I: IntoIterator<Item = (K, V)>,
 713:     {
 714:         Splice::new(self, range, replace_with.into_iter())
 715:     }
 716:     /// Moves all key-value pairs from `other` into `self`, leaving `other` empty.
 717:     ///
 718:     /// This is equivalent to calling [`insert`][Self::insert] for each
 719:     /// key-value pair from `other` in order, which means that for keys that
 720:     /// already exist in `self`, their value is updated in the current position.
 721:     ///
 722:     /// # Examples
 723:     ///
 724:     /// ```
 725:     /// use indexmap::IndexMap;
 726:     ///
 727:     /// // Note: Key (3) is present in both maps.
 728:     /// let mut a = IndexMap::from([(3, "c"), (2, "b"), (1, "a")]);
 729:     /// let mut b = IndexMap::from([(3, "d"), (4, "e"), (5, "f")]);
 730:     /// let old_capacity = b.capacity();
 731:     ///
 732:     /// a.append(&mut b);
 733:     ///
 734:     /// assert_eq!(a.len(), 5);
 735:     /// assert_eq!(b.len(), 0);
 736:     /// assert_eq!(b.capacity(), old_capacity);
 737:     ///
 738:     /// assert!(a.keys().eq(&[3, 2, 1, 4, 5]));
 739:     /// assert_eq!(a[&3], "d"); // "c" was overwritten.
 740:     /// ```
 741:     pub fn append<S2>(&mut self, other: &mut IndexMap<K, V, S2>) {
 742:         self.extend(other.drain(..));
 743:     }
 744: }
 745: impl<K, V, S> IndexMap<K, V, S>
 746: where
 747:     S: BuildHasher,
 748: {
 749:     pub(crate) fn hash<Q: ?Sized + Hash>(&self, key: &Q) -> HashValue {
 750:         let h = self.hash_builder.hash_one(key);
 751:         HashValue(h as usize)
 752:     }
 753:     /// Return `true` if an equivalent to `key` exists in the map.
 754:     ///
 755:     /// Computes in **O(1)** time (average).
 756:     pub fn contains_key<Q>(&self, key: &Q) -> bool
 757:     where
 758:         Q: ?Sized + Hash + Equivalent<K>,
 759:     {
 760:         self.get_index_of(key).is_some()
 761:     }
 762:     /// Return a reference to the stored value for `key`, if it is present,
 763:     /// else `None`.
 764:     ///
 765:     /// Computes in **O(1)** time (average).
 766:     pub fn get<Q>(&self, key: &Q) -> Option<&V>
 767:     where
 768:         Q: ?Sized + Hash + Equivalent<K>,
 769:     {
 770:         if let Some(i) = self.get_index_of(key) {
 771:             let entry = &self.as_entries()[i];
 772:             Some(&entry.value)
 773:         } else {
 774:             None
 775:         }
 776:     }
 777:     /// Return references to the stored key-value pair for the lookup `key`,
 778:     /// if it is present, else `None`.
 779:     ///
 780:     /// Computes in **O(1)** time (average).
 781:     pub fn get_key_value<Q>(&self, key: &Q) -> Option<(&K, &V)>
 782:     where
 783:         Q: ?Sized + Hash + Equivalent<K>,
 784:     {
 785:         if let Some(i) = self.get_index_of(key) {
 786:             let entry = &self.as_entries()[i];
 787:             Some((&entry.key, &entry.value))
 788:         } else {
 789:             None
 790:         }
 791:     }
 792:     /// Return the index with references to the stored key-value pair for the
 793:     /// lookup `key`, if it is present, else `None`.
 794:     ///
 795:     /// Computes in **O(1)** time (average).
 796:     pub fn get_full<Q>(&self, key: &Q) -> Option<(usize, &K, &V)>
 797:     where
 798:         Q: ?Sized + Hash + Equivalent<K>,
 799:     {
 800:         if let Some(i) = self.get_index_of(key) {
 801:             let entry = &self.as_entries()[i];
 802:             Some((i, &entry.key, &entry.value))
 803:         } else {
 804:             None
 805:         }
 806:     }
 807:     /// Return the item index for `key`, if it is present, else `None`.
 808:     ///
 809:     /// Computes in **O(1)** time (average).
 810:     pub fn get_index_of<Q>(&self, key: &Q) -> Option<usize>
 811:     where
 812:         Q: ?Sized + Hash + Equivalent<K>,
 813:     {
 814:         match self.as_entries() {
 815:             [] => None,
 816:             [x] => key.equivalent(&x.key).then_some(0),
 817:             _ => {
 818:                 let hash = self.hash(key);
 819:                 self.core.get_index_of(hash, key)
 820:             }
 821:         }
 822:     }
 823:     /// Return a mutable reference to the stored value for `key`,
 824:     /// if it is present, else `None`.
 825:     ///
 826:     /// Computes in **O(1)** time (average).
 827:     pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
 828:     where
 829:         Q: ?Sized + Hash + Equivalent<K>,
 830:     {
 831:         if let Some(i) = self.get_index_of(key) {
 832:             let entry = &mut self.as_entries_mut()[i];
 833:             Some(&mut entry.value)
 834:         } else {
 835:             None
 836:         }
 837:     }
 838:     /// Return a reference and mutable references to the stored key-value pair
 839:     /// for the lookup `key`, if it is present, else `None`.
 840:     ///
 841:     /// Computes in **O(1)** time (average).
 842:     pub fn get_key_value_mut<Q>(&mut self, key: &Q) -> Option<(&K, &mut V)>
 843:     where
 844:         Q: ?Sized + Hash + Equivalent<K>,
 845:     {
 846:         if let Some(i) = self.get_index_of(key) {
 847:             let entry = &mut self.as_entries_mut()[i];
 848:             Some((&entry.key, &mut entry.value))
 849:         } else {
 850:             None
 851:         }
 852:     }
 853:     /// Return the index with a reference and mutable reference to the stored
 854:     /// key-value pair for the lookup `key`, if it is present, else `None`.
 855:     ///
 856:     /// Computes in **O(1)** time (average).
 857:     pub fn get_full_mut<Q>(&mut self, key: &Q) -> Option<(usize, &K, &mut V)>
 858:     where
 859:         Q: ?Sized + Hash + Equivalent<K>,
 860:     {
 861:         if let Some(i) = self.get_index_of(key) {
 862:             let entry = &mut self.as_entries_mut()[i];
 863:             Some((i, &entry.key, &mut entry.value))
 864:         } else {
 865:             None
 866:         }
 867:     }
 868:     /// Return the values for `N` keys.
 869:     ///
 870:     /// ***Panics*** if any key is duplicated.
 871:     ///
 872:     /// # Examples
 873:     ///
 874:     /// ```
 875:     /// let mut map = indexmap::IndexMap::from([(1, 'a'), (3, 'b'), (2, 'c')]);
 876:     /// assert_eq!(
 877:     ///   map.get_disjoint_mut([&2, &1, &0]),
 878:     ///   [Some(&mut 'c'), Some(&mut 'a'), None],
 879:     /// );
 880:     /// ```
 881:     #[track_caller]
 882:     pub fn get_disjoint_mut<Q, const N: usize>(&mut self, keys: [&Q; N]) -> [Option<&mut V>; N]
 883:     where
 884:         Q: ?Sized + Hash + Equivalent<K>,
 885:     {
 886:         let indices = keys.map(|key| self.get_index_of(key));
 887:         disjoint::get_disjoint_opt_mut(self.as_entries_mut(), indices)
 888:             .map(|opt| opt.map(Bucket::value_mut))
 889:     }
 890:     /// Remove the key-value pair equivalent to `key` and return
 891:     /// its value.
 892:     ///
 893:     /// **NOTE:** This is equivalent to [`.swap_remove(key)`][Self::swap_remove], replacing this
 894:     /// entry's position with the last element, and it is deprecated in favor of calling that
 895:     /// explicitly. If you need to preserve the relative order of the keys in the map, use
 896:     /// [`.shift_remove(key)`][Self::shift_remove] instead.
 897:     #[deprecated(note = "`remove` disrupts the map order -- \
 898:         use `swap_remove` or `shift_remove` for explicit behavior.")]
 899:     pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
 900:     where
 901:         Q: ?Sized + Hash + Equivalent<K>,
 902:     {
 903:         self.swap_remove(key)
 904:     }
 905:     /// Remove and return the key-value pair equivalent to `key`.
 906:     ///
 907:     /// **NOTE:** This is equivalent to [`.swap_remove_entry(key)`][Self::swap_remove_entry],
 908:     /// replacing this entry's position with the last element, and it is deprecated in favor of
 909:     /// calling that explicitly. If you need to preserve the relative order of the keys in the map,
 910:     /// use [`.shift_remove_entry(key)`][Self::shift_remove_entry] instead.
 911:     #[deprecated(note = "`remove_entry` disrupts the map order -- \
 912:         use `swap_remove_entry` or `shift_remove_entry` for explicit behavior.")]
 913:     pub fn remove_entry<Q>(&mut self, key: &Q) -> Option<(K, V)>
 914:     where
 915:         Q: ?Sized + Hash + Equivalent<K>,
 916:     {
 917:         self.swap_remove_entry(key)
 918:     }
 919:     /// Remove the key-value pair equivalent to `key` and return
 920:     /// its value.
 921:     ///
 922:     /// Like [`Vec::swap_remove`], the pair is removed by swapping it with the
 923:     /// last element of the map and popping it off. **This perturbs
 924:     /// the position of what used to be the last element!**
 925:     ///
 926:     /// Return `None` if `key` is not in map.
 927:     ///
 928:     /// Computes in **O(1)** time (average).
 929:     pub fn swap_remove<Q>(&mut self, key: &Q) -> Option<V>
 930:     where
 931:         Q: ?Sized + Hash + Equivalent<K>,
 932:     {
 933:         self.swap_remove_full(key).map(third)
 934:     }
 935:     /// Remove and return the key-value pair equivalent to `key`.
 936:     ///
 937:     /// Like [`Vec::swap_remove`], the pair is removed by swapping it with the
 938:     /// last element of the map and popping it off. **This perturbs
 939:     /// the position of what used to be the last element!**
 940:     ///
 941:     /// Return `None` if `key` is not in map.
 942:     ///
 943:     /// Computes in **O(1)** time (average).
 944:     pub fn swap_remove_entry<Q>(&mut self, key: &Q) -> Option<(K, V)>
 945:     where
 946:         Q: ?Sized + Hash + Equivalent<K>,
 947:     {
 948:         match self.swap_remove_full(key) {
 949:             Some((_, key, value)) => Some((key, value)),
 950:             None => None,
 951:         }
 952:     }
 953:     /// Remove the key-value pair equivalent to `key` and return it and
 954:     /// the index it had.
 955:     ///
 956:     /// Like [`Vec::swap_remove`], the pair is removed by swapping it with the
 957:     /// last element of the map and popping it off. **This perturbs
 958:     /// the position of what used to be the last element!**
 959:     ///
 960:     /// Return `None` if `key` is not in map.
 961:     ///
 962:     /// Computes in **O(1)** time (average).
 963:     pub fn swap_remove_full<Q>(&mut self, key: &Q) -> Option<(usize, K, V)>
 964:     where
 965:         Q: ?Sized + Hash + Equivalent<K>,
 966:     {
 967:         match self.as_entries() {
 968:             [x] if key.equivalent(&x.key) => {
 969:                 let (k, v) = self.core.pop()?;
 970:                 Some((0, k, v))
 971:             }
 972:             [_] | [] => None,
 973:             _ => {
 974:                 let hash = self.hash(key);
 975:                 self.core.swap_remove_full(hash, key)
 976:             }
 977:         }
 978:     }
 979:     /// Remove the key-value pair equivalent to `key` and return
 980:     /// its value.
 981:     ///
 982:     /// Like [`Vec::remove`], the pair is removed by shifting all of the
 983:     /// elements that follow it, preserving their relative order.
 984:     /// **This perturbs the index of all of those elements!**
 985:     ///
 986:     /// Return `None` if `key` is not in map.
 987:     ///
 988:     /// Computes in **O(n)** time (average).
 989:     pub fn shift_remove<Q>(&mut self, key: &Q) -> Option<V>
 990:     where
 991:         Q: ?Sized + Hash + Equivalent<K>,
 992:     {
 993:         self.shift_remove_full(key).map(third)
 994:     }
 995:     /// Remove and return the key-value pair equivalent to `key`.
 996:     ///
 997:     /// Like [`Vec::remove`], the pair is removed by shifting all of the
 998:     /// elements that follow it, preserving their relative order.
 999:     /// **This perturbs the index of all of those elements!**
1000:     ///
1001:     /// Return `None` if `key` is not in map.
1002:     ///
1003:     /// Computes in **O(n)** time (average).
1004:     pub fn shift_remove_entry<Q>(&mut self, key: &Q) -> Option<(K, V)>
1005:     where
1006:         Q: ?Sized + Hash + Equivalent<K>,
1007:     {
1008:         match self.shift_remove_full(key) {
1009:             Some((_, key, value)) => Some((key, value)),
1010:             None => None,
1011:         }
1012:     }
1013:     /// Remove the key-value pair equivalent to `key` and return it and
1014:     /// the index it had.
1015:     ///
1016:     /// Like [`Vec::remove`], the pair is removed by shifting all of the
1017:     /// elements that follow it, preserving their relative order.
1018:     /// **This perturbs the index of all of those elements!**
1019:     ///
1020:     /// Return `None` if `key` is not in map.
1021:     ///
1022:     /// Computes in **O(n)** time (average).
1023:     pub fn shift_remove_full<Q>(&mut self, key: &Q) -> Option<(usize, K, V)>
1024:     where
1025:         Q: ?Sized + Hash + Equivalent<K>,
1026:     {
1027:         match self.as_entries() {
1028:             [x] if key.equivalent(&x.key) => {
1029:                 let (k, v) = self.core.pop()?;
1030:                 Some((0, k, v))
1031:             }
1032:             [_] | [] => None,
1033:             _ => {
1034:                 let hash = self.hash(key);
1035:                 self.core.shift_remove_full(hash, key)
1036:             }
1037:         }
1038:     }
1039: }
1040: impl<K, V, S> IndexMap<K, V, S> {
1041:     /// Remove the last key-value pair
1042:     ///
1043:     /// This preserves the order of the remaining elements.
1044:     ///
1045:     /// Computes in **O(1)** time (average).
1046:     #[doc(alias = "pop_last")] // like `BTreeMap`
1047:     pub fn pop(&mut self) -> Option<(K, V)> {
1048:         self.core.pop()
1049:     }
1050:     /// Removes and returns the last key-value pair from a map if the predicate
1051:     /// returns `true`, or [`None`] if the predicate returns false or the map
1052:     /// is empty (the predicate will not be called in that case).
1053:     ///
1054:     /// This preserves the order of the remaining elements.
1055:     ///
1056:     /// Computes in **O(1)** time (average).
1057:     ///
1058:     /// # Examples
1059:     ///
1060:     /// ```
1061:     /// use indexmap::IndexMap;
1062:     ///
1063:     /// let init = [(1, 'a'), (2, 'b'), (3, 'c'), (4, 'd')];
1064:     /// let mut map = IndexMap::from(init);
1065:     /// let pred = |key: &i32, _value: &mut char| *key % 2 == 0;
1066:     ///
1067:     /// assert_eq!(map.pop_if(pred), Some((4, 'd')));
1068:     /// assert_eq!(map.as_slice(), &init[..3]);
1069:     /// assert_eq!(map.pop_if(pred), None);
1070:     /// ```
1071:     pub fn pop_if(&mut self, predicate: impl FnOnce(&K, &mut V) -> bool) -> Option<(K, V)> {
1072:         let (last_key, last_value) = self.last_mut()?;
1073:         if predicate(last_key, last_value) {
1074:             self.core.pop()
1075:         } else {
1076:             None
1077:         }
1078:     }
1079:     /// Scan through each key-value pair in the map and keep those where the
1080:     /// closure `keep` returns `true`.
1081:     ///
1082:     /// The elements are visited in order, and remaining elements keep their
1083:     /// order.
1084:     ///
1085:     /// Computes in **O(n)** time (average).
1086:     pub fn retain<F>(&mut self, mut keep: F)
1087:     where
1088:         F: FnMut(&K, &mut V) -> bool,
1089:     {
1090:         self.core.retain_in_order(move |k, v| keep(k, v));
1091:     }
1092:     /// Sort the map's key-value pairs by the default ordering of the keys.
1093:     ///
1094:     /// This is a stable sort -- but equivalent keys should not normally coexist in
1095:     /// a map at all, so [`sort_unstable_keys`][Self::sort_unstable_keys] is preferred
1096:     /// because it is generally faster and doesn't allocate auxiliary memory.
1097:     ///
1098:     /// See [`sort_by`](Self::sort_by) for details.
1099:     pub fn sort_keys(&mut self)
1100:     where
1101:         K: Ord,
1102:     {
1103:         self.with_entries(move |entries| {
1104:             entries.sort_by(move |a, b| K::cmp(&a.key, &b.key));
1105:         });
1106:     }
1107:     /// Sort the map's key-value pairs in place using the comparison
1108:     /// function `cmp`.
1109:     ///
1110:     /// The comparison function receives two key and value pairs to compare (you
1111:     /// can sort by keys or values or their combination as needed).
1112:     ///
1113:     /// Computes in **O(n log n + c)** time and **O(n)** space where *n* is
1114:     /// the length of the map and *c* the capacity. The sort is stable.
1115:     pub fn sort_by<F>(&mut self, mut cmp: F)
1116:     where
1117:         F: FnMut(&K, &V, &K, &V) -> Ordering,
1118:     {
1119:         self.with_entries(move |entries| {
1120:             entries.sort_by(move |a, b| cmp(&a.key, &a.value, &b.key, &b.value));
1121:         });
1122:     }
1123:     /// Sort the key-value pairs of the map and return a by-value iterator of
1124:     /// the key-value pairs with the result.
1125:     ///
1126:     /// The sort is stable.
1127:     pub fn sorted_by<F>(self, mut cmp: F) -> IntoIter<K, V>
1128:     where
1129:         F: FnMut(&K, &V, &K, &V) -> Ordering,
1130:     {
1131:         let mut entries = self.into_entries();
1132:         entries.sort_by(move |a, b| cmp(&a.key, &a.value, &b.key, &b.value));
1133:         IntoIter::new(entries)
1134:     }
1135:     /// Sort the map's key-value pairs in place using a sort-key extraction function.
1136:     ///
1137:     /// Computes in **O(n log n + c)** time and **O(n)** space where *n* is
1138:     /// the length of the map and *c* the capacity. The sort is stable.
1139:     pub fn sort_by_key<T, F>(&mut self, mut sort_key: F)
1140:     where
1141:         T: Ord,
1142:         F: FnMut(&K, &V) -> T,
1143:     {
1144:         self.with_entries(move |entries| {
1145:             entries.sort_by_key(move |a| sort_key(&a.key, &a.value));
1146:         });
1147:     }
1148:     /// Sort the map's key-value pairs by the default ordering of the keys, but
1149:     /// may not preserve the order of equal elements.
1150:     ///
1151:     /// See [`sort_unstable_by`](Self::sort_unstable_by) for details.
1152:     pub fn sort_unstable_keys(&mut self)
1153:     where
1154:         K: Ord,
1155:     {
1156:         self.with_entries(move |entries| {
1157:             entries.sort_unstable_by(move |a, b| K::cmp(&a.key, &b.key));
1158:         });
1159:     }
1160:     /// Sort the map's key-value pairs in place using the comparison function `cmp`, but
1161:     /// may not preserve the order of equal elements.
1162:     ///
1163:     /// The comparison function receives two key and value pairs to compare (you
1164:     /// can sort by keys or values or their combination as needed).
1165:     ///
1166:     /// Computes in **O(n log n + c)** time where *n* is
1167:     /// the length of the map and *c* is the capacity. The sort is unstable.
1168:     pub fn sort_unstable_by<F>(&mut self, mut cmp: F)
1169:     where
1170:         F: FnMut(&K, &V, &K, &V) -> Ordering,
1171:     {
1172:         self.with_entries(move |entries| {
1173:             entries.sort_unstable_by(move |a, b| cmp(&a.key, &a.value, &b.key, &b.value));
1174:         });
1175:     }
1176:     /// Sort the key-value pairs of the map and return a by-value iterator of
1177:     /// the key-value pairs with the result.
1178:     ///
1179:     /// The sort is unstable.
1180:     #[inline]
1181:     pub fn sorted_unstable_by<F>(self, mut cmp: F) -> IntoIter<K, V>
1182:     where
1183:         F: FnMut(&K, &V, &K, &V) -> Ordering,
1184:     {
1185:         let mut entries = self.into_entries();
1186:         entries.sort_unstable_by(move |a, b| cmp(&a.key, &a.value, &b.key, &b.value));
1187:         IntoIter::new(entries)
1188:     }
1189:     /// Sort the map's key-value pairs in place using a sort-key extraction function.
1190:     ///
1191:     /// Computes in **O(n log n + c)** time where *n* is
1192:     /// the length of the map and *c* is the capacity. The sort is unstable.
1193:     pub fn sort_unstable_by_key<T, F>(&mut self, mut sort_key: F)
1194:     where
1195:         T: Ord,
1196:         F: FnMut(&K, &V) -> T,
1197:     {
1198:         self.with_entries(move |entries| {
1199:             entries.sort_unstable_by_key(move |a| sort_key(&a.key, &a.value));
1200:         });
1201:     }
1202:     /// Sort the map's key-value pairs in place using a sort-key extraction function.
1203:     ///
1204:     /// During sorting, the function is called at most once per entry, by using temporary storage
1205:     /// to remember the results of its evaluation. The order of calls to the function is
1206:     /// unspecified and may change between versions of `indexmap` or the standard library.
1207:     ///
1208:     /// Computes in **O(m n + n log n + c)** time () and **O(n)** space, where the function is
1209:     /// **O(m)**, *n* is the length of the map, and *c* the capacity. The sort is stable.
1210:     pub fn sort_by_cached_key<T, F>(&mut self, mut sort_key: F)
1211:     where
1212:         T: Ord,
1213:         F: FnMut(&K, &V) -> T,
1214:     {
1215:         self.with_entries(move |entries| {
1216:             entries.sort_by_cached_key(move |a| sort_key(&a.key, &a.value));
1217:         });
1218:     }
1219:     /// Search over a sorted map for a key.
1220:     ///
1221:     /// Returns the position where that key is present, or the position where it can be inserted to
1222:     /// maintain the sort. See [`slice::binary_search`] for more details.
1223:     ///
1224:     /// Computes in **O(log(n))** time, which is notably less scalable than looking the key up
1225:     /// using [`get_index_of`][IndexMap::get_index_of], but this can also position missing keys.
1226:     pub fn binary_search_keys(&self, x: &K) -> Result<usize, usize>
1227:     where
1228:         K: Ord,
1229:     {
1230:         self.as_slice().binary_search_keys(x)
1231:     }
1232:     /// Search over a sorted map with a comparator function.
1233:     ///
1234:     /// Returns the position where that value is present, or the position where it can be inserted
1235:     /// to maintain the sort. See [`slice::binary_search_by`] for more details.
1236:     ///
1237:     /// Computes in **O(log(n))** time.
1238:     #[inline]
1239:     pub fn binary_search_by<'a, F>(&'a self, f: F) -> Result<usize, usize>
1240:     where
1241:         F: FnMut(&'a K, &'a V) -> Ordering,
1242:     {
1243:         self.as_slice().binary_search_by(f)
1244:     }
1245:     /// Search over a sorted map with an extraction function.
1246:     ///
1247:     /// Returns the position where that value is present, or the position where it can be inserted
1248:     /// to maintain the sort. See [`slice::binary_search_by_key`] for more details.
1249:     ///
1250:     /// Computes in **O(log(n))** time.
1251:     #[inline]
1252:     pub fn binary_search_by_key<'a, B, F>(&'a self, b: &B, f: F) -> Result<usize, usize>
1253:     where
1254:         F: FnMut(&'a K, &'a V) -> B,
1255:         B: Ord,
1256:     {
1257:         self.as_slice().binary_search_by_key(b, f)
1258:     }
1259:     /// Checks if the keys of this map are sorted.
1260:     #[inline]
1261:     pub fn is_sorted(&self) -> bool
1262:     where
1263:         K: PartialOrd,
1264:     {
1265:         self.as_slice().is_sorted()
1266:     }
1267:     /// Checks if this map is sorted using the given comparator function.
1268:     #[inline]
1269:     pub fn is_sorted_by<'a, F>(&'a self, cmp: F) -> bool
1270:     where
1271:         F: FnMut(&'a K, &'a V, &'a K, &'a V) -> bool,
1272:     {
1273:         self.as_slice().is_sorted_by(cmp)
1274:     }
1275:     /// Checks if this map is sorted using the given sort-key function.
1276:     #[inline]
1277:     pub fn is_sorted_by_key<'a, F, T>(&'a self, sort_key: F) -> bool
1278:     where
1279:         F: FnMut(&'a K, &'a V) -> T,
1280:         T: PartialOrd,
1281:     {
1282:         self.as_slice().is_sorted_by_key(sort_key)
1283:     }
1284:     /// Returns the index of the partition point of a sorted map according to the given predicate
1285:     /// (the index of the first element of the second partition).
1286:     ///
1287:     /// See [`slice::partition_point`] for more details.
1288:     ///
1289:     /// Computes in **O(log(n))** time.
1290:     #[must_use]
1291:     pub fn partition_point<P>(&self, pred: P) -> usize
1292:     where
1293:         P: FnMut(&K, &V) -> bool,
1294:     {
1295:         self.as_slice().partition_point(pred)
1296:     }
1297:     /// Reverses the order of the map's key-value pairs in place.
1298:     ///
1299:     /// Computes in **O(n)** time and **O(1)** space.
1300:     pub fn reverse(&mut self) {
1301:         self.core.reverse()
1302:     }
1303:     /// Returns a slice of all the key-value pairs in the map.
1304:     ///
1305:     /// Computes in **O(1)** time.
1306:     pub fn as_slice(&self) -> &Slice<K, V> {
1307:         Slice::from_slice(self.as_entries())
1308:     }
1309:     /// Returns a mutable slice of all the key-value pairs in the map.
1310:     ///
1311:     /// Computes in **O(1)** time.
1312:     pub fn as_mut_slice(&mut self) -> &mut Slice<K, V> {
1313:         Slice::from_mut_slice(self.as_entries_mut())
1314:     }
1315:     /// Converts into a boxed slice of all the key-value pairs in the map.
1316:     ///
1317:     /// Note that this will drop the inner hash table and any excess capacity.
1318:     pub fn into_boxed_slice(self) -> Box<Slice<K, V>> {
1319:         Slice::from_boxed(self.into_entries().into_boxed_slice())
1320:     }
1321:     /// Get a key-value pair by index
1322:     ///
1323:     /// Valid indices are `0 <= index < self.len()`.
1324:     ///
1325:     /// Computes in **O(1)** time.
1326:     pub fn get_index(&self, index: usize) -> Option<(&K, &V)> {
1327:         self.as_entries().get(index).map(Bucket::refs)
1328:     }
1329:     /// Get a key-value pair by index
1330:     ///
1331:     /// Valid indices are `0 <= index < self.len()`.
1332:     ///
1333:     /// Computes in **O(1)** time.
1334:     pub fn get_index_mut(&mut self, index: usize) -> Option<(&K, &mut V)> {
1335:         self.as_entries_mut().get_mut(index).map(Bucket::ref_mut)
1336:     }
1337:     /// Get an entry in the map by index for in-place manipulation.
1338:     ///
1339:     /// Valid indices are `0 <= index < self.len()`.
1340:     ///
1341:     /// Computes in **O(1)** time.
1342:     pub fn get_index_entry(&mut self, index: usize) -> Option<IndexedEntry<'_, K, V>> {
1343:         IndexedEntry::new(&mut self.core, index)
1344:     }
1345:     /// Get an array of `N` key-value pairs by `N` indices
1346:     ///
1347:     /// Valid indices are *0 <= index < self.len()* and each index needs to be unique.
1348:     ///
1349:     /// # Examples
1350:     ///
1351:     /// ```
1352:     /// let mut map = indexmap::IndexMap::from([(1, 'a'), (3, 'b'), (2, 'c')]);
1353:     /// assert_eq!(map.get_disjoint_indices_mut([2, 0]), Ok([(&2, &mut 'c'), (&1, &mut 'a')]));
1354:     /// ```
1355:     pub fn get_disjoint_indices_mut<const N: usize>(
1356:         &mut self,
1357:         indices: [usize; N],
1358:     ) -> Result<[(&K, &mut V); N], GetDisjointMutError> {
1359:         self.as_mut_slice().get_disjoint_mut(indices)
1360:     }
1361:     /// Returns a slice of key-value pairs in the given range of indices.
1362:     ///
1363:     /// Valid indices are `0 <= index < self.len()`.
1364:     ///
1365:     /// Computes in **O(1)** time.
1366:     pub fn get_range<R: RangeBounds<usize>>(&self, range: R) -> Option<&Slice<K, V>> {
1367:         let entries = self.as_entries();
1368:         let range = try_simplify_range(range, entries.len())?;
1369:         entries.get(range).map(Slice::from_slice)
1370:     }
1371:     /// Returns a mutable slice of key-value pairs in the given range of indices.
1372:     ///
1373:     /// Valid indices are `0 <= index < self.len()`.
1374:     ///
1375:     /// Computes in **O(1)** time.
1376:     pub fn get_range_mut<R: RangeBounds<usize>>(&mut self, range: R) -> Option<&mut Slice<K, V>> {
1377:         let entries = self.as_entries_mut();
1378:         let range = try_simplify_range(range, entries.len())?;
1379:         entries.get_mut(range).map(Slice::from_mut_slice)
1380:     }
1381:     /// Get the first key-value pair
1382:     ///
1383:     /// Computes in **O(1)** time.
1384:     #[doc(alias = "first_key_value")] // like `BTreeMap`
1385:     pub fn first(&self) -> Option<(&K, &V)> {
1386:         self.as_entries().first().map(Bucket::refs)
1387:     }
1388:     /// Get the first key-value pair, with mutable access to the value
1389:     ///
1390:     /// Computes in **O(1)** time.
1391:     pub fn first_mut(&mut self) -> Option<(&K, &mut V)> {
1392:         self.as_entries_mut().first_mut().map(Bucket::ref_mut)
1393:     }
1394:     /// Get the first entry in the map for in-place manipulation.
1395:     ///
1396:     /// Computes in **O(1)** time.
1397:     pub fn first_entry(&mut self) -> Option<IndexedEntry<'_, K, V>> {
1398:         self.get_index_entry(0)
1399:     }
1400:     /// Get the last key-value pair
1401:     ///
1402:     /// Computes in **O(1)** time.
1403:     #[doc(alias = "last_key_value")] // like `BTreeMap`
1404:     pub fn last(&self) -> Option<(&K, &V)> {
1405:         self.as_entries().last().map(Bucket::refs)
1406:     }
1407:     /// Get the last key-value pair, with mutable access to the value
1408:     ///
1409:     /// Computes in **O(1)** time.
1410:     pub fn last_mut(&mut self) -> Option<(&K, &mut V)> {
1411:         self.as_entries_mut().last_mut().map(Bucket::ref_mut)
1412:     }
1413:     /// Get the last entry in the map for in-place manipulation.
1414:     ///
1415:     /// Computes in **O(1)** time.
1416:     pub fn last_entry(&mut self) -> Option<IndexedEntry<'_, K, V>> {
1417:         self.get_index_entry(self.len().checked_sub(1)?)
1418:     }
1419:     /// Remove the key-value pair by index
1420:     ///
1421:     /// Valid indices are `0 <= index < self.len()`.
1422:     ///
1423:     /// Like [`Vec::swap_remove`], the pair is removed by swapping it with the
1424:     /// last element of the map and popping it off. **This perturbs
1425:     /// the position of what used to be the last element!**
1426:     ///
1427:     /// Computes in **O(1)** time (average).
1428:     pub fn swap_remove_index(&mut self, index: usize) -> Option<(K, V)> {
1429:         self.core.swap_remove_index(index)
1430:     }
1431:     /// Remove the key-value pair by index
1432:     ///
1433:     /// Valid indices are `0 <= index < self.len()`.
1434:     ///
1435:     /// Like [`Vec::remove`], the pair is removed by shifting all of the
1436:     /// elements that follow it, preserving their relative order.
1437:     /// **This perturbs the index of all of those elements!**
1438:     ///
1439:     /// Computes in **O(n)** time (average).
1440:     pub fn shift_remove_index(&mut self, index: usize) -> Option<(K, V)> {
1441:         self.core.shift_remove_index(index)
1442:     }
1443:     /// Moves the position of a key-value pair from one index to another
1444:     /// by shifting all other pairs in-between.
1445:     ///
1446:     /// * If `from < to`, the other pairs will shift down while the targeted pair moves up.
1447:     /// * If `from > to`, the other pairs will shift up while the targeted pair moves down.
1448:     ///
1449:     /// ***Panics*** if `from` or `to` are out of bounds.
1450:     ///
1451:     /// Computes in **O(n)** time (average).
1452:     #[track_caller]
1453:     pub fn move_index(&mut self, from: usize, to: usize) {
1454:         self.core.move_index(from, to)
1455:     }
1456:     /// Swaps the position of two key-value pairs in the map.
1457:     ///
1458:     /// ***Panics*** if `a` or `b` are out of bounds.
1459:     ///
1460:     /// Computes in **O(1)** time (average).
1461:     #[track_caller]
1462:     pub fn swap_indices(&mut self, a: usize, b: usize) {
1463:         self.core.swap_indices(a, b)
1464:     }
1465: }
1466: /// Access [`IndexMap`] values corresponding to a key.
1467: ///
1468: /// # Examples
1469: ///
1470: /// ```
1471: /// use indexmap::IndexMap;
1472: ///
1473: /// let mut map = IndexMap::new();
1474: /// for word in "Lorem ipsum dolor sit amet".split_whitespace() {
1475: ///     map.insert(word.to_lowercase(), word.to_uppercase());
1476: /// }
1477: /// assert_eq!(map["lorem"], "LOREM");
1478: /// assert_eq!(map["ipsum"], "IPSUM");
1479: /// ```
1480: ///
1481: /// ```should_panic
1482: /// use indexmap::IndexMap;
1483: ///
1484: /// let mut map = IndexMap::new();
1485: /// map.insert("foo", 1);
1486: /// println!("{:?}", map["bar"]); // panics!
1487: /// ```
1488: impl<K, V, Q: ?Sized, S> Index<&Q> for IndexMap<K, V, S>
1489: where
1490:     Q: Hash + Equivalent<K>,
1491:     S: BuildHasher,
1492: {
1493:     type Output = V;
1494:     /// Returns a reference to the value corresponding to the supplied `key`.
1495:     ///
1496:     /// ***Panics*** if `key` is not present in the map.
1497:     fn index(&self, key: &Q) -> &V {
1498:         self.get(key).expect("no entry found for key")
1499:     }
1500: }
1501: /// Access [`IndexMap`] values corresponding to a key.
1502: ///
1503: /// Mutable indexing allows changing / updating values of key-value
1504: /// pairs that are already present.
1505: ///
1506: /// You can **not** insert new pairs with index syntax, use `.insert()`.
1507: ///
1508: /// # Examples
1509: ///
1510: /// ```
1511: /// use indexmap::IndexMap;
1512: ///
1513: /// let mut map = IndexMap::new();
1514: /// for word in "Lorem ipsum dolor sit amet".split_whitespace() {
1515: ///     map.insert(word.to_lowercase(), word.to_string());
1516: /// }
1517: /// let lorem = &mut map["lorem"];
1518: /// assert_eq!(lorem, "Lorem");
1519: /// lorem.retain(char::is_lowercase);
1520: /// assert_eq!(map["lorem"], "orem");
1521: /// ```
1522: ///
1523: /// ```should_panic
1524: /// use indexmap::IndexMap;
1525: ///
1526: /// let mut map = IndexMap::new();
1527: /// map.insert("foo", 1);
1528: /// map["bar"] = 1; // panics!
1529: /// ```
1530: impl<K, V, Q: ?Sized, S> IndexMut<&Q> for IndexMap<K, V, S>
1531: where
1532:     Q: Hash + Equivalent<K>,
1533:     S: BuildHasher,
1534: {
1535:     /// Returns a mutable reference to the value corresponding to the supplied `key`.
1536:     ///
1537:     /// ***Panics*** if `key` is not present in the map.
1538:     fn index_mut(&mut self, key: &Q) -> &mut V {
1539:         self.get_mut(key).expect("no entry found for key")
1540:     }
1541: }
1542: /// Access [`IndexMap`] values at indexed positions.
1543: ///
1544: /// See [`Index<usize> for Keys`][keys] to access a map's keys instead.
1545: ///
1546: /// [keys]: Keys#impl-Index<usize>-for-Keys<'a,+K,+V>
1547: ///
1548: /// # Examples
1549: ///
1550: /// ```
1551: /// use indexmap::IndexMap;
1552: ///
1553: /// let mut map = IndexMap::new();
1554: /// for word in "Lorem ipsum dolor sit amet".split_whitespace() {
1555: ///     map.insert(word.to_lowercase(), word.to_uppercase());
1556: /// }
1557: /// assert_eq!(map[0], "LOREM");
1558: /// assert_eq!(map[1], "IPSUM");
1559: /// map.reverse();
1560: /// assert_eq!(map[0], "AMET");
1561: /// assert_eq!(map[1], "SIT");
1562: /// map.sort_keys();
1563: /// assert_eq!(map[0], "AMET");
1564: /// assert_eq!(map[1], "DOLOR");
1565: /// ```
1566: ///
1567: /// ```should_panic
1568: /// use indexmap::IndexMap;
1569: ///
1570: /// let mut map = IndexMap::new();
1571: /// map.insert("foo", 1);
1572: /// println!("{:?}", map[10]); // panics!
1573: /// ```
1574: impl<K, V, S> Index<usize> for IndexMap<K, V, S> {
1575:     type Output = V;
1576:     /// Returns a reference to the value at the supplied `index`.
1577:     ///
1578:     /// ***Panics*** if `index` is out of bounds.
1579:     fn index(&self, index: usize) -> &V {
1580:         assert_index_lt(index, self.len());
1581:         &self.as_entries()[index].value
1582:     }
1583: }
1584: /// Access [`IndexMap`] values at indexed positions.
1585: ///
1586: /// Mutable indexing allows changing / updating indexed values
1587: /// that are already present.
1588: ///
1589: /// You can **not** insert new values with index syntax -- use [`.insert()`][IndexMap::insert].
1590: ///
1591: /// # Examples
1592: ///
1593: /// ```
1594: /// use indexmap::IndexMap;
1595: ///
1596: /// let mut map = IndexMap::new();
1597: /// for word in "Lorem ipsum dolor sit amet".split_whitespace() {
1598: ///     map.insert(word.to_lowercase(), word.to_string());
1599: /// }
1600: /// let lorem = &mut map[0];
1601: /// assert_eq!(lorem, "Lorem");
1602: /// lorem.retain(char::is_lowercase);
1603: /// assert_eq!(map["lorem"], "orem");
1604: /// ```
1605: ///
1606: /// ```should_panic
1607: /// use indexmap::IndexMap;
1608: ///
1609: /// let mut map = IndexMap::new();
1610: /// map.insert("foo", 1);
1611: /// map[10] = 1; // panics!
1612: /// ```
1613: impl<K, V, S> IndexMut<usize> for IndexMap<K, V, S> {
1614:     /// Returns a mutable reference to the value at the supplied `index`.
1615:     ///
1616:     /// ***Panics*** if `index` is out of bounds.
1617:     fn index_mut(&mut self, index: usize) -> &mut V {
1618:         assert_index_lt(index, self.len());
1619:         &mut self.as_entries_mut()[index].value
1620:     }
1621: }
1622: impl<K, V, S> FromIterator<(K, V)> for IndexMap<K, V, S>
1623: where
1624:     K: Hash + Eq,
1625:     S: BuildHasher + Default,
1626: {
1627:     /// Create an `IndexMap` from the sequence of key-value pairs in the
1628:     /// iterable.
1629:     ///
1630:     /// `from_iter` uses the same logic as `extend`. See
1631:     /// [`extend`][IndexMap::extend] for more details.
1632:     fn from_iter<I: IntoIterator<Item = (K, V)>>(iterable: I) -> Self {
1633:         let iter = iterable.into_iter();
1634:         let (low, _) = iter.size_hint();
1635:         let mut map = Self::with_capacity_and_hasher(low, <_>::default());
1636:         map.extend(iter);
1637:         map
1638:     }
1639: }
1640: #[cfg(feature = "std")]
1641: #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
1642: impl<K, V, const N: usize> From<[(K, V); N]> for IndexMap<K, V, RandomState>
1643: where
1644:     K: Hash + Eq,
1645: {
1646:     /// # Examples
1647:     ///
1648:     /// ```
1649:     /// use indexmap::IndexMap;
1650:     ///
1651:     /// let map1 = IndexMap::from([(1, 2), (3, 4)]);
1652:     /// let map2: IndexMap<_, _> = [(1, 2), (3, 4)].into();
1653:     /// assert_eq!(map1, map2);
1654:     /// ```
1655:     fn from(arr: [(K, V); N]) -> Self {
1656:         Self::from_iter(arr)
1657:     }
1658: }
1659: impl<K, V, S> Extend<(K, V)> for IndexMap<K, V, S>
1660: where
1661:     K: Hash + Eq,
1662:     S: BuildHasher,
1663: {
1664:     /// Extend the map with all key-value pairs in the iterable.
1665:     ///
1666:     /// This is equivalent to calling [`insert`][IndexMap::insert] for each of
1667:     /// them in order, which means that for keys that already existed
1668:     /// in the map, their value is updated but it keeps the existing order.
1669:     ///
1670:     /// New keys are inserted in the order they appear in the sequence. If
1671:     /// equivalents of a key occur more than once, the last corresponding value
1672:     /// prevails.
1673:     fn extend<I: IntoIterator<Item = (K, V)>>(&mut self, iterable: I) {
1674:         // (Note: this is a copy of `std`/`hashbrown`'s reservation logic.)
1675:         // Keys may be already present or show multiple times in the iterator.
1676:         // Reserve the entire hint lower bound if the map is empty.
1677:         // Otherwise reserve half the hint (rounded up), so the map
1678:         // will only resize twice in the worst case.
1679:         let iter = iterable.into_iter();
1680:         let (lower_len, _) = iter.size_hint();
1681:         let reserve = if self.is_empty() {
1682:             lower_len
1683:         } else {
1684:             lower_len.div_ceil(2)
1685:         };
1686:         self.reserve(reserve);
1687:         iter.for_each(move |(k, v)| {
1688:             self.insert(k, v);
1689:         });
1690:     }
1691: }
1692: impl<'a, K, V, S> Extend<(&'a K, &'a V)> for IndexMap<K, V, S>
1693: where
1694:     K: Hash + Eq + Copy,
1695:     V: Copy,
1696:     S: BuildHasher,
1697: {
1698:     /// Extend the map with all key-value pairs in the iterable.
1699:     ///
1700:     /// See the first extend method for more details.
1701:     fn extend<I: IntoIterator<Item = (&'a K, &'a V)>>(&mut self, iterable: I) {
1702:         self.extend(iterable.into_iter().map(|(&key, &value)| (key, value)));
1703:     }
1704: }
1705: impl<K, V, S> Default for IndexMap<K, V, S>
1706: where
1707:     S: Default,
1708: {
1709:     /// Return an empty [`IndexMap`]
1710:     fn default() -> Self {
1711:         Self::with_capacity_and_hasher(0, S::default())
1712:     }
1713: }
1714: impl<K, V1, S1, V2, S2> PartialEq<IndexMap<K, V2, S2>> for IndexMap<K, V1, S1>
1715: where
1716:     K: Hash + Eq,
1717:     V1: PartialEq<V2>,
1718:     S1: BuildHasher,
1719:     S2: BuildHasher,
1720: {
1721:     fn eq(&self, other: &IndexMap<K, V2, S2>) -> bool {
1722:         if self.len() != other.len() {
1723:             return false;
1724:         }
1725:         self.iter()
1726:             .all(|(key, value)| other.get(key).map_or(false, |v| *value == *v))
1727:     }
1728: }
1729: impl<K, V, S> Eq for IndexMap<K, V, S>
1730: where
1731:     K: Eq + Hash,
1732:     V: Eq,
1733:     S: BuildHasher,
1734: {
1735: }
````

## File: src/serde.rs
````rust
  1: #![cfg_attr(docsrs, doc(cfg(feature = "serde")))]
  2: use serde_core::de::value::{MapDeserializer, SeqDeserializer};
  3: use serde_core::de::{
  4:     Deserialize, Deserializer, Error, IntoDeserializer, MapAccess, SeqAccess, Visitor,
  5: };
  6: use serde_core::ser::{Serialize, Serializer};
  7: use core::fmt::{self, Formatter};
  8: use core::hash::{BuildHasher, Hash};
  9: use core::marker::PhantomData;
 10: use crate::{Bucket, IndexMap, IndexSet};
 11: /// Limit our preallocated capacity from a deserializer `size_hint()`.
 12: ///
 13: /// We do account for the `Bucket` overhead from its saved `hash` field, but we don't count the
 14: /// `RawTable` allocation or the fact that its raw capacity will be rounded up to a power of two.
 15: /// The "max" is an arbitrary choice anyway, not something that needs precise adherence.
 16: ///
 17: /// This is based on the internal `serde::de::size_hint::cautious(hint)` function.
 18: pub(crate) fn cautious_capacity<K, V>(hint: Option<usize>) -> usize {
 19:     const MAX_PREALLOC_BYTES: usize = 1024 * 1024;
 20:     Ord::min(
 21:         hint.unwrap_or(0),
 22:         MAX_PREALLOC_BYTES / size_of::<Bucket<K, V>>(),
 23:     )
 24: }
 25: impl<K, V, S> Serialize for IndexMap<K, V, S>
 26: where
 27:     K: Serialize,
 28:     V: Serialize,
 29: {
 30:     fn serialize<T>(&self, serializer: T) -> Result<T::Ok, T::Error>
 31:     where
 32:         T: Serializer,
 33:     {
 34:         serializer.collect_map(self)
 35:     }
 36: }
 37: struct IndexMapVisitor<K, V, S>(PhantomData<(K, V, S)>);
 38: impl<'de, K, V, S> Visitor<'de> for IndexMapVisitor<K, V, S>
 39: where
 40:     K: Deserialize<'de> + Eq + Hash,
 41:     V: Deserialize<'de>,
 42:     S: Default + BuildHasher,
 43: {
 44:     type Value = IndexMap<K, V, S>;
 45:     fn expecting(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
 46:         write!(formatter, "a map")
 47:     }
 48:     fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
 49:     where
 50:         A: MapAccess<'de>,
 51:     {
 52:         let capacity = cautious_capacity::<K, V>(map.size_hint());
 53:         let mut values = IndexMap::with_capacity_and_hasher(capacity, S::default());
 54:         while let Some((key, value)) = map.next_entry()? {
 55:             values.insert(key, value);
 56:         }
 57:         Ok(values)
 58:     }
 59: }
 60: impl<'de, K, V, S> Deserialize<'de> for IndexMap<K, V, S>
 61: where
 62:     K: Deserialize<'de> + Eq + Hash,
 63:     V: Deserialize<'de>,
 64:     S: Default + BuildHasher,
 65: {
 66:     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
 67:     where
 68:         D: Deserializer<'de>,
 69:     {
 70:         deserializer.deserialize_map(IndexMapVisitor(PhantomData))
 71:     }
 72: }
 73: impl<'de, K, V, S, E> IntoDeserializer<'de, E> for IndexMap<K, V, S>
 74: where
 75:     K: IntoDeserializer<'de, E> + Eq + Hash,
 76:     V: IntoDeserializer<'de, E>,
 77:     S: BuildHasher,
 78:     E: Error,
 79: {
 80:     type Deserializer = MapDeserializer<'de, <Self as IntoIterator>::IntoIter, E>;
 81:     fn into_deserializer(self) -> Self::Deserializer {
 82:         MapDeserializer::new(self.into_iter())
 83:     }
 84: }
 85: impl<T, S> Serialize for IndexSet<T, S>
 86: where
 87:     T: Serialize,
 88: {
 89:     fn serialize<Se>(&self, serializer: Se) -> Result<Se::Ok, Se::Error>
 90:     where
 91:         Se: Serializer,
 92:     {
 93:         serializer.collect_seq(self)
 94:     }
 95: }
 96: struct IndexSetVisitor<T, S>(PhantomData<(T, S)>);
 97: impl<'de, T, S> Visitor<'de> for IndexSetVisitor<T, S>
 98: where
 99:     T: Deserialize<'de> + Eq + Hash,
100:     S: Default + BuildHasher,
101: {
102:     type Value = IndexSet<T, S>;
103:     fn expecting(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
104:         write!(formatter, "a set")
105:     }
106:     fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
107:     where
108:         A: SeqAccess<'de>,
109:     {
110:         let capacity = cautious_capacity::<T, ()>(seq.size_hint());
111:         let mut values = IndexSet::with_capacity_and_hasher(capacity, S::default());
112:         while let Some(value) = seq.next_element()? {
113:             values.insert(value);
114:         }
115:         Ok(values)
116:     }
117: }
118: impl<'de, T, S> Deserialize<'de> for IndexSet<T, S>
119: where
120:     T: Deserialize<'de> + Eq + Hash,
121:     S: Default + BuildHasher,
122: {
123:     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
124:     where
125:         D: Deserializer<'de>,
126:     {
127:         deserializer.deserialize_seq(IndexSetVisitor(PhantomData))
128:     }
129: }
130: impl<'de, T, S, E> IntoDeserializer<'de, E> for IndexSet<T, S>
131: where
132:     T: IntoDeserializer<'de, E> + Eq + Hash,
133:     S: BuildHasher,
134:     E: Error,
135: {
136:     type Deserializer = SeqDeserializer<<Self as IntoIterator>::IntoIter, E>;
137:     fn into_deserializer(self) -> Self::Deserializer {
138:         SeqDeserializer::new(self.into_iter())
139:     }
140: }
````

## File: src/set.rs
````rust
   1: //! A hash set implemented using [`IndexMap`]
   2: mod iter;
   3: mod mutable;
   4: mod slice;
   5: #[cfg(test)]
   6: mod tests;
   7: pub use self::iter::{
   8:     Difference, Drain, ExtractIf, Intersection, IntoIter, Iter, Splice, SymmetricDifference, Union,
   9: };
  10: pub use self::mutable::MutableValues;
  11: pub use self::slice::Slice;
  12: use crate::TryReserveError;
  13: #[cfg(feature = "rayon")]
  14: pub use crate::rayon::set as rayon;
  15: #[cfg(feature = "std")]
  16: use std::hash::RandomState;
  17: use crate::util::{assert_index_lt, try_simplify_range};
  18: use alloc::boxed::Box;
  19: use alloc::vec::Vec;
  20: use core::cmp::Ordering;
  21: use core::fmt;
  22: use core::hash::{BuildHasher, Hash};
  23: use core::ops::{BitAnd, BitOr, BitXor, Index, RangeBounds, Sub};
  24: use super::{Equivalent, IndexMap};
  25: type Bucket<T> = super::Bucket<T, ()>;
  26: /// A hash set where the iteration order of the values is independent of their
  27: /// hash values.
  28: ///
  29: /// The interface is closely compatible with the standard
  30: /// [`HashSet`][std::collections::HashSet],
  31: /// but also has additional features.
  32: ///
  33: /// # Order
  34: ///
  35: /// The values have a consistent order that is determined by the sequence of
  36: /// insertion and removal calls on the set. The order does not depend on the
  37: /// values or the hash function at all. Note that insertion order and value
  38: /// are not affected if a re-insertion is attempted once an element is
  39: /// already present.
  40: ///
  41: /// All iterators traverse the set *in order*.  Set operation iterators like
  42: /// [`IndexSet::union`] produce a concatenated order, as do their matching "bitwise"
  43: /// operators.  See their documentation for specifics.
  44: ///
  45: /// The insertion order is preserved, with **notable exceptions** like the
  46: /// [`.remove()`][Self::remove] or [`.swap_remove()`][Self::swap_remove] methods.
  47: /// Methods such as [`.sort_by()`][Self::sort_by] of
  48: /// course result in a new order, depending on the sorting order.
  49: ///
  50: /// # Indices
  51: ///
  52: /// The values are indexed in a compact range without holes in the range
  53: /// `0..self.len()`. For example, the method `.get_full` looks up the index for
  54: /// a value, and the method `.get_index` looks up the value by index.
  55: ///
  56: /// # Complexity
  57: ///
  58: /// Internally, `IndexSet<T, S>` just holds an [`IndexMap<T, (), S>`](IndexMap). Thus the complexity
  59: /// of the two are the same for most methods.
  60: ///
  61: /// # Examples
  62: ///
  63: /// ```
  64: /// use indexmap::IndexSet;
  65: ///
  66: /// // Collects which letters appear in a sentence.
  67: /// let letters: IndexSet<_> = "a short treatise on fungi".chars().collect();
  68: ///
  69: /// assert!(letters.contains(&'s'));
  70: /// assert!(letters.contains(&'t'));
  71: /// assert!(letters.contains(&'u'));
  72: /// assert!(!letters.contains(&'y'));
  73: /// ```
  74: #[cfg(feature = "std")]
  75: pub struct IndexSet<T, S = RandomState> {
  76:     pub(crate) map: IndexMap<T, (), S>,
  77: }
  78: #[cfg(not(feature = "std"))]
  79: pub struct IndexSet<T, S> {
  80:     pub(crate) map: IndexMap<T, (), S>,
  81: }
  82: impl<T, S> Clone for IndexSet<T, S>
  83: where
  84:     T: Clone,
  85:     S: Clone,
  86: {
  87:     fn clone(&self) -> Self {
  88:         IndexSet {
  89:             map: self.map.clone(),
  90:         }
  91:     }
  92:     fn clone_from(&mut self, other: &Self) {
  93:         self.map.clone_from(&other.map);
  94:     }
  95: }
  96: impl<T, S> fmt::Debug for IndexSet<T, S>
  97: where
  98:     T: fmt::Debug,
  99: {
 100:     #[cfg(not(feature = "test_debug"))]
 101:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
 102:         f.debug_set().entries(self.iter()).finish()
 103:     }
 104:     #[cfg(feature = "test_debug")]
 105:     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
 106:         // Let the inner `IndexMap` print all of its details
 107:         f.debug_struct("IndexSet").field("map", &self.map).finish()
 108:     }
 109: }
 110: #[cfg(feature = "std")]
 111: #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
 112: impl<T> IndexSet<T> {
 113:     /// Create a new set. (Does not allocate.)
 114:     pub fn new() -> Self {
 115:         IndexSet {
 116:             map: IndexMap::new(),
 117:         }
 118:     }
 119:     /// Create a new set with capacity for `n` elements.
 120:     /// (Does not allocate if `n` is zero.)
 121:     ///
 122:     /// Computes in **O(n)** time.
 123:     pub fn with_capacity(n: usize) -> Self {
 124:         IndexSet {
 125:             map: IndexMap::with_capacity(n),
 126:         }
 127:     }
 128: }
 129: impl<T, S> IndexSet<T, S> {
 130:     /// Create a new set with capacity for `n` elements.
 131:     /// (Does not allocate if `n` is zero.)
 132:     ///
 133:     /// Computes in **O(n)** time.
 134:     pub fn with_capacity_and_hasher(n: usize, hash_builder: S) -> Self {
 135:         IndexSet {
 136:             map: IndexMap::with_capacity_and_hasher(n, hash_builder),
 137:         }
 138:     }
 139:     /// Create a new set with `hash_builder`.
 140:     ///
 141:     /// This function is `const`, so it
 142:     /// can be called in `static` contexts.
 143:     pub const fn with_hasher(hash_builder: S) -> Self {
 144:         IndexSet {
 145:             map: IndexMap::with_hasher(hash_builder),
 146:         }
 147:     }
 148:     #[inline]
 149:     pub(crate) fn into_entries(self) -> Vec<Bucket<T>> {
 150:         self.map.into_entries()
 151:     }
 152:     #[inline]
 153:     pub(crate) fn as_entries(&self) -> &[Bucket<T>] {
 154:         self.map.as_entries()
 155:     }
 156:     pub(crate) fn with_entries<F>(&mut self, f: F)
 157:     where
 158:         F: FnOnce(&mut [Bucket<T>]),
 159:     {
 160:         self.map.with_entries(f);
 161:     }
 162:     /// Return the number of elements the set can hold without reallocating.
 163:     ///
 164:     /// This number is a lower bound; the set might be able to hold more,
 165:     /// but is guaranteed to be able to hold at least this many.
 166:     ///
 167:     /// Computes in **O(1)** time.
 168:     pub fn capacity(&self) -> usize {
 169:         self.map.capacity()
 170:     }
 171:     /// Return a reference to the set's `BuildHasher`.
 172:     pub fn hasher(&self) -> &S {
 173:         self.map.hasher()
 174:     }
 175:     /// Return the number of elements in the set.
 176:     ///
 177:     /// Computes in **O(1)** time.
 178:     pub fn len(&self) -> usize {
 179:         self.map.len()
 180:     }
 181:     /// Returns true if the set contains no elements.
 182:     ///
 183:     /// Computes in **O(1)** time.
 184:     pub fn is_empty(&self) -> bool {
 185:         self.map.is_empty()
 186:     }
 187:     /// Return an iterator over the values of the set, in their order
 188:     pub fn iter(&self) -> Iter<'_, T> {
 189:         Iter::new(self.as_entries())
 190:     }
 191:     /// Remove all elements in the set, while preserving its capacity.
 192:     ///
 193:     /// Computes in **O(n)** time.
 194:     pub fn clear(&mut self) {
 195:         self.map.clear();
 196:     }
 197:     /// Shortens the set, keeping the first `len` elements and dropping the rest.
 198:     ///
 199:     /// If `len` is greater than the set's current length, this has no effect.
 200:     pub fn truncate(&mut self, len: usize) {
 201:         self.map.truncate(len);
 202:     }
 203:     /// Clears the `IndexSet` in the given index range, returning those values
 204:     /// as a drain iterator.
 205:     ///
 206:     /// The range may be any type that implements [`RangeBounds<usize>`],
 207:     /// including all of the `std::ops::Range*` types, or even a tuple pair of
 208:     /// `Bound` start and end values. To drain the set entirely, use `RangeFull`
 209:     /// like `set.drain(..)`.
 210:     ///
 211:     /// This shifts down all entries following the drained range to fill the
 212:     /// gap, and keeps the allocated memory for reuse.
 213:     ///
 214:     /// ***Panics*** if the starting point is greater than the end point or if
 215:     /// the end point is greater than the length of the set.
 216:     #[track_caller]
 217:     pub fn drain<R>(&mut self, range: R) -> Drain<'_, T>
 218:     where
 219:         R: RangeBounds<usize>,
 220:     {
 221:         Drain::new(self.map.core.drain(range))
 222:     }
 223:     /// Creates an iterator which uses a closure to determine if a value should be removed,
 224:     /// for all values in the given range.
 225:     ///
 226:     /// If the closure returns true, then the value is removed and yielded.
 227:     /// If the closure returns false, the value will remain in the list and will not be yielded
 228:     /// by the iterator.
 229:     ///
 230:     /// The range may be any type that implements [`RangeBounds<usize>`],
 231:     /// including all of the `std::ops::Range*` types, or even a tuple pair of
 232:     /// `Bound` start and end values. To check the entire set, use `RangeFull`
 233:     /// like `set.extract_if(.., predicate)`.
 234:     ///
 235:     /// If the returned `ExtractIf` is not exhausted, e.g. because it is dropped without iterating
 236:     /// or the iteration short-circuits, then the remaining elements will be retained.
 237:     /// Use [`retain`] with a negated predicate if you do not need the returned iterator.
 238:     ///
 239:     /// [`retain`]: IndexSet::retain
 240:     ///
 241:     /// ***Panics*** if the starting point is greater than the end point or if
 242:     /// the end point is greater than the length of the set.
 243:     ///
 244:     /// # Examples
 245:     ///
 246:     /// Splitting a set into even and odd values, reusing the original set:
 247:     ///
 248:     /// ```
 249:     /// use indexmap::IndexSet;
 250:     ///
 251:     /// let mut set: IndexSet<i32> = (0..8).collect();
 252:     /// let extracted: IndexSet<i32> = set.extract_if(.., |v| v % 2 == 0).collect();
 253:     ///
 254:     /// let evens = extracted.into_iter().collect::<Vec<_>>();
 255:     /// let odds = set.into_iter().collect::<Vec<_>>();
 256:     ///
 257:     /// assert_eq!(evens, vec![0, 2, 4, 6]);
 258:     /// assert_eq!(odds, vec![1, 3, 5, 7]);
 259:     /// ```
 260:     #[track_caller]
 261:     pub fn extract_if<F, R>(&mut self, range: R, pred: F) -> ExtractIf<'_, T, F>
 262:     where
 263:         F: FnMut(&T) -> bool,
 264:         R: RangeBounds<usize>,
 265:     {
 266:         ExtractIf::new(&mut self.map.core, range, pred)
 267:     }
 268:     /// Splits the collection into two at the given index.
 269:     ///
 270:     /// Returns a newly allocated set containing the elements in the range
 271:     /// `[at, len)`. After the call, the original set will be left containing
 272:     /// the elements `[0, at)` with its previous capacity unchanged.
 273:     ///
 274:     /// ***Panics*** if `at > len`.
 275:     #[track_caller]
 276:     pub fn split_off(&mut self, at: usize) -> Self
 277:     where
 278:         S: Clone,
 279:     {
 280:         Self {
 281:             map: self.map.split_off(at),
 282:         }
 283:     }
 284:     /// Reserve capacity for `additional` more values.
 285:     ///
 286:     /// Computes in **O(n)** time.
 287:     pub fn reserve(&mut self, additional: usize) {
 288:         self.map.reserve(additional);
 289:     }
 290:     /// Reserve capacity for `additional` more values, without over-allocating.
 291:     ///
 292:     /// Unlike `reserve`, this does not deliberately over-allocate the entry capacity to avoid
 293:     /// frequent re-allocations. However, the underlying data structures may still have internal
 294:     /// capacity requirements, and the allocator itself may give more space than requested, so this
 295:     /// cannot be relied upon to be precisely minimal.
 296:     ///
 297:     /// Computes in **O(n)** time.
 298:     pub fn reserve_exact(&mut self, additional: usize) {
 299:         self.map.reserve_exact(additional);
 300:     }
 301:     /// Try to reserve capacity for `additional` more values.
 302:     ///
 303:     /// Computes in **O(n)** time.
 304:     pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
 305:         self.map.try_reserve(additional)
 306:     }
 307:     /// Try to reserve capacity for `additional` more values, without over-allocating.
 308:     ///
 309:     /// Unlike `try_reserve`, this does not deliberately over-allocate the entry capacity to avoid
 310:     /// frequent re-allocations. However, the underlying data structures may still have internal
 311:     /// capacity requirements, and the allocator itself may give more space than requested, so this
 312:     /// cannot be relied upon to be precisely minimal.
 313:     ///
 314:     /// Computes in **O(n)** time.
 315:     pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
 316:         self.map.try_reserve_exact(additional)
 317:     }
 318:     /// Shrink the capacity of the set as much as possible.
 319:     ///
 320:     /// Computes in **O(n)** time.
 321:     pub fn shrink_to_fit(&mut self) {
 322:         self.map.shrink_to_fit();
 323:     }
 324:     /// Shrink the capacity of the set with a lower limit.
 325:     ///
 326:     /// Computes in **O(n)** time.
 327:     pub fn shrink_to(&mut self, min_capacity: usize) {
 328:         self.map.shrink_to(min_capacity);
 329:     }
 330: }
 331: impl<T, S> IndexSet<T, S>
 332: where
 333:     T: Hash + Eq,
 334:     S: BuildHasher,
 335: {
 336:     /// Insert the value into the set.
 337:     ///
 338:     /// If an equivalent item already exists in the set, it returns
 339:     /// `false` leaving the original value in the set and without
 340:     /// altering its insertion order. Otherwise, it inserts the new
 341:     /// item and returns `true`.
 342:     ///
 343:     /// Computes in **O(1)** time (amortized average).
 344:     pub fn insert(&mut self, value: T) -> bool {
 345:         self.map.insert(value, ()).is_none()
 346:     }
 347:     /// Insert the value into the set, and get its index.
 348:     ///
 349:     /// If an equivalent item already exists in the set, it returns
 350:     /// the index of the existing item and `false`, leaving the
 351:     /// original value in the set and without altering its insertion
 352:     /// order. Otherwise, it inserts the new item and returns the index
 353:     /// of the inserted item and `true`.
 354:     ///
 355:     /// Computes in **O(1)** time (amortized average).
 356:     pub fn insert_full(&mut self, value: T) -> (usize, bool) {
 357:         let (index, existing) = self.map.insert_full(value, ());
 358:         (index, existing.is_none())
 359:     }
 360:     /// Insert the value into the set at its ordered position among sorted values.
 361:     ///
 362:     /// This is equivalent to finding the position with
 363:     /// [`binary_search`][Self::binary_search], and if needed calling
 364:     /// [`insert_before`][Self::insert_before] for a new value.
 365:     ///
 366:     /// If the sorted item is found in the set, it returns the index of that
 367:     /// existing item and `false`, without any change. Otherwise, it inserts the
 368:     /// new item and returns its sorted index and `true`.
 369:     ///
 370:     /// If the existing items are **not** already sorted, then the insertion
 371:     /// index is unspecified (like [`slice::binary_search`]), but the value
 372:     /// is moved to or inserted at that position regardless.
 373:     ///
 374:     /// Computes in **O(n)** time (average). Instead of repeating calls to
 375:     /// `insert_sorted`, it may be faster to call batched [`insert`][Self::insert]
 376:     /// or [`extend`][Self::extend] and only call [`sort`][Self::sort] or
 377:     /// [`sort_unstable`][Self::sort_unstable] once.
 378:     pub fn insert_sorted(&mut self, value: T) -> (usize, bool)
 379:     where
 380:         T: Ord,
 381:     {
 382:         let (index, existing) = self.map.insert_sorted(value, ());
 383:         (index, existing.is_none())
 384:     }
 385:     /// Insert the value into the set at its ordered position among values
 386:     /// sorted by `cmp`.
 387:     ///
 388:     /// This is equivalent to finding the position with
 389:     /// [`binary_search_by`][Self::binary_search_by], then calling
 390:     /// [`insert_before`][Self::insert_before].
 391:     ///
 392:     /// If the existing items are **not** already sorted, then the insertion
 393:     /// index is unspecified (like [`slice::binary_search`]), but the value
 394:     /// is moved to or inserted at that position regardless.
 395:     ///
 396:     /// Computes in **O(n)** time (average).
 397:     pub fn insert_sorted_by<F>(&mut self, value: T, mut cmp: F) -> (usize, bool)
 398:     where
 399:         F: FnMut(&T, &T) -> Ordering,
 400:     {
 401:         let (index, existing) = self
 402:             .map
 403:             .insert_sorted_by(value, (), |a, (), b, ()| cmp(a, b));
 404:         (index, existing.is_none())
 405:     }
 406:     /// Insert the value into the set at its ordered position among values
 407:     /// using a sort-key extraction function.
 408:     ///
 409:     /// This is equivalent to finding the position with
 410:     /// [`binary_search_by_key`][Self::binary_search_by_key] with `sort_key(key)`,
 411:     /// then calling [`insert_before`][Self::insert_before].
 412:     ///
 413:     /// If the existing items are **not** already sorted, then the insertion
 414:     /// index is unspecified (like [`slice::binary_search`]), but the value
 415:     /// is moved to or inserted at that position regardless.
 416:     ///
 417:     /// Computes in **O(n)** time (average).
 418:     pub fn insert_sorted_by_key<B, F>(&mut self, value: T, mut sort_key: F) -> (usize, bool)
 419:     where
 420:         B: Ord,
 421:         F: FnMut(&T) -> B,
 422:     {
 423:         let (index, existing) = self.map.insert_sorted_by_key(value, (), |k, _| sort_key(k));
 424:         (index, existing.is_none())
 425:     }
 426:     /// Insert the value into the set before the value at the given index, or at the end.
 427:     ///
 428:     /// If an equivalent item already exists in the set, it returns `false` leaving the
 429:     /// original value in the set, but moved to the new position. The returned index
 430:     /// will either be the given index or one less, depending on how the value moved.
 431:     /// (See [`shift_insert`](Self::shift_insert) for different behavior here.)
 432:     ///
 433:     /// Otherwise, it inserts the new value exactly at the given index and returns `true`.
 434:     ///
 435:     /// ***Panics*** if `index` is out of bounds.
 436:     /// Valid indices are `0..=set.len()` (inclusive).
 437:     ///
 438:     /// Computes in **O(n)** time (average).
 439:     ///
 440:     /// # Examples
 441:     ///
 442:     /// ```
 443:     /// use indexmap::IndexSet;
 444:     /// let mut set: IndexSet<char> = ('a'..='z').collect();
 445:     ///
 446:     /// // The new value '*' goes exactly at the given index.
 447:     /// assert_eq!(set.get_index_of(&'*'), None);
 448:     /// assert_eq!(set.insert_before(10, '*'), (10, true));
 449:     /// assert_eq!(set.get_index_of(&'*'), Some(10));
 450:     ///
 451:     /// // Moving the value 'a' up will shift others down, so this moves *before* 10 to index 9.
 452:     /// assert_eq!(set.insert_before(10, 'a'), (9, false));
 453:     /// assert_eq!(set.get_index_of(&'a'), Some(9));
 454:     /// assert_eq!(set.get_index_of(&'*'), Some(10));
 455:     ///
 456:     /// // Moving the value 'z' down will shift others up, so this moves to exactly 10.
 457:     /// assert_eq!(set.insert_before(10, 'z'), (10, false));
 458:     /// assert_eq!(set.get_index_of(&'z'), Some(10));
 459:     /// assert_eq!(set.get_index_of(&'*'), Some(11));
 460:     ///
 461:     /// // Moving or inserting before the endpoint is also valid.
 462:     /// assert_eq!(set.len(), 27);
 463:     /// assert_eq!(set.insert_before(set.len(), '*'), (26, false));
 464:     /// assert_eq!(set.get_index_of(&'*'), Some(26));
 465:     /// assert_eq!(set.insert_before(set.len(), '+'), (27, true));
 466:     /// assert_eq!(set.get_index_of(&'+'), Some(27));
 467:     /// assert_eq!(set.len(), 28);
 468:     /// ```
 469:     #[track_caller]
 470:     pub fn insert_before(&mut self, index: usize, value: T) -> (usize, bool) {
 471:         let (index, existing) = self.map.insert_before(index, value, ());
 472:         (index, existing.is_none())
 473:     }
 474:     /// Insert the value into the set at the given index.
 475:     ///
 476:     /// If an equivalent item already exists in the set, it returns `false` leaving
 477:     /// the original value in the set, but moved to the given index.
 478:     /// Note that existing values **cannot** be moved to `index == set.len()`!
 479:     /// (See [`insert_before`](Self::insert_before) for different behavior here.)
 480:     ///
 481:     /// Otherwise, it inserts the new value at the given index and returns `true`.
 482:     ///
 483:     /// ***Panics*** if `index` is out of bounds.
 484:     /// Valid indices are `0..set.len()` (exclusive) when moving an existing value, or
 485:     /// `0..=set.len()` (inclusive) when inserting a new value.
 486:     ///
 487:     /// Computes in **O(n)** time (average).
 488:     ///
 489:     /// # Examples
 490:     ///
 491:     /// ```
 492:     /// use indexmap::IndexSet;
 493:     /// let mut set: IndexSet<char> = ('a'..='z').collect();
 494:     ///
 495:     /// // The new value '*' goes exactly at the given index.
 496:     /// assert_eq!(set.get_index_of(&'*'), None);
 497:     /// assert_eq!(set.shift_insert(10, '*'), true);
 498:     /// assert_eq!(set.get_index_of(&'*'), Some(10));
 499:     ///
 500:     /// // Moving the value 'a' up to 10 will shift others down, including the '*' that was at 10.
 501:     /// assert_eq!(set.shift_insert(10, 'a'), false);
 502:     /// assert_eq!(set.get_index_of(&'a'), Some(10));
 503:     /// assert_eq!(set.get_index_of(&'*'), Some(9));
 504:     ///
 505:     /// // Moving the value 'z' down to 9 will shift others up, including the '*' that was at 9.
 506:     /// assert_eq!(set.shift_insert(9, 'z'), false);
 507:     /// assert_eq!(set.get_index_of(&'z'), Some(9));
 508:     /// assert_eq!(set.get_index_of(&'*'), Some(10));
 509:     ///
 510:     /// // Existing values can move to len-1 at most, but new values can insert at the endpoint.
 511:     /// assert_eq!(set.len(), 27);
 512:     /// assert_eq!(set.shift_insert(set.len() - 1, '*'), false);
 513:     /// assert_eq!(set.get_index_of(&'*'), Some(26));
 514:     /// assert_eq!(set.shift_insert(set.len(), '+'), true);
 515:     /// assert_eq!(set.get_index_of(&'+'), Some(27));
 516:     /// assert_eq!(set.len(), 28);
 517:     /// ```
 518:     ///
 519:     /// ```should_panic
 520:     /// use indexmap::IndexSet;
 521:     /// let mut set: IndexSet<char> = ('a'..='z').collect();
 522:     ///
 523:     /// // This is an invalid index for moving an existing value!
 524:     /// set.shift_insert(set.len(), 'a');
 525:     /// ```
 526:     #[track_caller]
 527:     pub fn shift_insert(&mut self, index: usize, value: T) -> bool {
 528:         self.map.shift_insert(index, value, ()).is_none()
 529:     }
 530:     /// Adds a value to the set, replacing the existing value, if any, that is
 531:     /// equal to the given one, without altering its insertion order. Returns
 532:     /// the replaced value.
 533:     ///
 534:     /// Computes in **O(1)** time (average).
 535:     pub fn replace(&mut self, value: T) -> Option<T> {
 536:         self.replace_full(value).1
 537:     }
 538:     /// Adds a value to the set, replacing the existing value, if any, that is
 539:     /// equal to the given one, without altering its insertion order. Returns
 540:     /// the index of the item and its replaced value.
 541:     ///
 542:     /// Computes in **O(1)** time (average).
 543:     pub fn replace_full(&mut self, value: T) -> (usize, Option<T>) {
 544:         let hash = self.map.hash(&value);
 545:         match self.map.core.replace_full(hash, value, ()) {
 546:             (i, Some((replaced, ()))) => (i, Some(replaced)),
 547:             (i, None) => (i, None),
 548:         }
 549:     }
 550:     /// Replaces the value at the given index. The new value does not need to be
 551:     /// equivalent to the one it is replacing, but it must be unique to the rest
 552:     /// of the set.
 553:     ///
 554:     /// Returns `Ok(old_value)` if successful, or `Err((other_index, value))` if
 555:     /// an equivalent value already exists at a different index. The set will be
 556:     /// unchanged in the error case.
 557:     ///
 558:     /// ***Panics*** if `index` is out of bounds.
 559:     ///
 560:     /// Computes in **O(1)** time (average).
 561:     #[track_caller]
 562:     pub fn replace_index(&mut self, index: usize, value: T) -> Result<T, (usize, T)> {
 563:         self.map.replace_index(index, value)
 564:     }
 565:     /// Return an iterator over the values that are in `self` but not `other`.
 566:     ///
 567:     /// Values are produced in the same order that they appear in `self`.
 568:     pub fn difference<'a, S2>(&'a self, other: &'a IndexSet<T, S2>) -> Difference<'a, T, S2>
 569:     where
 570:         S2: BuildHasher,
 571:     {
 572:         Difference::new(self, other)
 573:     }
 574:     /// Return an iterator over the values that are in `self` or `other`,
 575:     /// but not in both.
 576:     ///
 577:     /// Values from `self` are produced in their original order, followed by
 578:     /// values from `other` in their original order.
 579:     pub fn symmetric_difference<'a, S2>(
 580:         &'a self,
 581:         other: &'a IndexSet<T, S2>,
 582:     ) -> SymmetricDifference<'a, T, S, S2>
 583:     where
 584:         S2: BuildHasher,
 585:     {
 586:         SymmetricDifference::new(self, other)
 587:     }
 588:     /// Return an iterator over the values that are in both `self` and `other`.
 589:     ///
 590:     /// Values are produced in the same order that they appear in `self`.
 591:     pub fn intersection<'a, S2>(&'a self, other: &'a IndexSet<T, S2>) -> Intersection<'a, T, S2>
 592:     where
 593:         S2: BuildHasher,
 594:     {
 595:         Intersection::new(self, other)
 596:     }
 597:     /// Return an iterator over all values that are in `self` or `other`.
 598:     ///
 599:     /// Values from `self` are produced in their original order, followed by
 600:     /// values that are unique to `other` in their original order.
 601:     pub fn union<'a, S2>(&'a self, other: &'a IndexSet<T, S2>) -> Union<'a, T, S>
 602:     where
 603:         S2: BuildHasher,
 604:     {
 605:         Union::new(self, other)
 606:     }
 607:     /// Creates a splicing iterator that replaces the specified range in the set
 608:     /// with the given `replace_with` iterator and yields the removed items.
 609:     /// `replace_with` does not need to be the same length as `range`.
 610:     ///
 611:     /// The `range` is removed even if the iterator is not consumed until the
 612:     /// end. It is unspecified how many elements are removed from the set if the
 613:     /// `Splice` value is leaked.
 614:     ///
 615:     /// The input iterator `replace_with` is only consumed when the `Splice`
 616:     /// value is dropped. If a value from the iterator matches an existing entry
 617:     /// in the set (outside of `range`), then the original will be unchanged.
 618:     /// Otherwise, the new value will be inserted in the replaced `range`.
 619:     ///
 620:     /// ***Panics*** if the starting point is greater than the end point or if
 621:     /// the end point is greater than the length of the set.
 622:     ///
 623:     /// # Examples
 624:     ///
 625:     /// ```
 626:     /// use indexmap::IndexSet;
 627:     ///
 628:     /// let mut set = IndexSet::from([0, 1, 2, 3, 4]);
 629:     /// let new = [5, 4, 3, 2, 1];
 630:     /// let removed: Vec<_> = set.splice(2..4, new).collect();
 631:     ///
 632:     /// // 1 and 4 kept their positions, while 5, 3, and 2 were newly inserted.
 633:     /// assert!(set.into_iter().eq([0, 1, 5, 3, 2, 4]));
 634:     /// assert_eq!(removed, &[2, 3]);
 635:     /// ```
 636:     #[track_caller]
 637:     pub fn splice<R, I>(&mut self, range: R, replace_with: I) -> Splice<'_, I::IntoIter, T, S>
 638:     where
 639:         R: RangeBounds<usize>,
 640:         I: IntoIterator<Item = T>,
 641:     {
 642:         Splice::new(self, range, replace_with.into_iter())
 643:     }
 644:     /// Moves all values from `other` into `self`, leaving `other` empty.
 645:     ///
 646:     /// This is equivalent to calling [`insert`][Self::insert] for each value
 647:     /// from `other` in order, which means that values that already exist
 648:     /// in `self` are unchanged in their current position.
 649:     ///
 650:     /// See also [`union`][Self::union] to iterate the combined values by
 651:     /// reference, without modifying `self` or `other`.
 652:     ///
 653:     /// # Examples
 654:     ///
 655:     /// ```
 656:     /// use indexmap::IndexSet;
 657:     ///
 658:     /// let mut a = IndexSet::from([3, 2, 1]);
 659:     /// let mut b = IndexSet::from([3, 4, 5]);
 660:     /// let old_capacity = b.capacity();
 661:     ///
 662:     /// a.append(&mut b);
 663:     ///
 664:     /// assert_eq!(a.len(), 5);
 665:     /// assert_eq!(b.len(), 0);
 666:     /// assert_eq!(b.capacity(), old_capacity);
 667:     ///
 668:     /// assert!(a.iter().eq(&[3, 2, 1, 4, 5]));
 669:     /// ```
 670:     pub fn append<S2>(&mut self, other: &mut IndexSet<T, S2>) {
 671:         self.map.append(&mut other.map);
 672:     }
 673: }
 674: impl<T, S> IndexSet<T, S>
 675: where
 676:     S: BuildHasher,
 677: {
 678:     /// Return `true` if an equivalent to `value` exists in the set.
 679:     ///
 680:     /// Computes in **O(1)** time (average).
 681:     pub fn contains<Q>(&self, value: &Q) -> bool
 682:     where
 683:         Q: ?Sized + Hash + Equivalent<T>,
 684:     {
 685:         self.map.contains_key(value)
 686:     }
 687:     /// Return a reference to the value stored in the set, if it is present,
 688:     /// else `None`.
 689:     ///
 690:     /// Computes in **O(1)** time (average).
 691:     pub fn get<Q>(&self, value: &Q) -> Option<&T>
 692:     where
 693:         Q: ?Sized + Hash + Equivalent<T>,
 694:     {
 695:         self.map.get_key_value(value).map(|(x, &())| x)
 696:     }
 697:     /// Return item index and value
 698:     pub fn get_full<Q>(&self, value: &Q) -> Option<(usize, &T)>
 699:     where
 700:         Q: ?Sized + Hash + Equivalent<T>,
 701:     {
 702:         self.map.get_full(value).map(|(i, x, &())| (i, x))
 703:     }
 704:     /// Return item index, if it exists in the set
 705:     ///
 706:     /// Computes in **O(1)** time (average).
 707:     pub fn get_index_of<Q>(&self, value: &Q) -> Option<usize>
 708:     where
 709:         Q: ?Sized + Hash + Equivalent<T>,
 710:     {
 711:         self.map.get_index_of(value)
 712:     }
 713:     /// Remove the value from the set, and return `true` if it was present.
 714:     ///
 715:     /// **NOTE:** This is equivalent to [`.swap_remove(value)`][Self::swap_remove], replacing this
 716:     /// value's position with the last element, and it is deprecated in favor of calling that
 717:     /// explicitly. If you need to preserve the relative order of the values in the set, use
 718:     /// [`.shift_remove(value)`][Self::shift_remove] instead.
 719:     #[deprecated(note = "`remove` disrupts the set order -- \
 720:         use `swap_remove` or `shift_remove` for explicit behavior.")]
 721:     pub fn remove<Q>(&mut self, value: &Q) -> bool
 722:     where
 723:         Q: ?Sized + Hash + Equivalent<T>,
 724:     {
 725:         self.swap_remove(value)
 726:     }
 727:     /// Remove the value from the set, and return `true` if it was present.
 728:     ///
 729:     /// Like [`Vec::swap_remove`], the value is removed by swapping it with the
 730:     /// last element of the set and popping it off. **This perturbs
 731:     /// the position of what used to be the last element!**
 732:     ///
 733:     /// Return `false` if `value` was not in the set.
 734:     ///
 735:     /// Computes in **O(1)** time (average).
 736:     pub fn swap_remove<Q>(&mut self, value: &Q) -> bool
 737:     where
 738:         Q: ?Sized + Hash + Equivalent<T>,
 739:     {
 740:         self.map.swap_remove(value).is_some()
 741:     }
 742:     /// Remove the value from the set, and return `true` if it was present.
 743:     ///
 744:     /// Like [`Vec::remove`], the value is removed by shifting all of the
 745:     /// elements that follow it, preserving their relative order.
 746:     /// **This perturbs the index of all of those elements!**
 747:     ///
 748:     /// Return `false` if `value` was not in the set.
 749:     ///
 750:     /// Computes in **O(n)** time (average).
 751:     pub fn shift_remove<Q>(&mut self, value: &Q) -> bool
 752:     where
 753:         Q: ?Sized + Hash + Equivalent<T>,
 754:     {
 755:         self.map.shift_remove(value).is_some()
 756:     }
 757:     /// Removes and returns the value in the set, if any, that is equal to the
 758:     /// given one.
 759:     ///
 760:     /// **NOTE:** This is equivalent to [`.swap_take(value)`][Self::swap_take], replacing this
 761:     /// value's position with the last element, and it is deprecated in favor of calling that
 762:     /// explicitly. If you need to preserve the relative order of the values in the set, use
 763:     /// [`.shift_take(value)`][Self::shift_take] instead.
 764:     #[deprecated(note = "`take` disrupts the set order -- \
 765:         use `swap_take` or `shift_take` for explicit behavior.")]
 766:     pub fn take<Q>(&mut self, value: &Q) -> Option<T>
 767:     where
 768:         Q: ?Sized + Hash + Equivalent<T>,
 769:     {
 770:         self.swap_take(value)
 771:     }
 772:     /// Removes and returns the value in the set, if any, that is equal to the
 773:     /// given one.
 774:     ///
 775:     /// Like [`Vec::swap_remove`], the value is removed by swapping it with the
 776:     /// last element of the set and popping it off. **This perturbs
 777:     /// the position of what used to be the last element!**
 778:     ///
 779:     /// Return `None` if `value` was not in the set.
 780:     ///
 781:     /// Computes in **O(1)** time (average).
 782:     pub fn swap_take<Q>(&mut self, value: &Q) -> Option<T>
 783:     where
 784:         Q: ?Sized + Hash + Equivalent<T>,
 785:     {
 786:         self.map.swap_remove_entry(value).map(|(x, ())| x)
 787:     }
 788:     /// Removes and returns the value in the set, if any, that is equal to the
 789:     /// given one.
 790:     ///
 791:     /// Like [`Vec::remove`], the value is removed by shifting all of the
 792:     /// elements that follow it, preserving their relative order.
 793:     /// **This perturbs the index of all of those elements!**
 794:     ///
 795:     /// Return `None` if `value` was not in the set.
 796:     ///
 797:     /// Computes in **O(n)** time (average).
 798:     pub fn shift_take<Q>(&mut self, value: &Q) -> Option<T>
 799:     where
 800:         Q: ?Sized + Hash + Equivalent<T>,
 801:     {
 802:         self.map.shift_remove_entry(value).map(|(x, ())| x)
 803:     }
 804:     /// Remove the value from the set return it and the index it had.
 805:     ///
 806:     /// Like [`Vec::swap_remove`], the value is removed by swapping it with the
 807:     /// last element of the set and popping it off. **This perturbs
 808:     /// the position of what used to be the last element!**
 809:     ///
 810:     /// Return `None` if `value` was not in the set.
 811:     pub fn swap_remove_full<Q>(&mut self, value: &Q) -> Option<(usize, T)>
 812:     where
 813:         Q: ?Sized + Hash + Equivalent<T>,
 814:     {
 815:         self.map.swap_remove_full(value).map(|(i, x, ())| (i, x))
 816:     }
 817:     /// Remove the value from the set return it and the index it had.
 818:     ///
 819:     /// Like [`Vec::remove`], the value is removed by shifting all of the
 820:     /// elements that follow it, preserving their relative order.
 821:     /// **This perturbs the index of all of those elements!**
 822:     ///
 823:     /// Return `None` if `value` was not in the set.
 824:     pub fn shift_remove_full<Q>(&mut self, value: &Q) -> Option<(usize, T)>
 825:     where
 826:         Q: ?Sized + Hash + Equivalent<T>,
 827:     {
 828:         self.map.shift_remove_full(value).map(|(i, x, ())| (i, x))
 829:     }
 830: }
 831: impl<T, S> IndexSet<T, S> {
 832:     /// Remove the last value
 833:     ///
 834:     /// This preserves the order of the remaining elements.
 835:     ///
 836:     /// Computes in **O(1)** time (average).
 837:     #[doc(alias = "pop_last")] // like `BTreeSet`
 838:     pub fn pop(&mut self) -> Option<T> {
 839:         self.map.pop().map(|(x, ())| x)
 840:     }
 841:     /// Removes and returns the last value from a set if the predicate
 842:     /// returns `true`, or [`None`] if the predicate returns false or the set
 843:     /// is empty (the predicate will not be called in that case).
 844:     ///
 845:     /// This preserves the order of the remaining elements.
 846:     ///
 847:     /// Computes in **O(1)** time (average).
 848:     ///
 849:     /// # Examples
 850:     ///
 851:     /// ```
 852:     /// use indexmap::IndexSet;
 853:     ///
 854:     /// let mut set = IndexSet::from([1, 2, 3, 4]);
 855:     /// let pred = |x: &i32| *x % 2 == 0;
 856:     ///
 857:     /// assert_eq!(set.pop_if(pred), Some(4));
 858:     /// assert_eq!(set.as_slice(), &[1, 2, 3]);
 859:     /// assert_eq!(set.pop_if(pred), None);
 860:     /// ```
 861:     pub fn pop_if(&mut self, predicate: impl FnOnce(&T) -> bool) -> Option<T> {
 862:         let last = self.last()?;
 863:         if predicate(last) { self.pop() } else { None }
 864:     }
 865:     /// Scan through each value in the set and keep those where the
 866:     /// closure `keep` returns `true`.
 867:     ///
 868:     /// The elements are visited in order, and remaining elements keep their
 869:     /// order.
 870:     ///
 871:     /// Computes in **O(n)** time (average).
 872:     pub fn retain<F>(&mut self, mut keep: F)
 873:     where
 874:         F: FnMut(&T) -> bool,
 875:     {
 876:         self.map.retain(move |x, &mut ()| keep(x))
 877:     }
 878:     /// Sort the set's values by their default ordering.
 879:     ///
 880:     /// This is a stable sort -- but equivalent values should not normally coexist in
 881:     /// a set at all, so [`sort_unstable`][Self::sort_unstable] is preferred
 882:     /// because it is generally faster and doesn't allocate auxiliary memory.
 883:     ///
 884:     /// See [`sort_by`](Self::sort_by) for details.
 885:     pub fn sort(&mut self)
 886:     where
 887:         T: Ord,
 888:     {
 889:         self.map.sort_keys()
 890:     }
 891:     /// Sort the set's values in place using the comparison function `cmp`.
 892:     ///
 893:     /// Computes in **O(n log n)** time and **O(n)** space. The sort is stable.
 894:     pub fn sort_by<F>(&mut self, mut cmp: F)
 895:     where
 896:         F: FnMut(&T, &T) -> Ordering,
 897:     {
 898:         self.map.sort_by(move |a, (), b, ()| cmp(a, b));
 899:     }
 900:     /// Sort the values of the set and return a by-value iterator of
 901:     /// the values with the result.
 902:     ///
 903:     /// The sort is stable.
 904:     pub fn sorted_by<F>(self, mut cmp: F) -> IntoIter<T>
 905:     where
 906:         F: FnMut(&T, &T) -> Ordering,
 907:     {
 908:         let mut entries = self.into_entries();
 909:         entries.sort_by(move |a, b| cmp(&a.key, &b.key));
 910:         IntoIter::new(entries)
 911:     }
 912:     /// Sort the set's values in place using a key extraction function.
 913:     ///
 914:     /// Computes in **O(n log n)** time and **O(n)** space. The sort is stable.
 915:     pub fn sort_by_key<K, F>(&mut self, mut sort_key: F)
 916:     where
 917:         K: Ord,
 918:         F: FnMut(&T) -> K,
 919:     {
 920:         self.with_entries(move |entries| {
 921:             entries.sort_by_key(move |a| sort_key(&a.key));
 922:         });
 923:     }
 924:     /// Sort the set's values by their default ordering.
 925:     ///
 926:     /// See [`sort_unstable_by`](Self::sort_unstable_by) for details.
 927:     pub fn sort_unstable(&mut self)
 928:     where
 929:         T: Ord,
 930:     {
 931:         self.map.sort_unstable_keys()
 932:     }
 933:     /// Sort the set's values in place using the comparison function `cmp`.
 934:     ///
 935:     /// Computes in **O(n log n)** time. The sort is unstable.
 936:     pub fn sort_unstable_by<F>(&mut self, mut cmp: F)
 937:     where
 938:         F: FnMut(&T, &T) -> Ordering,
 939:     {
 940:         self.map.sort_unstable_by(move |a, _, b, _| cmp(a, b))
 941:     }
 942:     /// Sort the values of the set and return a by-value iterator of
 943:     /// the values with the result.
 944:     pub fn sorted_unstable_by<F>(self, mut cmp: F) -> IntoIter<T>
 945:     where
 946:         F: FnMut(&T, &T) -> Ordering,
 947:     {
 948:         let mut entries = self.into_entries();
 949:         entries.sort_unstable_by(move |a, b| cmp(&a.key, &b.key));
 950:         IntoIter::new(entries)
 951:     }
 952:     /// Sort the set's values in place using a key extraction function.
 953:     ///
 954:     /// Computes in **O(n log n)** time. The sort is unstable.
 955:     pub fn sort_unstable_by_key<K, F>(&mut self, mut sort_key: F)
 956:     where
 957:         K: Ord,
 958:         F: FnMut(&T) -> K,
 959:     {
 960:         self.with_entries(move |entries| {
 961:             entries.sort_unstable_by_key(move |a| sort_key(&a.key));
 962:         });
 963:     }
 964:     /// Sort the set's values in place using a key extraction function.
 965:     ///
 966:     /// During sorting, the function is called at most once per entry, by using temporary storage
 967:     /// to remember the results of its evaluation. The order of calls to the function is
 968:     /// unspecified and may change between versions of `indexmap` or the standard library.
 969:     ///
 970:     /// Computes in **O(m n + n log n + c)** time () and **O(n)** space, where the function is
 971:     /// **O(m)**, *n* is the length of the map, and *c* the capacity. The sort is stable.
 972:     pub fn sort_by_cached_key<K, F>(&mut self, mut sort_key: F)
 973:     where
 974:         K: Ord,
 975:         F: FnMut(&T) -> K,
 976:     {
 977:         self.with_entries(move |entries| {
 978:             entries.sort_by_cached_key(move |a| sort_key(&a.key));
 979:         });
 980:     }
 981:     /// Search over a sorted set for a value.
 982:     ///
 983:     /// Returns the position where that value is present, or the position where it can be inserted
 984:     /// to maintain the sort. See [`slice::binary_search`] for more details.
 985:     ///
 986:     /// Computes in **O(log(n))** time, which is notably less scalable than looking the value up
 987:     /// using [`get_index_of`][IndexSet::get_index_of], but this can also position missing values.
 988:     pub fn binary_search(&self, x: &T) -> Result<usize, usize>
 989:     where
 990:         T: Ord,
 991:     {
 992:         self.as_slice().binary_search(x)
 993:     }
 994:     /// Search over a sorted set with a comparator function.
 995:     ///
 996:     /// Returns the position where that value is present, or the position where it can be inserted
 997:     /// to maintain the sort. See [`slice::binary_search_by`] for more details.
 998:     ///
 999:     /// Computes in **O(log(n))** time.
1000:     #[inline]
1001:     pub fn binary_search_by<'a, F>(&'a self, f: F) -> Result<usize, usize>
1002:     where
1003:         F: FnMut(&'a T) -> Ordering,
1004:     {
1005:         self.as_slice().binary_search_by(f)
1006:     }
1007:     /// Search over a sorted set with an extraction function.
1008:     ///
1009:     /// Returns the position where that value is present, or the position where it can be inserted
1010:     /// to maintain the sort. See [`slice::binary_search_by_key`] for more details.
1011:     ///
1012:     /// Computes in **O(log(n))** time.
1013:     #[inline]
1014:     pub fn binary_search_by_key<'a, B, F>(&'a self, b: &B, f: F) -> Result<usize, usize>
1015:     where
1016:         F: FnMut(&'a T) -> B,
1017:         B: Ord,
1018:     {
1019:         self.as_slice().binary_search_by_key(b, f)
1020:     }
1021:     /// Checks if the values of this set are sorted.
1022:     #[inline]
1023:     pub fn is_sorted(&self) -> bool
1024:     where
1025:         T: PartialOrd,
1026:     {
1027:         self.as_slice().is_sorted()
1028:     }
1029:     /// Checks if this set is sorted using the given comparator function.
1030:     #[inline]
1031:     pub fn is_sorted_by<'a, F>(&'a self, cmp: F) -> bool
1032:     where
1033:         F: FnMut(&'a T, &'a T) -> bool,
1034:     {
1035:         self.as_slice().is_sorted_by(cmp)
1036:     }
1037:     /// Checks if this set is sorted using the given sort-key function.
1038:     #[inline]
1039:     pub fn is_sorted_by_key<'a, F, K>(&'a self, sort_key: F) -> bool
1040:     where
1041:         F: FnMut(&'a T) -> K,
1042:         K: PartialOrd,
1043:     {
1044:         self.as_slice().is_sorted_by_key(sort_key)
1045:     }
1046:     /// Returns the index of the partition point of a sorted set according to the given predicate
1047:     /// (the index of the first element of the second partition).
1048:     ///
1049:     /// See [`slice::partition_point`] for more details.
1050:     ///
1051:     /// Computes in **O(log(n))** time.
1052:     #[must_use]
1053:     pub fn partition_point<P>(&self, pred: P) -> usize
1054:     where
1055:         P: FnMut(&T) -> bool,
1056:     {
1057:         self.as_slice().partition_point(pred)
1058:     }
1059:     /// Reverses the order of the set's values in place.
1060:     ///
1061:     /// Computes in **O(n)** time and **O(1)** space.
1062:     pub fn reverse(&mut self) {
1063:         self.map.reverse()
1064:     }
1065:     /// Returns a slice of all the values in the set.
1066:     ///
1067:     /// Computes in **O(1)** time.
1068:     pub fn as_slice(&self) -> &Slice<T> {
1069:         Slice::from_slice(self.as_entries())
1070:     }
1071:     /// Converts into a boxed slice of all the values in the set.
1072:     ///
1073:     /// Note that this will drop the inner hash table and any excess capacity.
1074:     pub fn into_boxed_slice(self) -> Box<Slice<T>> {
1075:         Slice::from_boxed(self.into_entries().into_boxed_slice())
1076:     }
1077:     /// Get a value by index
1078:     ///
1079:     /// Valid indices are `0 <= index < self.len()`.
1080:     ///
1081:     /// Computes in **O(1)** time.
1082:     pub fn get_index(&self, index: usize) -> Option<&T> {
1083:         self.as_entries().get(index).map(Bucket::key_ref)
1084:     }
1085:     /// Returns a slice of values in the given range of indices.
1086:     ///
1087:     /// Valid indices are `0 <= index < self.len()`.
1088:     ///
1089:     /// Computes in **O(1)** time.
1090:     pub fn get_range<R: RangeBounds<usize>>(&self, range: R) -> Option<&Slice<T>> {
1091:         let entries = self.as_entries();
1092:         let range = try_simplify_range(range, entries.len())?;
1093:         entries.get(range).map(Slice::from_slice)
1094:     }
1095:     /// Get the first value
1096:     ///
1097:     /// Computes in **O(1)** time.
1098:     pub fn first(&self) -> Option<&T> {
1099:         self.as_entries().first().map(Bucket::key_ref)
1100:     }
1101:     /// Get the last value
1102:     ///
1103:     /// Computes in **O(1)** time.
1104:     pub fn last(&self) -> Option<&T> {
1105:         self.as_entries().last().map(Bucket::key_ref)
1106:     }
1107:     /// Remove the value by index
1108:     ///
1109:     /// Valid indices are `0 <= index < self.len()`.
1110:     ///
1111:     /// Like [`Vec::swap_remove`], the value is removed by swapping it with the
1112:     /// last element of the set and popping it off. **This perturbs
1113:     /// the position of what used to be the last element!**
1114:     ///
1115:     /// Computes in **O(1)** time (average).
1116:     pub fn swap_remove_index(&mut self, index: usize) -> Option<T> {
1117:         self.map.swap_remove_index(index).map(|(x, ())| x)
1118:     }
1119:     /// Remove the value by index
1120:     ///
1121:     /// Valid indices are `0 <= index < self.len()`.
1122:     ///
1123:     /// Like [`Vec::remove`], the value is removed by shifting all of the
1124:     /// elements that follow it, preserving their relative order.
1125:     /// **This perturbs the index of all of those elements!**
1126:     ///
1127:     /// Computes in **O(n)** time (average).
1128:     pub fn shift_remove_index(&mut self, index: usize) -> Option<T> {
1129:         self.map.shift_remove_index(index).map(|(x, ())| x)
1130:     }
1131:     /// Moves the position of a value from one index to another
1132:     /// by shifting all other values in-between.
1133:     ///
1134:     /// * If `from < to`, the other values will shift down while the targeted value moves up.
1135:     /// * If `from > to`, the other values will shift up while the targeted value moves down.
1136:     ///
1137:     /// ***Panics*** if `from` or `to` are out of bounds.
1138:     ///
1139:     /// Computes in **O(n)** time (average).
1140:     #[track_caller]
1141:     pub fn move_index(&mut self, from: usize, to: usize) {
1142:         self.map.move_index(from, to)
1143:     }
1144:     /// Swaps the position of two values in the set.
1145:     ///
1146:     /// ***Panics*** if `a` or `b` are out of bounds.
1147:     ///
1148:     /// Computes in **O(1)** time (average).
1149:     #[track_caller]
1150:     pub fn swap_indices(&mut self, a: usize, b: usize) {
1151:         self.map.swap_indices(a, b)
1152:     }
1153: }
1154: /// Access [`IndexSet`] values at indexed positions.
1155: ///
1156: /// # Examples
1157: ///
1158: /// ```
1159: /// use indexmap::IndexSet;
1160: ///
1161: /// let mut set = IndexSet::new();
1162: /// for word in "Lorem ipsum dolor sit amet".split_whitespace() {
1163: ///     set.insert(word.to_string());
1164: /// }
1165: /// assert_eq!(set[0], "Lorem");
1166: /// assert_eq!(set[1], "ipsum");
1167: /// set.reverse();
1168: /// assert_eq!(set[0], "amet");
1169: /// assert_eq!(set[1], "sit");
1170: /// set.sort();
1171: /// assert_eq!(set[0], "Lorem");
1172: /// assert_eq!(set[1], "amet");
1173: /// ```
1174: ///
1175: /// ```should_panic
1176: /// use indexmap::IndexSet;
1177: ///
1178: /// let mut set = IndexSet::new();
1179: /// set.insert("foo");
1180: /// println!("{:?}", set[10]); // panics!
1181: /// ```
1182: impl<T, S> Index<usize> for IndexSet<T, S> {
1183:     type Output = T;
1184:     /// Returns a reference to the value at the supplied `index`.
1185:     ///
1186:     /// ***Panics*** if `index` is out of bounds.
1187:     fn index(&self, index: usize) -> &T {
1188:         assert_index_lt(index, self.len());
1189:         &self.map.as_entries()[index].key
1190:     }
1191: }
1192: impl<T, S> FromIterator<T> for IndexSet<T, S>
1193: where
1194:     T: Hash + Eq,
1195:     S: BuildHasher + Default,
1196: {
1197:     fn from_iter<I: IntoIterator<Item = T>>(iterable: I) -> Self {
1198:         let iter = iterable.into_iter().map(|x| (x, ()));
1199:         IndexSet {
1200:             map: IndexMap::from_iter(iter),
1201:         }
1202:     }
1203: }
1204: #[cfg(feature = "std")]
1205: #[cfg_attr(docsrs, doc(cfg(feature = "std")))]
1206: impl<T, const N: usize> From<[T; N]> for IndexSet<T, RandomState>
1207: where
1208:     T: Eq + Hash,
1209: {
1210:     /// # Examples
1211:     ///
1212:     /// ```
1213:     /// use indexmap::IndexSet;
1214:     ///
1215:     /// let set1 = IndexSet::from([1, 2, 3, 4]);
1216:     /// let set2: IndexSet<_> = [1, 2, 3, 4].into();
1217:     /// assert_eq!(set1, set2);
1218:     /// ```
1219:     fn from(arr: [T; N]) -> Self {
1220:         Self::from_iter(arr)
1221:     }
1222: }
1223: impl<T, S> Extend<T> for IndexSet<T, S>
1224: where
1225:     T: Hash + Eq,
1226:     S: BuildHasher,
1227: {
1228:     fn extend<I: IntoIterator<Item = T>>(&mut self, iterable: I) {
1229:         let iter = iterable.into_iter().map(|x| (x, ()));
1230:         self.map.extend(iter);
1231:     }
1232: }
1233: impl<'a, T, S> Extend<&'a T> for IndexSet<T, S>
1234: where
1235:     T: Hash + Eq + Copy + 'a,
1236:     S: BuildHasher,
1237: {
1238:     fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iterable: I) {
1239:         let iter = iterable.into_iter().copied();
1240:         self.extend(iter);
1241:     }
1242: }
1243: impl<T, S> Default for IndexSet<T, S>
1244: where
1245:     S: Default,
1246: {
1247:     /// Return an empty [`IndexSet`]
1248:     fn default() -> Self {
1249:         IndexSet {
1250:             map: IndexMap::default(),
1251:         }
1252:     }
1253: }
1254: impl<T, S1, S2> PartialEq<IndexSet<T, S2>> for IndexSet<T, S1>
1255: where
1256:     T: Hash + Eq,
1257:     S1: BuildHasher,
1258:     S2: BuildHasher,
1259: {
1260:     fn eq(&self, other: &IndexSet<T, S2>) -> bool {
1261:         self.len() == other.len() && self.is_subset(other)
1262:     }
1263: }
1264: impl<T, S> Eq for IndexSet<T, S>
1265: where
1266:     T: Eq + Hash,
1267:     S: BuildHasher,
1268: {
1269: }
1270: impl<T, S> IndexSet<T, S>
1271: where
1272:     T: Eq + Hash,
1273:     S: BuildHasher,
1274: {
1275:     /// Returns `true` if `self` has no elements in common with `other`.
1276:     pub fn is_disjoint<S2>(&self, other: &IndexSet<T, S2>) -> bool
1277:     where
1278:         S2: BuildHasher,
1279:     {
1280:         if self.len() <= other.len() {
1281:             self.iter().all(move |value| !other.contains(value))
1282:         } else {
1283:             other.iter().all(move |value| !self.contains(value))
1284:         }
1285:     }
1286:     /// Returns `true` if all elements of `self` are contained in `other`.
1287:     pub fn is_subset<S2>(&self, other: &IndexSet<T, S2>) -> bool
1288:     where
1289:         S2: BuildHasher,
1290:     {
1291:         self.len() <= other.len() && self.iter().all(move |value| other.contains(value))
1292:     }
1293:     /// Returns `true` if all elements of `other` are contained in `self`.
1294:     pub fn is_superset<S2>(&self, other: &IndexSet<T, S2>) -> bool
1295:     where
1296:         S2: BuildHasher,
1297:     {
1298:         other.is_subset(self)
1299:     }
1300: }
1301: impl<T, S1, S2> BitAnd<&IndexSet<T, S2>> for &IndexSet<T, S1>
1302: where
1303:     T: Eq + Hash + Clone,
1304:     S1: BuildHasher + Default,
1305:     S2: BuildHasher,
1306: {
1307:     type Output = IndexSet<T, S1>;
1308:     /// Returns the set intersection, cloned into a new set.
1309:     ///
1310:     /// Values are collected in the same order that they appear in `self`.
1311:     fn bitand(self, other: &IndexSet<T, S2>) -> Self::Output {
1312:         self.intersection(other).cloned().collect()
1313:     }
1314: }
1315: impl<T, S1, S2> BitOr<&IndexSet<T, S2>> for &IndexSet<T, S1>
1316: where
1317:     T: Eq + Hash + Clone,
1318:     S1: BuildHasher + Default,
1319:     S2: BuildHasher,
1320: {
1321:     type Output = IndexSet<T, S1>;
1322:     /// Returns the set union, cloned into a new set.
1323:     ///
1324:     /// Values from `self` are collected in their original order, followed by
1325:     /// values that are unique to `other` in their original order.
1326:     fn bitor(self, other: &IndexSet<T, S2>) -> Self::Output {
1327:         self.union(other).cloned().collect()
1328:     }
1329: }
1330: impl<T, S1, S2> BitXor<&IndexSet<T, S2>> for &IndexSet<T, S1>
1331: where
1332:     T: Eq + Hash + Clone,
1333:     S1: BuildHasher + Default,
1334:     S2: BuildHasher,
1335: {
1336:     type Output = IndexSet<T, S1>;
1337:     /// Returns the set symmetric-difference, cloned into a new set.
1338:     ///
1339:     /// Values from `self` are collected in their original order, followed by
1340:     /// values from `other` in their original order.
1341:     fn bitxor(self, other: &IndexSet<T, S2>) -> Self::Output {
1342:         self.symmetric_difference(other).cloned().collect()
1343:     }
1344: }
1345: impl<T, S1, S2> Sub<&IndexSet<T, S2>> for &IndexSet<T, S1>
1346: where
1347:     T: Eq + Hash + Clone,
1348:     S1: BuildHasher + Default,
1349:     S2: BuildHasher,
1350: {
1351:     type Output = IndexSet<T, S1>;
1352:     /// Returns the set difference, cloned into a new set.
1353:     ///
1354:     /// Values are collected in the same order that they appear in `self`.
1355:     fn sub(self, other: &IndexSet<T, S2>) -> Self::Output {
1356:         self.difference(other).cloned().collect()
1357:     }
1358: }
````

## File: src/sval.rs
````rust
 1: #![cfg_attr(docsrs, doc(cfg(feature = "sval")))]
 2: use crate::{IndexMap, IndexSet};
 3: use sval::{Stream, Value};
 4: impl<K: Value, V: Value, S> Value for IndexMap<K, V, S> {
 5:     fn stream<'sval, ST: Stream<'sval> + ?Sized>(&'sval self, stream: &mut ST) -> sval::Result {
 6:         stream.map_begin(Some(self.len()))?;
 7:         for (k, v) in self {
 8:             stream.map_key_begin()?;
 9:             stream.value(k)?;
10:             stream.map_key_end()?;
11:             stream.map_value_begin()?;
12:             stream.value(v)?;
13:             stream.map_value_end()?;
14:         }
15:         stream.map_end()
16:     }
17: }
18: impl<K: Value, S> Value for IndexSet<K, S> {
19:     fn stream<'sval, ST: Stream<'sval> + ?Sized>(&'sval self, stream: &mut ST) -> sval::Result {
20:         stream.seq_begin(Some(self.len()))?;
21:         for value in self {
22:             stream.seq_value_begin()?;
23:             stream.value(value)?;
24:             stream.seq_value_end()?;
25:         }
26:         stream.seq_end()
27:     }
28: }
````

## File: src/util.rs
````rust
 1: use core::ops::{Bound, Range, RangeBounds};
 2: pub(crate) fn third<A, B, C>(t: (A, B, C)) -> C {
 3:     t.2
 4: }
 5: #[inline]
 6: #[track_caller]
 7: pub(crate) fn assert_index_lt(index: usize, len: usize) {
 8:     assert!(
 9:         index < len,
10:         "index out of bounds: the len is {len} but the index is {index}",
11:     );
12: }
13: #[inline]
14: #[track_caller]
15: pub(crate) fn assert_index_le(index: usize, len: usize) {
16:     assert!(
17:         index <= len,
18:         "index out of bounds: the len is {len} but the index is {index}. Expected index <= len"
19:     );
20: }
21: #[track_caller]
22: pub(crate) fn simplify_range<R>(range: R, len: usize) -> Range<usize>
23: where
24:     R: RangeBounds<usize>,
25: {
26:     let start = match range.start_bound() {
27:         Bound::Unbounded => 0,
28:         Bound::Included(&i) if i <= len => i,
29:         Bound::Excluded(&i) if i < len => i + 1,
30:         Bound::Included(i) | Bound::Excluded(i) => {
31:             panic!("range start index {i} out of range for slice of length {len}")
32:         }
33:     };
34:     let end = match range.end_bound() {
35:         Bound::Unbounded => len,
36:         Bound::Excluded(&i) if i <= len => i,
37:         Bound::Included(&i) if i < len => i + 1,
38:         Bound::Included(i) | Bound::Excluded(i) => {
39:             panic!("range end index {i} out of range for slice of length {len}")
40:         }
41:     };
42:     if start > end {
43:         panic!(
44:             "range start index {:?} should be <= range end index {:?}",
45:             range.start_bound(),
46:             range.end_bound()
47:         );
48:     }
49:     start..end
50: }
51: pub(crate) fn try_simplify_range<R>(range: R, len: usize) -> Option<Range<usize>>
52: where
53:     R: RangeBounds<usize>,
54: {
55:     let start = match range.start_bound() {
56:         Bound::Unbounded => 0,
57:         Bound::Included(&i) if i <= len => i,
58:         Bound::Excluded(&i) if i < len => i + 1,
59:         _ => return None,
60:     };
61:     let end = match range.end_bound() {
62:         Bound::Unbounded => len,
63:         Bound::Excluded(&i) if i <= len => i,
64:         Bound::Included(&i) if i < len => i + 1,
65:         _ => return None,
66:     };
67:     if start > end {
68:         return None;
69:     }
70:     Some(start..end)
71: }
72: // Generic slice equality -- copied from the standard library but adding a custom comparator,
73: // allowing for our `Bucket` wrapper on either or both sides.
74: pub(crate) fn slice_eq<T, U>(left: &[T], right: &[U], eq: impl Fn(&T, &U) -> bool) -> bool {
75:     if left.len() != right.len() {
76:         return false;
77:     }
78:     // Implemented as explicit indexing rather
79:     // than zipped iterators for performance reasons.
80:     // See PR https://github.com/rust-lang/rust/pull/116846
81:     for i in 0..left.len() {
82:         // bound checks are optimized away
83:         if !eq(&left[i], &right[i]) {
84:             return false;
85:         }
86:     }
87:     true
88: }
````
