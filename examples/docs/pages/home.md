# Welcome to Sunny Lang

A modern programming language that is **transparent like sunlight**.

## Why Sunny?

- **Safe by default**: No null, no exceptions. `Shadow` handles all uncertainty
- **Enforced conventions**: Resource Action naming is built into the language
- **Simple syntax**: Clean, Go-inspired syntax with no semicolons
- **Built-in HTTP**: Define functions, get a web server for free

## Quick Example

```sunny
fn productIndex() -> List {
    out [
        {"id": 1, "name": "Sunny T-Shirt"},
        {"id": 2, "name": "Sunny Mug"}
    ]
}
```

This automatically maps to `GET /product` when you run:

```
sun serve app.sunny
```

---

[Get started now](/doc/getting-started)
