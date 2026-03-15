# Syntax Reference

## Keywords

| Keyword | Purpose |
|---------|---------|
| `lit` | Immutable binding |
| `glow` | Mutable binding |
| `fn` | Function definition |
| `out` | Return a value |
| `if` / `else` | Conditional |
| `for` / `in` | Loop over a list |
| `match` / `is` | Pattern matching |
| `ray` | Concurrent block |
| `and` / `or` / `not` | Logical operators |

## Types

- `Int` - integers: `42`, `-7`
- `Float` - decimals: `3.14`
- `String` - text: `"hello"` or `'hello'`
- `Bool` - `true` or `false`
- `List` - arrays: `[1, 2, 3]`
- `Map` - objects: `{"key": "value"}`
- `Shadow` - error wrapper: `Shadow("message")`

## Built-in Methods

### List

- `.length` - number of items
- `.push(val)` - add to end (returns new list)
- `.pop()` - remove last (returns new list)
- `.first()` / `.last()` - get first or last item
- `.contains(val)` - check membership
- `.reverse()` - reverse order
- `.join(sep)` - join into string

### String

- `.length` - character count
- `.trim()` - remove whitespace
- `.upper()` / `.lower()` - case conversion
- `.contains(sub)` - check substring
- `.split(sep)` - split into list
- `.starts_with(s)` / `.ends_with(s)` - prefix/suffix check

### Map

- `.length` - number of entries
- `.keys()` / `.values()` - get keys or values as list
- `.has(key)` - check if key exists
- `.remove(key)` - remove entry (returns new map)

### Shadow

- `.message` - get error message
- `.wrap(prefix)` - add context: `Shadow("err").wrap("DB")` = `Shadow("DB: err")`
- `.unwrap_or(default)` - get default value if shadow
