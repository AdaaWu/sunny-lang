# Getting Started

## Installation

Build from source with Rust:

```
git clone https://github.com/user/sunny-lang
cd sunny-lang
cargo build --release
```

## Your First Program

Create a file called `hello.sunny`:

```sunny
lit name = "World"
print("Hello, " + name + "!")
```

Run it:

```
sun hello.sunny
```

## Variables

Sunny has two kinds of bindings:

- `lit` - immutable (like the sun, it never changes)
- `glow` - mutable (it can change over time)

```sunny
lit pi = 3.14
glow count = 0
count = count + 1
```

## Functions

Functions use the `fn` keyword and must follow **Resource Action** naming:

```sunny
fn userShow(id: Int) -> String | Shadow {
    if id < 0 {
        out Shadow("invalid id")
    }
    out "User found"
}
```

Valid suffixes: `Index`, `Show`, `Store`, `Update`, `Remove`.

## Running a Server

```sunny
fn helloIndex() -> Map {
    out {"message": "Hello from Sunny!"}
}
```

```
sun serve app.sunny 3000
```

Your API is now live at `http://localhost:3000/hello`.
