```javascript

/* index */

for item at index in items {}

for (index, item) in items.indexed {}

/* zip - Option A */

for good, store in goods, stores {}

for good at index, store in goods, stores {}

for good, store at index in goods, stores {}

for good at i, store at j in goods, stores {}

/* zip - Option B */

for (item at index, user) in (items, users).zipped {}

for (item at i, user at j) in (items, users).zipped {}

for ((index, item), user) in (items.indexed, users).zipped {}

/* zip - Option C */

for good in goods, store in stores {}

for good at i in goods, store at j in stores {}

for (user, card) in (users, cards).zipped {}

```
