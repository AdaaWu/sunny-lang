# Shadow: Error Handling

## Philosophy

Sunny has **no null** and **no exceptions**. Instead, any operation that might fail returns a `Shadow`.

> A Shadow is what appears when sunlight is blocked. It doesn't crash your program - it simply tells you something went wrong.

## Creating Shadows

```sunny
lit err = Shadow("something went wrong")
```

Division by zero automatically produces a Shadow:

```sunny
lit result = 10 / 0
// result is Shadow("division by zero")
```

## Handling Shadows with match

The `match` expression forces you to handle both success and failure:

```sunny
fn dataShow(id: Int) -> String | Shadow {
    if id < 0 {
        out Shadow("ID must be positive")
    }
    out "Valid Data"
}

lit result = dataShow(-1)
match result {
    is String val -> print(val)
    is Shadow s -> print(s.message)
}
```

## Shadow Methods

- `.message` - get the error message as a String
- `.wrap(context)` - add context to the error
- `.unwrap_or(default)` - extract a default value

```sunny
lit err = Shadow("not found")
lit wrapped = err.wrap("DB")
print(wrapped.message)
// "DB: not found"

lit fallback = err.unwrap_or("default value")
print(fallback)
// "default value"
```

## Why Not Exceptions?

Exceptions are invisible. You can't tell from a function signature whether it might throw. Shadow makes errors **explicit** in the return type:

```sunny
fn userShow(id: Int) -> Map | Shadow {
    // caller KNOWS this might return a Shadow
}
```

This is the Solaris Principle: *code must be transparent like sunlight*.
