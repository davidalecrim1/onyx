# Rope

A rope is a binary tree where each leaf holds a small string chunk and each internal node stores the total character count of its left subtree. This makes insert and delete at arbitrary positions O(log n) instead of O(n).

## Why not String

A `String` is a contiguous heap allocation. Inserting one character at position 5 in a 100 000-character file means shifting every byte after it. For an editor applying hundreds of mutations per second this is the wrong data structure.

## How it is used here

`ropey::Rope` is the storage backend for `Buffer`. The editor never materialises the full document as a flat string except when rendering a single visible line (`Buffer::line`) or when a caller explicitly needs it (`Buffer::to_string`). All cursor arithmetic — `line_to_char`, `remove`, `insert` — operates on the tree directly.

## What it does not solve

Ropes do not help with undo history (a separate list of inverse operations) or syntax highlighting (which requires an incremental parse tree). Those are out of scope for this milestone.
